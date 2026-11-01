//! Admin trigger for the pending-checkout reconciler.

use axum::extract::State;
use axum::Json;
use serde_json::json;

use crate::auth::ApiKeyAuth;
use crate::routes::helpers::JsonResult;
use crate::services::reconcile;
use crate::state::AppState;

/// `POST /api/v1/reconcile/pending` — run one reconcile pass now (secret key).
pub async fn run_pending(_auth: ApiKeyAuth, State(state): State<AppState>) -> JsonResult {
    let stats = reconcile::run_once(&state).await;
    Ok(Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": {
            "scanned": stats.scanned,
            "polled": stats.polled,
            "succeeded": stats.succeeded,
            "errors": stats.errors,
        }
    })))
}
