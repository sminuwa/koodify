//! HMAC-SHA256 verification for inbound `Zainpay-Signature` webhooks.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Compute the hex-encoded HMAC-SHA256 digest of `raw_body` with `secret`.
pub fn sign_webhook_body(secret: impl AsRef<[u8]>, raw_body: impl AsRef<[u8]>) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_ref()).expect("HMAC accepts any key length");
    mac.update(raw_body.as_ref());
    hex::encode(mac.finalize().into_bytes())
}

/// Normalize a signature header to bare lowercase hex.
/// Accepts optional `sha256=` / `v1=` prefixes and mixed-case hex.
fn normalize_signature_header(signature_header: &str) -> String {
    let mut s = signature_header.trim();
    if let Some(rest) = s.strip_prefix("sha256=") {
        s = rest.trim();
    } else if let Some(rest) = s.strip_prefix("SHA256=") {
        s = rest.trim();
    } else if let Some(rest) = s.strip_prefix("v1=") {
        s = rest.trim();
    }
    // Some gateways send `t=…,v1=hex` — take the last hex-looking token.
    if s.contains(',') {
        if let Some(part) = s
            .split(',')
            .rev()
            .map(str::trim)
            .find(|p| {
                let p = p.strip_prefix("v1=").unwrap_or(p);
                !p.is_empty() && p.bytes().all(|b| b.is_ascii_hexdigit())
            })
        {
            s = part.strip_prefix("v1=").unwrap_or(part);
        }
    }
    s.to_ascii_lowercase()
}

/// Verify a `Zainpay-Signature` header (hex HMAC-SHA256 of the raw body).
///
/// Uses a constant-time comparison of the hex digests (case-insensitive).
pub fn verify_webhook_signature(
    secret: impl AsRef<[u8]>,
    raw_body: impl AsRef<[u8]>,
    signature_header: &str,
) -> bool {
    let expected = sign_webhook_body(secret, raw_body);
    let received = normalize_signature_header(signature_header);
    if expected.len() != received.len() || received.is_empty() {
        return false;
    }
    expected.as_bytes().ct_eq(received.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_signature() {
        let body = br#"{"event":"deposit.success","data":{"txnRef":"abc"}}"#;
        let secret = b"test-secret";
        let sig = sign_webhook_body(secret, body);
        assert!(verify_webhook_signature(secret, body, &sig));
        assert!(verify_webhook_signature(secret, body, &sig.to_ascii_uppercase()));
        assert!(verify_webhook_signature(
            secret,
            body,
            &format!("sha256={sig}")
        ));
        assert!(!verify_webhook_signature(secret, body, "deadbeef"));
        assert!(!verify_webhook_signature(b"other", body, &sig));
    }
}
