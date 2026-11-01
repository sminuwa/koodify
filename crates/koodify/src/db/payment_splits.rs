use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSplit {
    pub txn_ref: String,
    pub primary_zainbox_code: String,
    pub checkout_amount: String,
    pub total_kobo: i64,
    pub plan_snapshot: Value,
    pub status: String,
    pub source_account: Option<String>,
    pub source_bank_code: Option<String>,
    pub error: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSplitLeg {
    pub id: Uuid,
    pub txn_ref: String,
    pub leg_index: i32,
    pub zainbox_code: String,
    pub role: String,
    pub amount_kobo: i64,
    pub share_percent: Option<f64>,
    pub share_amount: Option<String>,
    pub status: String,
    pub dest_account: Option<String>,
    pub dest_bank_code: Option<String>,
    pub transfer_txn_ref: Option<String>,
    pub payment_ref: Option<String>,
    pub error: Option<String>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewSplitLeg<'a> {
    pub leg_index: i32,
    pub zainbox_code: &'a str,
    pub role: &'a str,
    pub amount_kobo: i64,
    pub share_percent: Option<f64>,
    pub share_amount: Option<&'a str>,
}

pub async fn insert_pending(
    pool: &PgPool,
    txn_ref: &str,
    primary_zainbox_code: &str,
    checkout_amount: &str,
    total_kobo: i64,
    plan_snapshot: &Value,
    legs: &[NewSplitLeg<'_>],
) -> Result<PaymentSplit, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let split = sqlx::query_as::<_, PaymentSplit>(
        r#"
        INSERT INTO payment_splits (
            txn_ref, primary_zainbox_code, checkout_amount, total_kobo,
            plan_snapshot, status
        )
        VALUES ($1, $2, $3, $4, $5, 'pending')
        ON CONFLICT (txn_ref) DO UPDATE SET
            primary_zainbox_code = EXCLUDED.primary_zainbox_code,
            checkout_amount = EXCLUDED.checkout_amount,
            total_kobo = EXCLUDED.total_kobo,
            plan_snapshot = EXCLUDED.plan_snapshot,
            status = CASE
                WHEN payment_splits.status IN ('success', 'running', 'partial')
                    THEN payment_splits.status
                ELSE 'pending'
            END,
            updated_at = now()
        RETURNING
            txn_ref, primary_zainbox_code, checkout_amount, total_kobo, plan_snapshot,
            status, source_account, source_bank_code, error,
            claimed_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(txn_ref)
    .bind(primary_zainbox_code)
    .bind(checkout_amount)
    .bind(total_kobo)
    .bind(plan_snapshot)
    .fetch_one(&mut *tx)
    .await?;

    // Replace legs only while still pending (avoid clobbering an in-flight run).
    if split.status == "pending" {
        sqlx::query("DELETE FROM payment_split_legs WHERE txn_ref = $1")
            .bind(txn_ref)
            .execute(&mut *tx)
            .await?;

        for leg in legs {
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO payment_split_legs (
                    id, txn_ref, leg_index, zainbox_code, role, amount_kobo,
                    share_percent, share_amount, status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
                "#,
            )
            .bind(id)
            .bind(txn_ref)
            .bind(leg.leg_index)
            .bind(leg.zainbox_code)
            .bind(leg.role)
            .bind(leg.amount_kobo)
            .bind(leg.share_percent)
            .bind(leg.share_amount)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(split)
}

/// How long a `running` claim is exclusive before another worker may reclaim it.
/// Prevents crash mid-`execute_split` from leaving funds stuck forever.
pub const RUNNING_LEASE_SECS: i64 = 600;

/// Atomically claim a pending split for execution. Returns the row if this caller won.
pub async fn claim_pending(pool: &PgPool, txn_ref: &str) -> Result<Option<PaymentSplit>, sqlx::Error> {
    claim_for_run(pool, txn_ref, false).await
}

/// Claim for execution.
///
/// - Always claims `pending`.
/// - When `allow_retry` is true, also reclaims `failed` / `partial`.
/// - Always reclaims `running` whose lease (`claimed_at`) is older than
///   [`RUNNING_LEASE_SECS`] (crash recovery).
pub async fn claim_for_run(
    pool: &PgPool,
    txn_ref: &str,
    allow_retry: bool,
) -> Result<Option<PaymentSplit>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let split = sqlx::query_as::<_, PaymentSplit>(
        r#"
        UPDATE payment_splits
        SET status = 'running',
            claimed_at = now(),
            completed_at = NULL,
            updated_at = now(),
            error = NULL
        WHERE txn_ref = $1
          AND (
            status = 'pending'
            OR ($2 AND status IN ('failed', 'partial'))
            OR (
              status = 'running'
              AND COALESCE(claimed_at, updated_at)
                  < now() - make_interval(secs => $3::double precision)
            )
          )
        RETURNING
            txn_ref, primary_zainbox_code, checkout_amount, total_kobo, plan_snapshot,
            status, source_account, source_bank_code, error,
            claimed_at, completed_at, created_at, updated_at
        "#,
    )
    .bind(txn_ref)
    .bind(allow_retry)
    .bind(RUNNING_LEASE_SECS as f64)
    .fetch_optional(&mut *tx)
    .await?;

    if split.is_some() {
        // Re-queue transfer legs that did not succeed; keep successful ones.
        // Preserve transfer_txn_ref / payment_ref so retries verify the same
        // idempotency key instead of minting a new one (avoids double-pay).
        sqlx::query(
            r#"
            UPDATE payment_split_legs
            SET status = 'pending',
                error = NULL,
                updated_at = now()
            WHERE txn_ref = $1
              AND role = 'transfer'
              AND status <> 'success'
            "#,
        )
        .bind(txn_ref)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(split)
}

pub async fn get(pool: &PgPool, txn_ref: &str) -> Result<Option<PaymentSplit>, sqlx::Error> {
    sqlx::query_as::<_, PaymentSplit>(
        r#"
        SELECT
            txn_ref, primary_zainbox_code, checkout_amount, total_kobo, plan_snapshot,
            status, source_account, source_bank_code, error,
            claimed_at, completed_at, created_at, updated_at
        FROM payment_splits
        WHERE txn_ref = $1
        "#,
    )
    .bind(txn_ref)
    .fetch_optional(pool)
    .await
}

pub async fn list_legs(pool: &PgPool, txn_ref: &str) -> Result<Vec<PaymentSplitLeg>, sqlx::Error> {
    sqlx::query_as::<_, PaymentSplitLeg>(
        r#"
        SELECT
            id, txn_ref, leg_index, zainbox_code, role, amount_kobo,
            share_percent, share_amount, status, dest_account, dest_bank_code,
            transfer_txn_ref, payment_ref, error, attempts, created_at, updated_at
        FROM payment_split_legs
        WHERE txn_ref = $1
        ORDER BY leg_index ASC
        "#,
    )
    .bind(txn_ref)
    .fetch_all(pool)
    .await
}

pub async fn set_source(
    pool: &PgPool,
    txn_ref: &str,
    account: &str,
    bank_code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE payment_splits
        SET source_account = $2,
            source_bank_code = $3,
            updated_at = now()
        WHERE txn_ref = $1
          AND (source_account IS NULL OR source_account = '')
        "#,
    )
    .bind(txn_ref)
    .bind(account)
    .bind(bank_code)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_finished(
    pool: &PgPool,
    txn_ref: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE payment_splits
        SET status = $2,
            error = $3,
            completed_at = now(),
            updated_at = now()
        WHERE txn_ref = $1
        "#,
    )
    .bind(txn_ref)
    .bind(status)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_leg_result(
    pool: &PgPool,
    leg_id: Uuid,
    status: &str,
    dest_account: Option<&str>,
    dest_bank_code: Option<&str>,
    transfer_txn_ref: Option<&str>,
    payment_ref: Option<&str>,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE payment_split_legs
        SET status = $2,
            dest_account = COALESCE($3, dest_account),
            dest_bank_code = COALESCE($4, dest_bank_code),
            transfer_txn_ref = COALESCE($5, transfer_txn_ref),
            payment_ref = COALESCE($6, payment_ref),
            error = $7,
            attempts = attempts + 1,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(leg_id)
    .bind(status)
    .bind(dest_account)
    .bind(dest_bank_code)
    .bind(transfer_txn_ref)
    .bind(payment_ref)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

/// Summary JSON for status enrichment.
pub async fn summary_json(pool: &PgPool, txn_ref: &str) -> Result<Option<Value>, sqlx::Error> {
    let Some(split) = get(pool, txn_ref).await? else {
        return Ok(None);
    };
    let legs = list_legs(pool, txn_ref).await?;
    Ok(Some(serde_json::json!({
        "txnRef": split.txn_ref,
        "primaryZainboxCode": split.primary_zainbox_code,
        "checkoutAmount": split.checkout_amount,
        "totalKobo": split.total_kobo,
        "status": split.status,
        "sourceAccount": split.source_account,
        "sourceBankCode": split.source_bank_code,
        "error": split.error,
        "legs": legs.iter().map(|l| serde_json::json!({
            "index": l.leg_index,
            "zainboxCode": l.zainbox_code,
            "role": l.role,
            "amountKobo": l.amount_kobo,
            "percent": l.share_percent,
            "amount": l.share_amount,
            "status": l.status,
            "destAccount": l.dest_account,
            "transferTxnRef": l.transfer_txn_ref,
            "paymentRef": l.payment_ref,
            "error": l.error,
            "attempts": l.attempts,
        })).collect::<Vec<_>>(),
    })))
}
