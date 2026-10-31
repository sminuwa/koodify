use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use client::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};
use tracing::warn;

use crate::auth::{ApiKeyAuth, PublicApiKeyAuth};
use crate::db;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::{payer, proxy};
use crate::state::AppState;

/// Checkout requires publishable key + an active seeded `zainboxCode`.
async fn require_checkout_zainbox(
    pool: &sqlx::PgPool,
    body: &Value,
) -> Result<String, (StatusCode, String)> {
    let code = body
        .get("zainboxCode")
        .or_else(|| body.get("zainbox_code"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "zainboxCode is required for checkout".into(),
            )
        })?;

    match db::zainboxes::get_by_code(pool, code).await {
        Ok(Some(z)) if z.is_active => Ok(code.to_string()),
        Ok(Some(_)) => Err((
            StatusCode::FORBIDDEN,
            format!("zainboxCode `{code}` is inactive"),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!(
                "unknown zainboxCode `{code}` — run the seeder so checkout can resolve it"
            ),
        )),
        Err(err) => {
            warn!(error = %err, "zainbox lookup failed");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to verify zainboxCode".into(),
            ))
        }
    }
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

/// InlineJS / bank-transfer checkout — publishable key + zainboxCode.
/// Logs payer against `txnRef`.
pub async fn initialize_payment_inline(
    _auth: PublicApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let _zainbox = require_checkout_zainbox(&state.pool, &body).await?;
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

/// Continues a checkout session (`e` from initialize). Publishable key.
pub async fn generate_temporary_virtual_account(
    _auth: PublicApiKeyAuth,
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

/// Status poll for checkout (`txnRef`). Publishable key.
pub async fn temporary_virtual_account_payment_status(
    _auth: PublicApiKeyAuth,
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
