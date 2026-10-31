use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
use client::{ApiResponse, VirtualAccountData};

use super::models::VirtualAccountRecord;

pub async fn insert_success(
    pool: &PgPool,
    zainbox_code: &str,
    metadata: &Value,
    response: &ApiResponse<VirtualAccountData>,
    data: &VirtualAccountData,
) -> Result<VirtualAccountRecord, sqlx::Error> {
    let id = Uuid::new_v4();
    let raw_response = serde_json::to_value(response).unwrap_or(Value::Null);

    sqlx::query_as::<_, VirtualAccountRecord>(
        r#"
        INSERT INTO virtual_accounts (
            id,
            account_number,
            account_name,
            bank_name,
            email,
            zainbox_code,
            zainpay_code,
            metadata,
            raw_response
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id,
            account_number,
            account_name,
            bank_name,
            email,
            zainbox_code,
            zainpay_code,
            metadata,
            raw_response,
            created_at
        "#,
    )
    .bind(id)
    .bind(&data.account_number)
    .bind(&data.account_name)
    .bind(&data.bank_name)
    .bind(&data.email)
    .bind(zainbox_code)
    .bind(&response.code)
    .bind(metadata)
    .bind(raw_response)
    .fetch_one(pool)
    .await
}

pub async fn find_by_account_number(
    pool: &PgPool,
    account_number: &str,
) -> Result<Option<VirtualAccountRecord>, sqlx::Error> {
    sqlx::query_as::<_, VirtualAccountRecord>(
        r#"
        SELECT
            id,
            account_number,
            account_name,
            bank_name,
            email,
            zainbox_code,
            zainpay_code,
            metadata,
            raw_response,
            created_at
        FROM virtual_accounts
        WHERE account_number = $1
        "#,
    )
    .bind(account_number)
    .fetch_optional(pool)
    .await
}
