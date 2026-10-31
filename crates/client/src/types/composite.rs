use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::envelope::ApiResponse;

pub(crate) fn composite_ok<T>(description: &str, data: T) -> ApiResponse<T> {
    ApiResponse {
        code: "00".into(),
        status: "200 OK".into(),
        description: description.into(),
        data: Some(data),
    }
}

/// `profile` + `accounts/balance` for one zainbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxSnapshot {
    pub profile: ApiResponse<Value>,
    pub balances: ApiResponse<Value>,
}

/// Virtual accounts inventory + balances for one zainbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxWalletView {
    pub virtual_accounts: ApiResponse<Value>,
    pub balances: ApiResponse<Value>,
}

/// Transactions ledger + deposit summary for one zainbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxActivity {
    pub transactions: ApiResponse<Value>,
    pub deposit_summary: ApiResponse<Value>,
}

/// Settlement config GET + internal settlement account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementContext {
    pub settlement: ApiResponse<Value>,
    pub internal_account: ApiResponse<Value>,
}

/// VA balance + VA transactions for one account number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountWallet {
    pub balance: ApiResponse<Value>,
    pub transactions: ApiResponse<Value>,
}

/// Bank catalogue + name enquiry for one account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameResolve {
    pub banks: ApiResponse<Value>,
    pub name_enquiry: ApiResponse<Value>,
}

/// All subscriptions + active subscriptions for one zainbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionsBundle {
    pub all: ApiResponse<Value>,
    pub active: ApiResponse<Value>,
}

/// Zainbox profile + billing config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZainboxProfileBilling {
    pub profile: ApiResponse<Value>,
    pub billing_config: ApiResponse<Value>,
}

/// Merchant-wide transaction summary + transaction list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerchantActivity {
    pub summary: ApiResponse<Value>,
    pub transactions: ApiResponse<Value>,
}

/// Transfer network status + name enquiry (payout preflight).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutPrep {
    pub network_status: ApiResponse<Value>,
    pub name_enquiry: ApiResponse<Value>,
}

/// Funds transfer response + immediate verify.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferAndVerify {
    pub transfer: ApiResponse<Value>,
    pub verify: ApiResponse<Value>,
}

/// Dynamic deposit status + deposit verify for one `txnRef`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicDepositCheck {
    pub deposit_status: ApiResponse<Value>,
    pub deposit_verify: ApiResponse<Value>,
}

/// Settlement create + history for the same zainbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementCreateWithHistory {
    pub create: ApiResponse<Value>,
    pub history: ApiResponse<Value>,
}
