use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;
use client::FundsTransferRequest;

use crate::auth::ApiKeyAuth;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::{payer, proxy};
use crate::state::AppState;

pub async fn funds_transfer(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<FundsTransferRequest, _>(
        &state,
        "transfer.funds",
        body,
        |req| async move { zainpay.transfers().funds_transfer(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
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
    Ok(Json(
        payer::enrich_response_from_db(&state.pool, &txn_ref, response).await,
    ))
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
