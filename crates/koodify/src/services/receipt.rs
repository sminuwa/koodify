//! Fire-and-forget payment emails on successful checkout / deposit:
//! 1. payer receipt → checkout email
//! 2. merchant notice → zainbox `email_notification`

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

fn first_notification_email(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    raw.split([',', ';'])
        .map(str::trim)
        .find(|s| !s.is_empty() && s.contains('@'))
        .map(str::to_string)
}

struct PaymentMailContext {
    payer_email: Option<String>,
    payer_name: Option<String>,
    merchant_email: Option<String>,
    business_name: Option<String>,
    amount_label: String,
    currency: String,
    paid_at: Option<String>,
    narration: Option<String>,
    sender_bank: Option<String>,
    sender_account: Option<String>,
    sender_name: Option<String>,
    custom_params: Vec<(String, String)>,
}

async fn resolve_zainbox(
    state: &AppState,
    response: &Value,
    init_req: Option<&Value>,
    zainbox_code: Option<&str>,
) -> Option<db::zainboxes::Zainbox> {
    let code = zainbox_code
        .map(str::to_string)
        .or_else(|| first_req_str(init_req, &["zainboxCode", "zainbox_code"]))
        .or_else(|| first_data_str(response, &["zainboxCode"]));
    let Some(code) = code else {
        return None;
    };
    match db::zainboxes::get_by_code(&state.pool, &code).await {
        Ok(zb) => zb,
        Err(err) => {
            warn!(error = %err, %code, "failed to load zainbox for payment emails");
            None
        }
    }
}

async fn build_context(
    state: &AppState,
    txn_ref: &str,
    response: &Value,
) -> PaymentMailContext {
    let payer_val = payer::load_payer(&state.pool, txn_ref).await;
    let records = db::transactions::find_by_resource_key(&state.pool, txn_ref)
        .await
        .unwrap_or_default();

    let init_req = records.iter().find_map(|r| r.request_body.clone());
    let enriched = match deposit_overlay_from_records(&records) {
        Some(overlay) => merge_data_overlay(response, &overlay),
        None => response.clone(),
    };

    let payer_email = payer_val
        .as_ref()
        .and_then(|p| {
            p.get("email")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .or_else(|| first_data_str(&enriched, &["email", "emailAddress"]))
        .or_else(|| first_req_str(init_req.as_ref(), &["emailAddress", "email"]));
    let payer_name = payer_val.as_ref().and_then(|p| {
        p.get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    });

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
    let zainbox = resolve_zainbox(state, &enriched, init_req.as_ref(), zainbox_code).await;

    let business_name = first_data_str(&enriched, &["zainboxName", "businessName"]).or_else(|| {
        zainbox.as_ref().and_then(|zb| {
            let name = zb.name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
    });

    let merchant_email =
        first_notification_email(zainbox.as_ref().and_then(|z| z.email_notification.as_deref()));

    let mut custom_rows = custom_param_rows(custom.as_ref(), &skip);
    if let Some(ref n) = narration {
        custom_rows.retain(|(_, v)| v != n);
    }

    PaymentMailContext {
        payer_email,
        payer_name,
        merchant_email,
        business_name,
        amount_label: amount_label_from(&enriched, init_req.as_ref()),
        currency: currency_from(&enriched, init_req.as_ref()),
        paid_at: first_data_str(&enriched, &["paymentDate", "txnDate", "transactionDate"]),
        narration,
        sender_bank: first_data_str(&enriched, &["bankName", "senderBank", "sourceBank"]),
        sender_account: first_data_str(&enriched, &["sender", "senderAccount", "sourceAccount"]),
        sender_name: first_data_str(&enriched, &["senderName"]),
        custom_params: custom_rows,
    }
}

fn to_mail(ctx: &PaymentMailContext, to: String, txn_ref: &str) -> mail::ReceiptEmail {
    mail::ReceiptEmail {
        to,
        payer_name: ctx.payer_name.clone(),
        payer_email: ctx.payer_email.clone(),
        amount_label: ctx.amount_label.clone(),
        currency: ctx.currency.clone(),
        reference: txn_ref.to_string(),
        paid_at: ctx.paid_at.clone(),
        business_name: ctx.business_name.clone(),
        narration: ctx.narration.clone(),
        sender_bank: ctx.sender_bank.clone(),
        sender_account: ctx.sender_account.clone(),
        sender_name: ctx.sender_name.clone(),
        custom_params: ctx.custom_params.clone(),
    }
}

/// Send payer receipt + merchant notice once each when payment is confirmed.
pub async fn maybe_send_on_success(state: &AppState, txn_ref: &str, response: &Value) {
    if !payment_status_success(response) {
        return;
    }
    if state.config.resend_api_key.is_none() {
        warn!(
            txn_ref,
            "RESEND_API_KEY unset on this server — skipping payer/merchant emails"
        );
        return;
    }

    let ctx = build_context(state, txn_ref, response).await;

    // --- payer receipt ---
    match db::transactions::claim_receipt_email(&state.pool, txn_ref).await {
        Ok(true) => {
            if let Some(email) = ctx.payer_email.clone() {
                let receipt = to_mail(&ctx, email, txn_ref);
                if let Err(err) = mail::send_payment_receipt(&state.config, receipt).await {
                    warn!(error = %err, txn_ref, "payer receipt email failed — clearing claim for retry");
                    let _ = db::transactions::clear_receipt_email(&state.pool, txn_ref).await;
                }
            } else {
                warn!(txn_ref, "no payer email — clearing claim so a later poll can retry");
                let _ = db::transactions::clear_receipt_email(&state.pool, txn_ref).await;
            }
        }
        Ok(false) => {}
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to claim receipt email send");
        }
    }

    // --- merchant / zainbox notice ---
    match db::transactions::claim_merchant_notice(&state.pool, txn_ref).await {
        Ok(true) => {
            if let Some(email) = ctx.merchant_email.clone() {
                let notice = to_mail(&ctx, email, txn_ref);
                if let Err(err) = mail::send_merchant_notice(&state.config, notice).await {
                    warn!(error = %err, txn_ref, "merchant notice email failed — clearing claim for retry");
                    let _ = db::transactions::clear_merchant_notice(&state.pool, txn_ref).await;
                }
            } else {
                // No destination configured — keep claim so we don't log forever.
                warn!(
                    txn_ref,
                    "zainbox has no email_notification — skipping merchant notice"
                );
            }
        }
        Ok(false) => {}
        Err(err) => {
            warn!(error = %err, txn_ref, "failed to claim merchant notice send");
        }
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
