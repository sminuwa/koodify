use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AdminAuth;
use crate::db;
use crate::db::kyc::UpsertKyc;
use crate::db::models::KycProfile;
use crate::services::merchant_provision;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertKycRequest {
    /// Integer `customers.id` (1, 2, 3, …).
    pub customer_id: i64,
    pub title: String,
    pub first_name: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    pub surname: String,
    pub email: String,
    pub mobile_number: String,
    pub bvn: String,
    pub dob: String,
    pub gender: String,
    pub address: String,
    pub state: String,
    #[serde(default)]
    pub bank_type: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    /// Use `pending` while under review; set `approved` to provision the merchant zainbox.
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListKycQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn after_kyc_write(
    state: &AppState,
    kyc: KycProfile,
) -> Result<Json<Value>, (StatusCode, String)> {
    let provision = if db::kyc::is_approved_status(&kyc.status) {
        // Keep portal compliance tiles in sync with the KYC decision.
        let _ = db::compliance::set_all_sections_status(&state.pool, kyc.customer_id, "approved")
            .await
            .map_err(map_db)?;

        match merchant_provision::maybe_provision_after_kyc(
            &state.pool,
            &state.zainpay,
            &state.config,
            &kyc,
        )
        .await
        {
            Ok(Some(result)) => Some(json!({
                "zainboxCode": result.zainbox.zainbox_code,
                "publicKey": result.public_key,
                "createdNewZainbox": result.created_new_zainbox,
                // Secrets are merchant-issued from Credentials — not returned on approve.
                "secretsReadyToIssue": true,
                "mintedSecrets": result.minted_secrets.iter().map(|s| json!({
                    "environment": s.environment,
                    "secretKey": s.secret_key,
                    "secretKeyPrefix": s.prefix,
                })).collect::<Vec<_>>(),
            })),
            Ok(None) => None,
            Err(err) => {
                // KYC row is already saved/approved — surface provision failure for retry.
                return Err((
                    StatusCode::BAD_GATEWAY,
                    format!(
                        "KYC saved as approved but zainbox provision failed: {err}. \
                         Retry POST /customers/{}/provision",
                        kyc.customer_id
                    ),
                ));
            }
        }
    } else {
        None
    };

    let mut body = serde_json::to_value(&kyc).unwrap_or(Value::Null);
    if let Some(obj) = body.as_object_mut() {
        if let Some(p) = provision {
            obj.insert("provision".into(), p);
        }
    }
    Ok(Json(body))
}

/// `PUT /kyc` — create or replace KYC for a customer.
///
/// When `status` is `approved`, Koodify creates (or syncs) a zainbox named after
/// the customer's `displayName`, assigns a publishable `publicKey`, and mirrors
/// credentials. Existing integrations that use other statuses are unchanged.
pub async fn upsert(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<UpsertKycRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if body.bvn.trim().is_empty() || body.email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "bvn and email are required".into()));
    }

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

    let kyc = db::kyc::upsert(
        &state.pool,
        UpsertKyc {
            customer_id: body.customer_id,
            title: body.title.trim(),
            first_name: body.first_name.trim(),
            middle_name: body.middle_name.as_deref().map(str::trim),
            surname: body.surname.trim(),
            email: body.email.trim(),
            mobile_number: body.mobile_number.trim(),
            bvn: body.bvn.trim(),
            dob: body.dob.trim(),
            gender: body.gender.trim(),
            address: body.address.trim(),
            state: body.state.trim(),
            bank_type: body.bank_type.as_deref().map(str::trim),
            zainbox_code: body.zainbox_code.as_deref().map(str::trim),
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    .map_err(map_db)?;

    after_kyc_write(&state, kyc).await
}

/// `GET /kyc?status=`
pub async fn list(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Query(q): Query<ListKycQuery>,
) -> Result<Json<Vec<KycProfile>>, (StatusCode, String)> {
    db::kyc::list(&state.pool, q.status.as_deref())
        .await
        .map(Json)
        .map_err(map_db)
}

/// `GET /kyc/by-customer/{customer_id}`
pub async fn by_customer(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(customer_id): Path<i64>,
) -> Result<Json<KycProfile>, (StatusCode, String)> {
    match db::kyc::find_by_customer_id(&state.pool, customer_id).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "kyc profile not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /kyc/{id}` — KYC row UUID.
pub async fn get_one(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<KycProfile>, (StatusCode, String)> {
    match db::kyc::find_by_id(&state.pool, id).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "kyc profile not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /kyc/{id}/status`
///
/// Setting `status` to `approved` provisions the merchant zainbox + public key.
pub async fn update_status(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    let kyc = match db::kyc::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => row,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "kyc profile not found".into())),
        Err(err) => return Err(map_db(err)),
    };
    after_kyc_write(&state, kyc).await
}
