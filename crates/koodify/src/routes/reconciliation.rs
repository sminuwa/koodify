use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::auth::AdminAuth;
use crate::routes::helpers::{map_upstream, JsonResult};
use crate::services::payer;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankDepositQuery {
    pub bank_type: String,
    pub verification_type: String,
    pub account_number: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardPaymentQuery {
    pub txn_ref: String,
}

pub async fn verify_deposit(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    let response = state
        .zainpay
        .reconciliation()
        .verify_deposit(&txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    let response = payer::enrich_response_from_db(&state.pool, &txn_ref, response).await;
    Ok(Json(crate::services::payment_status::enrich(response)))
}

pub async fn reconcile_bank_deposit(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Query(q): Query<BankDepositQuery>,
) -> JsonResult {
    state
        .zainpay
        .reconciliation()
        .reconcile_bank_deposit(
            &q.bank_type,
            &q.verification_type,
            &q.account_number,
            q.session_id.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn reconcile_card_payment(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Query(q): Query<CardPaymentQuery>,
) -> JsonResult {
    let response = state
        .zainpay
        .reconciliation()
        .reconcile_card_payment(&q.txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    Ok(Json(
        payer::enrich_response_from_db(&state.pool, &q.txn_ref, response).await,
    ))
}
