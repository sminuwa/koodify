use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::state::AppState;

/// Simple shared-secret auth via `X-Api-Key`.
pub struct ApiKeyAuth;

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let provided = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if provided.is_empty() || provided != state.config.gateway_api_key {
            return Err((StatusCode::UNAUTHORIZED, "invalid or missing X-Api-Key".into()));
        }

        Ok(ApiKeyAuth)
    }
}
