//! Shared payment-success side effects (receipts, split, merchant webhook).
//!
//! Used by status poll, inbound Zainpay webhook, and the pending reconciler so
//! customers always get the same enriched Koodify outcome.

use serde_json::{json, Value};
use tracing::{info, warn};

use crate::db;
use crate::services::activity::{self, SuccessSource};
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

fn zainbox_from(response: &Value) -> Option<&str> {
    response
        .pointer("/data/zainboxCode")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Run idempotent post-success work and return the response enriched with
/// `data.paymentSplit` when a split exists.
///
/// `source` is persisted to `activity_events` so ops can see whether Path 1
/// (Zainpay webhook), status poll, or Path 2 (reconciler) triggered delivery.
pub async fn apply_side_effects(
    state: &AppState,
    txn_ref: &str,
    response: &Value,
    deposit_account: Option<&str>,
    deposit_bank: Option<&str>,
    source: SuccessSource,
) -> Value {
    if !payment_status_success(response) {
        return response.clone();
    }

    let zainbox = zainbox_from(response);
    info!(
        txn_ref,
        source = source.as_str(),
        "payment success side effects via {}",
        source.label()
    );
    activity::record_success_side_effects(
        state,
        source,
        txn_ref,
        zainbox,
        json!({
            "depositAccount": deposit_account,
            "depositBank": deposit_bank,
        }),
    )
    .await;

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
    merchant_webhook::maybe_deliver_on_success(state, txn_ref, &enriched, source).await;

    if let Err(err) = db::transactions::mark_payment_success_handled(&state.pool, txn_ref).await {
        warn!(error = %err, txn_ref, "failed to mark paymentSuccessHandled");
    }

    activity::record(
        state,
        "payment.success.side_effects.done",
        source.as_str(),
        Some(txn_ref),
        zainbox,
        format!(
            "Success side effects finished via {} for {txn_ref}",
            source.label()
        ),
        json!({}),
    )
    .await;

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
