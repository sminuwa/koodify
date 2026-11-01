//! HMAC verification for inbound `Zainpay-Signature` webhooks.
//!
//! Zainpay docs are thin on the exact encoding. We accept the common variants:
//! HMAC-SHA256 of the raw body as lowercase/uppercase hex, `sha256=` / `v1=`
//! prefixed hex, and standard / URL-safe Base64 digests.

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

fn hmac_sha256_raw(secret: &[u8], raw_body: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(raw_body);
    let bytes = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn ct_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

fn ct_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Normalize a signature header to bare lowercase hex when it looks like hex.
/// Also returns the trimmed original (minus common prefixes) for base64 attempts.
fn normalize_signature_header(signature_header: &str) -> (String, String) {
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
    let bare = s.trim_matches('"').trim().to_string();
    (bare.to_ascii_lowercase(), bare)
}

fn verify_one_secret(secret: &[u8], raw_body: &[u8], signature_header: &str) -> bool {
    let digest = hmac_sha256_raw(secret, raw_body);
    let expected_hex = hex::encode(digest);
    let (received_hex, received_raw) = normalize_signature_header(signature_header);

    if ct_eq_str(&expected_hex, &received_hex) {
        return true;
    }

    // Observed Zainpay deliveries with 62-char hex (truncated SHA-256 HMAC).
    // Accept a prefix match only when the provided hex is long enough (>= 48).
    if received_hex.len() >= 48
        && received_hex.len() < expected_hex.len()
        && received_hex.bytes().all(|b| b.is_ascii_hexdigit())
        && ct_eq_str(&expected_hex[..received_hex.len()], &received_hex)
    {
        return true;
    }

    // Base64 (std + url-safe, with/without padding).
    let b64_std = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(digest)
    };
    let b64_url = {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    };
    if ct_eq_str(&b64_std, &received_raw) || ct_eq_str(&b64_url, &received_raw) {
        return true;
    }
    // Compare decoded bytes if the header decodes as base64 of length 32.
    for engine in [
        &base64::engine::general_purpose::STANDARD,
        &base64::engine::general_purpose::STANDARD_NO_PAD,
        &base64::engine::general_purpose::URL_SAFE,
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ] {
        use base64::Engine;
        if let Ok(bytes) = engine.decode(received_raw.as_bytes()) {
            if ct_eq_bytes(&digest, &bytes) {
                return true;
            }
        }
    }

    false
}

/// Verify a `Zainpay-Signature` header against one secret.
pub fn verify_webhook_signature(
    secret: impl AsRef<[u8]>,
    raw_body: impl AsRef<[u8]>,
    signature_header: &str,
) -> bool {
    verify_one_secret(secret.as_ref(), raw_body.as_ref(), signature_header)
}

/// Try each secret until one verifies. Empty secrets are skipped.
pub fn verify_webhook_signature_any(
    secrets: &[impl AsRef<[u8]>],
    raw_body: impl AsRef<[u8]>,
    signature_header: &str,
) -> bool {
    let body = raw_body.as_ref();
    secrets.iter().any(|s| {
        let key = s.as_ref();
        !key.is_empty() && verify_one_secret(key, body, signature_header)
    })
}

/// Classify a signature header for ops diagnostics (no secret material).
pub fn describe_signature_header(signature_header: &str) -> serde_json::Value {
    let trimmed = signature_header.trim();
    let (hexish, bare) = normalize_signature_header(trimmed);
    let hex_ok = !hexish.is_empty() && hexish.bytes().all(|b| b.is_ascii_hexdigit());
    serde_json::json!({
        "signaturePresent": !trimmed.is_empty(),
        "signatureLen": trimmed.len(),
        "normalizedLen": bare.len(),
        "looksLikeHex": hex_ok,
        "signaturePrefix": trimmed.chars().take(12).collect::<String>(),
    })
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

    #[test]
    fn accepts_base64_digest() {
        use base64::Engine;
        let body = br#"{"event":"deposit.success"}"#;
        let secret = b"secret";
        let digest = hmac_sha256_raw(secret, body);
        let b64 = base64::engine::general_purpose::STANDARD.encode(digest);
        assert!(verify_webhook_signature(secret, body, &b64));
    }

    #[test]
    fn any_secret_matches() {
        let body = b"payload";
        let sig = sign_webhook_body(b"right", body);
        let ok: &[&[u8]] = &[b"wrong", b"right"];
        let bad: &[&[u8]] = &[b"wrong", b"also-wrong"];
        assert!(verify_webhook_signature_any(ok, body, &sig));
        assert!(!verify_webhook_signature_any(bad, body, &sig));
    }

    #[test]
    fn accepts_truncated_hex_prefix() {
        let body = br#"{"event":"deposit.success"}"#;
        let secret = b"secret";
        let full = sign_webhook_body(secret, body);
        let truncated = &full[..62];
        assert!(verify_webhook_signature(secret, body, truncated));
        assert!(!verify_webhook_signature(secret, body, &full[..16]));
    }
}
