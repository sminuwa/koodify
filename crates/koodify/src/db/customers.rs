use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Customer;

const COLUMNS: &str = r#"
    uuid, id, display_name, email, mobile_number, zainbox_code, metadata, status,
    created_at, updated_at
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
