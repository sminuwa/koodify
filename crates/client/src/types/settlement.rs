use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAccountInput {
    pub bank_code: String,
    pub account_number: String,
    pub percentage: String,
    /// Optional fixed Naira amount (Zainpay accepts alongside percentage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSettlementRequest {
    pub name: String,
    pub schedule_type: String,
    pub schedule_period: String,
    pub zainbox_code: String,
    /// Live Zainpay expects a JSON boolean (`false` disables auto-settlement).
    pub status: bool,
    pub settlement_account_list: Vec<SettlementAccountInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
