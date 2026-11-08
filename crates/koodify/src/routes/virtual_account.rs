use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
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

/// `GET /api/v1/vas` — list virtual accounts for the authenticated merchant zainbox.
///
/// Merchants never send `zainboxCode`; it is taken from the secret key.
/// Platform admin keys should use `GET /api/v1/zainbox/{code}/vas` instead.
pub async fn list_mine(auth: ApiKeyAuth, State(state): State<AppState>) -> JsonResult {
    let Some(merchant) = auth.merchant() else {
        return Err((
            StatusCode::BAD_REQUEST,
            "admin keys must call GET /api/v1/zainbox/{zainboxCode}/vas".into(),
        ));
    };

    state
        .zainpay
        .zainbox()
        .virtual_accounts(&merchant.zainbox_code)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(map_upstream)
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

    crate::services::merchant_response::shape_envelope(&mut response_json);
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

/// Zainpay DVA docs: fixed account name, duration 300–259200 (JSON number), public callBackUrl.
fn apply_dva_defaults(
    state: &AppState,
    request: &mut CreateDynamicVirtualAccountRequest,
    metadata: &mut Value,
) -> Result<(), (StatusCode, String)> {
    // Docs: "Account Name is fixed as Zainpay Checkout".
    request.account_name = "Zainpay Checkout".into();

    if request.duration.trim().is_empty() {
        request.duration = "900".into();
    }
    let duration_secs: u64 = request.duration.trim().parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "duration must be an integer number of seconds".into(),
        )
    })?;
    if !(300..=259_200).contains(&duration_secs) {
        return Err((
            StatusCode::BAD_REQUEST,
            "duration must be between 300 and 259200 seconds".into(),
        ));
    }
    request.duration = duration_secs.to_string();

    // Keep merchant callback in metadata for outbound webhooks, then point Zainpay
    // at Koodify inbound when configured (avoids localhost / unreachable URLs → 502).
    let merchant_callback = request.call_back_url.trim().to_string();
    if !merchant_callback.is_empty()
        && !state
            .config
            .is_inbound_zainpay_webhook_url(&merchant_callback)
    {
        if let Some(obj) = metadata.as_object_mut() {
            obj.entry("callBackUrl")
                .or_insert_with(|| json!(merchant_callback));
        }
    }
    if let Some(inbound) = state.config.inbound_zainpay_webhook_url() {
        request.call_back_url = inbound;
    } else if request.call_back_url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "callBackUrl is required (or set PUBLIC_BASE_URL so Koodify can fill the inbound webhook)".into(),
        ));
    }

    Ok(())
}

fn resolve_zainbox_for_create(
    auth: &ApiKeyAuth,
    request: &mut CreateVirtualAccountRequest,
) -> Result<(), (StatusCode, String)> {
    resolve_zainbox_code_for_auth(auth, &mut request.zainbox_code)
}

fn resolve_zainbox_code_for_auth(
    auth: &ApiKeyAuth,
    zainbox_code: &mut String,
) -> Result<(), (StatusCode, String)> {
    let provided = zainbox_code.trim();
    if let Some(merchant) = auth.merchant() {
        if provided.is_empty() {
            *zainbox_code = merchant.zainbox_code.clone();
            return Ok(());
        }
        auth.ensure_zainbox(provided)?;
        *zainbox_code = provided.to_string();
        return Ok(());
    }

    // Platform admin must still name the target zainbox (multi-tenant ops).
    if provided.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "zainboxCode is required when using the platform admin API key".into(),
        ));
    }
    *zainbox_code = provided.to_string();
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

/// `POST /api/v1/dva/create` — temporary/dynamic VA for a one-time amount.
///
/// Merchants authenticate with `sk_…`. They may omit `zainboxCode`, `bankType`,
/// `accountName`, `duration`, and `callBackUrl`. Zainpay does **not** take BVN /
/// DOB / gender on this endpoint (unlike permanent VA create).
pub async fn create_dynamic(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let (mut zainpay_body, mut metadata) = crate::services::proxy::split_body(body);
    crate::services::payer::peel_payer_fields(&mut zainpay_body, &mut metadata);
    crate::services::payer::peel_custom_param(&mut zainpay_body, &mut metadata);

    let mut request: CreateDynamicVirtualAccountRequest =
        serde_json::from_value(zainpay_body).map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                format!("invalid DVA create body: {err}"),
            )
        })?;

    resolve_zainbox_code_for_auth(&auth, &mut request.zainbox_code)?;
    apply_dva_defaults(&state, &mut request, &mut metadata)?;

    if request.email.trim().is_empty()
        || request.amount.trim().is_empty()
        || request.txn_ref.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "email, amount, and txnRef are required".into(),
        ));
    }

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
            .create_dynamic(&request)
            .await
            .map_err(map_upstream)?
    } else {
        create_dynamic_with_adaptive_bank(&state, &request).await?
    };

    let request_json = serde_json::to_value(&request).unwrap_or(Value::Null);
    let mut response_json = serde_json::to_value(&response).unwrap_or(Value::Null);

    if response.is_success() {
        let resource_key = response
            .data
            .as_ref()
            .and_then(|d| d.get("accountNumber").or_else(|| d.get("txnRef")))
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        if let Err(err) = db::transactions::insert(
            &state.pool,
            "virtual_account.create_dynamic",
            resource_key.as_deref(),
            Some(request.zainbox_code.as_str()),
            &metadata,
            Some(&request_json),
            &response_json,
            &response.code,
        )
        .await
        {
            warn!(error = %err, "failed to persist DVA transaction");
        }
        response_json = merge_metadata_into_response(response_json, metadata);
    }

    // Middleware also shapes; apply here so unit/proxy paths stay uniform without HTTP.
    let mut response_json = crate::services::api_error::enrich(response_json);
    crate::services::merchant_response::shape_envelope(&mut response_json);
    Ok(Json(response_json))
}

async fn create_dynamic_with_adaptive_bank(
    state: &AppState,
    base: &CreateDynamicVirtualAccountRequest,
) -> Result<client::ApiResponse<Value>, (StatusCode, String)> {
    let order = state.bank_router.attempt_order();
    let mut last_http_err: Option<client::ZainpayError> = None;
    let mut last_app_fail: Option<client::ApiResponse<Value>> = None;

    for bank in order {
        let mut attempt = base.clone();
        attempt.bank_type = bank.clone();
        let started = std::time::Instant::now();
        match state.zainpay.virtual_accounts().create_dynamic(&attempt).await {
            Ok(resp) if resp.is_success() => {
                state
                    .bank_router
                    .record_outcome(&bank, true, started.elapsed());
                tracing::info!(%bank, "DVA issued via bank router");
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
                    "DVA bank attempt failed at application layer — trying next"
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
                    "DVA bank attempt failed at HTTP layer — trying next"
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

async fn ensure_account_owned(
    auth: &ApiKeyAuth,
    state: &AppState,
    account_number: &str,
) -> Result<(), (StatusCode, String)> {
    if auth.is_admin() {
        return Ok(());
    }
    match virtual_account::get_by_account_number(state, account_number).await {
        Ok(Some(row)) => auth.ensure_zainbox(&row.zainbox_code),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            "virtual account not found (or not created via this gateway)".into(),
        )),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn balance(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
) -> JsonResult {
    ensure_account_owned(&auth, &state, &account_number).await?;
    virtual_account::balance_with_metadata(&state, &account_number)
        .await
        .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
        .map_err(|e| match e {
            CreateError::Upstream(err) => map_upstream(err),
        })
}

pub async fn get_one(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
) -> Result<Json<StoredVirtualAccount>, (StatusCode, String)> {
    match virtual_account::get_by_account_number(&state, &account_number).await {
        Ok(Some(row)) => {
            auth.ensure_zainbox(&row.zainbox_code)?;
            Ok(Json(row))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "virtual account not found".into())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn update_status(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(mut body): Json<Value>,
) -> JsonResult {
    auth.scope_zainbox_body(&mut body)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(mut body): Json<Value>,
) -> JsonResult {
    auth.scope_zainbox_body(&mut body)?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(account_number): Path<String>,
    Query(q): Query<TxnQuery>,
) -> JsonResult {
    ensure_account_owned(&auth, &state, &account_number).await?;
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let account = body
        .get("accountNumber")
        .or_else(|| body.get("account_number"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !account.is_empty() {
        ensure_account_owned(&auth, &state, account).await?;
    }
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
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let account = body
        .get("accountNumber")
        .or_else(|| body.get("account_number"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !account.is_empty() {
        ensure_account_owned(&auth, &state, account).await?;
    }
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

