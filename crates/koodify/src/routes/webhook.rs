//! Path 1 — inbound Zainpay notifications.
//!
//! `POST /webhooks/zainpay` is **open** (no `X-Api-Key`, no HMAC gate).
//! Zainpay posts a plain JSON event. We:
//! 1. Ack quickly with HTTP 200 when the body is JSON
//! 2. De-dupe by body hash
//! 3. Confirm the `txnRef` is paid via Zainpay’s own APIs (source of truth)
//! 4. Run the same success side effects as Path 2 / status poll
//!
//! HMAC (`Zainpay-Signature`) is ignored for accept/reject — it has been
//! unreliable across environments. Spoofed posts without a real paid txn
//! are no-ops because upstream confirm fails.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_upstream, JsonResult};
use crate::services::activity::{self, SuccessSource};
use crate::services::payment_success;
use crate::state::AppState;

pub async fn repush_deposit(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .webhooks()
        .repush_deposit(&txn_ref)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn repush_card_payment(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .webhooks()
        .repush_card_payment(&txn_ref)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

fn first_str(v: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = v
            .get(*key)
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Some(s.to_string());
        }
    }
    None
}

fn data_object(payload: &Value) -> &Value {
    payload.get("data").unwrap_or(payload)
}

fn extract_txn_ref(payload: &Value) -> Option<String> {
    let data = data_object(payload);
    first_str(
        data,
        &[
            "txnRef",
            "txn_ref",
            "paymentRef",
            "payment_ref",
            "internalTxnRef",
            "internal_txn_ref",
        ],
    )
    .or_else(|| {
        first_str(
            payload,
            &[
                "txnRef",
                "txn_ref",
                "paymentRef",
                "payment_ref",
                "internalTxnRef",
                "internal_txn_ref",
            ],
        )
    })
}

fn event_name(payload: &Value) -> String {
    payload
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn looks_like_deposit(payload: &Value) -> bool {
    let name = event_name(payload).to_ascii_lowercase();
    if name.contains("deposit") || name.contains("success") {
        return true;
    }
    let data = data_object(payload);
    data.get("depositedAmount").is_some()
        || data.get("txnType").and_then(|v| v.as_str()) == Some("deposit")
        || first_str(data, &["paymentStatus"]).is_some_and(|s| {
            matches!(
                s.to_ascii_lowercase().as_str(),
                "success" | "successful" | "paid" | "completed"
            )
        })
}

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

/// Confirm the txn is paid using Zainpay (same checks Path 2 uses).
async fn confirm_paid(state: &AppState, txn_ref: &str) -> Option<Value> {
    if let Ok(resp) = state
        .zainpay
        .cards()
        .temporary_virtual_account_payment_status(txn_ref)
        .await
    {
        let value = serde_json::to_value(&resp).unwrap_or(Value::Null);
        if resp.is_success() && payment_status_success(&value) {
            return Some(value);
        }
    }

    if let Ok(resp) = state.zainpay.reconciliation().verify_deposit(txn_ref).await {
        if resp.is_success() {
            return Some(serde_json::to_value(&resp).unwrap_or(Value::Null));
        }
    }

    None
}

fn synthetic_from_webhook(payload: &Value) -> (Option<String>, Option<String>, Value) {
    let data = data_object(payload);
    let deposit_account = first_str(
        data,
        &["accountNumber", "account_number", "beneficiaryAccountNumber"],
    );
    let deposit_bank = first_str(data, &["bankCode", "bank_code", "beneficiaryBankCode"])
        .or_else(|| {
            first_str(data, &["bankName"]).filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        });

    let amount = first_str(data, &["depositedAmount", "amountAfterCharges", "amount"]);
    let mut map = serde_json::Map::new();
    map.insert("paymentStatus".into(), json!("success"));
    if let Some(a) = amount {
        map.insert("depositedAmount".into(), json!(a.clone()));
        map.insert("amount".into(), json!(a));
    }
    for (k, keys) in [
        ("paymentDate", &["paymentDate", "txnDate"][..]),
        ("narration", &["narration"][..]),
        ("bankName", &["bankName"][..]),
        ("sender", &["sender"][..]),
        ("senderName", &["senderName"][..]),
        ("zainboxName", &["zainboxName"][..]),
        ("zainboxCode", &["zainboxCode"][..]),
        ("accountNumber", &["accountNumber", "beneficiaryAccountNumber"][..]),
        ("bankCode", &["bankCode", "beneficiaryBankCode"][..]),
    ] {
        if let Some(v) = first_str(data, keys) {
            map.insert(k.into(), json!(v));
        }
    }

    (deposit_account, deposit_bank, json!({ "data": map }))
}

/// Open inbound webhook — Path 1.
pub async fn inbound(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<&'static str, (StatusCode, String)> {
    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid webhook JSON: {e}"),
        )
    })?;

    let event = event_name(&payload);
    let txn_ref = extract_txn_ref(&payload);
    let zainbox_code = first_str(data_object(&payload), &["zainboxCode", "zainbox_code"]);
    let delivery_hash = hex::encode(Sha256::digest(&body));

    let first_delivery = db::webhook_deliveries::try_claim(
        &state.pool,
        &delivery_hash,
        txn_ref.as_deref(),
        &event,
    )
    .await
    .map_err(|err| {
        tracing::error!(error = %err, "webhook delivery claim failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record webhook delivery".into(),
        )
    })?;

    if !first_delivery {
        activity::record(
            &state,
            "inbound.webhook.duplicate",
            SuccessSource::ZainpayWebhook.as_str(),
            txn_ref.as_deref(),
            zainbox_code.as_deref(),
            format!("Duplicate Zainpay webhook ignored ({event})"),
            json!({ "event": event, "deliveryHash": delivery_hash }),
        )
        .await;
        return Ok("OK");
    }

    activity::record(
        &state,
        "inbound.webhook.received",
        SuccessSource::ZainpayWebhook.as_str(),
        txn_ref.as_deref(),
        zainbox_code.as_deref(),
        format!("Zainpay webhook received ({event})"),
        json!({
            "event": event,
            "deliveryHash": delivery_hash,
            "bodyBytes": body.len(),
            "looksLikeDeposit": looks_like_deposit(&payload),
        }),
    )
    .await;

    if let Err(err) = db::transactions::insert(
        &state.pool,
        &format!("webhook.{event}"),
        txn_ref.as_deref(),
        zainbox_code.as_deref(),
        &Value::Object(Default::default()),
        Some(&payload),
        &payload,
        "00",
    )
    .await
    {
        tracing::error!(error = %err, "failed to persist inbound webhook");
    }

    let Some(txn_ref) = txn_ref else {
        activity::record(
            &state,
            "inbound.webhook.ignored",
            SuccessSource::ZainpayWebhook.as_str(),
            None,
            zainbox_code.as_deref(),
            "Inbound webhook had no txnRef — acknowledged",
            json!({ "event": event }),
        )
        .await;
        return Ok("OK");
    };

    if !looks_like_deposit(&payload) {
        return Ok("OK");
    }

    // Trust anchor: Zainpay must confirm the payment. Webhook body alone is not enough.
    let confirmed = confirm_paid(&state, &txn_ref).await;
    if confirmed.is_none() {
        activity::record(
            &state,
            "inbound.webhook.unconfirmed",
            SuccessSource::ZainpayWebhook.as_str(),
            Some(txn_ref.as_str()),
            zainbox_code.as_deref(),
            "Webhook received but Zainpay has not confirmed payment yet",
            json!({ "event": event }),
        )
        .await;
        // Still 200 — do not make Zainpay retry forever; Path 2 will catch up.
        return Ok("OK");
    }

    let (deposit_account, deposit_bank, synthetic) = synthetic_from_webhook(&payload);
    // Prefer fields from the confirmed Zainpay status when present.
    let mut synthetic = synthetic;
    if let Some(conf) = confirmed {
        if let Some(status) = conf.pointer("/data/paymentStatus").cloned() {
            if let Some(obj) = synthetic.get_mut("data").and_then(|d| d.as_object_mut()) {
                obj.insert("paymentStatus".into(), status);
            }
        }
    }

    payment_success::apply_side_effects(
        &state,
        &txn_ref,
        &synthetic,
        deposit_account.as_deref(),
        deposit_bank.as_deref(),
        SuccessSource::ZainpayWebhook,
    )
    .await;

    Ok("OK")
}
