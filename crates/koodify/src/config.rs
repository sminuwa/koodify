use std::env;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub zainpay_base_url: String,
    pub zainpay_api_key: String,
    /// Secret key for admin / privileged gateway routes (`X-Api-Key`).
    pub gateway_api_key: String,
    /// HMAC secret for inbound `Zainpay-Signature` verification.
    /// Docs refer to this as the dashboard secret key (`ZAINPAY_WEBHOOK_SECRET`).
    pub zainpay_webhook_secret: Option<String>,
    /// Resend API key for payer receipt emails (`RESEND_API_KEY`). Optional —
    /// when unset, receipt emails are skipped.
    pub resend_api_key: Option<String>,
    /// From header for receipts, e.g. `Koodify <payments@koodify.ng>`.
    pub resend_from: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind: env::var("GATEWAY_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            zainpay_base_url: env::var("ZAINPAY_BASE_URL")
                .unwrap_or_else(|_| "https://sandbox.zainpay.ng".into()),
            zainpay_api_key: env::var("ZAINPAY_API_KEY").context("ZAINPAY_API_KEY is required")?,
            gateway_api_key: env::var("GATEWAY_API_KEY").context("GATEWAY_API_KEY is required")?,
            zainpay_webhook_secret: env::var("ZAINPAY_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty()),
            resend_api_key: env::var("RESEND_API_KEY").ok().filter(|s| !s.trim().is_empty()),
            resend_from: env::var("RESEND_FROM")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Koodify <onboarding@resend.dev>".into()),
        })
    }

    /// Build config for tests / in-process usage without reading the process env.
    pub fn new(
        bind: impl Into<String>,
        database_url: impl Into<String>,
        zainpay_base_url: impl Into<String>,
        zainpay_api_key: impl Into<String>,
        gateway_api_key: impl Into<String>,
    ) -> Self {
        Self {
            bind: bind.into(),
            database_url: database_url.into(),
            zainpay_base_url: zainpay_base_url.into(),
            zainpay_api_key: zainpay_api_key.into(),
            gateway_api_key: gateway_api_key.into(),
            zainpay_webhook_secret: None,
            resend_api_key: None,
            resend_from: "Koodify <onboarding@resend.dev>".into(),
        }
    }

    pub fn with_webhook_secret(mut self, secret: impl Into<String>) -> Self {
        let secret = secret.into();
        self.zainpay_webhook_secret = if secret.is_empty() {
            None
        } else {
            Some(secret)
        };
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.gateway_api_key.is_empty() {
            return Err(anyhow!("GATEWAY_API_KEY must not be empty"));
        }
        if self.zainpay_api_key.is_empty() {
            return Err(anyhow!("ZAINPAY_API_KEY must not be empty"));
        }
        Ok(())
    }
}
