use std::sync::Arc;

use sqlx::PgPool;
use client::ZainpayClient;

use crate::config::Config;
use crate::services::bank_router::BankRouter;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub zainpay: ZainpayClient,
    pub bank_router: Arc<BankRouter>,
}
