//! Uniform merchant-facing JSON envelopes.
//!
//! Upstream Zainpay uses `{ code, status, description, data: { … } }`.
//! Koodify flattens object `data` into the top level so clients never dig through
//! nested `body` / `data` for ordinary fields. Array `data` is kept as-is (list
//! endpoints). `zainboxCode` is never echoed to merchants.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;
use serde_json::{json, Value};

const ENVELOPE_KEYS: &[&str] = &["code", "status", "description"];

/// Flatten object `data` onto the root and strip `zainboxCode`.
pub fn shape_envelope(response: &mut Value) {
    redact_zainbox_codes(response);

    let Some(root) = response.as_object_mut() else {
        return;
    };

    let Some(data_value) = root.remove("data") else {
        return;
    };

    match data_value {
        Value::Object(data) => {
            for (key, value) in data {
                if ENVELOPE_KEYS.contains(&key.as_str()) {
                    continue;
                }
                // Prefer values already set at the root (e.g. route-level hoists).
                root.entry(key).or_insert(value);
            }
        }
        Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Bool(_) => {
            root.insert("data".into(), data_value);
        }
        Value::Null => {}
    }
}

fn redact_zainbox_codes(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("zainboxCode");
            map.remove("zainbox_code");
            for child in map.values_mut() {
                redact_zainbox_codes(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_zainbox_codes(item);
            }
        }
        _ => {}
    }
}

fn contains_zainbox_key(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.contains_key("zainboxCode")
                || map.contains_key("zainbox_code")
                || map.values().any(contains_zainbox_key)
        }
        Value::Array(items) => items.iter().any(contains_zainbox_key),
        _ => false,
    }
}

/// Axum middleware: reshape JSON envelopes consistently for all routes.
pub async fn shape_json_middleware(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if !content_type.contains("application/json") {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return (parts.status, "failed to read response body").into_response();
        }
    };

    let Ok(mut value) = serde_json::from_slice::<Value>(&bytes) else {
        return Response::from_parts(parts, Body::from(bytes));
    };

    if !value.is_object() {
        return Response::from_parts(parts, Body::from(bytes));
    }

    // Only reshape Zainpay-style envelopes or payloads that still leak zainboxCode.
    let looks_like_envelope = value.get("code").is_some() || value.get("data").is_some();
    if !looks_like_envelope && !contains_zainbox_key(&value) {
        return Response::from_parts(parts, Body::from(bytes));
    }

    shape_envelope(&mut value);
    let body_bytes = match serde_json::to_vec(&value) {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::from(bytes)),
    };

    parts.headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from(body_bytes.len() as u64),
    );
    Response::from_parts(parts, Body::from(body_bytes))
}

/// Build a uniform error envelope (plain-text errors can adopt this later).
#[allow(dead_code)]
pub fn error_envelope(http_status: u16, description: impl Into<String>) -> Value {
    let description = description.into();
    json!({
        "code": format!("{http_status}"),
        "status": http_status.to_string(),
        "description": description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_object_data_and_redacts_zainbox() {
        let mut response = json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": {
                "accountName": "Zainpay Checkout",
                "accountNumber": "9006043531",
                "paymentStatus": "pending",
                "timeToLive": 900,
                "txnRef": "DVA_1",
                "zainboxCode": "secret_box",
                "metadata": { "source": "sample" }
            }
        });
        shape_envelope(&mut response);
        assert_eq!(response["code"], "00");
        assert_eq!(response["accountNumber"], "9006043531");
        assert_eq!(response["paymentStatus"], "pending");
        assert_eq!(response["metadata"]["source"], "sample");
        assert!(response.get("data").is_none());
        assert!(response.get("zainboxCode").is_none());
    }

    #[test]
    fn keeps_array_data() {
        let mut response = json!({
            "code": "00",
            "description": "successful",
            "data": [{ "id": 1 }, { "id": 2 }]
        });
        shape_envelope(&mut response);
        assert!(response["data"].is_array());
        assert_eq!(response["data"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn preserves_existing_root_keys() {
        let mut response = json!({
            "code": "00",
            "txnRef": "ROOT_REF",
            "data": { "txnRef": "DATA_REF", "amount": "100" }
        });
        shape_envelope(&mut response);
        assert_eq!(response["txnRef"], "ROOT_REF");
        assert_eq!(response["amount"], "100");
    }
}
