//! Uniform merchant-facing JSON envelopes.
//!
//! Canonical shape for every successful API response:
//!
//! ```json
//! {
//!   "code": "00",
//!   "status": "200 OK",
//!   "description": "successful",
//!   "data": { /* resource fields + metadata */ }
//! }
//! ```
//!
//! List endpoints use `"data": [ … ]`. `zainboxCode` is never echoed to merchants.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;
use serde_json::{json, Map, Value};

const ENVELOPE_KEYS: &[&str] = &["code", "status", "description", "data"];

/// Keep `{ code, status, description, data }` and strip `zainboxCode`.
///
/// Any non-envelope fields that landed on the root (legacy / partial shapes) are
/// moved under `data` so merchants always read resource fields from one place.
pub fn shape_envelope(response: &mut Value) {
    redact_zainbox_codes(response);

    let Some(root) = response.as_object_mut() else {
        return;
    };

    // Collect stray top-level resource fields into `data`.
    let mut stray = Map::new();
    let keys: Vec<String> = root.keys().cloned().collect();
    for key in keys {
        if ENVELOPE_KEYS.contains(&key.as_str()) {
            continue;
        }
        if let Some(value) = root.remove(&key) {
            stray.insert(key, value);
        }
    }

    if stray.is_empty() {
        return;
    }

    match root.get_mut("data") {
        Some(Value::Object(data)) => {
            for (key, value) in stray {
                data.entry(key).or_insert(value);
            }
        }
        Some(Value::Null) | None => {
            root.insert("data".into(), Value::Object(stray));
        }
        Some(Value::Array(_)) => {
            // List envelope — keep array `data`; put extras alongside under metadata-ish wrap.
            // Prefer not to destroy list payloads: nest extras as sibling object only if empty array?
            // Leave list `data` alone; drop stray to avoid ambiguous shapes.
        }
        Some(other) => {
            let wrapped = other.clone();
            let mut data = Map::new();
            data.insert("value".into(), wrapped);
            for (key, value) in stray {
                data.entry(key).or_insert(value);
            }
            root.insert("data".into(), Value::Object(data));
        }
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

/// Axum middleware: enforce the canonical envelope on JSON responses.
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
        "data": null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_data_object_and_redacts_zainbox() {
        let mut response = json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": {
                "accountName": "Zainpay Checkout",
                "accountNumber": "9006043531",
                "paymentStatus": "pending",
                "zainboxCode": "secret_box",
                "metadata": { "source": "sample" }
            }
        });
        shape_envelope(&mut response);
        assert_eq!(response["code"], "00");
        assert_eq!(response["status"], "200 OK");
        assert_eq!(response["data"]["accountNumber"], "9006043531");
        assert_eq!(response["data"]["metadata"]["source"], "sample");
        assert!(response["data"].get("zainboxCode").is_none());
        assert!(response.get("accountNumber").is_none());
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
    fn nests_stray_root_fields_under_data() {
        let mut response = json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "accountNumber": "5006079921",
            "data": { "email": "ada@example.com" }
        });
        shape_envelope(&mut response);
        assert_eq!(response["data"]["accountNumber"], "5006079921");
        assert_eq!(response["data"]["email"], "ada@example.com");
        assert!(response.get("accountNumber").is_none());
    }
}
