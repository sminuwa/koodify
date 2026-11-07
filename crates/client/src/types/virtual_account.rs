use serde::{Deserialize, Serialize};

/// Request body for `POST /virtual-account/create/request`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVirtualAccountRequest {
    pub address: String,
    /// Optional for merchants — Koodify fills via Adaptive Bank/DVA Routing when empty.
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
    /// Optional for merchants — taken from the `sk_…` credential scope when empty.
    #[serde(default)]
    pub zainbox_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
}

/// Successful create payload inside `ApiResponse.data`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountData {
    pub account_name: String,
    pub account_number: String,
    #[serde(default)]
    pub account_type: String,
    pub bank_name: String,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDynamicVirtualAccountRequest {
    pub bank_type: String,
    pub email: String,
    /// Amount in kobo.
    pub amount: String,
    pub zainbox_code: String,
    pub txn_ref: String,
    pub duration: String,
    pub account_name: String,
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
