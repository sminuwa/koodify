use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::db::customers::{CreateCustomer, PatchCustomer, UpdateCustomer};
use crate::db::models::{Credential, Customer, KycProfile};
use crate::db::zainboxes::Zainbox;
use crate::services::merchant_provision;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomerRequest {
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    /// Use `pending` for self-register; seed/admin may keep `active`.
    #[serde(default = "default_status")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerRequest {
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default = "default_status")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchCustomerRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub status: Option<String>,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCustomersQuery {
    pub status: Option<String>,
    /// Lookup single customer by email (login prep). When set, returns one detail or 404.
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailQuery {
    /// When true, include credential `zainboxCode` (private/server id).
    #[serde(default, alias = "revealPrivateKey")]
    pub reveal_zainbox_code: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxSummary {
    pub zainbox_code: String,
    pub name: String,
    pub public_key: Option<String>,
    pub is_active: bool,
    pub email_notification: Option<String>,
    pub callback_url: Option<String>,
    pub tags: Option<String>,
}

impl From<&Zainbox> for ZainboxSummary {
    fn from(z: &Zainbox) -> Self {
        Self {
            zainbox_code: z.zainbox_code.clone(),
            name: z.name.clone(),
            public_key: z.public_key.clone(),
            is_active: z.is_active,
            email_notification: z.email_notification.clone(),
            callback_url: z.callback_url.clone(),
            tags: z.tags.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KycSummary {
    pub id: Uuid,
    pub status: String,
    pub zainbox_code: Option<String>,
    pub email: String,
    pub first_name: String,
    pub surname: String,
}

impl From<&KycProfile> for KycSummary {
    fn from(k: &KycProfile) -> Self {
        Self {
            id: k.id,
            status: k.status.clone(),
            zainbox_code: k.zainbox_code.clone(),
            email: k.email.clone(),
            first_name: k.first_name.clone(),
            surname: k.surname.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSummary {
    pub id: Uuid,
    pub environment: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zainbox_code: Option<String>,
    pub zainbox_code_set: bool,
    pub status: String,
}

impl CredentialSummary {
    fn from_row(row: &Credential, reveal: bool) -> Self {
        let zainbox_code_set = !row.zainbox_code.is_empty();
        Self {
            id: row.id,
            environment: row.environment.clone(),
            public_key: row.public_key.clone(),
            zainbox_code: if reveal {
                Some(row.zainbox_code.clone())
            } else {
                None
            },
            zainbox_code_set,
            status: row.status.clone(),
        }
    }
}

/// Dashboard-ready customer payload (register / login / merchant UI).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDetail {
    #[serde(flatten)]
    pub customer: Customer,
    /// True when `password_hash` is set (login-ready); hash itself is never exposed.
    pub has_password: bool,
    pub zainbox: Option<ZainboxSummary>,
    /// Publishable key for inline checkout (from assigned zainbox).
    pub public_key: Option<String>,
    pub kyc: Option<KycSummary>,
    pub credentials: Vec<CredentialSummary>,
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// Soft uniqueness for self-register: reject if another customer already uses this email.
async fn ensure_email_available(
    state: &AppState,
    email: Option<&str>,
    except_customer_id: Option<i64>,
) -> Result<(), (StatusCode, String)> {
    let Some(email) = email.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    if let Some(existing) = db::customers::find_by_email(&state.pool, email)
        .await
        .map_err(map_db)?
    {
        if except_customer_id != Some(existing.id) {
            return Err((StatusCode::CONFLICT, "email already registered".into()));
        }
    }
    Ok(())
}

async fn load_detail(
    state: &AppState,
    customer: Customer,
    reveal: bool,
) -> Result<CustomerDetail, (StatusCode, String)> {
    let has_password = db::customers::has_password(&state.pool, customer.id)
        .await
        .map_err(map_db)?;

    let zainbox = if let Some(code) = customer.zainbox_code.as_deref() {
        db::zainboxes::get_by_code(&state.pool, code)
            .await
            .map_err(map_db)?
            .map(|z| ZainboxSummary::from(&z))
    } else {
        None
    };

    let public_key = zainbox
        .as_ref()
        .and_then(|z| z.public_key.clone())
        .filter(|s| !s.is_empty());

    let kyc = db::kyc::find_by_customer_id(&state.pool, customer.id)
        .await
        .map_err(map_db)?
        .map(|k| KycSummary::from(&k));

    let credentials = db::credentials::list_for_customer(&state.pool, customer.id)
        .await
        .map_err(map_db)?
        .iter()
        .map(|c| CredentialSummary::from_row(c, reveal))
        .collect();

    Ok(CustomerDetail {
        customer,
        has_password,
        zainbox,
        public_key,
        kyc,
        credentials,
    })
}

/// `POST /customers` — create a customer (`id` auto-assigned: 1, 2, 3, …).
///
/// Self-register style: omit `zainboxCode`, use `status: "pending"`, then submit KYC.
/// After KYC is `approved`, a zainbox + public key are provisioned automatically.
pub async fn create(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.display_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "displayName is required".into()));
    }

    let email = body.email.as_deref().map(str::trim).filter(|s| !s.is_empty());
    // Enforce email uniqueness on API create (seed path bypasses this and may share emails).
    ensure_email_available(&state, email, None).await?;

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    db::customers::create(
        &state.pool,
        CreateCustomer {
            display_name: body.display_name.trim(),
            email,
            mobile_number: body.mobile_number.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            zainbox_code: body.zainbox_code.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    .map(Json)
    .map_err(map_db)
}

/// `PUT /customers/{id}`
pub async fn update(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.display_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "displayName is required".into()));
    }

    let email = body.email.as_deref().map(str::trim).filter(|s| !s.is_empty());
    ensure_email_available(&state, email, Some(id)).await?;

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    match db::customers::update(
        &state.pool,
        id,
        UpdateCustomer {
            display_name: body.display_name.trim(),
            email,
            mobile_number: body.mobile_number.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            zainbox_code: body.zainbox_code.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /customers/{id}` — partial update for merchant profile edits.
pub async fn patch(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body
        .display_name
        .as_deref()
        .is_some_and(|s| s.trim().is_empty())
    {
        return Err((StatusCode::BAD_REQUEST, "displayName cannot be empty".into()));
    }

    let email = body
        .email
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if body.email.is_some() {
        ensure_email_available(&state, email, Some(id)).await?;
    }

    match db::customers::patch(
        &state.pool,
        id,
        PatchCustomer {
            display_name: body.display_name.as_deref().map(str::trim),
            email,
            mobile_number: body
                .mobile_number
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            zainbox_code: body
                .zainbox_code
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            metadata: body.metadata.as_ref(),
            status: body.status.as_deref().map(str::trim),
        },
    )
    .await
    {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /customers?status=` or `GET /customers?email=` (single detail for login prep).
pub async fn list(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListCustomersQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if let Some(email) = q.email.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let customer = db::customers::find_by_email(&state.pool, email)
            .await
            .map_err(map_db)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;
        let detail = load_detail(&state, customer, false).await?;
        return Ok(Json(serde_json::to_value(detail).unwrap_or(Value::Null)));
    }

    let rows = db::customers::list(&state.pool, q.status.as_deref())
        .await
        .map_err(map_db)?;
    Ok(Json(serde_json::to_value(rows).unwrap_or(Value::Null)))
}

/// `GET /customers/{id}` — customer + zainbox / public key / KYC / credentials (dashboard).
pub async fn get_one(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<DetailQuery>,
) -> Result<Json<CustomerDetail>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;
    load_detail(&state, customer, q.reveal_zainbox_code)
        .await
        .map(Json)
}

/// `GET /customers/by-uuid/{uuid}`
pub async fn by_uuid(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Query(q): Query<DetailQuery>,
) -> Result<Json<CustomerDetail>, (StatusCode, String)> {
    let customer = db::customers::find_by_uuid(&state.pool, uuid)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;
    load_detail(&state, customer, q.reveal_zainbox_code)
        .await
        .map(Json)
}

/// `GET /customers/{id}/zainbox` — assigned zainbox + publishable public key.
pub async fn zainbox(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;

    let Some(code) = customer.zainbox_code.as_deref().filter(|s| !s.is_empty()) else {
        return Err((
            StatusCode::NOT_FOUND,
            "customer has no assigned zainbox yet (approve KYC to provision)".into(),
        ));
    };

    let zb = db::zainboxes::get_by_code(&state.pool, code)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "zainbox not found in local cache".into()))?;

    Ok(Json(json!({
        "customerId": customer.id,
        "zainbox": ZainboxSummary::from(&zb),
        "publicKey": zb.public_key,
    })))
}

/// `POST /customers/{id}/provision` — create/sync zainbox + keys (admin retry / manual).
pub async fn provision(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let result = merchant_provision::ensure_merchant_zainbox(
        &state.pool,
        &state.zainpay,
        &state.config,
        id,
    )
    .await
    .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;

    Ok(Json(json!({
        "code": "00",
        "description": if result.created_new_zainbox {
            "zainbox created and credentials synced"
        } else {
            "zainbox already linked; public key and credentials synced"
        },
        "data": {
            "customerId": result.customer.id,
            "zainboxCode": result.zainbox.zainbox_code,
            "publicKey": result.public_key,
            "createdNewZainbox": result.created_new_zainbox,
            "credentials": result.credentials.iter().map(|c| json!({
                "environment": c.environment,
                "publicKey": c.public_key,
                "status": c.status,
            })).collect::<Vec<_>>(),
        }
    })))
}

/// `PATCH /customers/{id}/status`
pub async fn update_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    match db::customers::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}
