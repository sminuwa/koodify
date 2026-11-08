use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub email: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub email: Option<&'a str>,
}

pub async fn find_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, display_name, email, status, created_at, updated_at
        FROM users
        WHERE lower(username) = lower($1)
        "#,
    )
    .bind(username.trim())
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, display_name, email, status, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as("SELECT count(*)::bigint FROM users")
        .fetch_one(pool)
        .await?;
    Ok(n)
}

pub async fn create(pool: &PgPool, input: CreateUser<'_>) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password_hash, display_name, email, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        RETURNING id, username, password_hash, display_name, email, status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(input.username.trim())
    .bind(input.password_hash)
    .bind(input.display_name.trim())
    .bind(input.email.map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(pool)
    .await
}

/// Ensure at least one active admin exists (from env defaults on first boot).
pub async fn ensure_bootstrap_admin(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
    display_name: &str,
    email: Option<&str>,
) -> Result<Option<User>, sqlx::Error> {
    if count(pool).await? > 0 {
        return Ok(None);
    }
    let user = create(
        pool,
        CreateUser {
            username,
            password_hash,
            display_name,
            email,
        },
    )
    .await?;
    Ok(Some(user))
}
