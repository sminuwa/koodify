//! Application-level `code` values from Zainpay status-codes docs.

/// Successful application result.
pub const SUCCESS: &str = "00";

pub const INVALID_ZAINBOX: &str = "02";
pub const SUBSCRIPTION_ERROR: &str = "03";
pub const GENERAL_ERROR: &str = "07";
/// Async funds transfer accepted / queued (often with HTTP 202).
pub const FUNDS_TRANSFER_QUEUED: &str = "21";
pub const INSUFFICIENT_BALANCE: &str = "22";
pub const INVALID_ACCOUNT: &str = "23";
pub const NAME_ENQUIRY_FAILED: &str = "24";
pub const TRANSFER_FAILED: &str = "25";
pub const DUPLICATE_TXN_REF: &str = "26";
pub const VA_ERROR: &str = "30";
pub const CARD_VALIDATION: &str = "99";

/// Returns `true` when `code` is the success value `"00"`.
pub fn is_success_code(code: &str) -> bool {
    code == SUCCESS
}

/// Human-readable label for known application codes; `None` if unknown.
pub fn describe_code(code: &str) -> Option<&'static str> {
    Some(match code {
        SUCCESS => "success",
        INVALID_ZAINBOX => "invalid zainbox / application error",
        SUBSCRIPTION_ERROR => "subscription error",
        GENERAL_ERROR => "general error",
        FUNDS_TRANSFER_QUEUED => "funds transfer queued (async)",
        INSUFFICIENT_BALANCE => "insufficient balance",
        INVALID_ACCOUNT => "invalid account",
        NAME_ENQUIRY_FAILED => "name enquiry failed",
        TRANSFER_FAILED => "transfer failed",
        DUPLICATE_TXN_REF => "duplicate transaction reference",
        VA_ERROR => "virtual account error",
        CARD_VALIDATION => "card validation error",
        _ => return None,
    })
}
