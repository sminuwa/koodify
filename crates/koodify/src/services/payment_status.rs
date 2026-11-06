//! Canonical payment-status normalizer (DX issues 04 + 05).
//!
//! Zainpay returns overlapping signals: HTTP/application `code=00` while the
//! payment is still pending, `code=04` that may mean “txn not found” *or*
//! expired / other failures, and status strings like `Successful` / `paid`.
//! Merchants on the gateway should read `data.koodify` instead of guessing.

use serde_json::{json, Value};

/// Stable merchant-facing payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    NotFound,
    Mismatch,
    Expired,
    Unknown,
}

impl PaymentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::NotFound => "not_found",
            Self::Mismatch => "mismatch",
            Self::Expired => "expired",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_paid(self) -> bool {
        self == Self::Success
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Pending => "Awaiting the expected deposit. Do not fulfill yet.",
            Self::Success => "Expected amount received. Safe to fulfill.",
            Self::Failed => "Payment failed. Do not fulfill; ask the customer to retry.",
            Self::NotFound => {
                "No payment session found for this reference. Confirm txnRef — this is not the same as pending."
            }
            Self::Mismatch => {
                "Deposited amount differs from expected. Do not fulfill; Zainpay may auto-refund (DVA) or require bank/ops follow-up."
            }
            Self::Expired => {
                "Virtual account lifetime ended. Do not fulfill; create a new checkout if needed."
            }
            Self::Unknown => {
                "Status is ambiguous upstream. Do not fulfill; re-poll or check GET /api/v1/activity/{txnRef}."
            }
        }
    }
}

/// Parsed view of a Zainpay (or composite) payment status body.
#[derive(Debug, Clone)]
pub struct NormalizedPayment {
    pub status: PaymentStatus,
    pub upstream_code: Option<String>,
    pub upstream_description: Option<String>,
    pub upstream_payment_status: Option<String>,
    pub amount: Option<String>,
    pub deposited_amount: Option<String>,
    pub txn_ref: Option<String>,
}

impl NormalizedPayment {
    pub fn is_paid(&self) -> bool {
        self.status.is_paid()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "paymentStatus": self.status.as_str(),
            "isPaid": self.status.is_paid(),
            "hint": self.status.hint(),
            "amount": self.amount,
            "depositedAmount": self.deposited_amount,
            "txnRef": self.txn_ref,
            "upstream": {
                "code": self.upstream_code,
                "description": self.upstream_description,
                "paymentStatus": self.upstream_payment_status,
            }
        })
    }
}

/// Classify a Zainpay-shaped JSON envelope (or nested composite).
pub fn classify(response: &Value) -> NormalizedPayment {
    let upstream_code = first_str(response, &["code"]).map(str::to_string);
    let upstream_description = first_str(response, &["description"]).map(str::to_string);

    let data = data_object(response);
    let nested = nested_status_data(data);

    let upstream_payment_status = status_string(data)
        .or_else(|| nested.and_then(status_string))
        .map(|s| s.to_string());

    let amount = first_str(data, &["amount", "expectedAmount", "txnAmount"])
        .or_else(|| nested.and_then(|n| first_str(n, &["amount", "expectedAmount", "txnAmount"])))
        .map(str::to_string);
    let deposited_amount = first_str(data, &["depositedAmount", "amountPaid", "totalTxnAmount"])
        .or_else(|| {
            nested.and_then(|n| first_str(n, &["depositedAmount", "amountPaid", "totalTxnAmount"]))
        })
        .map(str::to_string);
    let txn_ref = first_str(data, &["txnRef", "txn_ref", "transactionRef"])
        .or_else(|| {
            nested.and_then(|n| first_str(n, &["txnRef", "txn_ref", "transactionRef"]))
        })
        .map(str::to_string);

    let status = resolve_status(
        upstream_code.as_deref(),
        upstream_description.as_deref(),
        upstream_payment_status.as_deref(),
    );

    NormalizedPayment {
        status,
        upstream_code,
        upstream_description,
        upstream_payment_status,
        amount,
        deposited_amount,
        txn_ref,
    }
}

/// True only when the canonical status is explicit success.
pub fn is_paid(response: &Value) -> bool {
    classify(response).is_paid()
}

/// Attach `data.koodify` (canonical status) without removing upstream fields.
pub fn enrich(mut response: Value) -> Value {
    let normalized = classify(&response);
    let block = normalized.to_json();

    match response.get_mut("data") {
        Some(Value::Object(map)) => {
            map.insert("koodify".into(), block);
        }
        Some(other) => {
            // Rare: data is not an object — wrap.
            let previous = other.clone();
            *other = json!({ "value": previous, "koodify": block });
        }
        None => {
            if let Some(obj) = response.as_object_mut() {
                obj.insert("data".into(), json!({ "koodify": block }));
            } else {
                response = json!({
                    "code": response.get("code").cloned().unwrap_or(Value::Null),
                    "description": response.get("description").cloned().unwrap_or(Value::Null),
                    "data": { "koodify": block },
                    "raw": response,
                });
            }
        }
    }

    // Layer application-error hints (code 04, 26, …) into the same koodify object.
    crate::services::api_error::enrich(response)
}

fn resolve_status(
    code: Option<&str>,
    description: Option<&str>,
    payment_status: Option<&str>,
) -> PaymentStatus {
    let desc = description.unwrap_or("").to_ascii_lowercase();
    let status = payment_status.unwrap_or("").to_ascii_lowercase();

    // Explicit payment-status strings win when present.
    if !status.is_empty() {
        if matches!(
            status.as_str(),
            "success" | "successful" | "paid" | "completed"
        ) {
            return PaymentStatus::Success;
        }
        if status == "pending" || status == "initiated" || status == "processing" {
            return PaymentStatus::Pending;
        }
        if status == "mismatch" || status == "amount_mismatch" {
            return PaymentStatus::Mismatch;
        }
        if status == "expired" {
            return PaymentStatus::Expired;
        }
        if matches!(
            status.as_str(),
            "failed" | "failure" | "declined" | "rejected" | "cancelled" | "canceled"
        ) {
            return PaymentStatus::Failed;
        }
        if status.contains("not_found") || status.contains("not found") {
            return PaymentStatus::NotFound;
        }
    }

    // Application code / description when status string is absent or unfamiliar.
    if desc_contains_any(&desc, &["txn not found", "transaction not found", "not found"]) {
        return PaymentStatus::NotFound;
    }
    if desc_contains_any(&desc, &["mismatch", "underpay", "overpay"]) {
        return PaymentStatus::Mismatch;
    }
    // Prefer specific expiry wording over the generic code-04 docs phrase
    // ("Invalid / expired resource / application error").
    if is_specific_expired(&desc) {
        return PaymentStatus::Expired;
    }

    match code {
        Some("00") | Some("0") => PaymentStatus::Unknown,
        Some("04") => {
            // Documented catch-all: not found, expired, or other failure.
            // Without a specific description we refuse to guess → unknown.
            PaymentStatus::Unknown
        }
        Some("22") | Some("23") | Some("24") | Some("25") | Some("26") | Some("99") => {
            PaymentStatus::Failed
        }
        None if status.is_empty() && desc.is_empty() => PaymentStatus::Unknown,
        _ => {
            if desc_contains_any(&desc, &["fail", "decline", "rejected"]) {
                PaymentStatus::Failed
            } else {
                PaymentStatus::Unknown
            }
        }
    }
}

fn is_specific_expired(desc: &str) -> bool {
    if desc.is_empty() {
        return false;
    }
    // Generic Zainpay code-04 label — not a real expiry event.
    if desc.contains("invalid") && desc.contains("expired resource") {
        return false;
    }
    desc_contains_any(
        desc,
        &[
            "lifetime has expired",
            "dva expired",
            "account expired",
            "session expired",
            "expired dva",
            "has expired",
        ],
    ) || (desc.contains("expired") && !desc.contains("application error"))
}

fn desc_contains_any(desc: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| desc.contains(n))
}

fn data_object(response: &Value) -> &Value {
    response.get("data").unwrap_or(response)
}

/// Composite `dynamic_deposit_check` nests status under `depositStatus` / `deposit_status`.
fn nested_status_data(data: &Value) -> Option<&Value> {
    let nested = data
        .get("depositStatus")
        .or_else(|| data.get("deposit_status"))
        .or_else(|| data.get("status"))?;
    Some(nested.get("data").unwrap_or(nested))
}

fn status_string(data: &Value) -> Option<&str> {
    first_str(
        data,
        &[
            "paymentStatus",
            "payment_status",
            "txnStatus",
            "transactionStatus",
            "status",
        ],
    )
    .filter(|s| {
        // Avoid treating HTTP-ish "200 OK" envelope status as payment status.
        let lower = s.to_ascii_lowercase();
        !(lower.contains("ok") && lower.chars().any(|c| c.is_ascii_digit()))
    })
}

fn first_str<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    let obj = value.as_object()?;
    for key in keys {
        if let Some(s) = obj.get(*key).and_then(|v| v.as_str()).map(str::trim) {
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_variants() {
        for s in ["success", "Successful", "paid", "COMPLETED"] {
            let n = classify(&json!({
                "code": "00",
                "data": { "paymentStatus": s, "amount": "20000" }
            }));
            assert!(n.is_paid(), "{s}");
            assert_eq!(n.status, PaymentStatus::Success);
        }
    }

    #[test]
    fn code_00_pending_is_not_paid() {
        let n = classify(&json!({
            "code": "00",
            "description": "DVA status",
            "data": { "paymentStatus": "pending", "amount": "20000" }
        }));
        assert!(!n.is_paid());
        assert_eq!(n.status, PaymentStatus::Pending);
    }

    #[test]
    fn code_00_alone_is_unknown_not_paid() {
        let n = classify(&json!({
            "code": "00",
            "description": "ok",
            "data": { "amount": "20000" }
        }));
        assert!(!n.is_paid());
        assert_eq!(n.status, PaymentStatus::Unknown);
    }

    #[test]
    fn code_04_txn_not_found() {
        let n = classify(&json!({
            "code": "04",
            "description": "Txn not found",
            "data": null
        }));
        assert_eq!(n.status, PaymentStatus::NotFound);
        assert!(!n.is_paid());
    }

    #[test]
    fn code_04_generic_docs_phrase_is_unknown() {
        let n = classify(&json!({
            "code": "04",
            "description": "Invalid / expired resource / application error",
            "data": {}
        }));
        assert_eq!(n.status, PaymentStatus::Unknown);
    }

    #[test]
    fn specific_expiry_description() {
        let n = classify(&json!({
            "code": "04",
            "description": "DVA lifetime has expired",
            "data": {}
        }));
        assert_eq!(n.status, PaymentStatus::Expired);
    }

    #[test]
    fn code_04_bare_unknown() {
        let n = classify(&json!({
            "code": "04",
            "description": "application error",
            "data": {}
        }));
        assert_eq!(n.status, PaymentStatus::Unknown);
    }

    #[test]
    fn mismatch_and_expired_status() {
        assert_eq!(
            classify(&json!({ "code": "00", "data": { "paymentStatus": "mismatch" } })).status,
            PaymentStatus::Mismatch
        );
        assert_eq!(
            classify(&json!({ "code": "00", "data": { "status": "expired" } })).status,
            PaymentStatus::Expired
        );
    }

    #[test]
    fn nested_composite_deposit_status() {
        let n = classify(&json!({
            "code": "00",
            "data": {
                "depositStatus": {
                    "code": "00",
                    "data": { "paymentStatus": "success", "amount": "50000", "txnRef": "T1" }
                },
                "depositVerify": { "code": "00", "data": {} }
            }
        }));
        assert!(n.is_paid());
        assert_eq!(n.txn_ref.as_deref(), Some("T1"));
        assert_eq!(n.amount.as_deref(), Some("50000"));
    }

    #[test]
    fn enrich_attaches_koodify_block() {
        let out = enrich(json!({
            "code": "00",
            "description": "DVA status",
            "data": {
                "paymentStatus": "pending",
                "amount": "20000",
                "txnRef": "LIVE_1"
            }
        }));
        assert_eq!(out["data"]["paymentStatus"], "pending");
        assert_eq!(out["data"]["koodify"]["paymentStatus"], "pending");
        assert_eq!(out["data"]["koodify"]["isPaid"], false);
        assert!(out["data"]["koodify"]["hint"].as_str().unwrap().contains("Awaiting"));
        assert_eq!(out["data"]["koodify"]["upstream"]["code"], "00");
    }

    #[test]
    fn ignores_http_ok_as_payment_status() {
        let n = classify(&json!({
            "code": "00",
            "status": "200 OK",
            "data": { "paymentStatus": "pending" }
        }));
        assert_eq!(n.status, PaymentStatus::Pending);
    }
}
