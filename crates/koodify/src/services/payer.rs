//! Persist and re-attach payer contact + merchant `customParam` that Zainpay
//! does not echo.
//!
//! On payment init, koodify stores `{ email, phone, name? }` under
//! `operation_records.metadata.payer` (keyed by `txnRef`) and returns it as
//! `data.payer`. Optional merchant `customParam` / `customParams` objects are
//! stored as `metadata.customParam` and returned as `data.customParam`.
//! Status lookups by `txnRef` reload and merge the same objects.

use serde_json::{json, Map, Value};
use sqlx::PgPool;
use tracing::warn;

use crate::db;

/// Koodify-only fields stripped from the upstream body before deserialize.
const NAME_KEYS: &[&str] = &["name", "payerName", "fullName", "customerName"];

/// Pull merchant `customParam` / `customParams` off the Zainpay payload into
/// `metadata.customParam` (not forwarded upstream).
pub fn peel_custom_param(body: &mut Value, metadata: &mut Value) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };

    let incoming = obj
        .remove("customParam")
        .or_else(|| obj.remove("customParams"));

    let Some(incoming) = incoming else {
        return;
    };
    if incoming.is_null() {
        return;
    }

    let meta = match metadata.as_object_mut() {
        Some(m) => m,
        None => {
            *metadata = Value::Object(Map::new());
            metadata.as_object_mut().expect("object")
        }
    };

    match (meta.get_mut("customParam"), incoming) {
        (Some(Value::Object(existing)), Value::Object(incoming)) => {
            for (k, v) in incoming {
                existing.insert(k, v);
            }
        }
        (_, Value::Object(incoming)) => {
            meta.insert("customParam".into(), Value::Object(incoming));
        }
        (_, other) => {
            meta.insert("customParam".into(), other);
        }
    }
}

/// Pull `payer` / name aliases off the Zainpay payload into `metadata`.
///
/// Accepts:
/// - top-level `name` | `payerName` | `fullName` | `customerName`
/// - top-level `payer: { name, email, phone, … }`
pub fn peel_payer_fields(body: &mut Value, metadata: &mut Value) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };

    let meta = match metadata.as_object_mut() {
        Some(m) => m,
        None => {
            *metadata = Value::Object(Map::new());
            metadata.as_object_mut().expect("object")
        }
    };

    if let Some(payer) = obj.remove("payer") {
        match meta.get_mut("payer") {
            Some(Value::Object(existing)) => {
                if let Value::Object(incoming) = payer {
                    for (k, v) in incoming {
                        existing.insert(k, v);
                    }
                } else {
                    existing.insert("_payer".into(), payer);
                }
            }
            _ => {
                meta.insert("payer".into(), payer);
            }
        }
    }

    for key in NAME_KEYS {
        if let Some(v) = obj.remove(*key) {
            if !v.is_null() {
                // Prefer first non-empty name into metadata.payer.name
                let name_str = v.as_str().map(str::trim).unwrap_or_default();
                if !name_str.is_empty() {
                    let payer = meta
                        .entry("payer")
                        .or_insert_with(|| Value::Object(Map::new()));
                    if let Value::Object(p) = payer {
                        p.entry("name").or_insert(Value::String(name_str.to_string()));
                    }
                }
            }
        }
    }
}

/// Build a canonical payer object from the Zainpay request + metadata.
pub fn build_payer(request: &Value, metadata: &Value) -> Value {
    let mut payer = Map::new();

    if let Some(Value::Object(existing)) = metadata.get("payer") {
        for (k, v) in existing {
            payer.insert(k.clone(), v.clone());
        }
    }

    // Official Zainpay contact fields win when present
    if let Some(email) = first_str(
        request,
        &["emailAddress", "email"],
    ) {
        payer.insert("email".into(), Value::String(email));
    }
    if let Some(phone) = first_str(
        request,
        &["mobileNumber", "phone", "phoneNumber"],
    ) {
        payer.insert("phone".into(), Value::String(phone));
    }

    // Name only from metadata / peeled fields / optional request aliases
    if !payer.contains_key("name") {
        if let Some(name) = first_str(metadata, NAME_KEYS) {
            payer.insert("name".into(), Value::String(name));
        } else if let Some(name) = first_str(request, NAME_KEYS) {
            payer.insert("name".into(), Value::String(name));
        }
    }

    // Normalize nested payer.email / payer.phone aliases
    if let Some(email) = payer
        .get("emailAddress")
        .and_then(|v| v.as_str())
        .map(str::to_string)
    {
        payer.entry("email").or_insert(Value::String(email));
        payer.remove("emailAddress");
    }
    if let Some(phone) = payer
        .get("mobileNumber")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            payer
                .get("phoneNumber")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
    {
        payer.entry("phone").or_insert(Value::String(phone));
        payer.remove("mobileNumber");
        payer.remove("phoneNumber");
    }

    Value::Object(payer)
}

/// Ensure `metadata.payer` is set from request + existing metadata.
pub fn attach_payer_to_metadata(request: &Value, metadata: Value) -> (Value, Value) {
    let payer = build_payer(request, &metadata);
    let mut meta = match metadata {
        Value::Object(m) => m,
        other => {
            let mut m = Map::new();
            if !other.is_null() {
                m.insert("_metadata".into(), other);
            }
            m
        }
    };
    if payer.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
        meta.insert("payer".into(), payer.clone());
    }
    (Value::Object(meta), payer)
}

/// Insert `data.payer` (alongside any existing `data.metadata`).
pub fn merge_payer_into_response(mut response: Value, payer: Value) -> Value {
    if payer.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return response;
    }
    insert_data_field(&mut response, "payer", payer);
    response
}

/// Insert `data.customParam` (alongside any existing `data.metadata`).
pub fn merge_custom_param_into_response(mut response: Value, custom_param: Value) -> Value {
    if custom_param.is_null() {
        return response;
    }
    if custom_param
        .as_object()
        .map(|o| o.is_empty())
        .unwrap_or(false)
    {
        return response;
    }
    insert_data_field(&mut response, "customParam", custom_param);
    response
}

fn insert_data_field(response: &mut Value, key: &str, value: Value) {
    match response.get_mut("data") {
        Some(Value::Object(map)) => {
            map.insert(key.into(), value);
        }
        Some(Value::Null) => {
            response["data"] = json!({ key: value });
        }
        Some(other) => {
            let wrapped = other.clone();
            response["data"] = json!({ "value": wrapped, key: value });
        }
        None => {
            if let Some(map) = response.as_object_mut() {
                map.insert(key.into(), value);
            }
        }
    }
}

/// Load the most recent stored payer for a txnRef / resource key.
pub async fn load_payer(pool: &PgPool, resource_key: &str) -> Option<Value> {
    let records = match db::operations::find_by_resource_key(pool, resource_key).await {
        Ok(r) => r,
        Err(err) => {
            warn!(error = %err, resource_key, "failed to load payer for resource");
            return None;
        }
    };

    for rec in records {
        if let Some(payer) = rec.metadata.get("payer") {
            if payer.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
                return Some(payer.clone());
            }
        }
        if let Some(req) = rec.request_body.as_ref() {
            let payer = build_payer(req, &rec.metadata);
            if payer.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
                return Some(payer);
            }
        }
    }
    None
}

/// Load the most recent stored `customParam` for a txnRef / resource key.
pub async fn load_custom_param(pool: &PgPool, resource_key: &str) -> Option<Value> {
    let records = match db::operations::find_by_resource_key(pool, resource_key).await {
        Ok(r) => r,
        Err(err) => {
            warn!(error = %err, resource_key, "failed to load customParam for resource");
            return None;
        }
    };

    for rec in records {
        if let Some(custom) = rec
            .metadata
            .get("customParam")
            .or_else(|| rec.metadata.get("customParams"))
        {
            if !custom.is_null()
                && custom
                    .as_object()
                    .map(|o| !o.is_empty())
                    .unwrap_or(true)
            {
                return Some(custom.clone());
            }
        }
    }
    None
}

fn attach_into_data_metadata(response: &mut Value, key: &str, value: Value) {
    if let Some(Value::Object(data)) = response.get_mut("data") {
        match data.get_mut("metadata") {
            Some(Value::Object(meta)) => {
                meta.insert(key.into(), value);
            }
            None => {
                data.insert("metadata".into(), json!({ key: value }));
            }
            Some(_) => {}
        }
    }
}

/// After a Zainpay status call, attach stored payer + customParam for `txn_ref`.
pub async fn enrich_response_from_db(pool: &PgPool, txn_ref: &str, response: Value) -> Value {
    let mut response = response;

    if let Some(payer) = load_payer(pool, txn_ref).await {
        response = merge_payer_into_response(response, payer.clone());
        attach_into_data_metadata(&mut response, "payer", payer);
    }

    if let Some(custom_param) = load_custom_param(pool, txn_ref).await {
        response = merge_custom_param_into_response(response, custom_param.clone());
        attach_into_data_metadata(&mut response, "customParam", custom_param);
    }

    response
}

fn first_str(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(s) = value.get(*key).and_then(|v| v.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peels_name_and_builds_payer() {
        let mut body = json!({
            "amount": "500",
            "txnRef": "T1",
            "emailAddress": "a@b.c",
            "mobileNumber": "0801",
            "name": "Ada Lovelace",
            "zainboxCode": "Z"
        });
        let mut metadata = json!({});
        peel_payer_fields(&mut body, &mut metadata);
        assert!(body.get("name").is_none());
        assert_eq!(metadata["payer"]["name"], "Ada Lovelace");

        let (meta, payer) = attach_payer_to_metadata(&body, metadata);
        assert_eq!(payer["email"], "a@b.c");
        assert_eq!(payer["phone"], "0801");
        assert_eq!(payer["name"], "Ada Lovelace");
        assert_eq!(meta["payer"]["name"], "Ada Lovelace");
    }

    #[test]
    fn merges_payer_into_object_data() {
        let response = json!({ "code": "00", "data": { "paymentStatus": "success" } });
        let out = merge_payer_into_response(
            response,
            json!({ "email": "a@b.c", "phone": "0801", "name": "Ada" }),
        );
        assert_eq!(out["data"]["payer"]["email"], "a@b.c");
        assert_eq!(out["data"]["paymentStatus"], "success");
    }

    #[test]
    fn peels_and_merges_custom_param() {
        let mut body = json!({
            "amount": "500",
            "txnRef": "T1",
            "emailAddress": "a@b.c",
            "mobileNumber": "0801",
            "zainboxCode": "Z",
            "customParam": {
                "matric": "ND/12/2211",
                "session": "2026",
                "paymentType": "School Fees",
                "studentId": "STU001"
            }
        });
        let mut metadata = json!({});
        peel_custom_param(&mut body, &mut metadata);
        assert!(body.get("customParam").is_none());
        assert_eq!(metadata["customParam"]["matric"], "ND/12/2211");
        assert_eq!(metadata["customParam"]["studentId"], "STU001");

        let response = json!({ "code": "00", "data": { "paymentStatus": "success" } });
        let out = merge_custom_param_into_response(response, metadata["customParam"].clone());
        assert_eq!(out["data"]["customParam"]["paymentType"], "School Fees");
    }
}
