//! Admin activity / audit log lookups.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::routes::helpers::JsonResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<i64>,
}

/// `GET /api/v1/activity/{txnRef}` — timeline for one payment.
pub async fn by_txn_ref(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(txn_ref): Path<String>,
    Query(q): Query<ActivityQuery>,
) -> JsonResult {
    let limit = q.limit.unwrap_or(100);
    let rows = db::activity_events::list_by_txn_ref(&state.pool, &txn_ref, limit)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load activity: {e}"),
            )
        })?;
    Ok(Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": {
            "txnRef": txn_ref,
            "events": rows,
        }
    })))
}

/// `GET /api/v1/activity` — recent events across the gateway.
pub async fn recent(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ActivityQuery>,
) -> JsonResult {
    let limit = q.limit.unwrap_or(50);
    let rows = db::activity_events::list_recent(&state.pool, limit)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load activity: {e}"),
            )
        })?;
    Ok(Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": rows,
    })))
}
