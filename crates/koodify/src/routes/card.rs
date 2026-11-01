use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use client::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};
use tracing::warn;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::{map_proxy_error, map_upstream, JsonResult};
use crate::services::activity::SuccessSource;
use crate::services::{payer, payment_split, payment_success, proxy};
use crate::state::AppState;

fn take_str_field(body: &mut Value, keys: &[&str]) -> Option<String> {
    let obj = body.as_object_mut()?;
    for key in keys {
        if let Some(v) = obj.remove(*key) {
            if let Some(s) = v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Resolve merchant from publishable `publicKey` → active zainbox.
/// Injects `zainboxCode` for Zainpay and strips browser-only fields.
/// Open checkout requires `publicKey` (bare `zainboxCode` is rejected).
async fn resolve_checkout_merchant(
    pool: &sqlx::PgPool,
    body: &mut Value,
) -> Result<String, (StatusCode, String)> {
    let public_key = take_str_field(body, &["publicKey", "public_key"]);

    let Some(pk) = public_key else {
        // Strip legacy code so it cannot be forwarded as a silent identity.
        if let Some(obj) = body.as_object_mut() {
            obj.remove("zainboxCode");
            obj.remove("zainbox_code");
        }
        return Err((
            StatusCode::BAD_REQUEST,
            "publicKey is required for checkout".into(),
        ));
    };

    let zainbox = match db::zainboxes::get_by_public_key(pool, &pk).await {
        Ok(Some(z)) => z,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "invalid or unknown publicKey".into(),
            ));
        }
        Err(err) => {
            warn!(error = %err, "publicKey lookup failed");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to verify publicKey".into(),
            ));
        }
    };

    if !zainbox.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            "merchant publicKey is inactive".into(),
        ));
    }

    if let Some(obj) = body.as_object_mut() {
        obj.insert("zainboxCode".into(), json!(zainbox.zainbox_code));
        obj.remove("zainbox_code");
        obj.remove("publicKey");
        obj.remove("public_key");
    }

    Ok(zainbox.zainbox_code)
}

pub async fn initialize_payment(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(mut body): Json<Value>,
) -> JsonResult {
    let primary = body
        .get("zainboxCode")
        .or_else(|| body.get("zainbox_code"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let Some(primary) = primary else {
        return Err((
            StatusCode::BAD_REQUEST,
            "zainboxCode is required".into(),
        ));
    };
    // Normalize key for typed deserialize.
    if let Some(obj) = body.as_object_mut() {
        obj.insert("zainboxCode".into(), json!(primary));
        obj.remove("zainbox_code");
    }
    initialize_with_optional_split(&state, "card.initialize", primary, body, false).await
}

/// InlineJS / bank-transfer checkout — publishable `publicKey` → zainbox.
/// Logs payer against `txnRef`. Validates + schedules Zainbox Payment Split when requested.
pub async fn initialize_payment_inline(
    State(state): State<AppState>,
    Json(mut body): Json<Value>,
) -> JsonResult {
    let zainbox = resolve_checkout_merchant(&state.pool, &mut body).await?;
    initialize_with_optional_split(&state, "card.initialize_inline", zainbox, body, true).await
}

async fn initialize_with_optional_split(
    state: &AppState,
    operation: &str,
    primary_zainbox: String,
    body: Value,
    inline: bool,
) -> JsonResult {
    let (mut zainpay_body, mut metadata) = proxy::split_body(body);
    payer::peel_payer_fields(&mut zainpay_body, &mut metadata);
    payer::peel_custom_param(&mut zainpay_body, &mut metadata);
    payment_split::peel_split_fields(&mut zainpay_body, &mut metadata);

    // Ensure zainboxCode is present for typed deserialize (resolve already injected it).
    if zainpay_body.get("zainboxCode").and_then(|v| v.as_str()).is_none() {
        if let Some(obj) = zainpay_body.as_object_mut() {
            obj.insert("zainboxCode".into(), json!(primary_zainbox));
        }
    }

    let amount = zainpay_body
        .get("amount")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let txn_ref = zainpay_body
        .get("txnRef")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let plan = match payment_split::plan_from_checkout(
        state,
        &primary_zainbox,
        &amount,
        &metadata,
        inline,
    )
    .await
    {
        Ok(p) => p,
        Err(err) => {
            return Err((StatusCode::BAD_REQUEST, err.to_string()));
        }
    };

    if let Some(ref plan) = plan {
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("paymentSplit".into(), plan.snapshot.clone());
        }
    }

    let request: InitializeCardPaymentRequest = serde_json::from_value(zainpay_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let zainpay = state.zainpay.clone();
    let response = if inline {
        proxy::mutate_with_metadata(
            state,
            operation,
            request,
            metadata,
            |req| async move { zainpay.cards().initialize_payment_inline(&req).await },
        )
        .await
    } else {
        proxy::mutate_with_metadata(
            state,
            operation,
            request,
            metadata,
            |req| async move { zainpay.cards().initialize_payment(&req).await },
        )
        .await
    }
    .map_err(map_upstream)?;

    if let Some(plan) = plan {
        if txn_ref.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "txnRef is required when paymentSplit / splitPlanId is set".into(),
            ));
        }
        if let Err(err) = payment_split::persist_pending(&state.pool, &txn_ref, &plan).await
        {
            warn!(error = %err, %txn_ref, "failed to persist payment split plan");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "payment initialized upstream but failed to store payment split plan — retry or contact support".into(),
            ));
        }
        let response =
            payment_split::enrich_response(&state.pool, &txn_ref, response).await;
        return Ok(Json(response));
    }

    Ok(Json(response))
}

pub async fn recurring_purchase(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> JsonResult {
    let zainpay = state.zainpay.clone();
    proxy::mutate_json_with_metadata::<RecurringCardPurchaseRequest, _>(
        &state,
        "card.recurring_purchase",
        body,
        |req| async move { zainpay.cards().recurring_purchase(&req).await },
    )
    .await
    .map(Json)
    .map_err(map_proxy_error)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempVaQuery {
    /// Optional override. When set, Koodify uses this bank exactly (legacy / debug).
    /// When omitted, the gateway auto-picks from `VA_BANK_CANDIDATES` with failover.
    pub bank_type: Option<String>,
    pub e: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempVaStatusQuery {
    pub txn_ref: String,
}

/// Continues a checkout session (`e` from initialize). Open — session key is the gate.
///
/// Careful compatibility:
/// - Explicit `bankType` → single attempt, same as before (no auto-switch).
/// - Omitted `bankType` → sticky preferred bank + failover across candidates.
pub async fn generate_temporary_virtual_account(
    State(state): State<AppState>,
    Query(q): Query<TempVaQuery>,
) -> JsonResult {
    let forced = q
        .bank_type
        .as_deref()
        .map(str::trim)
        .map(|s| s.trim_matches('"'))
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    if let Some(bank) = forced {
        // Preserve historical behavior for callers that still pass bankType.
        return state
            .zainpay
            .cards()
            .generate_temporary_virtual_account(&bank, &q.e)
            .await
            .map(|r| Json(serde_json::to_value(r).unwrap_or(Value::Null)))
            .map_err(map_upstream);
    }

    let order = state.bank_router.attempt_order();
    let mut last_http_err: Option<client::ZainpayError> = None;
    let mut last_app_fail: Option<Value> = None;

    for bank in order {
        let started = std::time::Instant::now();
        match state
            .zainpay
            .cards()
            .generate_temporary_virtual_account(&bank, &q.e)
            .await
        {
            Ok(resp) if resp.is_success() => {
                state
                    .bank_router
                    .record_outcome(&bank, true, started.elapsed());
                tracing::info!(%bank, "temporary VA issued via bank router");
                return Ok(Json(serde_json::to_value(resp).unwrap_or(Value::Null)));
            }
            Ok(resp) => {
                state
                    .bank_router
                    .record_outcome(&bank, false, started.elapsed());
                warn!(
                    bank = %bank,
                    code = %resp.code,
                    description = %resp.description,
                    "temporary VA bank attempt failed at application layer — trying next"
                );
                last_app_fail = Some(serde_json::to_value(resp).unwrap_or(Value::Null));
            }
            Err(err) => {
                state
                    .bank_router
                    .record_outcome(&bank, false, started.elapsed());
                warn!(
                    bank = %bank,
                    error = %err,
                    "temporary VA bank attempt failed at HTTP layer — trying next"
                );
                last_http_err = Some(err);
            }
        }
    }

    if let Some(body) = last_app_fail {
        return Ok(Json(body));
    }
    Err(map_upstream(last_http_err.unwrap_or_else(|| {
        client::ZainpayError::UnexpectedStatus {
            status: 502,
            body: "no VA bank candidates available".into(),
        }
    })))
}

/// Status poll for checkout (`txnRef`). Open — txnRef is the gate.
pub async fn temporary_virtual_account_payment_status(
    State(state): State<AppState>,
    Query(q): Query<TempVaStatusQuery>,
) -> JsonResult {
    let response = state
        .zainpay
        .cards()
        .temporary_virtual_account_payment_status(&q.txn_ref)
        .await
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(map_upstream)?;
    let enriched = payer::enrich_response_from_db(&state.pool, &q.txn_ref, response).await;
    let deposit_account = payment_success::deposit_account_from(&enriched);
    let deposit_bank = payment_success::deposit_bank_from(&enriched);
    let enriched = payment_success::apply_side_effects(
        &state,
        &q.txn_ref,
        &enriched,
        deposit_account,
        deposit_bank,
        SuccessSource::StatusPoll,
    )
    .await;
    Ok(Json(enriched))
}
