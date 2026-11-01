use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use client::{verify_webhook_signature, WebhookEvent};

use client::WebhookEventType;

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

/// Inbound Zainpay webhook receiver.
///
/// Verifies `Zainpay-Signature` (HMAC-SHA256 hex of the raw body) when
/// `ZAINPAY_WEBHOOK_SECRET` is configured, de-duplicates by body hash,
/// persists the event to `transactions`, and acknowledges with HTTP 200.
pub async fn inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<&'static str, (StatusCode, String)> {
    let Some(secret) = state.config.zainpay_webhook_secret.as_deref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "ZAINPAY_WEBHOOK_SECRET is not configured".into(),
        ));
    };

    let signature = headers
        .get("Zainpay-Signature")
        .or_else(|| headers.get("zainpay-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !verify_webhook_signature(secret.as_bytes(), &body, signature) {
        activity::record(
            &state,
            "inbound.webhook.rejected_signature",
            SuccessSource::ZainpayWebhook.as_str(),
            None,
            None,
            "Inbound Zainpay webhook rejected — invalid signature",
            json!({ "bodyBytes": body.len() }),
        )
        .await;
        return Err((StatusCode::UNAUTHORIZED, "invalid Zainpay-Signature".into()));
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
            json!({ "event": event_name, "deliveryHash": delivery_hash }),
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
