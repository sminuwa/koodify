use sqlx::PgPool;
use client::ZainpayClient;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub zainpay: ZainpayClient,
}
