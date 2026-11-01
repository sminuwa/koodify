use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Zainbox {
    pub zainbox_code: String,
    pub name: String,
    pub callback_url: Option<String>,
    pub email_notification: Option<String>,
    pub tags: Option<String>,
    pub is_active: bool,
    /// Browser-safe publishable key (`pk_live_…`). Maps checkout → this zainbox.
    pub public_key: Option<String>,
    pub raw: Value,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UpsertZainbox<'a> {
    pub zainbox_code: &'a str,
    pub name: &'a str,
    pub callback_url: Option<&'a str>,
    pub email_notification: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub is_active: bool,
    pub raw: &'a Value,
}

const COLUMNS: &str = r#"
    zainbox_code, name, callback_url, email_notification, tags, is_active, public_key, raw,
    synced_at, created_at, updated_at
"#;

pub async fn upsert(pool: &PgPool, input: UpsertZainbox<'_>) -> Result<Zainbox, sqlx::Error> {
    // Preserve an existing public_key on conflict so merchant pages keep working.
    sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        INSERT INTO zainboxes (
            zainbox_code, name, callback_url, email_notification, tags, is_active, raw,
            synced_at, updated_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7, now(), now())
        ON CONFLICT (zainbox_code) DO UPDATE SET
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
    .bind(input.zainbox_code)
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
        ORDER BY name ASC, zainbox_code ASC
        "#
    ))
    .fetch_all(pool)
    .await
}

pub async fn get_by_code(pool: &PgPool, zainbox_code: &str) -> Result<Option<Zainbox>, sqlx::Error> {
    sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM zainboxes
        WHERE zainbox_code = $1
        "#
    ))
    .bind(zainbox_code)
    .fetch_optional(pool)
    .await
}

pub async fn get_by_public_key(
    pool: &PgPool,
    public_key: &str,
) -> Result<Option<Zainbox>, sqlx::Error> {
    sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM zainboxes
        WHERE public_key = $1
        "#
    ))
    .bind(public_key)
    .fetch_optional(pool)
    .await
}

/// Assign a publishable key if missing. Returns the (possibly new) key.
pub async fn ensure_public_key(pool: &PgPool, zainbox_code: &str) -> Result<String, sqlx::Error> {
    if let Some(existing) = get_by_code(pool, zainbox_code).await? {
        if let Some(pk) = existing.public_key.filter(|s| !s.is_empty()) {
            return Ok(pk);
        }
    }

    let public_key = format!("pk_live_{}", Uuid::new_v4().simple());
    let row = sqlx::query_as::<_, Zainbox>(&format!(
        r#"
        UPDATE zainboxes
        SET public_key = $2, updated_at = now()
        WHERE zainbox_code = $1
          AND (public_key IS NULL OR public_key = '')
        RETURNING {COLUMNS}
        "#
    ))
    .bind(zainbox_code)
    .bind(&public_key)
    .fetch_optional(pool)
    .await?;

    if let Some(z) = row {
        return Ok(z.public_key.unwrap_or(public_key));
    }

    // Race: another writer set it — re-read.
    match get_by_code(pool, zainbox_code).await? {
        Some(z) => Ok(z
            .public_key
            .filter(|s| !s.is_empty())
            .unwrap_or(public_key)),
        None => Err(sqlx::Error::RowNotFound),
    }
}
