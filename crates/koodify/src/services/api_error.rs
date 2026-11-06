//! Enrich Zainpay application-level error envelopes (DX issue 06).
//!
//! Opaque codes like `04` become a stable `data.koodify.error` object with
//! `koodifyCode`, `message`, and an actionable `hint`.

use client::types::codes::{self, describe_code};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichedError {
    pub koodify_code: &'static str,
    pub message: String,
    pub hint: String,
    pub upstream_code: String,
    pub upstream_description: Option<String>,
}

impl EnrichedError {
    pub fn to_json(&self) -> Value {
        json!({
            "koodifyCode": self.koodify_code,
            "message": self.message,
            "hint": self.hint,
            "upstreamCode": self.upstream_code,
            "upstreamDescription": self.upstream_description,
        })
    }
}

/// Classify a Zainpay envelope into an enriched error, if it is not success/queued.
pub fn classify(response: &Value) -> Option<EnrichedError> {
    let code = response
        .get("code")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;

    if code == codes::SUCCESS || code == codes::FUNDS_TRANSFER_QUEUED {
        return None;
    }

    let description = response
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let desc_l = description
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();

    let (koodify_code, message, hint) = resolve(code, &desc_l, description.as_deref());

    Some(EnrichedError {
        koodify_code,
        message: message.to_string(),
        hint: hint.to_string(),
        upstream_code: code.to_string(),
        upstream_description: description,
    })
}

/// Attach `data.koodify.error` when the envelope is an application-level failure.
/// Leaves success / queued responses unchanged. Merges with an existing `data.koodify`.
pub fn enrich(mut response: Value) -> Value {
    let Some(err) = classify(&response) else {
        return response;
    };
    let block = err.to_json();

    // Ensure `data` is an object we can merge into.
    let needs_data_object = !matches!(response.get("data"), Some(Value::Object(_)));
    if needs_data_object {
        let previous = response.get("data").cloned().unwrap_or(Value::Null);
        if let Some(obj) = response.as_object_mut() {
            let data = if previous.is_null() {
                json!({})
            } else {
                json!({ "value": previous })
            };
            obj.insert("data".into(), data);
        }
    }

    if let Some(Value::Object(map)) = response.get_mut("data") {
        match map.get_mut("koodify") {
            Some(Value::Object(k)) => {
                k.insert("error".into(), block);
            }
            _ => {
                map.insert("koodify".into(), json!({ "error": block }));
            }
        }
    }

    response
}

fn resolve(
    code: &str,
    desc_l: &str,
    description: Option<&str>,
) -> (&'static str, String, &'static str) {
    // Description-first for known operational traps (especially opaque `04`).
    if desc_l.contains("auto settlement") && desc_l.contains("transfer") {
        return (
            "transfer_disabled_auto_settlement",
            description
                .unwrap_or("Funds transfer disabled while auto settlement is enabled")
                .to_string(),
            "Disable auto settlement on the primary zainbox (POST /api/v1/settlements with status:false), then retry the transfer or payment split.",
        );
    }
    if desc_l.contains("txn not found") || desc_l.contains("transaction not found") {
        return (
            "txn_not_found",
            description.unwrap_or("Transaction not found").to_string(),
            "Confirm the txnRef. This is not the same as a pending payment — do not fulfill.",
        );
    }
    if desc_l.contains("duplicate") {
        return (
            "duplicate_txn_ref",
            description
                .unwrap_or("Duplicate transaction reference")
                .to_string(),
            "Use a fresh unique txnRef (or the same Idempotency-Key to replay the original response).",
        );
    }
    if desc_l.contains("lifetime has expired")
        || desc_l.contains("dva expired")
        || (desc_l.contains("expired") && !desc_l.contains("expired resource"))
    {
        return (
            "resource_expired",
            description.unwrap_or("Resource expired").to_string(),
            "Create a new checkout / DVA session; do not reuse the expired account number.",
        );
    }
    if desc_l.contains("insufficient") || desc_l.contains("low balance") {
        return (
            "insufficient_balance",
            description.unwrap_or("Insufficient balance").to_string(),
            "Fund the source virtual account / ISA, then retry.",
        );
    }

    let label = describe_code(code).unwrap_or("upstream application error");
    let message = description
        .map(str::to_string)
        .unwrap_or_else(|| label.to_string());

    match code {
        codes::INVALID_ZAINBOX => (
            "invalid_zainbox",
            message,
            "Check zainboxCode / publicKey mapping — the zainbox must exist and be active.",
        ),
        codes::SUBSCRIPTION_ERROR => (
            "subscription_error",
            message,
            "Verify the zainbox code used for service subscription endpoints.",
        ),
        codes::RESOURCE_OR_APPLICATION_ERROR => (
            "resource_or_application_error",
            message,
            "Zainpay code 04 is a catch-all (not found, expired, or failed op). Re-check txnRef, session lifetime, and request fields; treat as not paid.",
        ),
        codes::INTERNAL_CONFIG_ERROR => (
            "internal_config_error",
            message,
            "Upstream configuration issue — retry later or contact Zainpay support with the description.",
        ),
        codes::ALREADY_SUBSCRIBED => (
            "already_subscribed",
            message,
            "Subscription already active — treat as success or skip re-subscribe.",
        ),
        codes::ALREADY_UNSUBSCRIBED => (
            "already_unsubscribed",
            message,
            "Service already unsubscribed — no further action needed.",
        ),
        codes::PAYLOAD_VALIDATION => (
            "payload_validation",
            message,
            "Fix required fields / types against the endpoint schema, then retry.",
        ),
        codes::INSUFFICIENT_BALANCE => (
            "insufficient_balance",
            message,
            "Fund the source account / ISA before retrying the transfer.",
        ),
        codes::INVALID_DESTINATION_ACCOUNT => (
            "invalid_destination_account",
            message,
            "Run name enquiry and confirm bankCode + accountNumber, then retry.",
        ),
        codes::NO_WALLET_BALANCE => (
            "no_wallet_balance",
            message,
            "The account has no wallet balance — fund it or choose another source account.",
        ),
        codes::DUPLICATE_TXN_REF => (
            "duplicate_txn_ref",
            message,
            "Generate a new txnRef, or replay with the same Idempotency-Key if this was a retry.",
        ),
        codes::BILLING_ESTIMATION_ERROR => (
            "billing_estimation_error",
            message,
            "Transfer billing estimation failed — verify amount/channel fees and retry, or contact Zainpay.",
        ),
        codes::CARD_VALIDATION => (
            "payload_validation",
            message,
            "Fix payload / card validation errors and retry.",
        ),
        _ => (
            "upstream_error",
            message,
            "Inspect upstreamCode / upstreamDescription; do not assume payment success.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_not_enriched() {
        let out = enrich(json!({"code":"00","description":"ok","data":{}}));
        assert!(out.pointer("/data/koodify/error").is_none());
    }

    #[test]
    fn queued_not_enriched() {
        let out = enrich(json!({"code":"21","description":"queued","data":{}}));
        assert!(out.pointer("/data/koodify/error").is_none());
    }

    #[test]
    fn code_04_auto_settlement_hint() {
        let out = enrich(json!({
            "code": "04",
            "description": "Funds Transfer is Disabled for Auto Settlement Enabled Zainbox",
            "data": null
        }));
        let err = &out["data"]["koodify"]["error"];
        assert_eq!(err["koodifyCode"], "transfer_disabled_auto_settlement");
        assert!(err["hint"].as_str().unwrap().contains("Disable auto settlement"));
    }

    #[test]
    fn code_04_txn_not_found() {
        let out = enrich(json!({
            "code": "04",
            "description": "Txn not found",
            "data": {}
        }));
        assert_eq!(out["data"]["koodify"]["error"]["koodifyCode"], "txn_not_found");
    }

    #[test]
    fn code_04_generic_catch_all() {
        let out = enrich(json!({
            "code": "04",
            "description": "Invalid / expired resource / application error",
            "data": {}
        }));
        assert_eq!(
            out["data"]["koodify"]["error"]["koodifyCode"],
            "resource_or_application_error"
        );
    }

    #[test]
    fn duplicate_txn_ref() {
        let out = enrich(json!({
            "code": "26",
            "description": "Duplicate transaction ref number",
            "data": {}
        }));
        assert_eq!(out["data"]["koodify"]["error"]["koodifyCode"], "duplicate_txn_ref");
    }

    #[test]
    fn merges_with_existing_koodify_payment_block() {
        let out = enrich(json!({
            "code": "23",
            "description": "Insufficient wallet balance",
            "data": {
                "koodify": {
                    "paymentStatus": "unknown",
                    "isPaid": false
                }
            }
        }));
        assert_eq!(out["data"]["koodify"]["paymentStatus"], "unknown");
        assert_eq!(out["data"]["koodify"]["error"]["koodifyCode"], "insufficient_balance");
    }
}
