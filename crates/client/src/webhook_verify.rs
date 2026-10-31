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

/// Verify a `Zainpay-Signature` header (hex HMAC-SHA256 of the raw body).
///
/// Uses a constant-time comparison of the hex digests.
pub fn verify_webhook_signature(
    secret: impl AsRef<[u8]>,
    raw_body: impl AsRef<[u8]>,
    signature_header: &str,
) -> bool {
    let expected = sign_webhook_body(secret, raw_body);
    let received = signature_header.trim();
    if expected.len() != received.len() {
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
        assert!(!verify_webhook_signature(secret, body, "deadbeef"));
        assert!(!verify_webhook_signature(b"other", body, &sig));
    }
}
