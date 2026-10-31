use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Credential;

const COLUMNS: &str = r#"
    id, customer_id, environment, public_key, private_key, metadata, status, created_at, updated_at
"#;

pub struct UpsertCredential<'a> {
    pub customer_id: i64,
    pub environment: &'a str,
    pub public_key: &'a str,
    pub private_key: &'a str,
    pub metadata: &'a Value,
    pub status: &'a str,
}

pub async fn upsert(
    pool: &PgPool,
    input: UpsertCredential<'_>,
) -> Result<Credential, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, Credential>(&format!(
        r#"
        INSERT INTO credentials (
            id, customer_id, environment, public_key, private_key, metadata, status
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        ON CONFLICT (customer_id, environment) DO UPDATE SET
            public_key = EXCLUDED.public_key,
            private_key = EXCLUDED.private_key,
            metadata = EXCLUDED.metadata,
            status = EXCLUDED.status,
            updated_at = now()
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(input.customer_id)
    .bind(input.environment)
    .bind(input.public_key)
    .bind(input.private_key)
    .bind(input.metadata)
    .bind(input.status)
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>(&format!(
        r#"SELECT {COLUMNS} FROM credentials WHERE id = $1"#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_for_customer_env(
    pool: &PgPool,
    customer_id: i64,
    environment: &str,
) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM credentials
        WHERE customer_id = $1 AND environment = $2
        "#
    ))
    .bind(customer_id)
    .bind(environment)
    .fetch_optional(pool)
    .await
}

pub async fn list_for_customer(
    pool: &PgPool,
    customer_id: i64,
) -> Result<Vec<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM credentials
        WHERE customer_id = $1
        ORDER BY CASE environment WHEN 'test' THEN 0 ELSE 1 END
        "#
    ))
    .bind(customer_id)
    .fetch_all(pool)
    .await
}

pub async fn list(
    pool: &PgPool,
    customer_id: Option<i64>,
    environment: Option<&str>,
) -> Result<Vec<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM credentials
        WHERE ($1::bigint IS NULL OR customer_id = $1)
          AND ($2::text IS NULL OR environment = $2)
        ORDER BY customer_id, CASE environment WHEN 'test' THEN 0 ELSE 1 END
        "#
    ))
    .bind(customer_id)
    .bind(environment)
    .fetch_all(pool)
    .await
}

pub async fn set_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>(&format!(
        r#"
        UPDATE credentials
        SET status = $2, updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await
}
