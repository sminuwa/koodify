use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use client::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};

use crate::auth::ApiKeyAuth;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::{payer, proxy};
use crate::state::AppState;

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

/// InlineJS / bank-transfer checkout session — logs payer against `txnRef`.
pub async fn initialize_payment_inline(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
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

pub async fn generate_temporary_virtual_account(
    _auth: ApiKeyAuth,
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

pub async fn temporary_virtual_account_payment_status(
    _auth: ApiKeyAuth,
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
