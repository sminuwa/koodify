//! Signed admin portal sessions (`Authorization: Bearer at_…`).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::auth::{api_keys_equal, ApiKeyAuth, AuthKind};
use crate::db;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_TTL_SECS: i64 = 60 * 60 * 12; // 12 hours
const TOKEN_PREFIX: &str = "at_";

#[derive(Debug, Clone)]
pub struct AdminSession {
    pub user_id: Uuid,
    pub username: String,
}

impl AdminSession {
    pub fn issue(user_id: Uuid, gateway_api_key: &str) -> String {
        let exp = chrono::Utc::now().timestamp() + TOKEN_TTL_SECS;
        let payload = format!("{user_id}.{exp}");
        let sig = sign(gateway_api_key, &payload);
        let body = URL_SAFE_NO_PAD.encode(format!("{payload}.{sig}"));
        format!("{TOKEN_PREFIX}{body}")
    }

    pub fn verify(token: &str, gateway_api_key: &str) -> Result<Uuid, (StatusCode, String)> {
        let raw = token
            .strip_prefix(TOKEN_PREFIX)
            .ok_or_else(|| unauthorized("invalid admin session"))?;
        let decoded = URL_SAFE_NO_PAD
            .decode(raw.as_bytes())
            .map_err(|_| unauthorized("invalid admin session"))?;
        let text = String::from_utf8(decoded).map_err(|_| unauthorized("invalid admin session"))?;
        let mut parts = text.rsplitn(2, '.');
        let sig = parts
            .next()
            .ok_or_else(|| unauthorized("invalid admin session"))?;
        let payload = parts
            .next()
            .ok_or_else(|| unauthorized("invalid admin session"))?;
        let expected = sign(gateway_api_key, payload);
        if !constant_time_eq(sig.as_bytes(), expected.as_bytes()) {
            return Err(unauthorized("invalid admin session"));
        }
        let mut id_exp = payload.splitn(2, '.');
        let user_id: Uuid = id_exp
            .next()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| unauthorized("invalid admin session"))?;
        let exp: i64 = id_exp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| unauthorized("invalid admin session"))?;
        if chrono::Utc::now().timestamp() > exp {
            return Err(unauthorized("admin session expired"));
        }
        Ok(user_id)
    }
}

fn sign(secret: &str, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(b"koodify.admin.session.v1\0");
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

fn bearer(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

impl FromRequestParts<AppState> for AdminSession {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer(parts).ok_or_else(|| unauthorized("missing admin Bearer token"))?;
        let user_id = Self::verify(token, &state.config.gateway_api_key)?;
        let user = db::users::find_by_id(&state.pool, user_id)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("admin lookup failed: {err}"),
                )
            })?
            .ok_or_else(|| unauthorized("admin user not found"))?;
        if user.status != "active" {
            return Err(unauthorized("admin user is disabled"));
        }
        Ok(Self {
            user_id: user.id,
            username: user.username,
        })
    }
}

/// Platform `GATEWAY_API_KEY` **or** signed admin portal session (`at_…`).
pub struct AdminPortalAuth {
    pub kind: AdminPortalKind,
}

#[derive(Debug, Clone)]
pub enum AdminPortalKind {
    PlatformKey,
    User(AdminSession),
}

impl FromRequestParts<AppState> for AdminPortalAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Prefer explicit admin session when present.
        if let Some(token) = bearer(parts) {
            if token.starts_with(TOKEN_PREFIX) {
                let session = AdminSession::from_request_parts(parts, state).await?;
                return Ok(Self {
                    kind: AdminPortalKind::User(session),
                });
            }
        }

        // Fall back to platform admin API key (X-Api-Key).
        let provided = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !provided.is_empty() && api_keys_equal(provided, &state.config.gateway_api_key) {
            return Ok(Self {
                kind: AdminPortalKind::PlatformKey,
            });
        }

        if let Ok(auth) = ApiKeyAuth::from_request_parts(parts, state).await {
            if matches!(auth.kind, AuthKind::Admin) {
                return Ok(Self {
                    kind: AdminPortalKind::PlatformKey,
                });
            }
        }

        Err(unauthorized(
            "admin portal auth required (Bearer at_… or X-Api-Key)",
        ))
    }
}
