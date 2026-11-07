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

use crate::auth::{AdminAuth, ApiKeyAuth};
use crate::db;
use crate::metadata::CreateVirtualAccountInbound;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::proxy::{self, merge_metadata_into_response};
use crate::services::virtual_account::{self, CreateError, StoredVirtualAccount};
use crate::state::AppState;
use client::CreateVirtualAccountRequest;
use client::ApiResponse;
use client::VirtualAccountData;

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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(inbound): Json<CreateVirtualAccountInbound>,
) -> JsonResult {
    let (mut request, metadata) = inbound.split();
    resolve_zainbox_for_create(&auth, &mut request)?;
    apply_va_identity_defaults(&state.config, &mut request);

    let forced_bank = {
        let t = request.bank_type.trim().trim_matches('"');
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };

    let response = if let Some(bank) = forced_bank {
        request.bank_type = bank;
        state
            .zainpay
            .virtual_accounts()
            .create(&request)
            .await
            .map_err(map_upstream)?
    } else {
        create_with_adaptive_bank(&state, &request).await?
    };

    let zainbox_code = request.zainbox_code.clone();
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

/// Merchants may omit BVN / DOB / gender — fill Zainpay-compatible defaults.
fn apply_va_identity_defaults(config: &crate::config::Config, request: &mut CreateVirtualAccountRequest) {
    if request.bvn.trim().is_empty() {
        request.bvn = config.va_default_bvn.clone();
    }
    if request.dob.trim().is_empty() {
        request.dob = config.va_default_dob.clone();
    }
    if request.gender.trim().is_empty() {
        request.gender = config.va_default_gender.clone();
    }
}

fn resolve_zainbox_for_create(
    auth: &ApiKeyAuth,
    request: &mut CreateVirtualAccountRequest,
) -> Result<(), (StatusCode, String)> {
    let provided = request.zainbox_code.trim();
    if let Some(merchant) = auth.merchant() {
        if provided.is_empty() {
            request.zainbox_code = merchant.zainbox_code.clone();
            return Ok(());
        }
        auth.ensure_zainbox(provided)?;
        request.zainbox_code = provided.to_string();
        return Ok(());
    }

    // Platform admin must still name the target zainbox (multi-tenant ops).
    if provided.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "zainboxCode is required when using the platform admin API key".into(),
        ));
    }
    request.zainbox_code = provided.to_string();
    Ok(())
}

async fn create_with_adaptive_bank(
    state: &AppState,
    base: &CreateVirtualAccountRequest,
) -> Result<ApiResponse<VirtualAccountData>, (StatusCode, String)> {
    let order = state.bank_router.attempt_order();
    let mut last_http_err: Option<client::ZainpayError> = None;
    let mut last_app_fail: Option<ApiResponse<VirtualAccountData>> = None;

    for bank in order {
        let mut attempt = base.clone();
        attempt.bank_type = bank.clone();
        let started = std::time::Instant::now();
        match state.zainpay.virtual_accounts().create(&attempt).await {
            Ok(resp) if resp.is_success() => {
                state
                    .bank_router
                    .record_outcome(&bank, true, started.elapsed());
                tracing::info!(%bank, "static VA issued via bank router");
                return Ok(resp);
            }
            Ok(resp) => {
                state
                    .bank_router
                    .record_outcome(&bank, false, started.elapsed());
                warn!(
                    bank = %bank,
                    code = %resp.code,
                    description = %resp.description,
                    "static VA bank attempt failed at application layer — trying next"
                );
                last_app_fail = Some(resp);
            }
            Err(err) => {
                state
                    .bank_router
                    .record_outcome(&bank, false, started.elapsed());
                warn!(
                    bank = %bank,
                    error = %err,
                    "static VA bank attempt failed at HTTP layer — trying next"
                );
                last_http_err = Some(err);
            }
        }
    }

    if let Some(body) = last_app_fail {
        return Ok(body);
    }
    Err(map_upstream(last_http_err.unwrap_or_else(|| {
        client::ZainpayError::UnexpectedStatus {
            status: 502,
            body: "no VA bank candidates available".into(),
        }
    })))
}

pub async fn create_dynamic(
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
    _auth: AdminAuth,
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
