use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::db::models::TransactionRecord;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// Optional zainbox business key filter.
    pub zainbox_code: Option<String>,
}

/// Lookup stored transactions by resource key (txnRef, accountNumber, …).
///
/// Optional `?zainboxCode=` scopes to that zainbox business key.
pub async fn by_resource_key(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(resource_key): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<TransactionRecord>>, (StatusCode, String)> {
    let zb_filter = if let Some(m) = auth.merchant() {
        // Merchants are locked to their own zainbox.
        if let Some(requested) = q
            .zainbox_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            auth.ensure_zainbox(requested)?;
        }
        Some(m.zainbox_code.as_str())
    } else {
        q.zainbox_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    };

    let rows = if let Some(zb) = zb_filter {
        db::transactions::find_by_zainbox_and_resource(&state.pool, zb, &resource_key).await
    } else {
        db::transactions::find_by_resource_key(&state.pool, &resource_key).await
    };
    rows.map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// List transactions for a zainbox business key.
pub async fn by_zainbox_code(
    auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(zainbox_code): Path<String>,
) -> Result<Json<Vec<TransactionRecord>>, (StatusCode, String)> {
    auth.ensure_zainbox(&zainbox_code)?;
    db::transactions::find_by_zainbox_code(&state.pool, &zainbox_code)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
