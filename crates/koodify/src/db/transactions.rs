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

/// Atomically set a boolean metadata flag on the oldest matching txn.
/// Returns `true` if this caller won the claim (should send).
/// `flag` must be an internal constant (never user input).
async fn claim_metadata_flag(
    pool: &PgPool,
    resource_key: &str,
    flag: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE transactions
        SET metadata = jsonb_set(
            COALESCE(metadata, '{}'::jsonb),
            $2::text[],
            'true'::jsonb,
            true
        )
        WHERE id = (
            SELECT id
            FROM transactions
            WHERE resource_key = $1
              AND COALESCE(metadata->>$3, 'false') <> 'true'
            ORDER BY created_at ASC
            LIMIT 1
        )
        "#,
    )
    .bind(resource_key)
    .bind(vec![flag.to_string()])
    .bind(flag)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Payer receipt — once per txn.
pub async fn claim_receipt_email(pool: &PgPool, resource_key: &str) -> Result<bool, sqlx::Error> {
    claim_metadata_flag(pool, resource_key, "receiptEmailSent").await
}

/// Merchant / zainbox notification — once per txn.
pub async fn claim_merchant_notice(pool: &PgPool, resource_key: &str) -> Result<bool, sqlx::Error> {
    claim_metadata_flag(pool, resource_key, "merchantNoticeSent").await
}

/// Outbound merchant HTTP webhook — once per txn.
pub async fn claim_merchant_webhook(pool: &PgPool, resource_key: &str) -> Result<bool, sqlx::Error> {
    claim_metadata_flag(pool, resource_key, "merchantWebhookSent").await
}

/// Marks that success side effects were applied (emails / split / webhook attempts).
/// Used by the reconciler to stop re-polling confirmed payments.
pub async fn mark_payment_success_handled(
    pool: &PgPool,
    resource_key: &str,
) -> Result<bool, sqlx::Error> {
    claim_metadata_flag(pool, resource_key, "paymentSuccessHandled").await
}

/// Checkout init row used by the pending reconciler.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingCheckout {
    pub resource_key: String,
    pub zainbox_code: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Recent unpaid checkouts that still need a status poll.
///
/// Skips txns already covered by an inbound Zainpay `deposit.success` webhook,
/// already notified via merchant webhook, or polled within `poll_cooldown`.
pub async fn list_pending_checkouts(
    pool: &PgPool,
    lookback: chrono::Duration,
    grace: chrono::Duration,
    poll_cooldown: chrono::Duration,
    limit: i64,
) -> Result<Vec<PendingCheckout>, sqlx::Error> {
    let now = chrono::Utc::now();
    let oldest = now - lookback;
    let newest = now - grace;
    let cooldown_secs = poll_cooldown.num_seconds().max(0);

    sqlx::query_as::<_, PendingCheckout>(
        r#"
        SELECT DISTINCT ON (t.resource_key)
            t.resource_key,
            t.zainbox_code,
            t.created_at
        FROM transactions t
        WHERE t.operation IN ('card.initialize', 'card.initialize_inline')
          AND t.resource_key IS NOT NULL
          AND t.created_at >= $1
          AND t.created_at <= $2
          AND COALESCE(t.metadata->>'merchantWebhookSent', 'false') <> 'true'
          AND COALESCE(t.metadata->>'paymentSuccessHandled', 'false') <> 'true'
          AND NOT EXISTS (
              SELECT 1
              FROM webhook_deliveries w
              WHERE w.resource_key = t.resource_key
                AND w.event_name = 'deposit.success'
          )
          AND (
              COALESCE(t.metadata->>'reconcileLastPolledAt', '') = ''
              OR (t.metadata->>'reconcileLastPolledAt')::timestamptz
                 < now() - ($3 * INTERVAL '1 second')
          )
        ORDER BY t.resource_key, t.created_at DESC
        LIMIT $4
        "#,
    )
    .bind(oldest)
    .bind(newest)
    .bind(cooldown_secs)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Stamp `metadata.reconcileLastPolledAt` on the oldest matching txn row.
pub async fn touch_reconcile_polled(
    pool: &PgPool,
    resource_key: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE transactions
        SET metadata = jsonb_set(
            COALESCE(metadata, '{}'::jsonb),
            '{reconcileLastPolledAt}',
            to_jsonb($2::text),
            true
        )
        WHERE id = (
            SELECT id
            FROM transactions
            WHERE resource_key = $1
            ORDER BY created_at ASC
            LIMIT 1
        )
        "#,
    )
    .bind(resource_key)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
