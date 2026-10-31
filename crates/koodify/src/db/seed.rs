use anyhow::{bail, Context, Result};
use client::ZainpayClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use super::credentials::{self, UpsertCredential};
use super::customers::{self, CreateCustomer};
use super::zainboxes::{self, UpsertZainbox};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZainboxListItem {
    code_name: String,
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
/// Each customer's credential `private_key` is the Zainpay zainbox `codeName`.
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
                        .map(|c| c == item.code_name)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .unwrap_or(raw);

        zainboxes::upsert(
            pool,
            UpsertZainbox {
                code_name: &item.code_name,
                name: &item.name,
                callback_url: item.callback_url.as_deref(),
                email_notification: item.email_notification.as_deref(),
                tags: item.tags.as_deref(),
                is_active: item.is_active,
                raw: &raw,
            },
        )
        .await
        .with_context(|| format!("upsert zainbox {}", item.code_name))?;
    }

    let boxes = zainboxes::list(pool).await?;
    println!("creating {} customer(s) from local zainboxes…", boxes.len());

    for zb in &boxes {
        let status = if zb.is_active { "active" } else { "inactive" };
        let metadata = json!({
            "source": "zainbox-sync",
            "zainboxCode": zb.code_name,
            "tags": zb.tags,
            "callbackUrl": zb.callback_url,
        });

        let customer = customers::create(
            pool,
            CreateCustomer {
                display_name: &zb.name,
                email: zb.email_notification.as_deref(),
                mobile_number: None,
                zainbox_code: Some(&zb.code_name),
                metadata: &metadata,
                status,
            },
        )
        .await
        .with_context(|| format!("create customer for {}", zb.code_name))?;

        let cred_meta = json!({ "zainboxCode": zb.code_name });

        // private_key = Zainpay zainbox id (codeName) for every environment.
        for environment in ["test", "live"] {
            credentials::upsert(
                pool,
                UpsertCredential {
                    customer_id: customer.id,
                    environment,
                    public_key: &api_key,
                    private_key: &zb.code_name,
                    metadata: &cred_meta,
                    status: "active",
                },
            )
            .await
            .with_context(|| {
                format!(
                    "upsert {} credentials for customer {} ({})",
                    environment, customer.id, zb.code_name
                )
            })?;
        }

        println!(
            "  id={} uuid={} zainbox={} — {}",
            customer.id, customer.uuid, zb.code_name, zb.name
        );
    }

    Ok(())
}
