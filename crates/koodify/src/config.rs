use std::env;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub zainpay_base_url: String,
    pub zainpay_api_key: String,
    /// Platform admin secret (`X-Api-Key`). Not shared with merchants.
    /// Merchants use per-credential `sk_test_` / `sk_live_` secrets instead.
    pub gateway_api_key: String,
    /// HMAC secret for inbound `Zainpay-Signature` verification.
    /// Docs refer to this as the dashboard secret key (`ZAINPAY_WEBHOOK_SECRET`).
    pub zainpay_webhook_secret: Option<String>,
    /// Public HTTPS origin for this gateway (`PUBLIC_BASE_URL`), e.g. `https://api.koodify.ng`.
    /// Used to build the inbound Zainpay webhook URL written onto zainbox `callbackUrl`.
    pub public_base_url: Option<String>,
    /// Resend API key for payer receipt emails (`RESEND_API_KEY`). Optional —
    /// when unset, receipt emails are skipped.
    pub resend_api_key: Option<String>,
    /// From header for receipts, e.g. `Koodify <payments@koodify.ng>`.
    pub resend_from: String,
    /// Adaptive Bank/DVA Routing candidates (`VA_BANK_CANDIDATES`).
    /// Default is `gtBank` only — add others only after confirming Zainpay temp/DVA support.
    pub va_bank_candidates: Vec<String>,
    /// How long the preferred DVA bank stays sticky (`VA_BANK_STICKY_SECS`, default 600).
    pub va_bank_sticky_secs: u64,
    /// Legacy fallback for outbound merchant webhook HMAC (`MERCHANT_WEBHOOK_SECRET`).
    /// Prefer each merchant's `sk_…` (wrapped on the credential). When unset and no
    /// wrapped secret exists, falls back to `GATEWAY_API_KEY`.
    pub merchant_webhook_secret: Option<String>,
    /// Background pending-checkout reconciler (`RECONCILE_ENABLED`, default true).
    pub reconcile_enabled: bool,
    /// Seconds between reconcile passes (`RECONCILE_INTERVAL_SECS`, default 60, min 15).
    pub reconcile_interval_secs: u64,
    /// How far back to scan unpaid checkouts (`RECONCILE_LOOKBACK_HOURS`, default 48).
    pub reconcile_lookback_hours: u64,
    /// Ignore checkouts newer than this (`RECONCILE_GRACE_SECS`, default 45) so webhook/status win first.
    pub reconcile_grace_secs: u64,
    /// Min seconds between polls of the same pending txn (`RECONCILE_POLL_COOLDOWN_SECS`, default 120).
    pub reconcile_poll_cooldown_secs: u64,
    /// Max pending txns polled per pass (`RECONCILE_BATCH_SIZE`, default 40).
    pub reconcile_batch_size: u32,
    /// Default BVN for permanent VA create when the merchant omits it (`VA_DEFAULT_BVN`).
    /// Zainpay requires 11 digits starting with `22`.
    pub va_default_bvn: String,
    /// Default DOB (`DD-MM-YYYY`) when omitted (`VA_DEFAULT_DOB`).
    pub va_default_dob: String,
    /// Default gender (`M` / `F`) when omitted (`VA_DEFAULT_GENDER`).
    pub va_default_gender: String,
    /// When true, merchant-submitted KYC is auto-approved and provisions keys
    /// (`MERCHANT_KYC_AUTO_APPROVE`, default false). Use for local/sandbox dashboards.
    pub merchant_kyc_auto_approve: bool,
    /// Directory for compliance document uploads (`COMPLIANCE_STORAGE_DIR`, default `./data/compliance`).
    pub compliance_storage_dir: String,
    /// Bootstrap admin username when `users` is empty (`ADMIN_USERNAME`, default `admin`).
    pub admin_bootstrap_username: String,
    /// Bootstrap admin password (`ADMIN_PASSWORD`, default `Admin123!`).
    pub admin_bootstrap_password: String,
    /// Bootstrap admin display name (`ADMIN_DISPLAY_NAME`).
    pub admin_bootstrap_display_name: String,
    /// Bootstrap admin email (`ADMIN_EMAIL`).
    pub admin_bootstrap_email: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let va_bank_candidates = crate::services::bank_router::parse_candidates(
            env::var("VA_BANK_CANDIDATES").ok().as_deref(),
        );
        crate::services::bank_router::warn_if_experimental(&va_bank_candidates);
        let va_bank_sticky_secs = env::var("VA_BANK_STICKY_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600)
            .max(60);

        Ok(Self {
            bind: env::var("GATEWAY_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            zainpay_base_url: env::var("ZAINPAY_BASE_URL")
                .unwrap_or_else(|_| "https://sandbox.zainpay.ng".into()),
            zainpay_api_key: env::var("ZAINPAY_API_KEY").context("ZAINPAY_API_KEY is required")?,
            gateway_api_key: env::var("GATEWAY_API_KEY").context("GATEWAY_API_KEY is required")?,
            zainpay_webhook_secret: env::var("ZAINPAY_WEBHOOK_SECRET")
                .ok()
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .ok()
                .map(|s| s.trim().trim_end_matches('/').to_string())
                .filter(|s| !s.is_empty()),
            resend_api_key: env::var("RESEND_API_KEY").ok().filter(|s| !s.trim().is_empty()),
            resend_from: env::var("RESEND_FROM")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Koodify <onboarding@resend.dev>".into()),
            va_bank_candidates,
            va_bank_sticky_secs,
            merchant_webhook_secret: env::var("MERCHANT_WEBHOOK_SECRET")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            reconcile_enabled: env::var("RECONCILE_ENABLED")
                .ok()
                .map(|s| !matches!(s.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"))
                .unwrap_or(true),
            reconcile_interval_secs: env::var("RECONCILE_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60)
                .max(15),
            reconcile_lookback_hours: env::var("RECONCILE_LOOKBACK_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(48)
                .max(1),
            reconcile_grace_secs: env::var("RECONCILE_GRACE_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(45)
                .max(5),
            reconcile_poll_cooldown_secs: env::var("RECONCILE_POLL_COOLDOWN_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120)
                .max(30),
            reconcile_batch_size: env::var("RECONCILE_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(40)
                .clamp(1, 200),
            va_default_bvn: env::var("VA_DEFAULT_BVN")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "22123456789".into()),
            va_default_dob: env::var("VA_DEFAULT_DOB")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "15-06-1990".into()),
            va_default_gender: env::var("VA_DEFAULT_GENDER")
                .ok()
                .map(|s| s.trim().to_ascii_uppercase())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "M".into()),
            merchant_kyc_auto_approve: env::var("MERCHANT_KYC_AUTO_APPROVE")
                .ok()
                .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(false),
            compliance_storage_dir: env::var("COMPLIANCE_STORAGE_DIR")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "./data/compliance".into()),
            admin_bootstrap_username: env::var("ADMIN_USERNAME")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "admin".into()),
            admin_bootstrap_password: env::var("ADMIN_PASSWORD")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Admin123!".into()),
            admin_bootstrap_display_name: env::var("ADMIN_DISPLAY_NAME")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Koodify Admin".into()),
            admin_bootstrap_email: env::var("ADMIN_EMAIL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        })
    }

    /// Secret used to sign outbound merchant webhook bodies.
    pub fn merchant_webhook_signing_key(&self) -> &str {
        self.merchant_webhook_secret
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(self.gateway_api_key.as_str())
    }

    /// Absolute URL Zainpay should POST deposit events to (`POST /webhooks/zainpay`).
    /// `None` when `PUBLIC_BASE_URL` is unset.
    pub fn inbound_zainpay_webhook_url(&self) -> Option<String> {
        self.public_base_url
            .as_ref()
            .map(|base| format!("{base}/webhooks/zainpay"))
    }

    /// True when `url` is this gateway's inbound Zainpay webhook (avoid merchant self-POST loops).
    pub fn is_inbound_zainpay_webhook_url(&self, url: &str) -> bool {
        let Some(ours) = self.inbound_zainpay_webhook_url() else {
            return url.trim().ends_with("/webhooks/zainpay");
        };
        url.trim().eq_ignore_ascii_case(ours.trim())
            || url.trim().ends_with("/webhooks/zainpay")
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
            public_base_url: None,
            resend_api_key: None,
            resend_from: "Koodify <onboarding@resend.dev>".into(),
            va_bank_candidates: vec!["gtBank".into()],
            va_bank_sticky_secs: 600,
            merchant_webhook_secret: None,
            reconcile_enabled: false,
            reconcile_interval_secs: 60,
            reconcile_lookback_hours: 48,
            reconcile_grace_secs: 45,
            reconcile_poll_cooldown_secs: 120,
            reconcile_batch_size: 40,
            va_default_bvn: "22123456789".into(),
            va_default_dob: "15-06-1990".into(),
            va_default_gender: "M".into(),
            merchant_kyc_auto_approve: false,
            compliance_storage_dir: "./data/compliance".into(),
            admin_bootstrap_username: "admin".into(),
            admin_bootstrap_password: "Admin123!".into(),
            admin_bootstrap_display_name: "Koodify Admin".into(),
            admin_bootstrap_email: Some("admin@koodify.ng".into()),
        }
    }

    pub fn with_public_base_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into().trim().trim_end_matches('/').to_string();
        self.public_base_url = if url.is_empty() { None } else { Some(url) };
        self
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
