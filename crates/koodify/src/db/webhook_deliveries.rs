use sqlx::PgPool;

/// Try to record a delivery. Returns `true` if this is the first time we've
/// seen `delivery_hash` (caller should process side effects). Returns `false`
/// on unique conflict (replay — acknowledge without re-running money side effects).
pub async fn try_claim(
    pool: &PgPool,
    delivery_hash: &str,
    resource_key: Option<&str>,
    event_name: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO webhook_deliveries (delivery_hash, resource_key, event_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (delivery_hash) DO NOTHING
        "#,
    )
    .bind(delivery_hash)
    .bind(resource_key)
    .bind(event_name)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
