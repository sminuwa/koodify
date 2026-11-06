pub mod activity_events;
pub mod credentials;
pub mod customers;
pub mod idempotency;
pub mod kyc;
pub mod models;
pub mod payment_splits;
pub mod seed;
pub mod split_plans;
pub mod transactions;
pub mod virtual_accounts;
pub mod webhook_deliveries;
pub mod zainboxes;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
