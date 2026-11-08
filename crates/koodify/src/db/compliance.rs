use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub const SECTIONS: &[&str] = &["business", "representative", "documents", "declaration"];

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceSection {
    pub id: Uuid,
    pub customer_id: i64,
    pub section: String,
    pub status: String,
    pub data: Value,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceDocument {
    pub id: Uuid,
    pub customer_id: i64,
    pub doc_type: String,
    pub original_name: String,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

pub fn is_valid_section(section: &str) -> bool {
    SECTIONS.iter().any(|s| *s == section)
}

pub async fn upsert_section(
    pool: &PgPool,
    customer_id: i64,
    section: &str,
    status: &str,
    data: &Value,
) -> Result<ComplianceSection, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, ComplianceSection>(
        r#"
        INSERT INTO compliance_sections (id, customer_id, section, status, data, submitted_at)
        VALUES ($1, $2, $3, $4, $5, now())
        ON CONFLICT (customer_id, section) DO UPDATE SET
            status = EXCLUDED.status,
            data = EXCLUDED.data,
            submitted_at = now(),
            reviewed_at = NULL,
            updated_at = now()
        RETURNING
            id, customer_id, section, status, data,
            submitted_at, reviewed_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(customer_id)
    .bind(section)
    .bind(status)
    .bind(data)
    .fetch_one(pool)
    .await
}

pub async fn list_sections(
    pool: &PgPool,
    customer_id: i64,
) -> Result<Vec<ComplianceSection>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceSection>(
        r#"
        SELECT
            id, customer_id, section, status, data,
            submitted_at, reviewed_at, created_at, updated_at
        FROM compliance_sections
        WHERE customer_id = $1
        ORDER BY section ASC
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
}

pub async fn find_section(
    pool: &PgPool,
    customer_id: i64,
    section: &str,
) -> Result<Option<ComplianceSection>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceSection>(
        r#"
        SELECT
            id, customer_id, section, status, data,
            submitted_at, reviewed_at, created_at, updated_at
        FROM compliance_sections
        WHERE customer_id = $1 AND section = $2
        "#,
    )
    .bind(customer_id)
    .bind(section)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_document(
    pool: &PgPool,
    customer_id: i64,
    doc_type: &str,
    original_name: &str,
    content_type: Option<&str>,
    byte_size: i64,
    storage_path: &str,
) -> Result<ComplianceDocument, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query_as::<_, ComplianceDocument>(
        r#"
        INSERT INTO compliance_documents (
            id, customer_id, doc_type, original_name, content_type, byte_size, storage_path
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (customer_id, doc_type) DO UPDATE SET
            original_name = EXCLUDED.original_name,
            content_type = EXCLUDED.content_type,
            byte_size = EXCLUDED.byte_size,
            storage_path = EXCLUDED.storage_path,
            created_at = now()
        RETURNING
            id, customer_id, doc_type, original_name, content_type,
            byte_size, storage_path, created_at
        "#,
    )
    .bind(id)
    .bind(customer_id)
    .bind(doc_type)
    .bind(original_name)
    .bind(content_type)
    .bind(byte_size)
    .bind(storage_path)
    .fetch_one(pool)
    .await
}

/// Mark every existing compliance section for a customer with the given status
/// (used when admin approves KYC so portal sections match the KYC decision).
pub async fn set_all_sections_status(
    pool: &PgPool,
    customer_id: i64,
    status: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE compliance_sections
        SET status = $2,
            reviewed_at = now(),
            updated_at = now()
        WHERE customer_id = $1
        "#,
    )
    .bind(customer_id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn list_documents(
    pool: &PgPool,
    customer_id: i64,
) -> Result<Vec<ComplianceDocument>, sqlx::Error> {
    sqlx::query_as::<_, ComplianceDocument>(
        r#"
        SELECT
            id, customer_id, doc_type, original_name, content_type,
            byte_size, storage_path, created_at
        FROM compliance_documents
        WHERE customer_id = $1
        ORDER BY doc_type ASC
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
}
