//! Fire-and-forget activity / audit logging.

use serde_json::{json, Value};
use tracing::warn;

use crate::db;
use crate::state::AppState;

/// Who triggered payment-success side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuccessSource {
    /// Inbound Zainpay `deposit.success` webhook (Path 1).
    ZainpayWebhook,
    /// Browser / client status poll.
    StatusPoll,
    /// Background pending-checkout reconciler (Path 2).
    Reconciler,
}

impl SuccessSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZainpayWebhook => "zainpay_webhook",
            Self::StatusPoll => "status_poll",
            Self::Reconciler => "reconciler",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ZainpayWebhook => "Zainpay inbound webhook (Path 1)",
            Self::StatusPoll => "checkout status poll",
            Self::Reconciler => "Koodify reconciler (Path 2)",
        }
    }
}

pub async fn record(
    state: &AppState,
    event: &str,
    source: &str,
    txn_ref: Option<&str>,
    zainbox_code: Option<&str>,
    summary: impl Into<String>,
    payload: Value,
) {
    let summary = summary.into();
    if let Err(err) = db::activity_events::insert(
        &state.pool,
        event,
        source,
        txn_ref,
        zainbox_code,
        Some(summary.as_str()),
        &payload,
    )
    .await
    {
        warn!(
            error = %err,
            event,
            source,
            txn_ref = txn_ref.unwrap_or(""),
            "failed to persist activity event"
        );
    }
}

pub async fn record_success_side_effects(
    state: &AppState,
    source: SuccessSource,
    txn_ref: &str,
    zainbox_code: Option<&str>,
    extra: Value,
) {
    let mut payload = extra;
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("trigger".into(), json!(source.as_str()));
        obj.insert("triggerLabel".into(), json!(source.label()));
    }
    record(
        state,
        "payment.success.side_effects",
        source.as_str(),
        Some(txn_ref),
        zainbox_code,
        format!(
            "Success side effects started via {} for {txn_ref}",
            source.label()
        ),
        payload,
    )
    .await;
}
