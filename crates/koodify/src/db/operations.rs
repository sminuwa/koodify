use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::OperationRecord;

pub async fn insert(
    pool: &PgPool,
    operation: &str,
    resource_key: Option<&str>,
    metadata: &Value,
    request_body: Option<&Value>,
    response_body: &Value,
    zainpay_code: &str,
) -> Result<OperationRecord, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, OperationRecord>(
        r#"
        INSERT INTO operation_records (
            id, operation, resource_key, metadata, request_body, response_body, zainpay_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id, operation, resource_key, metadata, request_body, response_body, zainpay_code, created_at
        "#,
    )
    .bind(id)
    .bind(operation)
    .bind(resource_key)
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
) -> Result<Vec<OperationRecord>, sqlx::Error> {
    sqlx::query_as::<_, OperationRecord>(
        r#"
        SELECT id, operation, resource_key, metadata, request_body, response_body, zainpay_code, created_at
        FROM operation_records
        WHERE resource_key = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(resource_key)
    .fetch_all(pool)
    .await
}
