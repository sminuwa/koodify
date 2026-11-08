//! Provision a merchant zainbox after KYC approval.
//!
//! Flow: customer registers → KYC submitted → status `approved` →
//! create Zainpay zainbox (business name) → local cache + publishable
//! `public_key` → link customer → mirror credentials (public_key from
//! zainbox; `zainbox_code` is the Zainpay box id; mint `sk_…` secrets).
//!
//! Checkout identity remains `zainboxes.public_key` only. Credentials keep a
//! mirrored `public_key` for dashboard/API compatibility — not a second source
//! of truth for payment init. Merchant server APIs use `sk_test_` / `sk_live_`.

use anyhow::{anyhow, bail, Context, Result};
use client::{CreateZainboxRequest, ZainpayClient};
use serde_json::{json, Value};
use sqlx::PgPool;
use tracing::{info, warn};

use crate::auth::{hash_secret_key, mint_secret_key, secret_key_prefix};
use crate::config::Config;
use crate::db;
use crate::db::credentials::UpsertCredential;
use crate::db::models::{Credential, Customer, KycProfile};
use crate::db::zainboxes::{self, UpsertZainbox, Zainbox};

#[derive(Debug, Clone)]
pub struct ProvisionedSecret {
    pub environment: String,
    /// Plaintext shown once (create / rotate). Never persisted.
    pub secret_key: String,
    pub prefix: String,
}

#[derive(Debug, Clone)]
pub struct ProvisionResult {
    pub customer: Customer,
    pub zainbox: Zainbox,
    pub public_key: String,
    pub credentials: Vec<Credential>,
    /// Newly minted secrets only (empty when secrets already existed).
    pub minted_secrets: Vec<ProvisionedSecret>,
    /// True when a new Zainpay zainbox was created this call.
    pub created_new_zainbox: bool,
}

/// Ensure a credential row has a merchant secret. Returns plaintext only when minted.
///
/// Also seals the secret for outbound webhook HMAC (merchants verify with the same `sk_…`).
pub async fn ensure_secret_key(
    pool: &PgPool,
    credential: &Credential,
    gateway_api_key: &str,
) -> Result<(Credential, Option<String>)> {
    if credential
        .secret_key_hash
        .as_deref()
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return Ok((credential.clone(), None));
    }
    let plaintext = mint_secret_key(&credential.environment);
    let hash = hash_secret_key(&plaintext);
    let prefix = secret_key_prefix(&plaintext);
    let wrapped = crate::services::secret_wrap::wrap_secret(gateway_api_key, &plaintext)
        .context("wrap merchant secret for webhook signing")?;
    let row = db::credentials::set_secret_hash(pool, credential.id, &hash, &prefix, &wrapped)
        .await
        .context("store merchant secret hash")?;
    Ok((row, Some(plaintext)))
}

/// Sync test+live credentials so `public_key` matches the zainbox.
///
/// When `mint_secrets` is false (admin KYC approval / provision), credential rows
/// are created without `sk_…` — merchants issue secrets themselves from Credentials.
/// Seed / ops paths may pass `mint_secrets: true`.
pub async fn sync_credentials_from_zainbox(
    pool: &PgPool,
    customer_id: i64,
    zainbox_code: &str,
    public_key: &str,
    gateway_api_key: &str,
    mint_secrets: bool,
) -> Result<(Vec<Credential>, Vec<ProvisionedSecret>)> {
    let meta = json!({ "zainboxCode": zainbox_code, "source": "zainbox-public-key" });
    let mut out = Vec::with_capacity(2);
    let mut minted = Vec::new();
    for environment in ["test", "live"] {
        let row = db::credentials::upsert(
            pool,
            UpsertCredential {
                customer_id,
                environment,
                public_key,
                zainbox_code,
                metadata: &meta,
                status: "active",
            },
        )
        .await
        .with_context(|| {
            format!("upsert {environment} credentials for customer {customer_id}")
        })?;
        if mint_secrets {
            let (row, secret) = ensure_secret_key(pool, &row, gateway_api_key).await?;
            if let Some(secret_key) = secret {
                minted.push(ProvisionedSecret {
                    environment: environment.to_string(),
                    prefix: secret_key_prefix(&secret_key),
                    secret_key,
                });
            }
            out.push(row);
        } else {
            out.push(row);
        }
    }
    Ok((out, minted))
}

fn extract_zainbox_code(data: &Value) -> Option<String> {
    if let Some(s) = data.as_str().filter(|s| !s.is_empty()) {
        return Some(s.to_string());
    }
    let obj = data.as_object()?;
    for key in ["codeName", "code_name", "zainboxCode", "zainbox_code", "id"] {
        if let Some(s) = obj.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
            return Some(s.to_string());
        }
    }
    None
}

fn extract_name(data: &Value, fallback: &str) -> String {
    data.as_object()
        .and_then(|o| o.get("name").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

/// Ensure the merchant has a zainbox + publishable key + credentials.
///
/// Idempotent: if `customers.zainbox_code` is already set, only ensures the
/// public key and credential mirror (no second Zainpay create).
pub async fn ensure_merchant_zainbox(
    pool: &PgPool,
    zainpay: &ZainpayClient,
    config: &Config,
    customer_id: i64,
) -> Result<ProvisionResult> {
    let customer = db::customers::find_by_id(pool, customer_id)
        .await
        .context("load customer")?
        .ok_or_else(|| anyhow!("customer {customer_id} not found"))?;

    let existing_code = customer
        .zainbox_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    if let Some(code) = existing_code {
        let inbound = config.inbound_zainpay_webhook_url();
        let zb = match zainboxes::get_by_code(pool, &code).await? {
            Some(z) => z,
            None => {
                // Local row missing — re-upsert a minimal cache from customer data.
                zainboxes::upsert(
                    pool,
                    UpsertZainbox {
                        zainbox_code: &code,
                        name: &customer.display_name,
                        callback_url: inbound.as_deref(),
                        email_notification: customer.email.as_deref(),
                        tags: Some("koodify-merchant"),
                        is_active: true,
                        raw: &json!({ "codeName": code, "name": customer.display_name }),
                    },
                )
                .await?
            }
        };
        let public_key = zainboxes::ensure_public_key(pool, &code).await?;
        // Admin provision unlocks the account; merchants mint sk_… themselves.
        let (credentials, minted_secrets) = sync_credentials_from_zainbox(
            pool,
            customer_id,
            &code,
            &public_key,
            &config.gateway_api_key,
            false,
        )
        .await?;
        if let Some(kyc) = db::kyc::find_by_customer_id(pool, customer_id).await? {
            if kyc.zainbox_code.as_deref() != Some(code.as_str()) {
                let _ = db::kyc::set_zainbox_code(pool, kyc.id, &code).await?;
            }
        }
        let customer = db::customers::find_by_id(pool, customer_id)
            .await?
            .unwrap_or(customer);
        let zainbox = zainboxes::get_by_code(pool, &code)
            .await?
            .unwrap_or(zb);
        return Ok(ProvisionResult {
            customer,
            zainbox,
            public_key,
            credentials,
            minted_secrets,
            created_new_zainbox: false,
        });
    }

    let business_name = customer.display_name.trim();
    if business_name.is_empty() {
        bail!("customer displayName is required to create a zainbox");
    }

    let callback_url = config
        .inbound_zainpay_webhook_url()
        .unwrap_or_else(|| "https://api.koodify.ng/webhooks/zainpay".into());

    let req = CreateZainboxRequest {
        name: business_name.to_string(),
        callback_url,
        email_notification: customer.email.clone(),
        description: Some(format!("Koodify merchant #{customer_id}")),
        tags: Some("koodify-merchant".into()),
        code_name_prefix: None,
        allow_auto_internal_transfer: Some(true),
    };

    let resp = zainpay
        .zainbox()
        .create(&req)
        .await
        .context("Zainpay zainbox create")?;

    if !resp.is_success() {
        bail!(
            "Zainpay zainbox create failed: code={} description={}",
            resp.code,
            resp.description
        );
    }

    let data = resp.data.clone().unwrap_or(Value::Null);
    let zainbox_code = extract_zainbox_code(&data).ok_or_else(|| {
        anyhow!("Zainpay create response missing codeName: {data}")
    })?;
    let name = extract_name(&data, business_name);

    let email_notification = data
        .as_object()
        .and_then(|o| o.get("emailNotification").and_then(|v| v.as_str()))
        .map(str::to_owned)
        .or_else(|| customer.email.clone());

    let inbound = config.inbound_zainpay_webhook_url();
    let callback = data
        .as_object()
        .and_then(|o| o.get("callbackUrl").and_then(|v| v.as_str()))
        .map(str::to_owned)
        .or(inbound);

    let zb = zainboxes::upsert(
        pool,
        UpsertZainbox {
            zainbox_code: &zainbox_code,
            name: &name,
            callback_url: callback.as_deref(),
            email_notification: email_notification.as_deref(),
            tags: Some("koodify-merchant"),
            is_active: true,
            raw: &data,
        },
    )
    .await
    .context("local zainbox upsert")?;

    let public_key = zainboxes::ensure_public_key(pool, &zainbox_code)
        .await
        .context("assign public_key")?;

    let customer = db::customers::set_zainbox_code(pool, customer_id, &zainbox_code)
        .await
        .context("link customer.zainbox_code")?
        .ok_or_else(|| anyhow!("customer {customer_id} vanished during provision"))?;

    if let Some(kyc) = db::kyc::find_by_customer_id(pool, customer_id).await? {
        let _ = db::kyc::set_zainbox_code(pool, kyc.id, &zainbox_code).await?;
    }

    let (credentials, minted_secrets) = sync_credentials_from_zainbox(
        pool,
        customer_id,
        &zainbox_code,
        &public_key,
        &config.gateway_api_key,
        false,
    )
    .await?;

    info!(
        customer_id,
        zainbox_code = %zainbox_code,
        public_key = %public_key,
        minted_secrets = minted_secrets.len(),
        "provisioned merchant zainbox after KYC approval (secrets issued by merchant)"
    );

    Ok(ProvisionResult {
        customer,
        zainbox: zb,
        public_key,
        credentials,
        minted_secrets,
        created_new_zainbox: true,
    })
}

/// After KYC reaches `approved`, provision (or re-sync) the merchant zainbox.
/// Returns `Ok(None)` when status is not approved.
pub async fn maybe_provision_after_kyc(
    pool: &PgPool,
    zainpay: &ZainpayClient,
    config: &Config,
    kyc: &KycProfile,
) -> Result<Option<ProvisionResult>> {
    if !db::kyc::is_approved_status(&kyc.status) {
        return Ok(None);
    }
    match ensure_merchant_zainbox(pool, zainpay, config, kyc.customer_id).await {
        Ok(result) => Ok(Some(result)),
        Err(err) => {
            warn!(
                error = %err,
                customer_id = kyc.customer_id,
                kyc_id = %kyc.id,
                "KYC approved but merchant zainbox provision failed"
            );
            Err(err)
        }
    }
}
