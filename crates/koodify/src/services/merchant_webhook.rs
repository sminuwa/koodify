//! Outbound merchant HTTP webhooks for `callBackUrl` / zainbox `callback_url`.
//!
//! On payment success (status poll or deposit webhook), POST a signed JSON body
//! once per `txnRef` so server-side fulfillment works even if the browser closes.

use std::time::Duration;

use client::sign_webhook_body;
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::db;
use crate::services::payer;
use crate::state::AppState;

const MAX_ATTEMPTS: u32 = 3;
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(8);

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

fn first_str(map: Option<&Value>, keys: &[&str]) -> Option<String> {
    let map = map?;
    for key in keys {
        if let Some(s) = map
            .get(*key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Some(s.to_string());
        }
    }
    None
}

/// Accept only absolute http(s) URLs. Reject blank / relative / non-http schemes.
pub fn normalize_callback_url(raw: &str) -> Option<String> {
    let url = raw.trim();
    if url.is_empty() {
        return None;
    }
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return None;
    }
    Some(url.to_string())
}

fn callback_from_request(req: &Value) -> Option<String> {
    first_str(
        Some(req),
        &["callBackUrl", "callbackUrl", "call_back_url", "callback_url"],
    )
    .and_then(|s| normalize_callback_url(&s))
}

async fn resolve_callback_url(
    state: &AppState,
    txn_ref: &str,
    records: &[db::models::TransactionRecord],
) -> Option<String> {
    for rec in records {
        if let Some(req) = rec.request_body.as_ref() {
            if let Some(url) = callback_from_request(req) {
                if state.config.is_inbound_zainpay_webhook_url(&url) {
                    // Checkout accidentally pointed at Path 1 — do not merchant-POST to ourselves.
                    continue;
                }
                return Some(url);
            }
        }
    }

    let code = records.iter().find_map(|r| r.zainbox_code.as_deref());
    let Some(code) = code else {
        return None;
    };
    match db::zainboxes::get_by_code(&state.pool, code).await {
        Ok(Some(zb)) => zb
            .callback_url
            .as_deref()
            .and_then(normalize_callback_url)
            // Zainbox callbackUrl on Zainpay points at Koodify inbound — not a merchant URL.
            .filter(|u| !state.config.is_inbound_zainpay_webhook_url(u)),
        Ok(None) => None,
        Err(err) => {
            warn!(error = %err, %code, txn_ref, "failed to load zainbox for merchant webhook");
            None
        }
    }
}

fn amount_naira(response: &Value, init_req: Option<&Value>) -> Option<String> {
    if let Some(s) = first_str(init_req, &["amount"]) {
        return Some(s);
    }
    response
        .pointer("/data/amount")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

async fn build_payload(state: &AppState, txn_ref: &str, response: &Value) -> Value {
    let records = db::transactions::find_by_resource_key(&state.pool, txn_ref)
        .await
        .unwrap_or_default();
    let init_req = records.iter().find_map(|r| r.request_body.clone());
    let zainbox_code = records
        .iter()
        .find_map(|r| r.zainbox_code.clone())
        .or_else(|| first_str(init_req.as_ref(), &["zainboxCode", "zainbox_code"]));

    let payer = payer::load_payer(&state.pool, txn_ref).await;
    let custom = payer::load_custom_param(&state.pool, txn_ref).await;
    let split = db::payment_splits::summary_json(&state.pool, txn_ref)
        .await
        .ok()
        .flatten();

    let mut payload = json!({
        "event": "payment.success",
        "status": "success",
        "reference": txn_ref,
        "description": "Payment was successful",
        "currencyCode": first_str(init_req.as_ref(), &["currencyCode", "currency"])
            .or_else(|| response.pointer("/data/currencyCode").and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or_else(|| "NGN".into()),
    });

    if let Some(obj) = payload.as_object_mut() {
        if let Some(amount) = amount_naira(response, init_req.as_ref()) {
            obj.insert("amount".into(), Value::String(amount));
        }
        if let Some(code) = zainbox_code {
            obj.insert("zainboxCode".into(), Value::String(code));
        }
        if let Some(p) = payer {
            obj.insert("payer".into(), p);
        }
        if let Some(c) = custom {
            obj.insert("customParam".into(), c);
        }
        if let Some(s) = split {
            obj.insert("paymentSplit".into(), s);
        }
    }

    payload
}

async fn post_signed(
    url: &str,
    body: &[u8],
    signature: &str,
) -> Result<reqwest::StatusCode, String> {
    let client = reqwest::Client::builder()
        .timeout(ATTEMPT_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Koodify-Webhook/1.0")
        .header("X-Koodify-Signature", signature)
        .header("X-Koodify-Event", "payment.success")
        .body(body.to_vec())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp.status())
}

/// Deliver a signed POST to the merchant callback once payment is confirmed.
pub async fn maybe_deliver_on_success(state: &AppState, txn_ref: &str, response: &Value) {
    if !payment_status_success(response) {
        return;
    }

    let records = match db::transactions::find_by_resource_key(&state.pool, txn_ref).await {
        Ok(r) => r,
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to load txn for merchant webhook");
            return;
        }
    };

    let Some(url) = resolve_callback_url(state, txn_ref, &records).await else {
        return;
    };

    match db::transactions::claim_merchant_webhook(&state.pool, txn_ref).await {
        Ok(true) => {}
        Ok(false) => return,
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to claim merchant webhook send");
            return;
        }
    }

    let payload = build_payload(state, txn_ref, response).await;
    let body = match serde_json::to_vec(&payload) {
        Ok(b) => b,
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to serialize merchant webhook body");
            return;
        }
    };

    let secret = state.config.merchant_webhook_signing_key();
    let signature = sign_webhook_body(secret.as_bytes(), &body);

    let mut last_err: Option<String> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match post_signed(&url, &body, &signature).await {
            Ok(status) if status.is_success() => {
                info!(
                    txn_ref,
                    %url,
                    attempt,
                    %status,
                    "merchant webhook delivered"
                );
                return;
            }
            Ok(status) => {
                last_err = Some(format!("HTTP {status}"));
                warn!(
                    txn_ref,
                    %url,
                    attempt,
                    %status,
                    "merchant webhook non-success response"
                );
            }
            Err(err) => {
                last_err = Some(err.clone());
                warn!(
                    error = %err,
                    txn_ref,
                    %url,
                    attempt,
                    "merchant webhook delivery failed"
                );
            }
        }
        if attempt < MAX_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(250 * u64::from(attempt))).await;
        }
    }

    warn!(
        txn_ref,
        %url,
        error = last_err.as_deref().unwrap_or("unknown"),
        "merchant webhook exhausted retries — marked sent to avoid storm; clear metadata.merchantWebhookSent to retry"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::verify_webhook_signature;

    #[test]
    fn normalize_accepts_http_https_only() {
        assert_eq!(
            normalize_callback_url(" https://pay.example/hook "),
            Some("https://pay.example/hook".into())
        );
        assert_eq!(
            normalize_callback_url("http://localhost:3000/paid"),
            Some("http://localhost:3000/paid".into())
        );
        assert!(normalize_callback_url("").is_none());
        assert!(normalize_callback_url("/relative").is_none());
        assert!(normalize_callback_url("ftp://x").is_none());
        assert!(normalize_callback_url("javascript:alert(1)").is_none());
    }

    #[test]
    fn signature_matches_client_helper() {
        let body = br#"{"event":"payment.success","reference":"ORDER_1"}"#;
        let secret = b"gateway-secret";
        let sig = sign_webhook_body(secret, body);
        assert!(verify_webhook_signature(secret, body, &sig));
    }

    #[test]
    fn callback_from_request_reads_aliases() {
        let req = json!({ "callBackUrl": "https://school.example/paid" });
        assert_eq!(
            callback_from_request(&req).as_deref(),
            Some("https://school.example/paid")
        );
        let req2 = json!({ "callback_url": "https://a.example/w" });
        assert_eq!(
            callback_from_request(&req2).as_deref(),
            Some("https://a.example/w")
        );
    }
}
