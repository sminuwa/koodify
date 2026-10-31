use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;
use client::{verify_webhook_signature, WebhookEvent};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_upstream, JsonResult};
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
/// `operation_records`, and acknowledges with HTTP 200.
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
    let payload = serde_json::to_value(&event).unwrap_or(Value::Null);

    if let Err(err) = db::operations::insert(
        &state.pool,
        &format!("webhook.{}", event.event.as_str()),
        resource_key.as_deref(),
        &Value::Object(Default::default()),
        Some(&payload),
        &payload,
        "00",
    )
    .await
    {
        tracing::error!(error = %err, "failed to persist inbound webhook");
    }

    Ok("OK")
}
