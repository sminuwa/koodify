use axum::http::StatusCode;
use serde_json::Value;

use crate::services::proxy::ProxyError;

pub fn map_proxy_error(err: ProxyError) -> (StatusCode, String) {
    match err {
        ProxyError::BadRequest(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        ProxyError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
        ProxyError::Upstream(e) => (StatusCode::BAD_GATEWAY, format!("zainpay upstream error: {e}")),
    }
}

pub fn map_upstream(err: client::ZainpayError) -> (StatusCode, String) {
    (StatusCode::BAD_GATEWAY, format!("zainpay upstream error: {err}"))
}

pub type JsonResult = Result<axum::Json<Value>, (StatusCode, String)>;
