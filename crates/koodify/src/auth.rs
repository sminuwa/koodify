use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::state::AppState;

fn header_api_key(parts: &Parts) -> &str {
    parts
        .headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
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

        if provided.is_empty() || provided != state.config.gateway_api_key {
            return Err((
                StatusCode::UNAUTHORIZED,
                "invalid or missing X-Api-Key (secret)".into(),
            ));
        }

        Ok(ApiKeyAuth)
    }
}
