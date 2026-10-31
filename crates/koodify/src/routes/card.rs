use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use client::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};
use tracing::warn;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::{payer, proxy};
use crate::state::AppState;

fn take_str_field(body: &mut Value, keys: &[&str]) -> Option<String> {
    let obj = body.as_object_mut()?;
    for key in keys {
        if let Some(v) = obj.remove(*key) {
            if let Some(s) = v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Resolve merchant from publishable `publicKey` → active zainbox.
/// Injects `zainboxCode` for Zainpay and strips browser-only fields.
/// Legacy: bare `zainboxCode` still accepted during migration.
async fn resolve_checkout_merchant(
    pool: &sqlx::PgPool,
    body: &mut Value,
) -> Result<String, (StatusCode, String)> {
    let public_key = take_str_field(body, &["publicKey", "public_key"]);
    let legacy_code = body
        .get("zainboxCode")
        .or_else(|| body.get("zainbox_code"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let zainbox = if let Some(pk) = public_key {
        match db::zainboxes::get_by_public_key(pool, &pk).await {
            Ok(Some(z)) => z,
            Ok(None) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "invalid or unknown publicKey".into(),
                ));
            }
            Err(err) => {
                warn!(error = %err, "publicKey lookup failed");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to verify publicKey".into(),
                ));
            }
        }
    } else if let Some(code) = legacy_code {
        match db::zainboxes::get_by_code(pool, &code).await {
            Ok(Some(z)) => z,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    format!(
                        "unknown zainboxCode `{code}` — prefer publicKey from the seeder"
                    ),
                ));
            }
            Err(err) => {
                warn!(error = %err, "zainbox lookup failed");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to verify zainboxCode".into(),
                ));
            }
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "publicKey is required for checkout".into(),
        ));
    };

    if !zainbox.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            "merchant publicKey is inactive".into(),
        ));
    }

    if let Some(obj) = body.as_object_mut() {
        obj.insert("zainboxCode".into(), json!(zainbox.code_name));
        obj.remove("zainbox_code");
        obj.remove("publicKey");
        obj.remove("public_key");
    }

    Ok(zainbox.code_name)
}

pub async fn initialize_payment(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<InitializeCardPaymentRequest, _>(
        &state,
        "card.initialize",
        body,
        |req| async move { zainpay.cards().initialize_payment(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

/// InlineJS / bank-transfer checkout — publishable `publicKey` → zainbox.
/// Logs payer against `txnRef`.
pub async fn initialize_payment_inline(
    State(state): State<AppState>,
    Json(mut body): Json<Value>,
) -> JsonResult {
    let _zainbox = resolve_checkout_merchant(&state.pool, &mut body).await?;
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<InitializeCardPaymentRequest, _>(
        &state,
        "card.initialize_inline",
        body,
        |req| async move { zainpay.cards().initialize_payment_inline(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn recurring_purchase(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<RecurringCardPurchaseRequest, _>(
        &state,
        "card.recurring_purchase",
        body,
        |req| async move { zainpay.cards().recurring_purchase(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempVaQuery {
    /// Zainpay expects a JSON-string bank key, e.g. `"gtBank"`.
    pub bank_type: String,
    pub e: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempVaStatusQuery {
    pub txn_ref: String,
}

/// Continues a checkout session (`e` from initialize). Open — session key is the gate.
pub async fn generate_temporary_virtual_account(
    State(state): State<AppState>,
    Query(q): Query<TempVaQuery>,
) -> JsonResult {
    // Accept either gtBank or "gtBank"
    let bank = q.bank_type.trim().trim_matches('"');
    state
        .zainpay
        .cards()
        .generate_temporary_virtual_account(bank, &q.e)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

/// Status poll for checkout (`txnRef`). Open — txnRef is the gate.
pub async fn temporary_virtual_account_payment_status(
    State(state): State<AppState>,
    Query(q): Query<TempVaStatusQuery>,
) -> JsonResult {
    let response = state
        .zainpay
        .cards()
        .temporary_virtual_account_payment_status(&q.txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    Ok(Json(
        payer::enrich_response_from_db(&state.pool, &q.txn_ref, response).await,
    ))
}
