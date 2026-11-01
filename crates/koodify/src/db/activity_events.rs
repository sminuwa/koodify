use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub event: String,
    pub source: String,
    pub txn_ref: Option<String>,
    pub zainbox_code: Option<String>,
    pub summary: Option<String>,
    pub payload: Value,
}

pub async fn insert(
    pool: &PgPool,
    event: &str,
    source: &str,
    txn_ref: Option<&str>,
    zainbox_code: Option<&str>,
    summary: Option<&str>,
    payload: &Value,
) -> Result<ActivityEvent, sqlx::Error> {
    sqlx::query_as::<_, ActivityEvent>(
        r#"
        INSERT INTO activity_events (
            event, source, txn_ref, zainbox_code, summary, payload
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, created_at, event, source, txn_ref, zainbox_code, summary, payload
        "#,
    )
    .bind(event)
    .bind(source)
    .bind(txn_ref)
    .bind(zainbox_code)
    .bind(summary)
    .bind(payload)
    .fetch_one(pool)
    .await
}

pub async fn list_by_txn_ref(
    pool: &PgPool,
    txn_ref: &str,
    limit: i64,
) -> Result<Vec<ActivityEvent>, sqlx::Error> {
    sqlx::query_as::<_, ActivityEvent>(
        r#"
        SELECT id, created_at, event, source, txn_ref, zainbox_code, summary, payload
        FROM activity_events
        WHERE txn_ref = $1
        ORDER BY created_at ASC, id ASC
        LIMIT $2
        "#,
    )
    .bind(txn_ref)
    .bind(limit.clamp(1, 500))
    .fetch_all(pool)
    .await
}

pub async fn list_recent(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ActivityEvent>, sqlx::Error> {
    sqlx::query_as::<_, ActivityEvent>(
        r#"
        SELECT id, created_at, event, source, txn_ref, zainbox_code, summary, payload
        FROM activity_events
        ORDER BY created_at DESC, id DESC
        LIMIT $1
        "#,
    )
    .bind(limit.clamp(1, 500))
    .fetch_all(pool)
    .await
}
