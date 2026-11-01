//! Shared payment-success side effects (receipts, split, merchant webhook).
//!
//! Used by status poll, inbound Zainpay webhook, and the pending reconciler so
//! customers always get the same enriched Koodify outcome.

use serde_json::Value;
use tracing::warn;

use crate::db;
use crate::services::{merchant_webhook, payment_split, receipt};
use crate::state::AppState;

fn payment_status_success(response: &Value) -> bool {
    let status = response
        .pointer("/data/paymentStatus")
        .or_else(|| response.get("paymentStatus"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        status.as_str(),
        "success" | "successful" | "paid" | "completed"
    )
}

/// Run idempotent post-success work and return the response enriched with
/// `data.paymentSplit` when a split exists.
pub async fn apply_side_effects(
    state: &AppState,
    txn_ref: &str,
    response: &Value,
    deposit_account: Option<&str>,
    deposit_bank: Option<&str>,
) -> Value {
    if !payment_status_success(response) {
        return response.clone();
    }

    receipt::maybe_send_on_success(state, txn_ref, response).await;
    payment_split::maybe_run_on_success(
        state,
        txn_ref,
        response,
        deposit_account,
        deposit_bank,
    )
    .await;
    let enriched = payment_split::enrich_response(&state.pool, txn_ref, response.clone()).await;
    merchant_webhook::maybe_deliver_on_success(state, txn_ref, &enriched).await;

    if let Err(err) = db::transactions::mark_payment_success_handled(&state.pool, txn_ref).await {
        warn!(error = %err, txn_ref, "failed to mark paymentSuccessHandled");
    }

    enriched
}

pub fn deposit_account_from(response: &Value) -> Option<&str> {
    response
        .pointer("/data/accountNumber")
        .or_else(|| response.pointer("/data/bankAccount"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub fn deposit_bank_from(response: &Value) -> Option<&str> {
    response
        .pointer("/data/bankCode")
        .or_else(|| response.pointer("/data/beneficiaryBankCode"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}
