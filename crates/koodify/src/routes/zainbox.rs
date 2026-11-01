use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};
use client::{CreateZainboxRequest, ServiceSubscriptionRequest, UpdateZainboxRequest};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::proxy;
use crate::state::AppState;

/// When `PUBLIC_BASE_URL` is set, force Zainpay `callbackUrl` to Koodify's inbound webhook.
fn apply_inbound_webhook_callback(state: &AppState, mut body: Value) -> Value {
    if let Some(url) = state.config.inbound_zainpay_webhook_url() {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("callbackUrl".into(), json!(url));
        }
    }
    body
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TxnQuery {
    pub count: Option<String>,
    pub txn_type: Option<String>,
    pub account_number: Option<String>,
    pub payment_channel: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,
    pub email: Option<String>,
    pub txn_ref: Option<String>,
    pub service_name: Option<String>,
}

pub async fn create(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let body = apply_inbound_webhook_callback(&state, body);
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<CreateZainboxRequest, _>(
        &state,
        "zainbox.create",
        body,
        |req| async move { zainpay.zainbox().create(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn list(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .list(q.status.as_deref())
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn update(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let body = apply_inbound_webhook_callback(&state, body);
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<UpdateZainboxRequest, _>(
        &state,
        "zainbox.update",
        body,
        |req| async move { zainpay.zainbox().update(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

/// Point every known zainbox's Zainpay `callbackUrl` at `POST /webhooks/zainpay`.
///
/// Requires `PUBLIC_BASE_URL`. Uses local `zainboxes` rows for code/name, then PATCHes Zainpay
/// and refreshes the local `callback_url` column.
pub async fn sync_webhook_callbacks(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
) -> JsonResult {
    let Some(webhook_url) = state.config.inbound_zainpay_webhook_url() else {
        return Err((
            StatusCode::BAD_REQUEST,
            "PUBLIC_BASE_URL is required to sync zainbox webhook callbacks".into(),
        ));
    };

    let boxes = db::zainboxes::list(&state.pool).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("list zainboxes: {err}"),
        )
    })?;

    let mut updated = Vec::new();
    let mut failed = Vec::new();
    let mut skipped = Vec::new();

    for zb in boxes {
        if zb
            .callback_url
            .as_deref()
            .is_some_and(|u| state.config.is_inbound_zainpay_webhook_url(u))
        {
            skipped.push(json!({
                "codeName": zb.code_name,
                "callbackUrl": zb.callback_url,
                "reason": "already_inbound",
            }));
            continue;
        }

        let req = UpdateZainboxRequest {
            code_name: zb.code_name.clone(),
            name: zb.name.clone(),
            callback_url: Some(webhook_url.clone()),
            email_notification: zb.email_notification.clone(),
            description: None,
            tags: zb.tags.clone(),
            allow_auto_internal_transfer: None,
            status: None,
        };

        match state.zainpay.zainbox().update(&req).await {
            Ok(resp) if resp.is_success() => {
                let raw = serde_json::to_value(&resp).unwrap_or(Value::Null);
                if let Err(err) = db::zainboxes::upsert(
                    &state.pool,
                    db::zainboxes::UpsertZainbox {
                        code_name: &zb.code_name,
                        name: &zb.name,
                        callback_url: Some(webhook_url.as_str()),
                        email_notification: zb.email_notification.as_deref(),
                        tags: zb.tags.as_deref(),
                        is_active: zb.is_active,
                        raw: &raw,
                    },
                )
                .await
                {
                    warn!(
                        error = %err,
                        code = %zb.code_name,
                        "zainbox webhook sync: Zainpay updated but local upsert failed"
                    );
                }
                info!(code = %zb.code_name, %webhook_url, "zainbox callbackUrl → inbound webhook");
                updated.push(json!({
                    "codeName": zb.code_name,
                    "callbackUrl": webhook_url,
                }));
            }
            Ok(resp) => {
                failed.push(json!({
                    "codeName": zb.code_name,
                    "code": resp.code,
                    "description": resp.description,
                }));
            }
            Err(err) => {
                failed.push(json!({
                    "codeName": zb.code_name,
                    "error": err.to_string(),
                }));
            }
        }
    }

    Ok(Json(json!({
        "code": "00",
        "description": "zainbox webhook callback sync finished",
        "data": {
            "webhookUrl": webhook_url,
            "updated": updated,
            "skipped": skipped,
            "failed": failed,
        }
    })))
}

pub async fn virtual_accounts(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .virtual_accounts(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn accounts_balance(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .accounts_balance(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn transactions(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .transactions(
            &code,
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.account_number.as_deref(),
            q.payment_channel.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn merchant_transactions(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .merchant_transactions(
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.account_number.as_deref(),
            q.payment_channel.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn deposit_summary(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .deposit_summary(&code, q.date_from.as_deref(), q.date_to.as_deref())
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn profile(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .profile(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn profile_v2(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .profile_v2(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn billing_config(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .billing_config(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn transactions_summary(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .transactions_summary(q.date_from.as_deref(), q.date_to.as_deref())
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn card_transactions(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .card_transactions(
            &code,
            q.count.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
            q.status.as_deref(),
            q.email.as_deref(),
            q.txn_ref.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn subscribe(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<ServiceSubscriptionRequest, _>(
        &state,
        "zainbox.subscribe",
        body,
        |req| async move { zainpay.zainbox().subscribe(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn unsubscribe(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<ServiceSubscriptionRequest, _>(
        &state,
        "zainbox.unsubscribe",
        body,
        |req| async move { zainpay.zainbox().unsubscribe(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn subscriptions_all(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .subscriptions_all(&code, q.service_name.as_deref())
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn subscriptions_active(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {
    state
        .zainpay
        .zainbox()
        .subscriptions_active(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}
