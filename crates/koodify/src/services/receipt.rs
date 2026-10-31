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

fn first_data_str(response: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = response
            .pointer(&format!("/data/{key}"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Some(s.to_string());
        }
        if let Some(s) = response
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

fn first_req_str(req: Option<&Value>, keys: &[&str]) -> Option<String> {
    let req = req?;
    for key in keys {
        if let Some(s) = req
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

fn humanize_key(key: &str) -> String {
    let mut out = String::new();
    for (i, ch) in key.chars().enumerate() {
        if ch == '_' || ch == '-' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            continue;
        }
        if i > 0
            && ch.is_ascii_uppercase()
            && out
                .chars()
                .last()
                .map(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                .unwrap_or(false)
        {
            out.push(' ');
        }
        if out.is_empty() || out.ends_with(' ') {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn value_as_display(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Array(_) | Value::Object(_) => Some(v.to_string()),
    }
}

fn custom_param_rows(custom: Option<&Value>, skip_keys: &[&str]) -> Vec<(String, String)> {
    let Some(Value::Object(map)) = custom else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (k, v) in map {
        let lower = k.to_ascii_lowercase();
        if skip_keys
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&lower) || s.eq_ignore_ascii_case(k))
        {
            continue;
        }
        if let Some(display) = value_as_display(v) {
            rows.push((humanize_key(k), display));
        }
    }
    rows.sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));
    rows
}

fn narration_from(
    response: &Value,
    init_req: Option<&Value>,
    custom: Option<&Value>,
) -> Option<String> {
    first_data_str(response, &["narration", "paymentDescription", "description"])
        .or_else(|| first_req_str(init_req, &["narration", "description", "purpose"]))
        .or_else(|| {
            let Value::Object(map) = custom? else {
                return None;
            };
            for key in ["narration", "description", "purpose", "paymentType"] {
                if let Some(display) = map.get(key).and_then(value_as_display) {
                    return Some(display);
                }
            }
            None
        })
}

/// Pull deposit detail fields from a previously stored webhook / verify payload.
fn deposit_overlay_from_records(records: &[db::models::TransactionRecord]) -> Option<Value> {
    for rec in records {
        let candidates = [
            rec.response_body.pointer("/data"),
            rec.response_body.get("data"),
            rec.request_body
                .as_ref()
                .and_then(|b| b.pointer("/data"))
                .or_else(|| rec.request_body.as_ref().and_then(|b| b.get("data"))),
            Some(&rec.response_body),
        ];
        for cand in candidates.into_iter().flatten() {
            let has_detail = cand.get("narration").is_some()
                || cand.get("sender").is_some()
                || cand.get("bankName").is_some()
                || cand.get("senderName").is_some()
                || cand.get("zainboxName").is_some();
            if has_detail {
                return Some(cand.clone());
            }
        }
    }
    None
}

fn merge_data_overlay(response: &Value, overlay: &Value) -> Value {
    let Some(overlay_obj) = overlay.as_object() else {
        return response.clone();
    };

    let mut out = response.clone();
    if let Some(Value::Object(data)) = out.get_mut("data") {
        for (k, v) in overlay_obj {
            let missing = !data.contains_key(k) || data.get(k).is_some_and(|x| x.is_null());
            if missing && !v.is_null() {
                data.insert(k.clone(), v.clone());
            }
        }
        return out;
    }

    let mut data = overlay_obj.clone();
    if let Some(status) = response
        .pointer("/data/paymentStatus")
        .or_else(|| response.get("paymentStatus"))
    {
        data.insert("paymentStatus".into(), status.clone());
    }
    Value::Object(serde_json::Map::from_iter([("data".into(), Value::Object(data))]))
}

async fn resolve_business_name(
    state: &AppState,
    response: &Value,
    init_req: Option<&Value>,
    zainbox_code: Option<&str>,
) -> Option<String> {
    if let Some(name) = first_data_str(response, &["zainboxName", "businessName"]) {
        return Some(name);
    }
    let code = zainbox_code
        .map(str::to_string)
        .or_else(|| first_req_str(init_req, &["zainboxCode", "zainbox_code"]))
        .or_else(|| first_data_str(response, &["zainboxCode"]));
    let Some(code) = code else {
        return None;
    };
    match db::zainboxes::get_by_code(&state.pool, &code).await {
        Ok(Some(zb)) => {
            let name = zb.name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        }
        Ok(None) => None,
        Err(err) => {
            warn!(error = %err, %code, "failed to load zainbox for receipt business name");
            None
        }
    }
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

    let records = db::transactions::find_by_resource_key(&state.pool, txn_ref)
        .await
        .unwrap_or_default();

    let init_req = records.iter().find_map(|r| r.request_body.clone());
    let enriched = match deposit_overlay_from_records(&records) {
        Some(overlay) => merge_data_overlay(response, &overlay),
        None => response.clone(),
    };

    let custom = payer::load_custom_param(&state.pool, txn_ref).await;
    let narration = narration_from(&enriched, init_req.as_ref(), custom.as_ref());
    let skip = [
        "narration",
        "description",
        "purpose",
        "paymentType",
        "payment_type",
    ];

    let zainbox_code = records.iter().find_map(|r| r.zainbox_code.as_deref());
    let business_name =
        resolve_business_name(state, &enriched, init_req.as_ref(), zainbox_code).await;

    let sender_account = first_data_str(&enriched, &["sender", "senderAccount", "sourceAccount"]);
    let sender_bank = first_data_str(&enriched, &["bankName", "senderBank", "sourceBank"]);
    let sender_name = first_data_str(&enriched, &["senderName"]);
    let paid_at = first_data_str(&enriched, &["paymentDate", "txnDate", "transactionDate"]);

    let mut custom_rows = custom_param_rows(custom.as_ref(), &skip);
    if let Some(ref n) = narration {
        custom_rows.retain(|(_, v)| v != n);
    }

    let receipt = mail::ReceiptEmail {
        to: email.to_string(),
        payer_name: name,
        amount_label: amount_label_from(&enriched, init_req.as_ref()),
        currency: currency_from(&enriched, init_req.as_ref()),
        reference: txn_ref.to_string(),
        paid_at,
        business_name,
        narration,
        sender_bank,
        sender_account,
        sender_name,
        custom_params: custom_rows,
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
    narration: Option<&str>,
    sender_bank: Option<&str>,
    sender_account: Option<&str>,
    sender_name: Option<&str>,
    business_name: Option<&str>,
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
    if let Some(n) = narration.filter(|s| !s.is_empty()) {
        data.insert("narration".into(), Value::String(n.to_string()));
    }
    if let Some(b) = sender_bank.filter(|s| !s.is_empty()) {
        data.insert("bankName".into(), Value::String(b.to_string()));
    }
    if let Some(s) = sender_account.filter(|s| !s.is_empty()) {
        data.insert("sender".into(), Value::String(s.to_string()));
    }
    if let Some(n) = sender_name.filter(|s| !s.is_empty()) {
        data.insert("senderName".into(), Value::String(n.to_string()));
    }
    if let Some(name) = business_name.filter(|s| !s.is_empty()) {
        data.insert("zainboxName".into(), Value::String(name.to_string()));
    }
    let response = Value::Object(serde_json::Map::from_iter([(
        "data".into(),
        Value::Object(data),
    )]));
    maybe_send_on_success(state, txn_ref, &response).await;
}
