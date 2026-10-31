use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAccountInput {
    pub bank_code: String,
    pub account_number: String,
    pub percentage: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSettlementRequest {
    pub name: String,
    pub schedule_type: String,
    pub schedule_period: String,
    pub zainbox_code: String,
    pub status: String,
    pub settlement_account_list: Vec<SettlementAccountInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
