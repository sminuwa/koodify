//! Persist Idempotency-Key outcomes for money-moving POSTs.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct IdempotencyRow {
    pub request_hash: String,
    pub state: String,
    pub response_body: Option<Value>,
    pub resource_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insert a pending claim. Returns `true` if this caller owns the key.
pub async fn try_begin(
    pool: &PgPool,
    scope: &str,
    idem_key: &str,
    request_hash: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO idempotency_keys (scope, idem_key, request_hash, state)
        VALUES ($1, $2, $3, 'pending')
        ON CONFLICT (scope, idem_key) DO NOTHING
        "#,
    )
    .bind(scope)
    .bind(idem_key)
    .bind(request_hash)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get(
    pool: &PgPool,
    scope: &str,
    idem_key: &str,
) -> Result<Option<IdempotencyRow>, sqlx::Error> {
    sqlx::query_as::<_, IdemRow>(
        r#"
        SELECT request_hash, state, response_body, resource_key, created_at, updated_at
        FROM idempotency_keys
        WHERE scope = $1 AND idem_key = $2
        "#,
    )
    .bind(scope)
    .bind(idem_key)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(Into::into))
}

/// Store the final JSON body and mark completed.
pub async fn complete(
    pool: &PgPool,
    scope: &str,
    idem_key: &str,
    response_body: &Value,
    resource_key: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE idempotency_keys
        SET state = 'completed',
            response_body = $3,
            resource_key = $4,
            updated_at = now()
        WHERE scope = $1 AND idem_key = $2
        "#,
    )
    .bind(scope)
    .bind(idem_key)
    .bind(response_body)
    .bind(resource_key)
    .execute(pool)
    .await?;
    Ok(())
}

/// Drop a pending claim so the client can safely retry (e.g. after transport failure).
pub async fn release_pending(
    pool: &PgPool,
    scope: &str,
    idem_key: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM idempotency_keys
        WHERE scope = $1 AND idem_key = $2 AND state = 'pending'
        "#,
    )
    .bind(scope)
    .bind(idem_key)
    .execute(pool)
    .await?;
    Ok(())
}

/// Reclaim a stale pending row (crashed worker) when hash still matches.
pub async fn try_reclaim_stale(
    pool: &PgPool,
    scope: &str,
    idem_key: &str,
    request_hash: &str,
    older_than_secs: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE idempotency_keys
        SET updated_at = now()
        WHERE scope = $1
          AND idem_key = $2
          AND state = 'pending'
          AND request_hash = $3
          AND updated_at < now() - make_interval(secs => $4::double precision)
        "#,
    )
    .bind(scope)
    .bind(idem_key)
    .bind(request_hash)
    .bind(older_than_secs as f64)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
struct IdemRow {
    request_hash: String,
    state: String,
    response_body: Option<Value>,
    resource_key: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<IdemRow> for IdempotencyRow {
    fn from(r: IdemRow) -> Self {
        Self {
            request_hash: r.request_hash,
            state: r.state,
            response_body: r.response_body,
            resource_key: r.resource_key,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
