//! Merchant compliance sections (business / documents / declaration) + document uploads.
//!
//! Session (`Authorization: Bearer mt_…`):
//! - `GET  /api/v1/auth/compliance`
//! - `PUT  /api/v1/auth/compliance/{section}`
//! - `POST /api/v1/auth/compliance/documents`

use std::path::PathBuf;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::fs;
use uuid::Uuid;

use crate::db;
use crate::db::compliance::{self, ComplianceDocument, ComplianceSection};
use crate::services::merchant_session::MerchantSession;
use crate::state::AppState;

fn ok(data: Value) -> Json<Value> {
    Json(json!({
        "code": "00",
        "status": "200 OK",
        "description": "successful",
        "data": data,
    }))
}

fn map_db(err: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn section_view(row: &ComplianceSection) -> Value {
    json!({
        "section": row.section,
        "status": row.status,
        "data": row.data,
        "submittedAt": row.submitted_at,
        "reviewedAt": row.reviewed_at,
        "updatedAt": row.updated_at,
    })
}

fn document_view(row: &ComplianceDocument) -> Value {
    json!({
        "docType": row.doc_type,
        "name": row.original_name,
        "mime": row.content_type,
        "size": row.byte_size,
        "uploadedAt": row.created_at,
    })
}

pub async fn compliance_snapshot(
    pool: &sqlx::PgPool,
    customer_id: i64,
) -> Result<Value, (StatusCode, String)> {
    let sections = compliance::list_sections(pool, customer_id)
        .await
        .map_err(map_db)?;
    let documents = compliance::list_documents(pool, customer_id)
        .await
        .map_err(map_db)?;

    let mut out = json!({});
    for key in compliance::SECTIONS {
        let row = sections.iter().find(|s| s.section == *key);
        if let Some(row) = row {
            let mut data = row.data.clone();
            if *key == "documents" {
                let mut files = json!({});
                for doc in &documents {
                    files[doc.doc_type.as_str()] = json!({
                        "name": doc.original_name,
                        "mime": doc.content_type,
                        "size": doc.byte_size,
                    });
                }
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("files".into(), files);
                }
            }
            out[*key] = json!({
                "status": row.status,
                "submittedAt": row.submitted_at,
                "data": data,
            });
        } else {
            out[*key] = json!({
                "status": "none",
                "submittedAt": Value::Null,
                "data": {},
            });
        }
    }
    Ok(out)
}

/// `GET /api/v1/auth/compliance`
pub async fn get_compliance(
    session: MerchantSession,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let data = compliance_snapshot(&state.pool, session.customer_id).await?;
    Ok(ok(data))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitSectionRequest {
    #[serde(default)]
    pub data: Value,
}

/// `PUT /api/v1/auth/compliance/{section}` — business / declaration (JSON body).
/// Representative continues to use `PUT /api/v1/auth/kyc`.
pub async fn submit_section(
    session: MerchantSession,
    State(state): State<AppState>,
    Path(section): Path<String>,
    Json(body): Json<SubmitSectionRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if section == "representative" {
        return Err((
            StatusCode::BAD_REQUEST,
            "use PUT /api/v1/auth/kyc for company representative".into(),
        ));
    }
    if section == "documents" {
        return Err((
            StatusCode::BAD_REQUEST,
            "use POST /api/v1/auth/compliance/documents for document uploads".into(),
        ));
    }
    if !compliance::is_valid_section(&section) {
        return Err((StatusCode::BAD_REQUEST, "unknown compliance section".into()));
    }

    let data = if body.data.is_null() {
        json!({})
    } else {
        body.data
    };

    let status = if state.config.merchant_kyc_auto_approve {
        "approved"
    } else {
        "pending"
    };

    let row = compliance::upsert_section(&state.pool, session.customer_id, &section, status, &data)
        .await
        .map_err(map_db)?;

    // Business name is the merchant display name — keep customers.display_name in sync.
    if section == "business" {
        if let Some(name) = data
            .get("businessName")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            db::customers::patch(
                &state.pool,
                session.customer_id,
                db::customers::PatchCustomer {
                    display_name: Some(name),
                    email: None,
                    mobile_number: None,
                    zainbox_code: None,
                    metadata: None,
                    status: None,
                },
            )
            .await
            .map_err(map_db)?;
        }
    }

    Ok(ok(section_view(&row)))
}

/// `POST /api/v1/auth/compliance/documents` — multipart: cacCertificate, utilityBill, notes?
pub async fn submit_documents(
    session: MerchantSession,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer_id = session.customer_id;
    let mut notes: Option<String> = None;
    let mut saved = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "notes" {
            notes = Some(
                field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
            );
            continue;
        }
        if name != "cacCertificate" && name != "utilityBill" {
            continue;
        }

        let original = field
            .file_name()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{name}.bin"));
        let content_type = field.content_type().map(|s| s.to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        if bytes.is_empty() {
            return Err((StatusCode::BAD_REQUEST, format!("{name} file is empty")));
        }
        if bytes.len() > 5 * 1024 * 1024 {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("{name} exceeds 5MB limit"),
            ));
        }

        let ext = std::path::Path::new(&original)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_ascii_lowercase();
        if !matches!(ext.as_str(), "pdf" | "jpg" | "jpeg" | "png") {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("{name} must be pdf, jpg, or png"),
            ));
        }

        let dir = PathBuf::from(&state.config.compliance_storage_dir).join(customer_id.to_string());
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let stored_name = format!("{name}-{}.{}", Uuid::new_v4(), ext);
        let abs = dir.join(&stored_name);
        fs::write(&abs, &bytes)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let rel = format!("{customer_id}/{stored_name}");
        let doc = compliance::upsert_document(
            &state.pool,
            customer_id,
            &name,
            &original,
            content_type.as_deref(),
            bytes.len() as i64,
            &rel,
        )
        .await
        .map_err(map_db)?;
        saved.push(doc);
    }

    let has_cac = saved.iter().any(|d| d.doc_type == "cacCertificate")
        || compliance::list_documents(&state.pool, customer_id)
            .await
            .map_err(map_db)?
            .iter()
            .any(|d| d.doc_type == "cacCertificate");
    let has_utility = saved.iter().any(|d| d.doc_type == "utilityBill")
        || compliance::list_documents(&state.pool, customer_id)
            .await
            .map_err(map_db)?
            .iter()
            .any(|d| d.doc_type == "utilityBill");

    if !has_cac || !has_utility {
        return Err((
            StatusCode::BAD_REQUEST,
            "cacCertificate and utilityBill are required".into(),
        ));
    }

    let docs = compliance::list_documents(&state.pool, customer_id)
        .await
        .map_err(map_db)?;
    let mut files = json!({});
    for doc in &docs {
        files[doc.doc_type.as_str()] = document_view(doc);
    }

    let data = json!({
        "notes": notes,
        "files": files,
    });
    let status = if state.config.merchant_kyc_auto_approve {
        "approved"
    } else {
        "pending"
    };
    let row = compliance::upsert_section(&state.pool, customer_id, "documents", status, &data)
        .await
        .map_err(map_db)?;

    Ok(ok(json!({
        "section": section_view(&row),
        "documents": docs.iter().map(document_view).collect::<Vec<_>>(),
    })))
}

/// Persist representative compliance row alongside `kyc_profiles` upsert.
pub async fn sync_representative_section(
    pool: &sqlx::PgPool,
    customer_id: i64,
    status: &str,
    data: Value,
) -> Result<(), (StatusCode, String)> {
    compliance::upsert_section(pool, customer_id, "representative", status, &data)
        .await
        .map_err(map_db)?;
    Ok(())
}
