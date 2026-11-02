use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Customer;

const COLUMNS: &str = r#"
    uuid, id, display_name, email, mobile_number, zainbox_code, metadata, status,
    password_hash, created_at, updated_at
"#;

pub struct CreateCustomer<'a> {
    pub display_name: &'a str,
    pub email: Option<&'a str>,
    pub mobile_number: Option<&'a str>,
    pub zainbox_code: Option<&'a str>,
    pub metadata: &'a Value,
    pub status: &'a str,
}

pub struct UpdateCustomer<'a> {
    pub display_name: &'a str,
    pub email: Option<&'a str>,
    pub mobile_number: Option<&'a str>,
    pub zainbox_code: Option<&'a str>,
    pub metadata: &'a Value,
    pub status: &'a str,
}

pub struct PatchCustomer<'a> {
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub mobile_number: Option<&'a str>,
    pub zainbox_code: Option<&'a str>,
    pub metadata: Option<&'a Value>,
    pub status: Option<&'a str>,
}

pub async fn create(pool: &PgPool, input: CreateCustomer<'_>) -> Result<Customer, sqlx::Error> {
    let uuid = Uuid::new_v4();
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        INSERT INTO customers (
            uuid, display_name, email, mobile_number, zainbox_code, metadata, status
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        RETURNING {COLUMNS}
        "#
    ))
    .bind(uuid)
    .bind(input.display_name)
    .bind(input.email)
    .bind(input.mobile_number)
    .bind(input.zainbox_code)
    .bind(input.metadata)
    .bind(input.status)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: i64,
    input: UpdateCustomer<'_>,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        UPDATE customers
        SET display_name = $2,
            email = $3,
            mobile_number = $4,
            zainbox_code = $5,
            metadata = $6,
            status = $7,
            updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(input.display_name)
    .bind(input.email)
    .bind(input.mobile_number)
    .bind(input.zainbox_code)
    .bind(input.metadata)
    .bind(input.status)
    .fetch_optional(pool)
    .await
}

/// Partial update — only non-`None` fields are written.
pub async fn patch(
    pool: &PgPool,
    id: i64,
    input: PatchCustomer<'_>,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        UPDATE customers
        SET display_name = COALESCE($2, display_name),
            email = COALESCE($3, email),
            mobile_number = COALESCE($4, mobile_number),
            zainbox_code = COALESCE($5, zainbox_code),
            metadata = COALESCE($6, metadata),
            status = COALESCE($7, status),
            updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(input.display_name)
    .bind(input.email)
    .bind(input.mobile_number)
    .bind(input.zainbox_code)
    .bind(input.metadata)
    .bind(input.status)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"SELECT {COLUMNS} FROM customers WHERE id = $1"#
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_uuid(pool: &PgPool, uuid: Uuid) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"SELECT {COLUMNS} FROM customers WHERE uuid = $1"#
    ))
    .bind(uuid)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        SELECT {COLUMNS}
        FROM customers
        WHERE email IS NOT NULL
          AND lower(email) = lower($1)
        "#
    ))
    .bind(email.trim())
    .fetch_optional(pool)
    .await
}

pub async fn list(
    pool: &PgPool,
    status: Option<&str>,
) -> Result<Vec<Customer>, sqlx::Error> {
    if let Some(status) = status {
        sqlx::query_as::<_, Customer>(&format!(
            r#"
            SELECT {COLUMNS}
            FROM customers
            WHERE status = $1
            ORDER BY id ASC
            "#
        ))
        .bind(status)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Customer>(&format!(
            r#"
            SELECT {COLUMNS}
            FROM customers
            ORDER BY id ASC
            "#
        ))
        .fetch_all(pool)
        .await
    }
}

pub async fn set_status(
    pool: &PgPool,
    id: i64,
    status: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        UPDATE customers
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

pub async fn set_zainbox_code(
    pool: &PgPool,
    id: i64,
    zainbox_code: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        UPDATE customers
        SET zainbox_code = $2, updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(zainbox_code)
    .fetch_optional(pool)
    .await
}

/// Store a password hash for future merchant login. Hashing is done by the caller.
pub async fn set_password_hash(
    pool: &PgPool,
    id: i64,
    password_hash: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(&format!(
        r#"
        UPDATE customers
        SET password_hash = $2, updated_at = now()
        WHERE id = $1
        RETURNING {COLUMNS}
        "#
    ))
    .bind(id)
    .bind(password_hash)
    .fetch_optional(pool)
    .await
}

pub async fn has_password(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT (password_hash IS NOT NULL AND password_hash <> '') AS has_pw
        FROM customers
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0).unwrap_or(false))
}
