use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Map, Value};
use tracing::warn;
use client::{ApiResponse, ZainpayError};

use crate::db;
use crate::services::payer;
use crate::state::AppState;

/// Split inbound JSON into Zainpay payload + metadata.
///
/// Supports:
/// - `{ "zainpay": { ... }, "metadata": { ... } }`
/// - `{ ...fields..., "metadata": { ... } }` (metadata key stripped before upstream)
pub fn split_body(mut body: Value) -> (Value, Value) {
    let mut metadata = Map::new();

    if let Some(Value::Object(obj)) = body.as_object_mut().and_then(|o| o.remove("metadata")) {
        metadata.extend(obj);
    } else if let Some(other) = body.as_object_mut().and_then(|o| o.remove("metadata")) {
        metadata.insert("_metadata".into(), other);
    }

    if let Some(zainpay) = body.get("zainpay").cloned() {
        return (zainpay, Value::Object(metadata));
    }

    (body, Value::Object(metadata))
}

/// Merge metadata into a successful envelope's `data` object (or wrap non-objects).
pub fn merge_metadata_into_response(mut response: Value, metadata: Value) -> Value {
    match response.get_mut("data") {
        Some(Value::Object(map)) => {
            map.insert("metadata".into(), metadata);
        }
        Some(Value::Null) => {
            response["data"] = json!({ "metadata": metadata });
        }
        Some(other) => {
            let wrapped = other.clone();
            response["data"] = json!({ "value": wrapped, "metadata": metadata });
        }
        None => {
            if let Some(map) = response.as_object_mut() {
                map.insert("metadata".into(), metadata);
            }
        }
    }
    response
}

fn extract_resource_key(request: &Value, response: &Value) -> Option<String> {
    let candidates = [
        "txnRef",
        "accountNumber",
        "paymentRef",
        "codeName",
    ];
    for key in candidates {
        if let Some(v) = request.get(key).and_then(|x| x.as_str()) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        if let Some(v) = response
            .pointer(&format!("/data/{key}"))
            .and_then(|x| x.as_str())
        {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

/// Business key: zainbox `codeName` from request / response / metadata.
pub fn extract_zainbox_code(request: &Value, response: &Value, metadata: &Value) -> Option<String> {
    let candidates = [
        request.get("zainboxCode").and_then(|v| v.as_str()),
        request.get("codeName").and_then(|v| v.as_str()),
        response
            .pointer("/data/zainboxCode")
            .and_then(|v| v.as_str()),
        response
            .pointer("/data/codeName")
            .and_then(|v| v.as_str()),
        metadata.get("zainboxCode").and_then(|v| v.as_str()),
    ];
    for c in candidates.into_iter().flatten() {
        let t = c.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    None
}

/// Call an upstream mutating endpoint, persist metadata on success, merge into response.
pub async fn mutate_with_metadata<Req, Fut>(
    state: &AppState,
    operation: &str,
    request: Req,
    metadata: Value,
    call: impl FnOnce(Req) -> Fut,
) -> Result<Value, ZainpayError>
where
    Req: Serialize,
    Fut: std::future::Future<Output = Result<ApiResponse<Value>, ZainpayError>>,
{
    let request_json = serde_json::to_value(&request).unwrap_or(Value::Null);
    let (metadata, payer) = payer::attach_payer_to_metadata(&request_json, metadata);
    let response = call(request).await?;
    let mut response_json = serde_json::to_value(&response).unwrap_or(Value::Null);

    if response.is_success() {
        let resource_key = extract_resource_key(&request_json, &response_json);
        let zainbox_code = extract_zainbox_code(&request_json, &response_json, &metadata);
        if let Err(err) = db::transactions::insert(
            &state.pool,
            operation,
            resource_key.as_deref(),
            zainbox_code.as_deref(),
            &metadata,
            Some(&request_json),
            &response_json,
            &response.code,
        )
        .await
        {
            warn!(error = %err, operation, "failed to persist transaction");
        }
        let custom_param = metadata.get("customParam").cloned();
        response_json = merge_metadata_into_response(response_json, metadata);
        response_json = payer::merge_payer_into_response(response_json, payer);
        if let Some(custom_param) = custom_param {
            response_json =
                payer::merge_custom_param_into_response(response_json, custom_param);
        }
    }

    Ok(response_json)
}

/// Same as [`mutate_with_metadata`] but accepts already-split JSON body deserialized into `Req`.
pub async fn mutate_json_with_metadata<Req, Fut>(
    state: &AppState,
    operation: &str,
    body: Value,
    call: impl FnOnce(Req) -> Fut,
) -> Result<Value, ProxyError>
where
    Req: DeserializeOwned + Serialize,
    Fut: std::future::Future<Output = Result<ApiResponse<Value>, ZainpayError>>,
{
    let (mut zainpay_body, mut metadata) = split_body(body);
    payer::peel_payer_fields(&mut zainpay_body, &mut metadata);
    payer::peel_custom_param(&mut zainpay_body, &mut metadata);
    crate::services::payment_split::peel_split_fields(&mut zainpay_body, &mut metadata);
    let request: Req = serde_json::from_value(zainpay_body).map_err(ProxyError::BadRequest)?;
    mutate_with_metadata(state, operation, request, metadata, call)
        .await
        .map_err(ProxyError::Upstream)
}

/// Typed VA create uses a richer response type — convert to Value after call.
pub async fn mutate_typed_with_metadata<Req, Res, Fut>(
    state: &AppState,
    operation: &str,
    request: Req,
    metadata: Value,
    call: impl FnOnce(Req) -> Fut,
) -> Result<Value, ZainpayError>
where
    Req: Serialize,
    Res: Serialize,
    Fut: std::future::Future<Output = Result<ApiResponse<Res>, ZainpayError>>,
{
    let request_json = serde_json::to_value(&request).unwrap_or(Value::Null);
    let (metadata, payer) = payer::attach_payer_to_metadata(&request_json, metadata);
    let response = call(request).await?;
    let mut response_json = serde_json::to_value(&response).unwrap_or(Value::Null);

    if response.is_success() {
        let resource_key = extract_resource_key(&request_json, &response_json);
        let zainbox_code = extract_zainbox_code(&request_json, &response_json, &metadata);
        if let Err(err) = db::transactions::insert(
            &state.pool,
            operation,
            resource_key.as_deref(),
            zainbox_code.as_deref(),
            &metadata,
            Some(&request_json),
            &response_json,
            &response.code,
        )
        .await
        {
            warn!(error = %err, operation, "failed to persist transaction");
        }
        let custom_param = metadata.get("customParam").cloned();
        response_json = merge_metadata_into_response(response_json, metadata);
        response_json = payer::merge_payer_into_response(response_json, payer);
        if let Some(custom_param) = custom_param {
            response_json =
                payer::merge_custom_param_into_response(response_json, custom_param);
        }
    }

    Ok(response_json)
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("invalid request body: {0}")]
    BadRequest(#[from] serde_json::Error),
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Upstream(#[from] ZainpayError),
}
