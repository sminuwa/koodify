//! Pending-checkout reconciler (Path 2).
//!
//! Independently of Zainpay webhook delivery, poll Zainpay for recent unpaid
//! checkouts. When status flips to success, run the same enriched side effects
//! (receipt / split / merchant webhook). Skip txns already covered by an inbound
//! `deposit.success` webhook so customers are not double-notified.

use std::time::Duration;

use serde_json::Value;
use tracing::{info, warn};

use crate::db;
use crate::services::activity::SuccessSource;
use crate::services::{payer, payment_status, payment_success};
use crate::state::AppState;

#[derive(Debug, Default, Clone)]
pub struct ReconcileStats {
    pub scanned: u32,
    pub polled: u32,
    pub succeeded: u32,
    pub errors: u32,
}

/// One reconcile pass over pending checkouts.
pub async fn run_once(state: &AppState) -> ReconcileStats {
    let mut stats = ReconcileStats::default();

    let lookback = chrono::Duration::hours(state.config.reconcile_lookback_hours as i64);
    let grace = chrono::Duration::seconds(state.config.reconcile_grace_secs as i64);
    let cooldown = chrono::Duration::seconds(state.config.reconcile_poll_cooldown_secs as i64);
    let limit = state.config.reconcile_batch_size as i64;

    let pending = match db::transactions::list_pending_checkouts(
        &state.pool,
        lookback,
        grace,
        cooldown,
        limit,
    )
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            warn!(error = %err, "reconcile: failed to list pending checkouts");
            stats.errors += 1;
            return stats;
        }
    };

    stats.scanned = pending.len() as u32;
    if pending.is_empty() {
        return stats;
    }

    info!(count = pending.len(), "reconcile: polling pending checkouts");

    for row in pending {
        let txn_ref = row.resource_key.as_str();
        if let Err(err) = db::transactions::touch_reconcile_polled(&state.pool, txn_ref).await {
            warn!(error = %err, txn_ref, "reconcile: failed to stamp poll time");
        }

        stats.polled += 1;
        let response = match state
            .zainpay
            .cards()
            .temporary_virtual_account_payment_status(txn_ref)
            .await
        {
            Ok(r) => serde_json::to_value(r).unwrap_or(Value::Null),
            Err(err) => {
                warn!(error = %err, txn_ref, "reconcile: Zainpay status poll failed");
                stats.errors += 1;
                continue;
            }
        };

        if !payment_status::is_paid(&response) {
            continue;
        }

        let enriched = payer::enrich_response_from_db(&state.pool, txn_ref, response).await;
        let deposit_account = payment_success::deposit_account_from(&enriched);
        let deposit_bank = payment_success::deposit_bank_from(&enriched);
        payment_success::apply_side_effects(
            state,
            txn_ref,
            &enriched,
            deposit_account,
            deposit_bank,
            SuccessSource::Reconciler,
        )
        .await;

        stats.succeeded += 1;
        info!(
            txn_ref,
            zainbox = ?row.zainbox_code,
            "reconcile: confirmed success — applied Koodify side effects (Path 2)"
        );
    }

    stats
}

/// Background loop. No-ops when `reconcile_enabled` is false.
pub fn spawn_loop(state: AppState) {
    if !state.config.reconcile_enabled {
        info!("reconcile: background loop disabled");
        return;
    }

    let interval = Duration::from_secs(state.config.reconcile_interval_secs.max(15));
    info!(
        ?interval,
        lookback_hours = state.config.reconcile_lookback_hours,
        batch = state.config.reconcile_batch_size,
        "reconcile: background loop starting"
    );

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // First tick completes immediately — skip so we don't stampede on boot.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let stats = run_once(&state).await;
            if stats.polled > 0 || stats.errors > 0 {
                info!(
                    scanned = stats.scanned,
                    polled = stats.polled,
                    succeeded = stats.succeeded,
                    errors = stats.errors,
                    "reconcile: pass complete"
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::services::payment_status;

    #[test]
    fn detects_success_status() {
        assert!(payment_status::is_paid(&json!({
            "data": { "paymentStatus": "success" }
        })));
        assert!(payment_status::is_paid(&json!({
            "data": { "paymentStatus": "PAID" }
        })));
        assert!(!payment_status::is_paid(&json!({
            "data": { "paymentStatus": "pending" }
        })));
    }
}
