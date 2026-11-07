use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::{
    hash_secret_key, mint_secret_key, secret_key_prefix, AdminAuth, ApiKeyAuth,
};
use crate::db;
use crate::db::credentials::UpsertCredential;
use crate::db::models::Credential;
use crate::services::merchant_provision;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCredentialRequest {
    /// Integer `customers.id` (1, 2, 3, …).
    pub customer_id: i64,
    /// `test` or `live`
    pub environment: String,
    pub public_key: String,
    /// Zainpay zainbox business id (`codeName`). Alias: `privateKey` (legacy).
    #[serde(alias = "privateKey", alias = "private_key")]
    pub zainbox_code: String,
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
    /// When true, include `zainboxCode` in the response. Default: redacted.
    #[serde(default, alias = "revealPrivateKey")]
    pub reveal_zainbox_code: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealQuery {
    /// When true, include `zainboxCode` in the response. Default: redacted.
    #[serde(default, alias = "revealPrivateKey")]
    pub reveal_zainbox_code: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

/// API view of a credential. Plaintext `secretKey` only on mint/rotate.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialView {
    pub id: Uuid,
    pub customer_id: i64,
    pub environment: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zainbox_code: Option<String>,
    /// True when a zainbox code is stored (even if redacted from this response).
    pub zainbox_code_set: bool,
    /// Display prefix of merchant secret (`sk_live_abcd…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key_prefix: Option<String>,
    /// True when a merchant secret hash is stored.
    pub secret_key_set: bool,
    /// Plaintext secret — only present right after mint/rotate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    pub metadata: Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CredentialView {
    fn from_row(row: Credential, reveal_zainbox: bool) -> Self {
        Self::from_row_with_secret(row, reveal_zainbox, None)
    }

    fn from_row_with_secret(
        row: Credential,
        reveal_zainbox: bool,
        secret_key: Option<String>,
    ) -> Self {
        let zainbox_code_set = !row.zainbox_code.is_empty();
        let secret_key_set = row
            .secret_key_hash
            .as_deref()
            .filter(|s| !s.is_empty())
            .is_some();
        Self {
            id: row.id,
            customer_id: row.customer_id,
            environment: row.environment,
            public_key: row.public_key,
            zainbox_code: if reveal_zainbox {
                Some(row.zainbox_code)
            } else {
                None
            },
            zainbox_code_set,
            secret_key_prefix: row.secret_key_prefix,
            secret_key_set,
            secret_key,
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

fn authorize_credential_read(
    auth: &ApiKeyAuth,
    row: &Credential,
) -> Result<(), (StatusCode, String)> {
    match auth.merchant() {
        None => Ok(()),
        Some(m) if m.customer_id == row.customer_id => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            "forbidden: API key cannot read another merchant's credentials".into(),
        )),
    }
}

/// `PUT /credentials` — upsert one env (`test` or `live`) for a customer (admin).
/// Response redacts `zainboxCode` unless `?revealZainboxCode=true`.
/// Mints a merchant `sk_…` when missing and returns plaintext once.
pub async fn upsert(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Query(q): Query<RevealQuery>,
    Json(body): Json<UpsertCredentialRequest>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    let public_key = body.public_key.trim();
    let zainbox_code = body.zainbox_code.trim();
    if public_key.is_empty() || zainbox_code.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "publicKey and zainboxCode are required".into(),
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
            zainbox_code,
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    .map_err(map_db)?;

    let (row, secret) =
        merchant_provision::ensure_secret_key(&state.pool, &row, &state.config.gateway_api_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CredentialView::from_row_with_secret(
        row,
        q.reveal_zainbox_code,
        secret,
    )))
}

/// `GET /credentials?customerId=&environment=&revealZainboxCode=`
pub async fn list(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListCredentialsQuery>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    let environment = match q.environment.as_deref() {
        Some(env) => Some(normalize_environment(env)?),
        None => None,
    };

    let customer_id = if let Some(m) = auth.merchant() {
        // Merchants may only list their own credentials.
        if let Some(requested) = q.customer_id {
            if requested != m.customer_id {
                return Err((
                    StatusCode::FORBIDDEN,
                    "forbidden: API key cannot list another merchant's credentials".into(),
                ));
            }
        }
        Some(m.customer_id)
    } else {
        q.customer_id
    };

    let rows = db::credentials::list(&state.pool, customer_id, environment)
        .await
        .map_err(map_db)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CredentialView::from_row(r, q.reveal_zainbox_code))
            .collect(),
    ))
}

/// `GET /credentials/by-customer/{customer_id}?revealZainboxCode=`
pub async fn by_customer(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(customer_id): Path<i64>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<Vec<CredentialView>>, (StatusCode, String)> {
    if let Some(m) = auth.merchant() {
        if m.customer_id != customer_id {
            return Err((
                StatusCode::FORBIDDEN,
                "forbidden: API key cannot read another merchant's credentials".into(),
            ));
        }
    }
    let rows = db::credentials::list_for_customer(&state.pool, customer_id)
        .await
        .map_err(map_db)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CredentialView::from_row(r, q.reveal_zainbox_code))
            .collect(),
    ))
}

/// `GET /credentials/by-customer/{customer_id}/{environment}?revealZainboxCode=`
pub async fn by_customer_env(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path((customer_id, environment)): Path<(i64, String)>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    if let Some(m) = auth.merchant() {
        if m.customer_id != customer_id {
            return Err((
                StatusCode::FORBIDDEN,
                "forbidden: API key cannot read another merchant's credentials".into(),
            ));
        }
    }
    let environment = normalize_environment(&environment)?;
    match db::credentials::find_for_customer_env(&state.pool, customer_id, environment).await {
        Ok(Some(row)) => Ok(Json(CredentialView::from_row(row, q.reveal_zainbox_code))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /credentials/{id}?revealZainboxCode=` — credential row UUID.
pub async fn get_one(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    match db::credentials::find_by_id(&state.pool, id).await {
        Ok(Some(row)) => {
            authorize_credential_read(&auth, &row)?;
            Ok(Json(CredentialView::from_row(row, q.reveal_zainbox_code)))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /credentials/{id}/status?revealZainboxCode=`
pub async fn update_status(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<RevealQuery>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    match db::credentials::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => Ok(Json(CredentialView::from_row(row, q.reveal_zainbox_code))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "credential not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `POST /credentials/{id}/rotate-secret` — mint a new `sk_…` (admin or owning merchant).
/// Returns plaintext `secretKey` once.
pub async fn rotate_secret(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<RevealQuery>,
) -> Result<Json<CredentialView>, (StatusCode, String)> {
    let row = db::credentials::find_by_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "credential not found".into()))?;

    authorize_credential_read(&auth, &row)?;

    let plaintext = mint_secret_key(&row.environment);
    let hash = hash_secret_key(&plaintext);
    let prefix = secret_key_prefix(&plaintext);
    let wrapped = crate::services::secret_wrap::wrap_secret(&state.config.gateway_api_key, &plaintext)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to wrap secret: {e}"),
            )
        })?;
    let row =
        db::credentials::set_secret_hash(&state.pool, row.id, &hash, &prefix, &wrapped)
            .await
            .map_err(map_db)?;

    Ok(Json(CredentialView::from_row_with_secret(
        row,
        q.reveal_zainbox_code,
        Some(plaintext),
    )))
}
