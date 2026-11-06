use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use client::CreateSettlementRequest;

use crate::auth::{AdminAuth, ApiKeyAuth};
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::proxy;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementQuery {
    pub zainbox_code: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    pub count: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,
}

pub async fn create(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<CreateSettlementRequest, _>(
        &state,
        "settlement.create",
        body,
        |req| async move { zainpay.settlements().create(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn get(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<SettlementQuery>,
) -> JsonResult {
    auth.ensure_zainbox(&q.zainbox_code)?;
    state
        .zainpay
        .settlements()
        .get(&q.zainbox_code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn history(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
    state
        .zainpay
        .settlements()
        .history(
            &code,
            q.count.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
            q.status.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn internal_account(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> JsonResult {

    auth.ensure_zainbox(&code)?;
    state
        .zainpay
        .settlements()
        .internal_account(&code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}
