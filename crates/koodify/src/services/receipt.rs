//! Fire-and-forget payer receipt on successful checkout / deposit.

use serde_json::Value;
use tracing::warn;

use crate::db;
use crate::services::{mail, payer};
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

fn format_naira(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return "—".into();
    }
    if let Ok(n) = t.parse::<f64>() {
        return if (n - n.trunc()).abs() < f64::EPSILON {
            format!("{n:.0}")
        } else {
            format!("{n:.2}")
        };
    }
    t.to_string()
}

/// Zainpay status/webhook amounts are usually kobo.
fn format_kobo_as_naira(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return "—".into();
    }
    if let Ok(kobo) = t.parse::<f64>() {
        let naira = kobo / 100.0;
        return if (naira - naira.trunc()).abs() < f64::EPSILON {
            format!("{naira:.0}")
        } else {
            format!("{naira:.2}")
        };
    }
    t.to_string()
}

fn amount_label_from(response: &Value, fallback_request: Option<&Value>) -> String {
    // Prefer the original checkout amount (already Naira).
    if let Some(s) = fallback_request.and_then(|r| r.get("amount").and_then(|v| v.as_str())) {
        return format_naira(s);
    }
    if let Some(s) = response.pointer("/data/amount").and_then(|v| v.as_str()) {
        return format_kobo_as_naira(s);
    }
    if let Some(n) = response.pointer("/data/amount").and_then(|v| v.as_f64()) {
        return format_kobo_as_naira(&n.to_string());
    }
    if let Some(s) = response
        .pointer("/data/depositedAmount")
        .and_then(|v| v.as_str())
    {
        return format_kobo_as_naira(s);
    }
    "—".into()
}

fn currency_from(response: &Value, fallback_request: Option<&Value>) -> String {
    fallback_request
        .and_then(|r| r.get("currencyCode").and_then(|v| v.as_str()))
        .or_else(|| {
            response
                .pointer("/data/currencyCode")
                .and_then(|v| v.as_str())
        })
        .or_else(|| response.pointer("/data/currency").and_then(|v| v.as_str()))
        .unwrap_or("NGN")
        .to_string()
}

/// Send receipt once when a payment is confirmed successful.
pub async fn maybe_send_on_success(state: &AppState, txn_ref: &str, response: &Value) {
    if !payment_status_success(response) {
        return;
    }
    if state.config.resend_api_key.is_none() {
        return;
    }

    let claimed = match db::transactions::claim_receipt_email(&state.pool, txn_ref).await {
        Ok(c) => c,
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to claim receipt email send");
            return;
        }
    };
    if !claimed {
        return;
    }

    let Some(payer_val) = payer::load_payer(&state.pool, txn_ref).await else {
        warn!(txn_ref, "no stored payer — cannot send receipt");
        return;
    };

    let email = payer_val
        .get("email")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if email.is_empty() {
        warn!(txn_ref, "payer has no email — skipping receipt");
        return;
    }

    let name = payer_val
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let init_req = db::transactions::find_by_resource_key(&state.pool, txn_ref)
        .await
        .ok()
        .and_then(|rows| rows.into_iter().find_map(|r| r.request_body));

    let receipt = mail::ReceiptEmail {
        to: email.to_string(),
        payer_name: name,
        amount_label: amount_label_from(response, init_req.as_ref()),
        currency: currency_from(response, init_req.as_ref()),
        reference: txn_ref.to_string(),
        paid_at: response
            .pointer("/data/paymentDate")
            .or_else(|| response.pointer("/data/txnDate"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    };

    if let Err(err) = mail::send_payment_receipt(&state.config, receipt).await {
        warn!(error = %err, txn_ref, "payer receipt email failed");
    }
}

/// Webhook path: build a synthetic status envelope and reuse the same sender.
pub async fn maybe_send_on_deposit_webhook(
    state: &AppState,
    txn_ref: &str,
    amount: Option<&str>,
    paid_at: Option<&str>,
) {
    let mut data = serde_json::Map::new();
    data.insert("paymentStatus".into(), Value::String("success".into()));
    if let Some(a) = amount.filter(|s| !s.is_empty()) {
        data.insert("depositedAmount".into(), Value::String(a.to_string()));
        data.insert("amount".into(), Value::String(a.to_string()));
    }
    if let Some(d) = paid_at.filter(|s| !s.is_empty()) {
        data.insert("paymentDate".into(), Value::String(d.to_string()));
    }
    let response = Value::Object(serde_json::Map::from_iter([(
        "data".into(),
        Value::Object(data),
    )]));
    maybe_send_on_success(state, txn_ref, &response).await;
}
