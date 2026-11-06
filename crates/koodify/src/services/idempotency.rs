//! Idempotency-Key helper for money-moving POSTs (DX issue 07).
//!
//! Header: `Idempotency-Key` (also accepts `X-Idempotency-Key`).
//! Same key + same body → replay stored JSON. Same key + different body → 409.
//! Keys are scoped (e.g. `transfer`) and retained for [`TTL_SECS`].

use axum::http::{HeaderMap, StatusCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::db;
use crate::state::AppState;

/// How long a completed key is replayable (24h).
pub const TTL_SECS: i64 = 24 * 60 * 60;
/// Pending rows older than this may be reclaimed after a crashed attempt.
pub const STALE_PENDING_SECS: i64 = 120;

pub const SCOPE_TRANSFER: &str = "transfer";
pub const SCOPE_TRANSFER_COMPOSITE: &str = "transfer_composite";

#[derive(Debug)]
pub enum BeginOutcome {
    /// Caller should execute the upstream request, then [`complete`] or [`release`].
    Proceed,
    /// Return this body as HTTP 200 (prior success/failure JSON).
    Replay(Value),
    /// Same key used with a different request body.
    Conflict,
    /// Another request with this key is still in flight.
    InFlight,
}

/// Read `Idempotency-Key` / `X-Idempotency-Key` (trimmed, max 256 chars).
pub fn extract_key(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get("Idempotency-Key")
        .or_else(|| headers.get("X-Idempotency-Key"))
        .or_else(|| headers.get("idempotency-key"))
        .and_then(|v| v.to_str().ok())?;
    let key = raw.trim();
    if key.is_empty() || key.len() > 256 {
        return None;
    }
    Some(key.to_string())
}

/// Stable hash of the Zainpay-bound request payload (not metadata).
pub fn hash_request(payload: &Value) -> String {
    let bytes = serde_json::to_vec(payload).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    hex::encode(digest)
}

pub async fn begin(
    state: &AppState,
    scope: &str,
    idem_key: &str,
    request_hash: &str,
) -> BeginOutcome {
    match db::idempotency::try_begin(&state.pool, scope, idem_key, request_hash).await {
        Ok(true) => return BeginOutcome::Proceed,
        Ok(false) => {}
        Err(err) => {
            warn!(error = %err, scope, idem_key, "idempotency begin failed — continuing without guard");
            return BeginOutcome::Proceed;
        }
    }

    let row = match db::idempotency::get(&state.pool, scope, idem_key).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            // Lost the race then row vanished — try once more.
            return match db::idempotency::try_begin(&state.pool, scope, idem_key, request_hash)
                .await
            {
                Ok(true) => BeginOutcome::Proceed,
                _ => BeginOutcome::InFlight,
            };
        }
        Err(err) => {
            warn!(error = %err, scope, idem_key, "idempotency get failed — continuing without guard");
            return BeginOutcome::Proceed;
        }
    };

    let age_secs = (chrono::Utc::now() - row.created_at).num_seconds();
    if age_secs > TTL_SECS {
        // Expired — delete and reclaim.
        let _ = sqlx::query(
            "DELETE FROM idempotency_keys WHERE scope = $1 AND idem_key = $2",
        )
        .bind(scope)
        .bind(idem_key)
        .execute(&state.pool)
        .await;
        return match db::idempotency::try_begin(&state.pool, scope, idem_key, request_hash).await
        {
            Ok(true) => BeginOutcome::Proceed,
            Ok(false) => BeginOutcome::InFlight,
            Err(_) => BeginOutcome::Proceed,
        };
    }

    if row.request_hash != request_hash {
        return BeginOutcome::Conflict;
    }

    if row.state == "completed" {
        if let Some(body) = row.response_body {
            return BeginOutcome::Replay(body);
        }
        return BeginOutcome::InFlight;
    }

    // Pending — try stale reclaim.
    match db::idempotency::try_reclaim_stale(
        &state.pool,
        scope,
        idem_key,
        request_hash,
        STALE_PENDING_SECS,
    )
    .await
    {
        Ok(true) => BeginOutcome::Proceed,
        Ok(false) => BeginOutcome::InFlight,
        Err(err) => {
            warn!(error = %err, scope, idem_key, "idempotency reclaim failed");
            BeginOutcome::InFlight
        }
    }
}

pub async fn complete(
    state: &AppState,
    scope: &str,
    idem_key: &str,
    response: &Value,
    resource_key: Option<&str>,
) {
    if let Err(err) =
        db::idempotency::complete(&state.pool, scope, idem_key, response, resource_key).await
    {
        warn!(error = %err, scope, idem_key, "idempotency complete failed");
    }
}

pub async fn release(state: &AppState, scope: &str, idem_key: &str) {
    if let Err(err) = db::idempotency::release_pending(&state.pool, scope, idem_key).await {
        warn!(error = %err, scope, idem_key, "idempotency release failed");
    }
}

pub fn conflict_error() -> (StatusCode, String) {
    (
        StatusCode::CONFLICT,
        "Idempotency-Key was reused with a different request body".into(),
    )
}

pub fn in_flight_error() -> (StatusCode, String) {
    (
        StatusCode::CONFLICT,
        "Idempotency-Key request is still in progress — retry shortly".into(),
    )
}

/// Hash the transfer-shaped payload after metadata peel (stable via typed re-serialize).
pub fn transfer_request_hash(body: &Value) -> Result<String, String> {
    let (mut payload, mut metadata) = crate::services::proxy::split_body(body.clone());
    crate::services::payer::peel_payer_fields(&mut payload, &mut metadata);
    crate::services::payer::peel_custom_param(&mut payload, &mut metadata);
    crate::services::payment_split::peel_split_fields(&mut payload, &mut metadata);
    // Prefer hashing the typed request so key order is stable.
    match serde_json::from_value::<client::FundsTransferRequest>(payload.clone()) {
        Ok(req) => {
            let v = serde_json::to_value(&req).unwrap_or(payload);
            Ok(hash_request(&v))
        }
        Err(err) => Err(format!("invalid transfer body: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_key_prefers_standard_header() {
        let mut headers = HeaderMap::new();
        headers.insert("Idempotency-Key", "abc-123".parse().unwrap());
        assert_eq!(extract_key(&headers).as_deref(), Some("abc-123"));
    }

    #[test]
    fn extract_key_accepts_x_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Idempotency-Key", "xyz".parse().unwrap());
        assert_eq!(extract_key(&headers).as_deref(), Some("xyz"));
    }

    #[test]
    fn hash_is_stable_for_same_json() {
        let a = hash_request(&json!({"amount":"100","txnRef":"T1"}));
        let b = hash_request(&json!({"amount":"100","txnRef":"T1"}));
        assert_eq!(a, b);
        assert_ne!(a, hash_request(&json!({"amount":"200","txnRef":"T1"})));
    }
}
