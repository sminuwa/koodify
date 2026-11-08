//! Admin portal APIs — login against `users`, dashboard, section + KYC approve.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

use crate::db;
use crate::db::kyc::is_approved_status;
use crate::routes::customers::load_detail;
use crate::routes::merchant_compliance::compliance_snapshot;
use crate::services::admin_session::{AdminPortalAuth, AdminSession};
use crate::services::mail;
use crate::services::merchant_provision;
use crate::services::password::{hash_password, verify_password};
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionStatusRequest {
    pub status: String,
}

/// `POST /api/v1/admin/auth/login`
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<AdminLoginRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let username = body.username.trim();
    let password = body.password;
    if username.is_empty() || password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "username and password are required".into(),
        ));
    }

    let user = db::users::find_by_username(&state.pool, username)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "invalid username or password".into()))?;

    if user.status != "active" {
        return Err((StatusCode::UNAUTHORIZED, "admin user is disabled".into()));
    }
    if !verify_password(&password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "invalid username or password".into()));
    }

    let token = AdminSession::issue(user.id, &state.config.gateway_api_key);
    Ok(ok(json!({
        "token": token,
        "tokenType": "Bearer",
        "user": {
            "id": user.id,
            "username": user.username,
            "displayName": user.display_name,
            "email": user.email,
        }
    })))
}

/// `GET /api/v1/admin/auth/me`
pub async fn me(
    session: AdminSession,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let user = db::users::find_by_id(&state.pool, session.user_id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "admin user not found".into()))?;
    Ok(ok(json!({
        "id": user.id,
        "username": user.username,
        "displayName": user.display_name,
        "email": user.email,
        "status": user.status,
    })))
}

/// `GET /api/v1/admin/dashboard`
pub async fn dashboard(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customers = db::customers::list(&state.pool, None)
        .await
        .map_err(map_db)?;

    let mut pending_kyc = 0i64;
    let mut approved_kyc = 0i64;
    let mut awaiting_provision = 0i64;
    let mut pending_sections = 0i64;
    let mut pending_queue: Vec<Value> = Vec::new();

    for c in &customers {
        let kyc = db::kyc::find_by_customer_id(&state.pool, c.id)
            .await
            .map_err(map_db)?;
        let sections = db::compliance::list_sections(&state.pool, c.id)
            .await
            .map_err(map_db)?;
        let section_pending = sections.iter().filter(|s| s.status == "pending").count() as i64;
        pending_sections += section_pending;

        let kyc_status = kyc
            .as_ref()
            .map(|k| k.status.as_str())
            .unwrap_or("none");
        if kyc_status == "pending" || section_pending > 0 {
            pending_kyc += 1;
            pending_queue.push(json!({
                "id": c.id,
                "displayName": c.display_name,
                "email": c.email,
                "kycStatus": kyc_status,
                "pendingSections": section_pending,
                "status": c.status,
            }));
        }
        if kyc.as_ref().is_some_and(|k| is_approved_status(&k.status)) {
            approved_kyc += 1;
            let has_box = c
                .zainbox_code
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some();
            if !has_box {
                awaiting_provision += 1;
            }
        }
    }

    pending_queue.sort_by(|a, b| {
        let an = a.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
        let bn = b.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    Ok(ok(json!({
        "merchants": customers.len(),
        "pendingKyc": pending_kyc,
        "approvedKyc": approved_kyc,
        "pendingComplianceSections": pending_sections,
        "awaitingProvision": awaiting_provision,
        "pendingQueue": pending_queue.into_iter().take(25).collect::<Vec<_>>(),
    })))
}

/// `GET /api/v1/admin/merchants`
pub async fn list_merchants(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customers = db::customers::list(&state.pool, None)
        .await
        .map_err(map_db)?;
    let mut out = Vec::with_capacity(customers.len());
    for c in customers {
        let kyc = db::kyc::find_by_customer_id(&state.pool, c.id)
            .await
            .map_err(map_db)?;
        let sections = db::compliance::list_sections(&state.pool, c.id)
            .await
            .map_err(map_db)?;
        let pending_sections = sections.iter().filter(|s| s.status == "pending").count();
        out.push(json!({
            "id": c.id,
            "displayName": c.display_name,
            "email": c.email,
            "mobileNumber": c.mobile_number,
            "status": c.status,
            "zainboxCode": c.zainbox_code,
            "kycStatus": kyc
                .as_ref()
                .map(|k| k.status.clone())
                .unwrap_or_else(|| "none".to_string()),
            "pendingSections": pending_sections,
            "createdAt": c.created_at,
        }));
    }
    Ok(ok(Value::Array(out)))
}

/// `GET /api/v1/admin/merchants/{id}`
pub async fn merchant_detail(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "merchant not found".into()))?;
    let detail = load_detail(&state, customer, true).await?;
    let mut data = serde_json::to_value(&detail).unwrap_or(Value::Null);
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "compliance".into(),
            compliance_snapshot(&state.pool, id).await?,
        );
    }
    Ok(ok(data))
}

/// `PATCH /api/v1/admin/merchants/{id}/compliance/{section}`
pub async fn set_compliance_section(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
    Path((id, section)): Path<(i64, String)>,
    Json(body): Json<SectionStatusRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !db::compliance::is_valid_section(&section) {
        return Err((StatusCode::BAD_REQUEST, "unknown compliance section".into()));
    }
    let status = body.status.trim().to_ascii_lowercase();
    if !matches!(status.as_str(), "approved" | "rejected" | "pending") {
        return Err((
            StatusCode::BAD_REQUEST,
            "status must be approved, rejected, or pending".into(),
        ));
    }

    let existing = db::compliance::find_section(&state.pool, id, &section)
        .await
        .map_err(map_db)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("compliance section `{section}` not submitted yet"),
            )
        })?;

    let row = db::compliance::upsert_section(
        &state.pool,
        id,
        &section,
        &status,
        &existing.data,
    )
    .await
    .map_err(map_db)?;

    if status == "approved" || status == "rejected" {
        let _ = sqlx::query(
            r#"
            UPDATE compliance_sections
            SET reviewed_at = now(), updated_at = now()
            WHERE customer_id = $1 AND section = $2
            "#,
        )
        .bind(id)
        .bind(&section)
        .execute(&state.pool)
        .await;
    }

    // Representative section mirrors KYC status when approved/rejected.
    if section == "representative" {
        if let Some(kyc) = db::kyc::find_by_customer_id(&state.pool, id)
            .await
            .map_err(map_db)?
        {
            let _ = db::kyc::set_status(&state.pool, kyc.id, &status)
                .await
                .map_err(map_db)?;
        }
    }

    Ok(ok(json!({
        "section": row.section,
        "status": status,
        "data": row.data,
        "submittedAt": row.submitted_at,
    })))
}

/// `POST /api/v1/admin/merchants/{id}/kyc/approve`
///
/// Approves KYC + all compliance sections, provisions zainbox, emails the merchant.
pub async fn approve_kyc(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let customer = db::customers::find_by_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "merchant not found".into()))?;

    let kyc = db::kyc::find_by_customer_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                "merchant has not submitted Company Representative KYC yet".into(),
            )
        })?;

    let kyc = db::kyc::set_status(&state.pool, kyc.id, "approved")
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "kyc profile not found".into()))?;

    let _ = db::compliance::set_all_sections_status(&state.pool, id, "approved")
        .await
        .map_err(map_db)?;

    let provision = merchant_provision::ensure_merchant_zainbox(
        &state.pool,
        &state.zainpay,
        &state.config,
        id,
    )
    .await
    .map_err(|err| {
        (
            StatusCode::BAD_GATEWAY,
            format!("KYC approved but zainbox provision failed: {err}"),
        )
    })?;

    let email = customer
        .email
        .as_deref()
        .or(Some(kyc.email.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if let Some(to) = email {
        if let Err(err) = mail::send_kyc_approved(
            &state.config,
            to,
            &customer.display_name,
            Some(provision.public_key.as_str()),
        )
        .await
        {
            warn!(error = %err, customer_id = id, "failed to send KYC approval email");
        }
    } else {
        warn!(customer_id = id, "no merchant email for KYC approval notice");
    }

    Ok(ok(json!({
        "customerId": id,
        "kycStatus": "approved",
        "provision": {
            "zainboxCode": provision.zainbox.zainbox_code,
            "publicKey": provision.public_key,
            "createdNewZainbox": provision.created_new_zainbox,
            "secretsReadyToIssue": true,
        },
        "emailSent": email.is_some(),
    })))
}

/// `POST /api/v1/admin/merchants/{id}/kyc/reject`
pub async fn reject_kyc(
    _auth: AdminPortalAuth,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let kyc = db::kyc::find_by_customer_id(&state.pool, id)
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "kyc profile not found".into()))?;
    let kyc = db::kyc::set_status(&state.pool, kyc.id, "rejected")
        .await
        .map_err(map_db)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "kyc profile not found".into()))?;
    Ok(ok(json!({
        "customerId": id,
        "kycStatus": kyc.status,
    })))
}

/// Create bootstrap admin if `users` is empty.
pub async fn ensure_bootstrap_admin(state: &AppState) -> anyhow::Result<()> {
    let hash = hash_password(&state.config.admin_bootstrap_password)
        .map_err(|(_, msg)| anyhow::anyhow!(msg))?;
    match db::users::ensure_bootstrap_admin(
        &state.pool,
        &state.config.admin_bootstrap_username,
        &hash,
        &state.config.admin_bootstrap_display_name,
        state.config.admin_bootstrap_email.as_deref(),
    )
    .await?
    {
        Some(user) => {
            tracing::info!(
                username = %user.username,
                "bootstrapped default admin user (change ADMIN_PASSWORD in production)"
            );
        }
        None => {}
    }
    Ok(())
}
