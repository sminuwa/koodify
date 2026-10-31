use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
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
pub async fn upsert(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<UpsertCredentialRequest>,
) -> Result<Json<Credential>, (StatusCode, String)> {
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

    db::credentials::upsert(
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
    .map(Json)
    .map_err(map_db)
}

/// `GET /credentials?customerId=&environment=`
pub async fn list(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListCredentialsQuery>,
) -> Result<Json<Vec<Credential>>, (StatusCode, String)> {
    let environment = match q.environment.as_deref() {
        Some(env) => Some(normalize_environment(env)?),
        None => None,
    };

    db::credentials::list(&state.pool, q.customer_id, environment)
        .await
        .map(Json)
        .map_err(map_db)
}

/// `GET /credentials/by-customer/{customer_id}`
pub async fn by_customer(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(customer_id): Path<i64>,
) -> Result<Json<Vec<Credential>>, (StatusCode, String)> {
    db::credentials::list_for_customer(&state.pool, customer_id)
        .await
        .map(Json)
        .map_err(map_db)
}

/// `GET /credentials/by-customer/{customer_id}/{environment}`
pub async fn by_customer_env(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path((customer_id, environment)): Path<(i64, String)>,
) -> Result<Json<Credential>, (StatusCode, String)> {
    let environment = normalize_environment(&environment)?;
    match db::credentials::find_for_customer_env(&state.pool, customer_id, environment).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /credentials/{id}` — credential row UUID.
pub async fn get_one(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Credential>, (StatusCode, String)> {
    match db::credentials::find_by_id(&state.pool, id).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /credentials/{id}/status`
pub async fn update_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<Credential>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    match db::credentials::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}
