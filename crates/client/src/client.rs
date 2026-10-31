use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ZainpayError;
use crate::modules::bank::BankApi;
use crate::modules::card::CardApi;
use crate::modules::composite::CompositeApi;
use crate::modules::reconciliation::ReconciliationApi;
use crate::modules::settlement::SettlementApi;
use crate::modules::transfer::TransferApi;
use crate::modules::virtual_account::VirtualAccountApi;
use crate::modules::webhook::WebhookApi;
use crate::modules::zainbox::ZainboxApi;

/// Known Zainpay environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Sandbox,
    Live,
}

impl Environment {
    pub fn base_url(self) -> &'static str {
        match self {
            Self::Sandbox => "https://sandbox.zainpay.ng",
            Self::Live => "https://api.zainpay.ng",
        }
    }

    pub fn mock_base_url(self) -> &'static str {
        "https://mock.zainpay.ng"
    }
}

/// HTTP client for the Zainpay API.
#[derive(Clone)]
pub struct ZainpayClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) mock_base_url: String,
    pub(crate) api_key: SecretString,
}

impl ZainpayClient {
    /// Create a client for a known environment.
    pub fn new(environment: Environment, api_key: impl Into<String>) -> Self {
        Self::builder()
            .base_url(environment.base_url())
            .mock_base_url(environment.mock_base_url())
            .api_key(api_key)
            .build()
            .expect("default reqwest client builds")
    }

    pub fn builder() -> ZainpayClientBuilder {
        ZainpayClientBuilder::default()
    }

    pub fn zainbox(&self) -> ZainboxApi<'_> {
        ZainboxApi { client: self }
    }

    pub fn virtual_accounts(&self) -> VirtualAccountApi<'_> {
        VirtualAccountApi { client: self }
    }

    pub fn bank(&self) -> BankApi<'_> {
        BankApi { client: self }
    }

    pub fn transfers(&self) -> TransferApi<'_> {
        TransferApi { client: self }
    }

    pub fn settlements(&self) -> SettlementApi<'_> {
        SettlementApi { client: self }
    }

    pub fn reconciliation(&self) -> ReconciliationApi<'_> {
        ReconciliationApi { client: self }
    }

    pub fn cards(&self) -> CardApi<'_> {
        CardApi { client: self }
    }

    pub fn webhooks(&self) -> WebhookApi<'_> {
        WebhookApi { client: self }
    }

    /// Convenience helpers that call two Zainpay endpoints as one response.
    pub fn composite(&self) -> CompositeApi<'_> {
        CompositeApi { client: self }
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    pub(crate) fn mock_url(&self, path: &str) -> String {
        format!("{}{}", self.mock_base_url.trim_end_matches('/'), path)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key.expose_secret())
    }

    pub(crate) async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T, ZainpayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let response = self
            .http
            .post(self.url(path))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        Self::parse_response(response).await
    }

    pub(crate) async fn patch_json<B, T>(&self, path: &str, body: &B) -> Result<T, ZainpayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let response = self
            .http
            .patch(self.url(path))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        Self::parse_response(response).await
    }

    pub(crate) async fn get_json<T>(&self, path: &str) -> Result<T, ZainpayError>
    where
        T: DeserializeOwned,
    {
        let response = self
            .http
            .get(self.url(path))
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::parse_response(response).await
    }

    pub(crate) async fn get_json_query<T, K, V>(
        &self,
        path: &str,
        query: &[(K, V)],
    ) -> Result<T, ZainpayError>
    where
        T: DeserializeOwned,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let pairs: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();
        let response = self
            .http
            .get(self.url(path))
            .header("Authorization", self.auth_header())
            .query(&pairs)
            .send()
            .await?;
        Self::parse_response(response).await
    }

    /// Unauthenticated POST against the mock host (sandbox VA funding).
    pub(crate) async fn post_mock_json<B, T>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ZainpayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let response = self
            .http
            .post(self.mock_url(path))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;
        Self::parse_response(response).await
    }

    async fn parse_response<T>(response: reqwest::Response) -> Result<T, ZainpayError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;

        if !(status.is_success() || status.as_u16() == 400 || status.as_u16() == 422) {
            return Err(ZainpayError::UnexpectedStatus {
                status: status.as_u16(),
                body,
            });
        }

        serde_json::from_str(&body).map_err(ZainpayError::Deserialize)
    }
}

/// Builder for [`ZainpayClient`].
#[derive(Debug, Default)]
pub struct ZainpayClientBuilder {
    base_url: Option<String>,
    mock_base_url: Option<String>,
    api_key: Option<SecretString>,
    http: Option<reqwest::Client>,
}

impl ZainpayClientBuilder {
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn mock_base_url(mut self, mock_base_url: impl Into<String>) -> Self {
        self.mock_base_url = Some(mock_base_url.into());
        self
    }

    pub fn environment(self, environment: Environment) -> Self {
        self.base_url(environment.base_url())
            .mock_base_url(environment.mock_base_url())
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(SecretString::from(api_key.into()));
        self
    }

    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    pub fn build(self) -> Result<ZainpayClient, ZainpayError> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| Environment::Sandbox.base_url().to_string());
        let mock_base_url = self
            .mock_base_url
            .unwrap_or_else(|| Environment::Sandbox.mock_base_url().to_string());
        let api_key = self
            .api_key
            .unwrap_or_else(|| SecretString::from(String::new()));
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder().build()?,
        };

        Ok(ZainpayClient {
            http,
            base_url,
            mock_base_url,
            api_key,
        })
    }
}
