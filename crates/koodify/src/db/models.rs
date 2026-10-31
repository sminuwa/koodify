use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VirtualAccountRecord {
    pub id: Uuid,
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub email: String,
    pub zainbox_code: String,
    pub zainpay_code: String,
    pub metadata: Value,
    pub raw_response: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OperationRecord {
    pub id: Uuid,
    pub operation: String,
    pub resource_key: Option<String>,
    pub metadata: Value,
    pub request_body: Option<Value>,
    pub response_body: Value,
    pub zainpay_code: String,
    pub created_at: DateTime<Utc>,
}

/// Customer registry.
/// - `uuid`: opaque technical id
/// - `id`: auto-increment integer 1, 2, 3, … (PK; child FKs use this)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub uuid: Uuid,
    pub id: i64,
    pub display_name: String,
    pub email: Option<String>,
    pub mobile_number: Option<String>,
    /// Zainpay zainbox `codeName` when this customer maps to a box.
    pub zainbox_code: Option<String>,
    pub metadata: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Customer KYC profile (`customer_id` → `customers.id`).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KycProfile {
    pub id: Uuid,
    pub customer_id: i64,
    pub title: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub surname: String,
    pub email: String,
    pub mobile_number: String,
    pub bvn: String,
    pub dob: String,
    pub gender: String,
    pub address: String,
    pub state: String,
    pub bank_type: Option<String>,
    pub zainbox_code: Option<String>,
    pub metadata: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Per-customer credential pair: one `test` and one `live`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub id: Uuid,
    pub customer_id: i64,
    /// `test` or `live`
    pub environment: String,
    pub public_key: String,
    pub private_key: String,
    pub metadata: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
