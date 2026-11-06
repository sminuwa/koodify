//! Application-level `code` values from Zainpay status-codes docs.

/// Successful application result.
pub const SUCCESS: &str = "00";

pub const INVALID_ZAINBOX: &str = "02";
pub const SUBSCRIPTION_ERROR: &str = "03";
/// Catch-all: invalid/expired resource, missing record, or failed operation.
pub const RESOURCE_OR_APPLICATION_ERROR: &str = "04";
pub const INTERNAL_CONFIG_ERROR: &str = "05";
pub const ALREADY_SUBSCRIBED: &str = "06";
pub const ALREADY_UNSUBSCRIBED: &str = "07";
/// Async funds transfer accepted / queued (often with HTTP 202).
pub const FUNDS_TRANSFER_QUEUED: &str = "21";
pub const PAYLOAD_VALIDATION: &str = "22";
pub const INSUFFICIENT_BALANCE: &str = "23";
pub const INVALID_DESTINATION_ACCOUNT: &str = "24";
pub const NO_WALLET_BALANCE: &str = "25";
pub const DUPLICATE_TXN_REF: &str = "26";
pub const BILLING_ESTIMATION_ERROR: &str = "30";
pub const CARD_VALIDATION: &str = "99";

/// Alias kept for older call sites (`07` = already unsubscribed per docs).
pub const GENERAL_ERROR: &str = ALREADY_UNSUBSCRIBED;
/// Alias: invalid destination account.
pub const INVALID_ACCOUNT: &str = INVALID_DESTINATION_ACCOUNT;
/// Alias: no wallet balance / failed transfer context.
pub const TRANSFER_FAILED: &str = NO_WALLET_BALANCE;
/// Alias: billing estimation during fund transfer.
pub const VA_ERROR: &str = BILLING_ESTIMATION_ERROR;

/// Returns `true` when `code` is the success value `"00"`.
pub fn is_success_code(code: &str) -> bool {
    code == SUCCESS
}

/// Human-readable label for known application codes; `None` if unknown.
pub fn describe_code(code: &str) -> Option<&'static str> {
    Some(match code {
        SUCCESS => "successful",
        INVALID_ZAINBOX => "invalid source / zainbox code",
        SUBSCRIPTION_ERROR => "invalid zainbox code (service subscription)",
        RESOURCE_OR_APPLICATION_ERROR => {
            "invalid / expired resource / application error (ambiguous catch-all)"
        }
        INTERNAL_CONFIG_ERROR => "internal config error",
        ALREADY_SUBSCRIBED => "service already subscribed",
        ALREADY_UNSUBSCRIBED => "service already unsubscribed",
        FUNDS_TRANSFER_QUEUED => "funds transfer queued (async)",
        PAYLOAD_VALIDATION => "payload validation error",
        INSUFFICIENT_BALANCE => "insufficient wallet balance / amount too low",
        INVALID_DESTINATION_ACCOUNT => "invalid destination account number",
        NO_WALLET_BALANCE => "account has no wallet balance",
        DUPLICATE_TXN_REF => "duplicate transaction reference",
        BILLING_ESTIMATION_ERROR => "billing estimation error during fund transfer",
        CARD_VALIDATION => "payload / card validation failure",
        _ => return None,
    })
}
