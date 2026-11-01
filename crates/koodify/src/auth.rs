use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::state::AppState;

fn header_api_key(parts: &Parts) -> &str {
    parts
        .headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

/// Constant-time equality for API keys (hashes first so length is not leaked).
fn api_keys_equal(provided: &str, expected: &str) -> bool {
    let a = Sha256::digest(provided.as_bytes());
    let b = Sha256::digest(expected.as_bytes());
    bool::from(a.ct_eq(&b))
}

/// Secret key auth for admin / privileged routes (`GATEWAY_API_KEY`).
pub struct ApiKeyAuth;

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let provided = header_api_key(parts);

        if provided.is_empty() || !api_keys_equal(provided, &state.config.gateway_api_key) {
            return Err((
                StatusCode::UNAUTHORIZED,
                "invalid or missing X-Api-Key (secret)".into(),
            ));
        }

        Ok(ApiKeyAuth)
    }
}

#[cfg(test)]
mod tests {
    use super::api_keys_equal;

    #[test]
    fn api_keys_equal_matches() {
        assert!(api_keys_equal("secret-key", "secret-key"));
        assert!(!api_keys_equal("secret-key", "other-key"));
        assert!(!api_keys_equal("", "secret-key"));
        assert!(!api_keys_equal("short", "much-longer-secret"));
    }
}
