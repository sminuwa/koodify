use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level webhook event envelope pushed to a Zainbox `callbackUrl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEvent {
    pub event: WebhookEventType,
    pub data: WebhookData,
}

/// Known webhook event names. Unknown values deserialize as [`WebhookEventType::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    #[serde(rename = "deposit.success")]
    DepositSuccess,
    #[serde(rename = "transfer.success")]
    TransferSuccess,
    #[serde(rename = "transfer.failed")]
    TransferFailed,
    #[serde(other)]
    Unknown,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DepositSuccess => "deposit.success",
            Self::TransferSuccess => "transfer.success",
            Self::TransferFailed => "transfer.failed",
            Self::Unknown => "unknown",
        }
    }
}

/// Union of fields seen across deposit and transfer webhook payloads.
///
/// Extra / future fields are preserved in [`WebhookData::extra`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookData {
    #[serde(default)]
    pub txn_ref: Option<String>,
    #[serde(default)]
    pub payment_ref: Option<String>,
    #[serde(default)]
    pub internal_txn_ref: Option<String>,
    #[serde(default)]
    pub txn_type: Option<String>,
    #[serde(default)]
    pub txn_date: Option<String>,
    #[serde(default)]
    pub payment_date: Option<String>,
    #[serde(default)]
    pub zainbox_code: Option<String>,
    #[serde(default)]
    pub zainbox_name: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub beneficiary_account_number: Option<String>,
    #[serde(default)]
    pub beneficiary_account_name: Option<String>,
    #[serde(default)]
    pub beneficiary_bank_code: Option<String>,
    #[serde(default)]
    pub bank_name: Option<String>,
    #[serde(default)]
    pub narration: Option<String>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub sender_name: Option<String>,
    #[serde(default)]
    pub deposited_amount: Option<String>,
    #[serde(default)]
    pub txn_charges_amount: Option<String>,
    #[serde(default)]
    pub amount_after_charges: Option<String>,
    #[serde(default)]
    pub amount: Option<WebhookAmount>,
    #[serde(default)]
    pub call_back_url: Option<String>,
    #[serde(default)]
    pub email_notification: Option<String>,
    #[serde(default)]
    pub card_token: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl WebhookData {
    /// Best-effort idempotency key: `txnRef`, else `paymentRef`, else `internalTxnRef`.
    pub fn idempotency_key(&self) -> Option<&str> {
        self.txn_ref
            .as_deref()
            .or(self.payment_ref.as_deref())
            .or(self.internal_txn_ref.as_deref())
    }
}

/// Nested amount object used on transfer webhooks (`{ "amount": 2500 }`, kobo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookAmount {
    #[serde(default)]
    pub amount: Option<serde_json::Value>,
}
