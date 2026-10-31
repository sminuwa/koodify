use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundsTransferRequest {
    pub destination_account_number: String,
    pub destination_bank_code: String,
    pub source_account_number: String,
    pub source_bank_code: String,
    /// Amount in kobo (as string per Zainpay examples).
    pub amount: String,
    pub txn_ref: String,
    pub zainbox_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_back_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narration: Option<String>,
}
