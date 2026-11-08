//! Signed merchant dashboard sessions (`Authorization: Bearer …`).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_TTL_SECS: i64 = 60 * 60 * 24 * 7; // 7 days
const TOKEN_PREFIX: &str = "mt_";

/// Authenticated merchant dashboard session (email/password login).
#[derive(Debug, Clone)]
pub struct MerchantSession {
    pub customer_id: i64,
}

impl MerchantSession {
    pub fn issue(customer_id: i64, gateway_api_key: &str) -> String {
        let exp = chrono::Utc::now().timestamp() + TOKEN_TTL_SECS;
        let payload = format!("{customer_id}.{exp}");
        let sig = sign(gateway_api_key, &payload);
        let body = URL_SAFE_NO_PAD.encode(format!("{payload}.{sig}"));
        format!("{TOKEN_PREFIX}{body}")
    }

    pub fn verify(token: &str, gateway_api_key: &str) -> Result<Self, (StatusCode, String)> {
        let raw = token
            .strip_prefix(TOKEN_PREFIX)
            .ok_or_else(|| unauthorized("invalid session token"))?;
        let decoded = URL_SAFE_NO_PAD
            .decode(raw.as_bytes())
            .map_err(|_| unauthorized("invalid session token"))?;
        let text = String::from_utf8(decoded).map_err(|_| unauthorized("invalid session token"))?;
        let mut parts = text.rsplitn(2, '.');
        let sig = parts
            .next()
            .ok_or_else(|| unauthorized("invalid session token"))?;
        let payload = parts
            .next()
            .ok_or_else(|| unauthorized("invalid session token"))?;
        let expected = sign(gateway_api_key, payload);
        if !constant_time_eq(sig.as_bytes(), expected.as_bytes()) {
            return Err(unauthorized("invalid session token"));
        }
        let mut id_exp = payload.splitn(2, '.');
        let customer_id: i64 = id_exp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| unauthorized("invalid session token"))?;
        let exp: i64 = id_exp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| unauthorized("invalid session token"))?;
        if chrono::Utc::now().timestamp() > exp {
            return Err(unauthorized("session expired"));
        }
        Ok(Self { customer_id })
    }
}

fn sign(secret: &str, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    if a.len() != b.len() {
        return false;
    }
    bool::from(a.ct_eq(b))
}

fn unauthorized(msg: &str) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.into())
}

fn extract_bearer(parts: &Parts) -> Option<&str> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

impl FromRequestParts<AppState> for MerchantSession {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts)
            .or_else(|| {
                parts
                    .headers
                    .get("x-merchant-token")
                    .and_then(|v| v.to_str().ok())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            })
            .ok_or_else(|| unauthorized("missing merchant session token"))?;
        Self::verify(token, &state.config.gateway_api_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_token() {
        let token = MerchantSession::issue(42, "gateway-secret");
        let session = MerchantSession::verify(&token, "gateway-secret").unwrap();
        assert_eq!(session.customer_id, 42);
        assert!(MerchantSession::verify(&token, "wrong").is_err());
    }
}
