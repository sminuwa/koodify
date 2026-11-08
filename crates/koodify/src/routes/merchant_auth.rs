//! Merchant dashboard auth — register / login / me / KYC / issue secret.
//!
//! Public (no API key):
//! - `POST /api/v1/auth/register`
//! - `POST /api/v1/auth/login`
//!
//! Session (`Authorization: Bearer mt_…`):
//! - `GET  /api/v1/auth/me`
//! - `PUT  /api/v1/auth/kyc`
//! - `POST /api/v1/auth/credentials/{environment}/issue-secret`

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db;
use crate::db::customers::CreateCustomer;
use crate::db::kyc::UpsertKyc;
use crate::routes::customers::load_detail;
use crate::services::merchant_provision;
use crate::services::merchant_session::MerchantSession;
use crate::services::password::{hash_password, validate_password_strength, verify_password};
use crate::state::AppState;

fn ok(data: Value) -> Json<Value> {
    Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": data,
    }))
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub display_name: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub mobile_number: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitKycRequest {
    pub title: String,
    pub first_name: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    pub surname: String,
    pub email: String,
    pub mobile_number: String,
    /// National identity number (NIN). Accepted as `nin` or legacy `bvn`.
    #[serde(alias = "nin")]
    pub bvn: String,
    pub dob: String,
    pub gender: String,
    pub address: String,
    pub state: String,
    #[serde(default)]
    pub bank_type: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

/// Soft uniqueness for self-register.
async fn ensure_email_available(
    state: &AppState,
    email: &str,
) -> Result<(), (StatusCode, String)> {
    if db::customers::find_by_email(&state.pool, email)
        .await
        .map_err(map_db)?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, "email already registered".into()));
    }
    Ok(())
}

async fn session_payload(
    state: &AppState,
    customer_id: i64,
    token: String,
    extra: Option<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, customer_id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;
    let detail = load_detail(state, customer, false).await?;
    let mut data = serde_json::to_value(&detail).unwrap_or(Value::Null);
    if let Some(obj) = data.as_object_mut() {
        obj.insert("token".into(), Value::String(token));
        if let Some(extra) = extra {
            if let Some(extra_obj) = extra.as_object() {
                for (k, v) in extra_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }
    Ok(ok(data))
}

/// `POST /api/v1/auth/register` — create pending merchant + password, return session.
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let display_name = body.display_name.trim();
    let email = body.email.trim().to_ascii_lowercase();
    if display_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "displayName is required".into()));
    }
    if email.is_empty() || !email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "valid email is required".into()));
    }
    validate_password_strength(body.password.trim())?;
    ensure_email_available(&state, &email).await?;

    let password_hash = hash_password(body.password.trim())?;
    let customer = db::customers::create(
        &state.pool,
        CreateCustomer {
            display_name,
            email: Some(email.as_str()),
            mobile_number: body
                .mobile_number
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            zainbox_code: None,
            metadata: &json!({ "source": "merchant-register" }),
            status: "pending",
        },
    )
    .await
    .map_err(map_db)?;

    db::customers::set_password_hash(&state.pool, customer.id, &password_hash)
        .await
        .map_err(map_db)?;

    let token = MerchantSession::issue(customer.id, &state.config.gateway_api_key);
    session_payload(&state, customer.id, token, None).await
}

/// `POST /api/v1/auth/login`
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let email = body.email.trim().to_ascii_lowercase();
    if email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "email is required".into()));
    }
    let customer = db::customers::find_by_email(&state.pool, &email)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "invalid email or password".into()))?;

    let hash = customer
        .password_hash
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "password not set for this account".into(),
            )
        })?;

    if !verify_password(body.password.trim(), hash) {
        return Err((StatusCode::UNAUTHORIZED, "invalid email or password".into()));
    }

    let token = MerchantSession::issue(customer.id, &state.config.gateway_api_key);
    session_payload(&state, customer.id, token, None).await
}

/// `GET /api/v1/auth/me`
pub async fn me(
    session: MerchantSession,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, session.customer_id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;
    let detail = load_detail(&state, customer, false).await?;
    Ok(ok(serde_json::to_value(detail).unwrap_or(Value::Null)))
}

/// `PUT /api/v1/auth/kyc` — merchant submits KYC (`pending`, or auto-approved in sandbox).
pub async fn submit_kyc(
    session: MerchantSession,
    State(state): State<AppState>,
    Json(body): Json<SubmitKycRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if body.bvn.trim().is_empty() || body.email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "bvn and email are required".into()));
    }

    let status = if state.config.merchant_kyc_auto_approve {
        "approved"
    } else {
        "pending"
    };

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    let kyc = db::kyc::upsert(
        &state.pool,
        UpsertKyc {
            customer_id: session.customer_id,
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
            zainbox_code: None,
            metadata: &metadata,
            status,
        },
    )
    .await
    .map_err(map_db)?;

    let section_data = json!({
        "title": body.title.trim(),
        "firstName": body.first_name.trim(),
        "middleName": body.middle_name.as_deref().map(str::trim),
        "surname": body.surname.trim(),
        "email": body.email.trim(),
        "mobileNumber": body.mobile_number.trim(),
        "nin": body.bvn.trim(),
        "dob": body.dob.trim(),
        "gender": body.gender.trim(),
        "address": body.address.trim(),
        "state": body.state.trim(),
    });
    crate::routes::merchant_compliance::sync_representative_section(
        &state.pool,
        session.customer_id,
        status,
        section_data,
    )
    .await?;

    let mut data = serde_json::to_value(&kyc).unwrap_or(Value::Null);

    if db::kyc::is_approved_status(&kyc.status) {
        match merchant_provision::maybe_provision_after_kyc(
            &state.pool,
            &state.zainpay,
            &state.config,
            &kyc,
        )
        .await
        {
            Ok(Some(result)) => {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert(
                        "provision".into(),
                        json!({
                            "zainboxCode": result.zainbox.zainbox_code,
                            "publicKey": result.public_key,
                            "createdNewZainbox": result.created_new_zainbox,
                            "mintedSecrets": result.minted_secrets.iter().map(|s| json!({
                                "environment": s.environment,
                                "secretKey": s.secret_key,
                                "secretKeyPrefix": s.prefix,
                            })).collect::<Vec<_>>(),
                        }),
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    format!(
                        "KYC saved as approved but zainbox provision failed: {err}. \
                         Retry later or contact support."
                    ),
                ));
            }
        }
    }

    Ok(ok(data))
}

/// `POST /api/v1/auth/credentials/{environment}/issue-secret`
///
/// Mints `sk_…` when missing (or rotates when `rotate=true` query — reserved).
/// Plaintext is returned **once**.
pub async fn issue_secret(
    session: MerchantSession,
    State(state): State<AppState>,
    Path(environment): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let env = match environment.trim().to_ascii_lowercase().as_str() {
        "test" => "test",
        "live" => "live",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "environment must be test or live".into(),
            ))
        }
    };

    let customer = db::customers::find_by_id(&state.pool, session.customer_id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "customer not found".into()))?;

    let zainbox_code = customer
        .zainbox_code
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                "complete KYC approval before issuing API secrets".into(),
            )
        })?;

    let public_key = db::zainboxes::get_by_code(&state.pool, zainbox_code)
        .await
        .map_err(map_db)?
        .and_then(|z| z.public_key)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                "zainbox has no public key yet".into(),
            )
        })?;

    let (credentials, minted) = merchant_provision::sync_credentials_from_zainbox(
        &state.pool,
        customer.id,
        zainbox_code,
        &public_key,
        &state.config.gateway_api_key,
    )
    .await
    .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()))?;

    let secret = minted
        .into_iter()
        .find(|s| s.environment == env)
        .or_else(|| {
            // Already had a secret — mint a fresh one via rotate semantics.
            None
        });

    if let Some(secret) = secret {
        return Ok(ok(json!({
            "environment": secret.environment,
            "secretKey": secret.secret_key,
            "secretKeyPrefix": secret.prefix,
            "publicKey": public_key,
            "minted": true,
        })));
    }

    // Secret already existed: rotate so the dashboard can show plaintext once.
    let row = credentials
        .into_iter()
        .find(|c| c.environment == env)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "credential row missing".into()))?;

    let plaintext = crate::auth::mint_secret_key(env);
    let hash = crate::auth::hash_secret_key(&plaintext);
    let prefix = crate::auth::secret_key_prefix(&plaintext);
    let wrapped = crate::services::secret_wrap::wrap_secret(&state.config.gateway_api_key, &plaintext)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    db::credentials::set_secret_hash(&state.pool, row.id, &hash, &prefix, &wrapped)
        .await
        .map_err(map_db)?;

    Ok(ok(json!({
        "environment": env,
        "secretKey": plaintext,
        "secretKeyPrefix": prefix,
        "publicKey": public_key,
        "minted": true,
        "rotated": true,
    })))
}
