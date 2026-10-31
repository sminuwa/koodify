use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::auth::ApiKeyAuth;
use crate::routes::helpers::{map_upstream, JsonResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameEnquiryQuery {
    pub bank_code: String,
    pub account_number: String,
}

pub async fn list(_auth: ApiKeyAuth, State(state): State<AppState>) -> JsonResult {
    state
        .zainpay
        .bank()
        .list()
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn name_enquiry(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<NameEnquiryQuery>,
) -> JsonResult {
    state
        .zainpay
        .bank()
        .name_enquiry(&q.bank_code, &q.account_number)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}
