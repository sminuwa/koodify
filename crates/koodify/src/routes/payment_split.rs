use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::JsonResult;
use crate::services::payment_split::{self, PaymentSplitInput, ShareInput};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSplitPlanBody {
    pub id: String,
    pub name: String,
    #[serde(alias = "primaryZainbox", alias = "primary_zainbox")]
    pub primary_zainbox_code: String,
    pub shares: Vec<ShareInput>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub metadata: Value,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPlansQuery {
    #[serde(alias = "primaryZainbox", alias = "primary_zainbox")]
    pub primary_zainbox_code: Option<String>,
}

/// `PUT /api/v1/split-plans` — create/update a reusable plan (secret key).
pub async fn upsert_plan(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<UpsertSplitPlanBody>,
) -> JsonResult {
    let id = body.id.trim();
    let name = body.name.trim();
    let primary = body.primary_zainbox_code.trim();
    if id.is_empty() || name.is_empty() || primary.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "id, name, and primaryZainboxCode are required".into(),
        ));
    }

    // Validate shape with a dummy positive amount (₦10,000).
    let input = PaymentSplitInput {
        primary_zainbox_code: Some(primary.to_string()),
        shares: body.shares.clone(),
    };
    let plan = payment_split::resolve_plan(primary, "10000", &input)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    payment_split::assert_boxes_active(&state.pool, &plan)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let shares_json = serde_json::to_value(&body.shares)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let metadata = if body.metadata.is_null() {
        json!({})
    } else {
        body.metadata
    };

    let stored = db::split_plans::upsert(
        &state.pool,
        id,
        name,
        primary,
        &shares_json,
        body.is_active,
        &metadata,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to save split plan: {e}"),
        )
    })?;

    Ok(Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": stored,
    })))
}

pub async fn get_plan(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> JsonResult {
    match db::split_plans::get(&state.pool, &id).await {
        Ok(Some(plan)) => Ok(Json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": plan,
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, format!("split plan `{id}` not found"))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to load split plan: {err}"),
        )),
    }
}

pub async fn list_plans(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListPlansQuery>,
) -> JsonResult {
    let Some(primary) = q
        .primary_zainbox_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Err((
            StatusCode::BAD_REQUEST,
            "primaryZainboxCode query param is required".into(),
        ));
    };
    match db::split_plans::list_for_primary(&state.pool, primary).await {
        Ok(plans) => Ok(Json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": plans,
        }))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to list split plans: {err}"),
        )),
    }
}

/// `GET /api/v1/payment-splits/{txnRef}` — run status + legs.
pub async fn get_payment_split(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    match db::payment_splits::summary_json(&state.pool, &txn_ref).await {
        Ok(Some(summary)) => Ok(Json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": summary,
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("no payment split for txnRef `{txn_ref}`"),
        )),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to load payment split: {err}"),
        )),
    }
}

/// `POST /api/v1/payment-splits/{txnRef}/run` — force execute a pending/failed/partial split.
pub async fn run_payment_split(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
) -> JsonResult {
    match payment_split::force_run(&state, &txn_ref).await {
        Ok(summary) => Ok(Json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": summary,
        }))),
        Err(err) => Err((StatusCode::BAD_REQUEST, err)),
    }
}
