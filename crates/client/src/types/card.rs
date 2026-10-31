use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeCardPaymentRequest {
    /// Amount in Naira (string), per Zainpay card docs.
    pub amount: String,
    pub txn_ref: String,
    pub zainbox_code: String,
    pub email_address: String,
    pub mobile_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_back_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_recurring_payment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringCardPurchaseRequest {
    pub amount: String,
    pub txn_ref: String,
    pub mobile_number: String,
    pub zainbox_code: String,
    pub email_address: String,
    pub call_back_url: String,
    pub card_token: String,
}
