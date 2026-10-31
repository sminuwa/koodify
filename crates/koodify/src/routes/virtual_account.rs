use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;
use client::{
    CreateDynamicVirtualAccountRequest, FundVirtualAccountRequest, UpdateVirtualAccountBvnRequest,
    UpdateVirtualAccountStatusRequest,
};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::metadata::CreateVirtualAccountInbound;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::proxy::{self, merge_metadata_into_response};
use crate::services::virtual_account::{self, CreateError, StoredVirtualAccount};
use crate::state::AppState;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TxnQuery {
    pub count: Option<String>,
    pub txn_type: Option<String>,
    pub payment_channel: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub async fn create(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(inbound): Json<CreateVirtualAccountInbound>,
) -> JsonResult {
    let (request, metadata) = inbound.split();
    let zainbox_code = request.zainbox_code.clone();

    let response = state
        .zainpay
        .virtual_accounts()
        .create(&request)
        .await
        .map_err(map_upstream)?;

    let mut response_json = serde_json::to_value(&response).unwrap_or(Value::Null);

    if response.is_success() {
        if let Some(ref data) = response.data {
            if let Err(err) = db::virtual_accounts::insert_success(
                &state.pool,
                &zainbox_code,
                &metadata,
                &response,
                data,
            )
            .await
            {
                warn!(error = %err, "failed to persist virtual account row");
            }
        }

        let request_json = serde_json::to_value(&request).unwrap_or(Value::Null);
        let resource_key = response
            .data
            .as_ref()
            .map(|d| d.account_number.clone());
        if let Err(err) = db::transactions::insert(
            &state.pool,
            "virtual_account.create",
            resource_key.as_deref(),
            Some(zainbox_code.as_str()),
            &metadata,
            Some(&request_json),
            &response_json,
            &response.code,
        )
        .await
        {
            warn!(error = %err, "failed to persist transaction");
        }

        response_json = merge_metadata_into_response(response_json, metadata);
    }

    Ok(Json(response_json))
}

pub async fn create_dynamic(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<CreateDynamicVirtualAccountRequest, _>(
        &state,
        "virtual_account.create_dynamic",
        body,
        |req| async move { zainpay.virtual_accounts().create_dynamic(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn balance(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
) -> JsonResult {
    virtual_account::balance_with_metadata(&state, &account_number)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(|e| match e {
            CreateError::Upstream(err) => map_upstream(err),
        })
}

pub async fn get_one(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
) -> Result<Json<StoredVirtualAccount>, (StatusCode, String)> {
    match virtual_account::get_by_account_number(&state, &account_number).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "virtual account not found".into())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn update_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<UpdateVirtualAccountStatusRequest, _>(
        &state,
        "virtual_account.update_status",
        body,
        |req| async move { zainpay.virtual_accounts().update_status(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn update_bvn(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<UpdateVirtualAccountBvnRequest, _>(
        &state,
        "virtual_account.update_bvn",
        body,
        |req| async move { zainpay.virtual_accounts().update_bvn(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn transactions(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    state
        .zainpay
        .virtual_accounts()
        .transactions(
            &account_number,
            q.count.as_deref(),
            q.txn_type.as_deref(),
            q.payment_channel.as_deref(),
            q.date_from.as_deref(),
            q.date_to.as_deref(),
        )
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
}

pub async fn dynamic_deposit_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    let response = state
        .zainpay
        .virtual_accounts()
        .dynamic_deposit_status(&txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    Ok(Json(
        crate::services::payer::enrich_response_from_db(&state.pool, &txn_ref, response).await,
    ))
}

pub async fn fund_static(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<FundVirtualAccountRequest, _>(
        &state,
        "virtual_account.fund_static",
        body,
        |req| async move { zainpay.virtual_accounts().fund_static(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

pub async fn fund_dynamic(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<FundVirtualAccountRequest, _>(
        &state,
        "virtual_account.fund_dynamic",
        body,
        |req| async move { zainpay.virtual_accounts().fund_dynamic(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}
