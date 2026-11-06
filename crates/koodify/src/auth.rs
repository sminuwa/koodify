//! Gateway authentication.
//!
//! - `GATEWAY_API_KEY` → platform **admin** (full access)
//! - Merchant `sk_test_…` / `sk_live_…` → **scoped** to that credential's zainbox
//! - Checkout uses publishable `pk_…` on open routes (not this module)

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::db;
use crate::state::AppState;

fn header_api_key(parts: &Parts) -> &str {
    parts
        .headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

/// Constant-time equality for API keys (hashes first so length is not leaked).
pub fn api_keys_equal(provided: &str, expected: &str) -> bool {
    let a = Sha256::digest(provided.as_bytes());
    let b = Sha256::digest(expected.as_bytes());
    bool::from(a.ct_eq(&b))
}

/// SHA-256 hex of a merchant secret (stored / looked up).
pub fn hash_secret_key(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

/// Mint a merchant secret for `test` or `live`.
pub fn mint_secret_key(environment: &str) -> String {
    let env = match environment {
        "test" => "test",
        _ => "live",
    };
    format!("sk_{env}_{}", Uuid::new_v4().simple())
}

/// Display prefix (`sk_live_abcd1234`).
pub fn secret_key_prefix(secret: &str) -> String {
    const KEEP: usize = 16;
    if secret.len() <= KEEP {
        return secret.to_string();
    }
    format!("{}…", &secret[..KEEP])
}

#[derive(Debug, Clone)]
pub struct MerchantPrincipal {
    pub customer_id: i64,
    pub zainbox_code: String,
    pub environment: String,
    pub credential_id: Uuid,
    pub secret_key_prefix: String,
}

#[derive(Debug, Clone)]
pub enum AuthKind {
    Admin,
    Merchant(MerchantPrincipal),
}

/// Accepts platform admin **or** merchant secret key.
/// Use [`ApiKeyAuth::ensure_zainbox`] on path/body zainbox codes.
pub struct ApiKeyAuth {
    pub kind: AuthKind,
}

impl ApiKeyAuth {
    pub fn is_admin(&self) -> bool {
        matches!(self.kind, AuthKind::Admin)
    }

    pub fn merchant(&self) -> Option<&MerchantPrincipal> {
        match &self.kind {
            AuthKind::Merchant(m) => Some(m),
            AuthKind::Admin => None,
        }
    }

    /// Admin always allowed; merchant only for their own `zainbox_code`.
    pub fn ensure_zainbox(&self, code: &str) -> Result<(), (StatusCode, String)> {
        match &self.kind {
            AuthKind::Admin => Ok(()),
            AuthKind::Merchant(m) if m.zainbox_code == code => Ok(()),
            AuthKind::Merchant(_) => Err((
                StatusCode::FORBIDDEN,
                "forbidden: API key is not authorized for this zainbox".into(),
            )),
        }
    }

    /// Reject merchant keys — platform operators only.
    pub fn require_admin(&self) -> Result<(), (StatusCode, String)> {
        if self.is_admin() {
            Ok(())
        } else {
            Err((
                StatusCode::FORBIDDEN,
                "forbidden: platform admin API key required".into(),
            ))
        }
    }
}

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let provided = header_api_key(parts);
        if provided.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                "invalid or missing X-Api-Key (secret)".into(),
            ));
        }

        if api_keys_equal(provided, &state.config.gateway_api_key) {
            return Ok(ApiKeyAuth {
                kind: AuthKind::Admin,
            });
        }

        // Merchant secrets are sk_test_… / sk_live_…
        if provided.starts_with("sk_test_") || provided.starts_with("sk_live_") {
            let hash = hash_secret_key(provided);
            match db::credentials::find_by_secret_hash(&state.pool, &hash).await {
                Ok(Some(row)) if row.status == "active" => {
                    let prefix = row
                        .secret_key_prefix
                        .clone()
                        .unwrap_or_else(|| secret_key_prefix(provided));
                    return Ok(ApiKeyAuth {
                        kind: AuthKind::Merchant(MerchantPrincipal {
                            customer_id: row.customer_id,
                            zainbox_code: row.zainbox_code,
                            environment: row.environment,
                            credential_id: row.id,
                            secret_key_prefix: prefix,
                        }),
                    });
                }
                Ok(Some(_)) => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "API key is inactive".into(),
                    ));
                }
                Ok(None) => {}
                Err(err) => {
                    tracing::error!(error = %err, "secret key lookup failed");
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to verify API key".into(),
                    ));
                }
            }
        }

        Err((
            StatusCode::UNAUTHORIZED,
            "invalid or missing X-Api-Key (secret)".into(),
        ))
    }
}

/// Platform admin only (`GATEWAY_API_KEY`). Merchants cannot use this extractor.
pub struct AdminAuth;

impl FromRequestParts<AppState> for AdminAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = ApiKeyAuth::from_request_parts(parts, state).await?;
        auth.require_admin()?;
        Ok(AdminAuth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_keys_equal_matches() {
        assert!(api_keys_equal("secret-key", "secret-key"));
        assert!(!api_keys_equal("secret-key", "other-key"));
        assert!(!api_keys_equal("", "secret-key"));
        assert!(!api_keys_equal("short", "much-longer-secret"));
    }

    #[test]
    fn mint_secret_key_prefixes() {
        let live = mint_secret_key("live");
        let test = mint_secret_key("test");
        assert!(live.starts_with("sk_live_"));
        assert!(test.starts_with("sk_test_"));
        assert_ne!(hash_secret_key(&live), hash_secret_key(&test));
    }

    #[test]
    fn ensure_zainbox_scopes_merchant() {
        let admin = ApiKeyAuth {
            kind: AuthKind::Admin,
        };
        assert!(admin.ensure_zainbox("any").is_ok());

        let merchant = ApiKeyAuth {
            kind: AuthKind::Merchant(MerchantPrincipal {
                customer_id: 1,
                zainbox_code: "main".into(),
                environment: "live".into(),
                credential_id: Uuid::nil(),
                secret_key_prefix: "sk_live_abcd…".into(),
            }),
        };
        assert!(merchant.ensure_zainbox("main").is_ok());
        assert!(merchant.ensure_zainbox("other").is_err());
        assert!(merchant.require_admin().is_err());
    }
}
