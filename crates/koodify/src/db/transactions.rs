use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::TransactionRecord;

pub async fn insert(
    pool: &PgPool,
    operation: &str,
    resource_key: Option<&str>,
    zainbox_code: Option<&str>,
    metadata: &Value,
    request_body: Option<&Value>,
    response_body: &Value,
    zainpay_code: &str,
) -> Result<TransactionRecord, sqlx::Error> {
    let id = Uuid::new_v4();
    let zainbox = zainbox_code
        .map(str::trim)
        .filter(|s| !s.is_empty());
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        INSERT INTO transactions (
            id, operation, resource_key, zainbox_code, metadata,
            request_body, response_body, zainpay_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id, operation, resource_key, zainbox_code, metadata,
            request_body, response_body, zainpay_code, created_at
        "#,
    )
    .bind(id)
    .bind(operation)
    .bind(resource_key)
    .bind(zainbox)
    .bind(metadata)
    .bind(request_body)
    .bind(response_body)
    .bind(zainpay_code)
    .fetch_one(pool)
    .await
}

pub async fn find_by_resource_key(
    pool: &PgPool,
    resource_key: &str,
) -> Result<Vec<TransactionRecord>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        SELECT id, operation, resource_key, zainbox_code, metadata,
               request_body, response_body, zainpay_code, created_at
        FROM transactions
        WHERE resource_key = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(resource_key)
    .fetch_all(pool)
    .await
}

pub async fn find_by_zainbox_code(
    pool: &PgPool,
    zainbox_code: &str,
) -> Result<Vec<TransactionRecord>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        SELECT id, operation, resource_key, zainbox_code, metadata,
               request_body, response_body, zainpay_code, created_at
        FROM transactions
        WHERE zainbox_code = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(zainbox_code)
    .fetch_all(pool)
    .await
}

pub async fn find_by_zainbox_and_resource(
    pool: &PgPool,
    zainbox_code: &str,
    resource_key: &str,
) -> Result<Vec<TransactionRecord>, sqlx::Error> {
    sqlx::query_as::<_, TransactionRecord>(
        r#"
        SELECT id, operation, resource_key, zainbox_code, metadata,
               request_body, response_body, zainpay_code, created_at
        FROM transactions
        WHERE zainbox_code = $1 AND resource_key = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(zainbox_code)
    .bind(resource_key)
    .fetch_all(pool)
    .await
}

/// Atomically mark a txn as receipt-emailed. Returns `true` if this caller
/// won the claim (should send). Prevents duplicate Resend calls on poll.
pub async fn claim_receipt_email(pool: &PgPool, resource_key: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE transactions
        SET metadata = jsonb_set(
            COALESCE(metadata, '{}'::jsonb),
            '{receiptEmailSent}',
            'true'::jsonb,
            true
        )
        WHERE id = (
            SELECT id
            FROM transactions
            WHERE resource_key = $1
              AND COALESCE(metadata->>'receiptEmailSent', 'false') <> 'true'
            ORDER BY created_at ASC
            LIMIT 1
        )
        "#,
    )
    .bind(resource_key)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
