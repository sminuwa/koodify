use anyhow::{bail, Context, Result};
use client::ZainpayClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use super::customers::{self, CreateCustomer};
use super::zainboxes::{self, UpsertZainbox};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZainboxListItem {
    /// Zainpay list payload uses `codeName`.
    #[serde(alias = "codeName", rename = "codeName")]
    zainbox_code: String,
    name: String,
    #[serde(default)]
    callback_url: Option<String>,
    #[serde(default)]
    email_notification: Option<String>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default = "default_active")]
    is_active: bool,
}

fn default_active() -> bool {
    true
}

/// Sync Zainpay zainboxes into local DB, then create one customer per box.
///
/// Each zainbox gets a browser-safe `public_key` (`pk_live_…`) stored on
/// `zainboxes` (source of truth for checkout). Credentials mirror that key and
/// mint merchant `sk_test_` / `sk_live_` secrets (shown once). `zainbox_code` is
/// the Zainpay box id — not an API secret.
///
/// New merchants (register → KYC approve) are provisioned by
/// `services::merchant_provision` instead of this seed path.
pub async fn run(pool: &PgPool) -> Result<()> {
    let api_key = std::env::var("ZAINPAY_API_KEY")
        .context("ZAINPAY_API_KEY is required to seed from live zainboxes")?;
    let base_url = std::env::var("ZAINPAY_BASE_URL")
        .unwrap_or_else(|_| "https://api.zainpay.ng".into());

    let zainpay = ZainpayClient::builder()
        .base_url(&base_url)
        .api_key(api_key.clone())
        .build()?;

    let response = zainpay
        .zainbox()
        .list(None)
        .await
        .context("failed to list zainboxes from Zainpay")?;

    if !response.is_success() {
        bail!(
            "zainbox list failed: code={} description={}",
            response.code,
            response.description
        );
    }

    let data = response.data.unwrap_or(Value::Null);
    let items: Vec<ZainboxListItem> = serde_json::from_value(data.clone())
        .context("unexpected zainbox list payload shape")?;

    if items.is_empty() {
        bail!("Zainpay returned zero zainboxes; nothing to seed");
    }

    // Drop customer-linked rows first (FK to zainboxes is ON DELETE SET NULL).
    sqlx::query(
        r#"
        TRUNCATE TABLE credentials, kyc_profiles, customers
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(pool)
    .await?;

    println!("syncing {} zainbox(es) into local db…", items.len());

    for item in &items {
        let raw = serde_json::to_value(item).unwrap_or(json!({}));
        // Prefer the original API object when available.
        let raw = data
            .as_array()
            .and_then(|arr| {
                arr.iter().find(|v| {
                    v.get("codeName")
                        .and_then(|c| c.as_str())
                        .map(|c| c == item.zainbox_code)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .unwrap_or(raw);

        zainboxes::upsert(
            pool,
            UpsertZainbox {
                zainbox_code: &item.zainbox_code,
                name: &item.name,
                callback_url: item.callback_url.as_deref(),
                email_notification: item.email_notification.as_deref(),
                tags: item.tags.as_deref(),
                is_active: item.is_active,
                raw: &raw,
            },
        )
        .await
        .with_context(|| format!("upsert zainbox {}", item.zainbox_code))?;
    }

    let boxes = zainboxes::list(pool).await?;
    println!("creating {} customer(s) from local zainboxes…", boxes.len());

    for zb in &boxes {
        let public_key = zainboxes::ensure_public_key(pool, &zb.zainbox_code)
            .await
            .with_context(|| format!("assign public_key for {}", zb.zainbox_code))?;

        let status = if zb.is_active { "active" } else { "inactive" };
        let metadata = json!({
            "source": "zainbox-sync",
            "zainboxCode": zb.zainbox_code,
            "tags": zb.tags,
            "callbackUrl": zb.callback_url,
        });

        let customer = customers::create(
            pool,
            CreateCustomer {
                display_name: &zb.name,
                email: zb.email_notification.as_deref(),
                mobile_number: None,
                zainbox_code: Some(&zb.zainbox_code),
                metadata: &metadata,
                status,
            },
        )
        .await
        .with_context(|| format!("create customer for {}", zb.zainbox_code))?;

        // public_key = checkout; zainbox_code = box id; sk_… minted if missing.
        let (creds, minted) = crate::services::merchant_provision::sync_credentials_from_zainbox(
            pool,
            customer.id,
            &zb.zainbox_code,
            &public_key,
        )
        .await
        .with_context(|| {
            format!(
                "sync credentials for customer {} ({})",
                customer.id, zb.zainbox_code
            )
        })?;
        let _ = creds;

        println!(
            "  id={} uuid={} publicKey={} zainbox={} — {}",
            customer.id, customer.uuid, public_key, zb.zainbox_code, zb.name
        );
        for s in &minted {
            println!(
                "    NEW {} secretKey={} (store now — shown once)",
                s.environment, s.secret_key
            );
        }
    }

    Ok(())
}
