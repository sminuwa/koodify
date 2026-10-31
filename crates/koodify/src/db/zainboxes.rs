use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Zainbox {
    pub code_name: String,
    pub name: String,
    pub callback_url: Option<String>,
    pub email_notification: Option<String>,
    pub tags: Option<String>,
    pub is_active: bool,
    pub raw: Value,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UpsertZainbox<'a> {
    pub code_name: &'a str,
    pub name: &'a str,
    pub callback_url: Option<&'a str>,
    pub email_notification: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub is_active: bool,
    pub raw: &'a Value,
}

const COLUMNS: &str = r#"
    code_name, name, callback_url, email_notification, tags, is_active, raw,
    synced_at, created_at, updated_at
"#;

pub async fn upsert(pool: &PgPool, input: UpsertZainbox<'_>) -> Result<Zainbox, sqlx::Error> {
    sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        INSERT INTO zainboxes (
            code_name, name, callback_url, email_notification, tags, is_active, raw,
            synced_at, updated_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7, now(), now())
        ON CONFLICT (code_name) DO UPDATE SET
            name = EXCLUDED.name,
            callback_url = EXCLUDED.callback_url,
            email_notification = EXCLUDED.email_notification,
            tags = EXCLUDED.tags,
            is_active = EXCLUDED.is_active,
            raw = EXCLUDED.raw,
            synced_at = now(),
            updated_at = now()
        RETURNING {COLUMNS}
        "#
    ))
    .bind(input.code_name)
    .bind(input.name)
    .bind(input.callback_url)
    .bind(input.email_notification)
    .bind(input.tags)
    .bind(input.is_active)
    .bind(input.raw)
    .fetch_one(pool)
    .await
}

pub async fn list(pool: &PgPool) -> Result<Vec<Zainbox>, sqlx::Error> {
    sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM zainboxes
        ORDER BY name ASC, code_name ASC
        "#
    ))
    .fetch_all(pool)
    .await
}
