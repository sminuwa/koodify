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
    /// Optional for platform admin; ignored for merchant `sk_…` (auto-scoped).
    pub zainbox_code: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxCodeQuery {
    pub zainbox_code: Option<String>,
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
    state
        .zainpay
        .composite()
        .zainbox_wallet_view(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/zainbox/{zainboxCode}/transactions` — transactions + deposit summary.
/// Alias: `/api/v1/zainbox/{zainboxCode}/activity` (deprecated).
pub async fn zainbox_activity(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<DateRangeQuery>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
    state
        .zainpay
        .composite()
        .subscriptions_bundle(&code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/va/{accountNumber}` — balance + transactions.
/// Alias: `/api/v1/virtual-account/{accountNumber}` (deprecated).
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

/// `GET /api/v1/wallet` — merchant `sk_…` auto-scopes to their zainbox; admin needs `?zainboxCode=`.
pub async fn merchant_wallet(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ZainboxCodeQuery>,
) -> JsonResult {
    let code = auth.resolve_zainbox(q.zainbox_code.as_deref())?;
    state
        .zainpay
        .composite()
        .zainbox_wallet_view(code)
        .await
        .map_err(map_upstream)
        .and_then(to_json)
}

/// `GET /api/v1/transactions` — summary + transaction list.
///
/// Merchant `sk_…` is scoped to their zainbox (same data as `/zainbox/{code}/transactions`).
/// Platform admin uses the merchant-wide Zainpay summary unless `?zainboxCode=` is set.
/// Aliases (deprecated): `/api/v1/merchant/transactions`, `/api/v1/merchant/activity`.
pub async fn merchant_activity(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<DateRangeQuery>,
) -> JsonResult {
    // Merchant sk_… (or admin with ?zainboxCode=) → zainbox-scoped activity.
    if auth.merchant().is_some()
        || q.zainbox_code
            .as_deref()
            .map(str::trim)
            .is_some_and(|s| !s.is_empty())
    {
        let code = auth.resolve_zainbox(q.zainbox_code.as_deref())?;
        return state
            .zainpay
            .composite()
            .zainbox_activity(
                code,
                q.count.as_deref(),
                q.txn_type.as_deref(),
                q.account_number.as_deref(),
                q.payment_channel.as_deref(),
                q.date_from.as_deref(),
                q.date_to.as_deref(),
            )
            .await
            .map_err(map_upstream)
            .and_then(to_json);
    }

    auth.require_admin()?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> JsonResult {
    auth.require_admin()?;
    use crate::services::idempotency;

    let idem_key = idempotency::extract_key(&headers);
    let claim = if let Some(ref key) = idem_key {
        let hash = idempotency::transfer_request_hash(&body)
            .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e))?;
        match idempotency::begin(&state, idempotency::SCOPE_TRANSFER_COMPOSITE, key, &hash).await {
            idempotency::BeginOutcome::Proceed => Some(key.clone()),
            idempotency::BeginOutcome::Replay(body) => return Ok(Json(body)),
            idempotency::BeginOutcome::Conflict => return Err(idempotency::conflict_error()),
            idempotency::BeginOutcome::InFlight => return Err(idempotency::in_flight_error()),
        }
    } else {
        None
    };

    let (payload, metadata) = proxy::split_body(body);
    let req: FundsTransferRequest = serde_json::from_value(payload).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("invalid transfer body: {e}"),
        )
    })?;
    let txn_ref = req.txn_ref.clone();
    let response = match state.zainpay.composite().transfer_and_verify(&req).await {
        Ok(r) => r,
        Err(err) => {
            if let Some(key) = claim.as_deref() {
                idempotency::release(&state, idempotency::SCOPE_TRANSFER_COMPOSITE, key).await;
            }
            return Err(map_upstream(err));
        }
    };
    let mut json = serde_json::to_value(&response).unwrap_or(Value::Null);
    if response.is_success() {
        json = proxy::merge_metadata_into_response(json, metadata);
    }
    json = crate::services::api_error::enrich(json);
    if let Some(key) = claim.as_deref() {
        idempotency::complete(
            &state,
            idempotency::SCOPE_TRANSFER_COMPOSITE,
            key,
            &json,
            Some(txn_ref.as_str()),
        )
        .await;
    }
    Ok(Json(json))
}

/// `GET /api/v1/dva/verify/{txnRef}` — dynamic status + deposit verify.
/// Alias: `/api/v1/deposit/{txnRef}` (deprecated).
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
    let json = crate::services::payer::enrich_response_from_db(&state.pool, &txn_ref, json).await;
    Ok(Json(crate::services::payment_status::enrich(json)))
}

/// `POST /api/v1/settlement` — create then history
pub async fn settlement_create_with_history(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<SettlementHistoryQuery>,
    Json(body): Json<Value>,
) -> JsonResult {
    auth.require_admin()?;
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
