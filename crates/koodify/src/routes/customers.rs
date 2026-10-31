use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::ApiKeyAuth;
use crate::db;
use crate::db::customers::{CreateCustomer, UpdateCustomer};
use crate::db::models::Customer;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomerRequest {
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default = "default_status")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerRequest {
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".into()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCustomersQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// `POST /customers` — create a customer (`id` auto-assigned: 1, 2, 3, …).
pub async fn create(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.display_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "displayName is required".into()));
    }

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    db::customers::create(
        &state.pool,
        CreateCustomer {
            display_name: body.display_name.trim(),
            email: body.email.as_deref().map(str::trim),
            mobile_number: body.mobile_number.as_deref().map(str::trim),
            zainbox_code: body.zainbox_code.as_deref().map(str::trim),
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    .map(Json)
    .map_err(map_db)
}

/// `PUT /customers/{id}`
pub async fn update(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.display_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "displayName is required".into()));
    }

    let metadata = match body.metadata {
        Value::Null => json!({}),
        other => other,
    };

    match db::customers::update(
        &state.pool,
        id,
        UpdateCustomer {
            display_name: body.display_name.trim(),
            email: body.email.as_deref().map(str::trim),
            mobile_number: body.mobile_number.as_deref().map(str::trim),
            zainbox_code: body.zainbox_code.as_deref().map(str::trim),
            metadata: &metadata,
            status: body.status.trim(),
        },
    )
    .await
    {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `GET /customers?status=`
pub async fn list(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Query(q): Query<ListCustomersQuery>,
) -> Result<Json<Vec<Customer>>, (StatusCode, String)> {
    db::customers::list(&state.pool, q.status.as_deref())
        .await
        .map(Json)
        .map_err(map_db)
}

/// `GET /customers/{id}`
pub async fn get_one(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    match db::customers::find_by_id(&state.pool, id).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}

/// `PATCH /customers/{id}/status`
pub async fn update_status(
    _auth: ApiKeyAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    if body.status.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "status is required".into()));
    }
    match db::customers::set_status(&state.pool, id, body.status.trim()).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "customer not found".into())),
        Err(err) => Err(map_db(err)),
    }
}
