use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::db::models::OperationRecord;
use crate::state::AppState;

/// Lookup stored metadata / operation history by resource key (txnRef, accountNumber, …).
pub async fn by_resource_key(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(resource_key): Path<String>,
) -> Result<Json<Vec<OperationRecord>>, (StatusCode, String)> {
    db::operations::find_by_resource_key(&state.pool, &resource_key)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
