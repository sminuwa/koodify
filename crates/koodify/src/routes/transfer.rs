use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use serde_json::Value;
use client::FundsTransferRequest;

use crate::auth::{AdminAuth, ApiKeyAuth};
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::{idempotency, payer, proxy};
use crate::state::AppState;

pub async fn funds_transfer(
    _auth: AdminAuth,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> JsonResult {
    let idem_key = idempotency::extract_key(&headers);
    let claim = if let Some(ref key) = idem_key {
        let hash = idempotency::transfer_request_hash(&body)
            .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e))?;
        match idempotency::begin(&state, idempotency::SCOPE_TRANSFER, key, &hash).await {
            idempotency::BeginOutcome::Proceed => Some(key.clone()),
            idempotency::BeginOutcome::Replay(body) => return Ok(Json(body)),
            idempotency::BeginOutcome::Conflict => return Err(idempotency::conflict_error()),
            idempotency::BeginOutcome::InFlight => return Err(idempotency::in_flight_error()),
        }
    } else {
        None
    };

    let zainpay = state.zainpay.clone();
    let result = proxy::mutate_json_with_metadata::<FundsTransferRequest, _>(
        &state,
        "transfer.funds",
        body,
        |req| async move { zainpay.transfers().funds_transfer(&req).await },
    )
    .await;

    match result {
        Ok(json) => {
            // proxy already runs api_error::enrich; keep stored replay enriched.
            if let Some(key) = claim.as_deref() {
                let resource = json
                    .pointer("/data/txnRef")
                    .or_else(|| json.pointer("/data/paymentRef"))
                    .and_then(|v| v.as_str());
                idempotency::complete(
                    &state,
                    idempotency::SCOPE_TRANSFER,
                    key,
                    &json,
                    resource,
                )
                .await;
            }
            Ok(Json(json))
        }
        Err(err) => {
            if let Some(key) = claim.as_deref() {
                idempotency::release(&state, idempotency::SCOPE_TRANSFER, key).await;
            }
            Err(map_proxy_error(err))
        }
    }
}

pub async fn verify(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    let response = state
        .zainpay
        .transfers()
        .verify(&txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    let response = payer::enrich_response_from_db(&state.pool, &txn_ref, response).await;
    Ok(Json(crate::services::api_error::enrich(response)))
}

pub async fn network_status(_auth: ApiKeyAuth, State(state): State<AppState>) -> JsonResult {
    state
        .zainpay
        .transfers()
        .network_status()
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}
