//! Zainpay gateway: wraps Zainpay endpoints and persists custom metadata.

pub mod auth;
pub mod config;
pub mod db;
pub mod metadata;
pub mod routes;
pub mod services;
pub mod state;

use anyhow::Result;
use axum::http::Method;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use client::ZainpayClient;

use crate::config::Config;
use crate::state::AppState;

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
}

/// Build the HTTP router and application state.
pub async fn build_app(config: Config) -> Result<(Router, AppState)> {
    config.validate()?;

    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    let zainpay = ZainpayClient::builder()
        .base_url(&config.zainpay_base_url)
        .api_key(config.zainpay_api_key.clone())
        .build()?;

    let bank_router = std::sync::Arc::new(services::bank_router::BankRouter::from_config(
        &config.va_bank_candidates,
        config.va_bank_sticky_secs,
    ));
    tracing::info!(
        candidates = ?config.va_bank_candidates,
        preferred = %bank_router.preferred(),
        sticky_secs = config.va_bank_sticky_secs,
        "Adaptive Bank/DVA Routing ready"
    );

    let state = AppState {
        config,
        pool,
        zainpay,
        bank_router,
    };

    let app = routes::router()
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    Ok((app, state))
}

/// Build an app against an existing pool and a custom Zainpay client (tests).
pub fn build_app_with(
    config: Config,
    pool: sqlx::PgPool,
    zainpay: ZainpayClient,
) -> Router {
    let bank_router = std::sync::Arc::new(services::bank_router::BankRouter::from_config(
        &config.va_bank_candidates,
        config.va_bank_sticky_secs,
    ));
    let state = AppState {
        config,
        pool,
        zainpay,
        bank_router,
    };

    routes::router()
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
