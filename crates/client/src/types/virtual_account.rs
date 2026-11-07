use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Accept JSON string or number into a `String` (Zainpay samples mix both).
pub fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(D::Error::custom(format!(
            "expected string or number, got {other}"
        ))),
    }
}

/// Serialize a numeric string as a JSON number (Zainpay DVA docs use `duration: 900`).
pub fn serialize_string_as_u64<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let n: u64 = value.trim().parse().map_err(|_| {
        serde::ser::Error::custom(format!("expected unsigned integer string, got {value:?}"))
    })?;
    serializer.serialize_u64(n)
}

/// Request body for `POST /virtual-account/create/request`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVirtualAccountRequest {
    pub address: String,
    /// Optional for merchants — Koodify fills via Adaptive Bank/DVA Routing when empty.
    #[serde(default)]
    pub bank_type: String,
    /// Optional for merchants — Koodify fills a Zainpay-compatible default when empty.
    #[serde(default)]
    pub bvn: String,
    /// Optional for merchants — Koodify fills a default `DD-MM-YYYY` when empty.
    #[serde(default)]
    pub dob: String,
    pub email: String,
    pub first_name: String,
    /// Optional for merchants — Koodify fills `M`/`F` default when empty.
    #[serde(default)]
    pub gender: String,
    pub mobile_number: String,
    pub state: String,
    pub surname: String,
    pub title: String,
    /// Optional for merchants — taken from the `sk_…` credential scope when empty.
    #[serde(default)]
    pub zainbox_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
}

/// Successful create payload inside `ApiResponse.data`.
///
/// Fields default so Zainpay **error** envelopes with `data: {}` (or partial
/// objects) still deserialize; callers must check [`crate::types::envelope::ApiResponse::is_success`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountData {
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub bank_name: String,
    #[serde(default)]
    pub email: String,
}

/// Balance payload for `GET /virtual-account/wallet/balance/v2/{accountNumber}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountBalance {
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub account_name: Option<String>,
    /// Some responses use `balance` (string); live often uses `balanceAmount`.
    #[serde(default)]
    pub balance: Option<serde_json::Value>,
    #[serde(default)]
    pub balance_amount: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_code: Option<String>,
    #[serde(default)]
    pub bank_type: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Request body for `POST /virtual-account/dynamic/create/request`.
///
/// Per Zainpay docs: amount is a kobo **string**; duration is a **number** (300–259200);
/// `accountName` is fixed as `"Zainpay Checkout"`. BVN / DOB / gender are **not** used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDynamicVirtualAccountRequest {
    /// Optional for merchants — Koodify Adaptive Bank/DVA Routing when empty.
    #[serde(default)]
    pub bank_type: String,
    #[serde(default)]
    pub email: String,
    /// Amount in kobo (string or number accepted from JSON; always sent as string).
    #[serde(default, deserialize_with = "deserialize_stringish")]
    pub amount: String,
    /// Optional for merchants — taken from the `sk_…` credential when empty.
    #[serde(default)]
    pub zainbox_code: String,
    #[serde(default)]
    pub txn_ref: String,
    /// Validity seconds (string or number on input). Serialized as a JSON number.
    #[serde(
        default,
        deserialize_with = "deserialize_stringish",
        serialize_with = "serialize_string_as_u64"
    )]
    pub duration: String,
    /// Fixed by Zainpay as `"Zainpay Checkout"` — Koodify overwrites empties/custom values.
    #[serde(default)]
    pub account_name: String,
    /// Optional when Koodify has `PUBLIC_BASE_URL` (filled with inbound webhook).
    #[serde(default)]
    pub call_back_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualAccountStatusRequest {
    pub zainbox_code: String,
    pub account_number: String,
    /// `true` = active, `false` = inactive (per Zainpay docs).
    pub status: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVirtualAccountBvnRequest {
    pub zainbox_code: String,
    pub account_number: String,
    pub bvn: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundVirtualAccountRequest {
    pub amount: String,
    pub account_number: String,
}
