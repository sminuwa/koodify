use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::db::credentials::UpsertCredential;
use crate::db::models::Credential;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCredentialRequest {
    /// Integer `customers.id` (1, 2, 3, …).
    pub customer_id: i64,
    /// `test` or `live`
    pub environment: String,
    pub public_key: String,
    pub private_key: String,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCredentialsQuery {
    pub customer_id: Option<i64>,
    pub environment: Option<String>,
    /// When true, include `privateKey` in the response. Default: redacted.
    #[serde(default)]
    pub reveal_private_key: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealQuery {
    /// When true, include `privateKey` in the response. Default: redacted.
    #[serde(default)]
    pub reveal_private_key: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

/// API view of a credential. `privateKey` is omitted unless explicitly revealed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialView {
    pub id: Uuid,
    pub customer_id: i64,
    pub environment: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    /// True when a private key is stored (even if redacted from this response).
    pub private_key_set: bool,
    pub metadata: Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CredentialView {
    fn from_row(row: Credential, reveal: bool) -> Self {
        let private_key_set = !row.private_key.is_empty();
        Self {
            id: row.id,
            customer_id: row.customer_id,
            environment: row.environment,
            public_key: row.public_key,
            private_key: if reveal {
                Some(row.private_key)
            } else {
                None
            },
            private_key_set,
            metadata: row.metadata,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn normalize_environment(raw: &str) -> Result<&'static str, (StatusCode, String)> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "test" | "sandbox" => Ok("test"),
        "live" | "production" | "prod" => Ok("live"),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "environment must be 'test' or 'live'".into(),
        )),
    }
}

/// `PUT /credentials` — upsert one env (`test` or `live`) for a customer.
/// Response redacts `privateKey` unless `?revealPrivateKey=true`.
pub async fn upsert(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<RevealQuery>,
    Json(body): Json<UpsertCredentialRequest>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    let public_key = body.public_key.trim();
    let private_key = body.private_key.trim();
    if public_key.is_empty() || private_key.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "publicKey and privateKey are required".into(),
        ));
    }

    let environment = normalize_environment(&body.environment)?;

    if db::customers::find_by_id(&state.pool, body.customer_id)
        .await
        .map_err(map_db)?
        .is_none()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "customerId {} does not exist; create the customer first",
                body.customer_id
            ),
        ));
    }

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    let row = db::credentials::upsert(
        &state.pool,
        UpsertCredential {
            customer_id: body.customer_id,
            environment,
            public_key,
            private_key,
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    .map_err(map_db)?;

    Ok(Json(CredentialView::from_row(row, q.reveal_private_key)))
}

/// `GET /credentials?customerId=&environment=&revealPrivateKey=`
pub async fn list(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListCredentialsQuery>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    let environment = match q.environment.as_deref() {
        Some(env) => Some(normalize_environment(env)?),
        None => None,
    };

    let rows = db::credentials::list(&state.pool, q.customer_id, environment)
        .await
        .map_err(map_db)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CredentialView::from_row(r, q.reveal_private_key))
            .collect(),
    ))
}

/// `GET /credentials/by-customer/{customer_id}?revealPrivateKey=`
pub async fn by_customer(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(customer_id): Path<i64>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    let rows = db::credentials::list_for_customer(&state.pool, customer_id)
        .await
        .map_err(map_db)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CredentialView::from_row(r, q.reveal_private_key))
            .collect(),
    ))
}

/// `GET /credentials/by-customer/{customer_id}/{environment}?revealPrivateKey=`
pub async fn by_customer_env(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path((customer_id, environment)): Path<(i64, String)>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    let environment = normalize_environment(&environment)?;
    match db::credentials::find_for_customer_env(&state.pool, customer_id, environment).await {
        Ok(Some(row)) => Ok(Json(CredentialView::from_row(row, q.reveal_private_key))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /credentials/{id}?revealPrivateKey=` — credential row UUID.
pub async fn get_one(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    match db::credentials::find_by_id(&state.pool, id).await {
        Ok(Some(row)) => Ok(Json(CredentialView::from_row(row, q.reveal_private_key))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /credentials/{id}/status?revealPrivateKey=`
pub async fn update_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<RevealQuery>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    match db::credentials::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => Ok(Json(CredentialView::from_row(row, q.reveal_private_key))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}
