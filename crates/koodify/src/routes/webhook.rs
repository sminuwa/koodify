use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use client::{
    describe_signature_header, verify_webhook_signature_any, WebhookEvent, WebhookEventType,
};

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

fn candidate_secrets(state: &AppState) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    if let Some(secret) = state.config.zainpay_webhook_secret.as_deref() {
        for part in secret.split([',', ';']) {
            let t = part.trim().trim_matches('"').trim_matches('\'');
            if !t.is_empty() {
                out.push(t.as_bytes().to_vec());
            }
        }
    }
    // Some Zainpay environments appear to sign with the API token itself.
    let api = state.config.zainpay_api_key.trim();
    if !api.is_empty() && !out.iter().any(|s| s.as_slice() == api.as_bytes()) {
        out.push(api.as_bytes().to_vec());
    }
    out
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

fn extract_txn_ref_loose(body: &[u8]) -> Result<(String, Value), String> {
    let value: Value =
        serde_json::from_slice(body).map_err(|e| format!("json_parse: {e}"))?;
    let data = value.get("data").unwrap_or(&value);
    let txn = first_str(
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
            &value,
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
    .ok_or_else(|| "missing_txn_ref".to_string())?;
    Ok((txn, value))
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

/// When HMAC fails, confirm the payment with Zainpay (same sources Path 2 uses).
/// Returns `Ok(txn_ref)` when accepted, `Err(reason)` for activity diagnostics.
async fn accept_via_upstream_verify(state: &AppState, body: &[u8]) -> Result<String, String> {
    let (txn_ref, parsed) = extract_txn_ref_loose(body)?;
    let event_name = parsed
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Temp-VA checkouts (LIVE_…) — same poll Path 2 uses.
    match state
        .zainpay
        .cards()
        .temporary_virtual_account_payment_status(&txn_ref)
        .await
    {
        Ok(resp) => {
            let value = serde_json::to_value(&resp).unwrap_or(Value::Null);
            if resp.is_success() && payment_status_success(&value) {
                tracing::warn!(
                    %txn_ref,
                    %event_name,
                    "inbound webhook HMAC mismatch — accepted via temp-VA status poll"
                );
                return Ok(txn_ref);
            }
        }
        Err(err) => {
            tracing::debug!(%txn_ref, error = %err, "temp-VA status poll failed during webhook fallback");
        }
    }

    // Generic deposit verify (permanent VA / wallet deposits).
    match state.zainpay.reconciliation().verify_deposit(&txn_ref).await {
        Ok(resp) if resp.is_success() => {
            tracing::warn!(
                %txn_ref,
                %event_name,
                "inbound webhook HMAC mismatch — accepted via deposit verify"
            );
            Ok(txn_ref)
        }
        Ok(resp) => Err(format!(
            "upstream_not_success code={} event={}",
            resp.code, event_name
        )),
        Err(err) => Err(format!("upstream_error: {err}")),
    }
}

/// Inbound Zainpay webhook receiver.
///
/// Verifies `Zainpay-Signature` when configured. If the HMAC does not match
/// (wrong dashboard secret / encoding), falls back to confirming `deposit.success`
/// via Zainpay's deposit-verify API so Path 1 is not stuck behind a bad secret.
pub async fn inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<&'static str, (StatusCode, String)> {
    let secrets = candidate_secrets(&state);
    if secrets.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "ZAINPAY_WEBHOOK_SECRET is not configured".into(),
        ));
    }

    let signature = headers
        .get("Zainpay-Signature")
        .or_else(|| headers.get("zainpay-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let mut auth_mode = "hmac";
    if !verify_webhook_signature_any(&secrets, &body, signature) {
        match accept_via_upstream_verify(&state, &body).await {
            Ok(txn_ref) => {
                auth_mode = "upstream_verify";
                activity::record(
                    &state,
                    "inbound.webhook.accepted_upstream_verify",
                    SuccessSource::ZainpayWebhook.as_str(),
                    Some(txn_ref.as_str()),
                    None,
                    "Inbound Zainpay webhook accepted after upstream verify (HMAC mismatch)",
                    {
                        let mut meta = describe_signature_header(signature);
                        if let Some(obj) = meta.as_object_mut() {
                            obj.insert("bodyBytes".into(), json!(body.len()));
                            obj.insert("secretCandidates".into(), json!(secrets.len()));
                            obj.insert("txnRef".into(), json!(txn_ref));
                        }
                        meta
                    },
                )
                .await;
            }
            Err(reason) => {
                let top_keys = serde_json::from_slice::<Value>(&body)
                    .ok()
                    .and_then(|v| v.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()));
                activity::record(
                    &state,
                    "inbound.webhook.rejected_signature",
                    SuccessSource::ZainpayWebhook.as_str(),
                    None,
                    None,
                    "Inbound Zainpay webhook rejected — invalid signature",
                    {
                        let mut meta = describe_signature_header(signature);
                        if let Some(obj) = meta.as_object_mut() {
                            obj.insert("bodyBytes".into(), json!(body.len()));
                            obj.insert("secretCandidates".into(), json!(secrets.len()));
                            obj.insert(
                                "secretLens".into(),
                                json!(secrets.iter().map(|s| s.len()).collect::<Vec<_>>()),
                            );
                            obj.insert("fallbackFail".into(), json!(reason));
                            if let Some(keys) = top_keys {
                                obj.insert("bodyTopKeys".into(), json!(keys));
                            }
                        }
                        meta
                    },
                )
                .await;
                return Err((StatusCode::UNAUTHORIZED, "invalid Zainpay-Signature".into()));
            }
        }
    }

    let event: WebhookEvent = serde_json::from_slice(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid webhook JSON: {e}"),
        )
    })?;

    let resource_key = event.data.idempotency_key().map(str::to_string);
    let event_name = event.event.as_str().to_string();
    let delivery_hash = hex::encode(Sha256::digest(&body));

    let first_delivery = db::webhook_deliveries::try_claim(
        &state.pool,
        &delivery_hash,
        resource_key.as_deref(),
        &event_name,
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
        tracing::info!(
            %delivery_hash,
            resource = ?resource_key,
            event = %event_name,
            "duplicate webhook delivery — acknowledging without re-processing"
        );
        activity::record(
            &state,
            "inbound.webhook.duplicate",
            SuccessSource::ZainpayWebhook.as_str(),
            resource_key.as_deref(),
            event.data.zainbox_code.as_deref(),
            format!("Duplicate Zainpay webhook ignored ({event_name})"),
            json!({ "event": event_name, "deliveryHash": delivery_hash, "authMode": auth_mode }),
        )
        .await;
        return Ok("OK");
    }

    let zainbox_code = event.data.zainbox_code.clone();
    let payload = serde_json::to_value(&event).unwrap_or(Value::Null);

    activity::record(
        &state,
        "inbound.webhook.received",
        SuccessSource::ZainpayWebhook.as_str(),
        resource_key.as_deref(),
        zainbox_code.as_deref(),
        format!("Zainpay webhook received ({event_name})"),
        json!({
            "event": event_name,
            "deliveryHash": delivery_hash,
            "authMode": auth_mode,
        }),
    )
    .await;

    if let Err(err) = db::transactions::insert(
        &state.pool,
        &format!("webhook.{event_name}"),
        resource_key.as_deref(),
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

    if event.event == WebhookEventType::DepositSuccess {
        if let Some(txn_ref) = resource_key.as_deref() {
            let amount = event
                .data
                .deposited_amount
                .as_deref()
                .or(event.data.amount_after_charges.as_deref());
            let paid_at = event
                .data
                .payment_date
                .as_deref()
                .or(event.data.txn_date.as_deref());
            let deposit_account = event
                .data
                .account_number
                .as_deref()
                .or(event.data.beneficiary_account_number.as_deref());
            let deposit_bank = event
                .data
                .beneficiary_bank_code
                .as_deref()
                .or(event.data.bank_name.as_deref().filter(|s| {
                    s.chars().all(|c| c.is_ascii_digit())
                }));

            let mut data = serde_json::Map::new();
            data.insert("paymentStatus".into(), json!("success"));
            if let Some(a) = amount {
                data.insert("depositedAmount".into(), json!(a));
                data.insert("amount".into(), json!(a));
            }
            if let Some(d) = paid_at {
                data.insert("paymentDate".into(), json!(d));
            }
            if let Some(n) = event.data.narration.as_deref() {
                data.insert("narration".into(), json!(n));
            }
            if let Some(b) = event.data.bank_name.as_deref() {
                data.insert("bankName".into(), json!(b));
            }
            if let Some(s) = event.data.sender.as_deref() {
                data.insert("sender".into(), json!(s));
            }
            if let Some(n) = event.data.sender_name.as_deref() {
                data.insert("senderName".into(), json!(n));
            }
            if let Some(name) = event.data.zainbox_name.as_deref() {
                data.insert("zainboxName".into(), json!(name));
            }
            if let Some(a) = deposit_account {
                data.insert("accountNumber".into(), json!(a));
            }
            if let Some(b) = deposit_bank {
                data.insert("bankCode".into(), json!(b));
            }
            if let Some(code) = zainbox_code.as_deref() {
                data.insert("zainboxCode".into(), json!(code));
            }

            let synthetic = json!({ "data": data });
            payment_success::apply_side_effects(
                &state,
                txn_ref,
                &synthetic,
                deposit_account,
                deposit_bank,
                SuccessSource::ZainpayWebhook,
            )
            .await;
        }
    }

    Ok("OK")
}
