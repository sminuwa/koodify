use serde::Deserialize;
use serde_json::{Map, Value};
use client::CreateVirtualAccountRequest;

/// Inbound create payload supporting both nested and flat custom fields.
///
/// Nested form:
/// ```json
/// { "zainpay": { ... }, "metadata": { "customerId": "cus_1" } }
/// ```
///
/// Flat form (drop-in compatible with Zainpay's body):
/// ```json
/// { "bankType": "gtBank", "...": "...", "customerId": "cus_1" }
/// ```
/// Unknown top-level keys become `metadata`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CreateVirtualAccountInbound {
    Nested {
        zainpay: CreateVirtualAccountRequest,
        #[serde(default)]
        metadata: Value,
    },
    Flat(FlatCreateVirtualAccount),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatCreateVirtualAccount {
    pub address: String,
    /// Merchants may omit this — Koodify applies Adaptive Bank/DVA Routing.
    #[serde(default)]
    pub bank_type: String,
    pub bvn: String,
    pub dob: String,
    pub email: String,
    pub first_name: String,
    pub gender: String,
    pub mobile_number: String,
    pub state: String,
    pub surname: String,
    pub title: String,
    /// Merchants may omit this — filled from the `sk_…` credential's zainbox.
    #[serde(default)]
    pub zainbox_code: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    /// Explicit metadata object if the client sends one alongside flat fields.
    #[serde(default)]
    pub metadata: Option<Value>,
    /// Any remaining custom keys.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl CreateVirtualAccountInbound {
    pub fn split(self) -> (CreateVirtualAccountRequest, Value) {
        match self {
            Self::Nested { zainpay, metadata } => (zainpay, normalize_metadata(metadata)),
            Self::Flat(flat) => {
                let mut metadata = Map::new();
                if let Some(Value::Object(obj)) = flat.metadata {
                    metadata.extend(obj);
                } else if let Some(other) = flat.metadata {
                    metadata.insert("_metadata".into(), other);
                }
                metadata.extend(flat.extra);

                let req = CreateVirtualAccountRequest {
                    address: flat.address,
                    bank_type: flat.bank_type,
                    bvn: flat.bvn,
                    dob: flat.dob,
                    email: flat.email,
                    first_name: flat.first_name,
                    gender: flat.gender,
                    mobile_number: flat.mobile_number,
                    state: flat.state,
                    surname: flat.surname,
                    title: flat.title,
                    zainbox_code: flat.zainbox_code,
                    middle_name: flat.middle_name,
                };

                (req, Value::Object(metadata))
            }
        }
    }
}

fn normalize_metadata(value: Value) -> Value {
    match value {
        Value::Null => Value::Object(Map::new()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nested_payload_splits_cleanly() {
        let inbound: CreateVirtualAccountInbound = serde_json::from_value(json!({
            "zainpay": {
                "address": "123 Main Street, Lagos",
                "bankType": "gtBank",
                "bvn": "12345678901",
                "dob": "15-06-1990",
                "email": "muhammad.abdullahi@example.com",
                "firstName": "Muhammad",
                "gender": "M",
                "mobileNumber": "08012345678",
                "state": "Lagos",
                "surname": "Abdullahi",
                "title": "Mr",
                "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL"
            },
            "metadata": { "customerId": "cus_123", "storeId": "store_9" }
        }))
        .unwrap();

        let (req, meta) = inbound.split();
        assert_eq!(req.bank_type, "gtBank");
        assert_eq!(meta["customerId"], "cus_123");
        assert_eq!(meta["storeId"], "store_9");
    }

    #[test]
    fn flat_payload_collects_unknown_keys() {
        let inbound: CreateVirtualAccountInbound = serde_json::from_value(json!({
            "address": "123 Main Street, Lagos",
            "bankType": "gtBank",
            "bvn": "12345678901",
            "dob": "15-06-1990",
            "email": "muhammad.abdullahi@example.com",
            "firstName": "Muhammad",
            "gender": "M",
            "mobileNumber": "08012345678",
            "state": "Lagos",
            "surname": "Abdullahi",
            "title": "Mr",
            "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL",
            "customerId": "cus_123",
            "tags": ["vip"]
        }))
        .unwrap();

        let (req, meta) = inbound.split();
        assert_eq!(req.first_name, "Muhammad");
        assert_eq!(meta["customerId"], "cus_123");
        assert_eq!(meta["tags"], json!(["vip"]));
        // Ensure known fields did not leak into metadata.
        assert!(meta.get("bankType").is_none());
        assert!(meta.get("zainboxCode").is_none());
    }

    #[test]
    fn merchant_payload_may_omit_bank_and_zainbox() {
        let inbound: CreateVirtualAccountInbound = serde_json::from_value(json!({
            "address": "12 Computing Lane, Lagos",
            "bvn": "12345678901",
            "dob": "10-12-1990",
            "email": "ada@example.com",
            "firstName": "Ada",
            "gender": "F",
            "mobileNumber": "08012345678",
            "state": "Lagos",
            "surname": "Lovelace",
            "title": "Ms",
            "metadata": { "customerId": "cus_demo_001" }
        }))
        .unwrap();

        let (req, meta) = inbound.split();
        assert!(req.bank_type.is_empty());
        assert!(req.zainbox_code.is_empty());
        assert_eq!(meta["customerId"], "cus_demo_001");
    }
}
