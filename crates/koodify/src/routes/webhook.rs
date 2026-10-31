use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;
use client::{verify_webhook_signature, WebhookEvent};

use client::WebhookEventType;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_upstream, JsonResult};
use crate::services::receipt;
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
/// `ZAINPAY_WEBHOOK_SECRET` is configured, persists the event to
/// `transactions`, and acknowledges with HTTP 200.
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
        return Err((StatusCode::UNAUTHORIZED, "invalid Zainpay-Signature".into()));
    }

    let event: WebhookEvent = serde_json::from_slice(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid webhook JSON: {e}"),
        )
    })?;

    let resource_key = event.data.idempotency_key().map(str::to_string);
    let zainbox_code = event.data.zainbox_code.clone();
    let payload = serde_json::to_value(&event).unwrap_or(Value::Null);

    if let Err(err) = db::transactions::insert(
        &state.pool,
        &format!("webhook.{}", event.event.as_str()),
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
            receipt::maybe_send_on_deposit_webhook(
                &state,
                txn_ref,
                amount,
                paid_at,
                event.data.narration.as_deref(),
                event.data.bank_name.as_deref(),
                event.data.sender.as_deref(),
                event.data.sender_name.as_deref(),
                event.data.zainbox_name.as_deref(),
            )
            .await;
        }
    }

    Ok("OK")
}
