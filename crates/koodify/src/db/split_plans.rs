use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SplitPlan {
    pub id: String,
    pub name: String,
    pub primary_zainbox_code: String,
    pub shares: Value,
    pub is_active: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn get(pool: &PgPool, id: &str) -> Result<Option<SplitPlan>, sqlx::Error> {
    sqlx::query_as::<_, SplitPlan>(
        r#"
        SELECT id, name, primary_zainbox_code, shares, is_active, metadata, created_at, updated_at
        FROM split_plans
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert(
    pool: &PgPool,
    id: &str,
    name: &str,
    primary_zainbox_code: &str,
    shares: &Value,
    is_active: bool,
    metadata: &Value,
) -> Result<SplitPlan, sqlx::Error> {
    sqlx::query_as::<_, SplitPlan>(
        r#"
        INSERT INTO split_plans (id, name, primary_zainbox_code, shares, is_active, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            primary_zainbox_code = EXCLUDED.primary_zainbox_code,
            shares = EXCLUDED.shares,
            is_active = EXCLUDED.is_active,
            metadata = EXCLUDED.metadata,
            updated_at = now()
        RETURNING id, name, primary_zainbox_code, shares, is_active, metadata, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(primary_zainbox_code)
    .bind(shares)
    .bind(is_active)
    .bind(metadata)
    .fetch_one(pool)
    .await
}

pub async fn list_for_primary(
    pool: &PgPool,
    primary_zainbox_code: &str,
) -> Result<Vec<SplitPlan>, sqlx::Error> {
    sqlx::query_as::<_, SplitPlan>(
        r#"
        SELECT id, name, primary_zainbox_code, shares, is_active, metadata, created_at, updated_at
        FROM split_plans
        WHERE primary_zainbox_code = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(primary_zainbox_code)
    .fetch_all(pool)
    .await
}
