//! Composite convenience routes — each calls two Zainpay endpoints via the SDK.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use client::{CreateSettlementRequest, FundsTransferRequest};

use crate::auth::ApiKeyAuth;
use crate::routes::helpers::{map_upstream, JsonResult};
use crate::services::proxy;
use crate::state::AppState;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DateRangeQuery {
    pub count: Option<String>,
    pub txn_type: Option<String>,
    pub account_number: Option<String>,
    pub payment_channel: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameResolveQuery {
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettlementHistoryQuery {
    pub count: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,
}

fn to_json<T: serde::Serialize>(r: T) -> JsonResult {
    Ok(Json(serde_json::to_value(r).unwrap_or(Value::Null)))
}

/// `GET /api/v1/zainbox/{zainboxCode}` — profile + balances
pub async fn zainbox_snapshot(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .zainbox_snapshot(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/wallet` — virtual accounts + balances
pub async fn zainbox_wallet_view(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .zainbox_wallet_view(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/activity` — transactions + deposit summary
pub async fn zainbox_activity(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<DateRangeQuery>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .zainbox_activity(
            &code,
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.account_number.as_deref(),
            q.payment_channel.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
        )
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/settlement` — settlement GET + internal account
pub async fn settlement_context(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .settlement_context(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/billing` — profile + billing config
pub async fn zainbox_profile_billing(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .zainbox_profile_billing(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/subscriptions` — all + active
pub async fn subscriptions_bundle(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .subscriptions_bundle(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/virtual-account/{accountNumber}` — balance + transactions
pub async fn virtual_account_wallet(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
    Query(q): Query<DateRangeQuery>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .virtual_account_wallet(
            &account_number,
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.payment_channel.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
        )
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/bank/resolve` — bank list + name enquiry
pub async fn name_resolve(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<NameResolveQuery>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .name_resolve(&q.bank_code, &q.account_number)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/merchant/activity` — summary + transaction list
pub async fn merchant_activity(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .merchant_activity(
            q.date_from.as_deref(),
            q.date_to.as_deref(),
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.account_number.as_deref(),
            q.payment_channel.as_deref(),
        )
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/payout/prep` — network status + name enquiry
pub async fn payout_prep(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<NameResolveQuery>,
) -> JsonResult {
    state
        .zainpay
        .composite()
        .payout_prep(&q.bank_code, &q.account_number)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `POST /api/v1/transfer` — transfer then verify
pub async fn transfer_and_verify(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let (payload, metadata) = proxy::split_body(body);
    let req: FundsTransferRequest = serde_json::from_value(payload).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("invalid transfer body: {e}"),
        )
    })?;
    let response = state
        .zainpay
        .composite()
        .transfer_and_verify(&req)
        .await
        .map_err(map_upstream)?;
    let mut json = serde_json::to_value(&response).unwrap_or(Value::Null);
    if response.is_success() {
        json = proxy::merge_metadata_into_response(json, metadata);
    }
    Ok(Json(json))
}

/// `GET /api/v1/deposit/{txnRef}` — dynamic status + deposit verify
pub async fn dynamic_deposit_check(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    let response = state
        .zainpay
        .composite()
        .dynamic_deposit_check(&txn_ref)
        .await
        .map_err(map_upstream)?;
    let json = serde_json::to_value(response).unwrap_or(Value::Null);
    Ok(Json(
        crate::services::payer::enrich_response_from_db(&state.pool, &txn_ref, json).await,
    ))
}

/// `POST /api/v1/settlement` — create then history
pub async fn settlement_create_with_history(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<SettlementHistoryQuery>,
    Json(body): Json<Value>,
) -> JsonResult {
    let (payload, metadata) = proxy::split_body(body);
    let req: CreateSettlementRequest = serde_json::from_value(payload).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("invalid settlement body: {e}"),
        )
    })?;
    let response = state
        .zainpay
        .composite()
        .settlement_create_with_history(
            &req,
            q.count.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
            q.status.as_deref(),
        )
        .await
        .map_err(map_upstream)?;
    let mut json = serde_json::to_value(&response).unwrap_or(Value::Null);
    if response.is_success() {
        json = proxy::merge_metadata_into_response(json, metadata);
    }
    Ok(Json(json))
}
