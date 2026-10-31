use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;
use crate::types::query::optional_query;
use crate::types::zainbox::{
    CreateZainboxRequest, ServiceSubscriptionRequest, UpdateZainboxRequest,
};

pub struct ZainboxApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl ZainboxApi<'_> {
    /// `POST /zainbox/create/request`
    pub async fn create(
        &self,
        req: &CreateZainboxRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.post_json("/zainbox/create/request", req).await
    }

    /// `GET /zainbox/list`
    pub async fn list(&self, status: Option<&str>) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = optional_query(&[("status", status.map(str::to_string))]);
        self.client.get_json_query("/zainbox/list", &q).await
    }

    /// `PATCH /zainbox/update`
    pub async fn update(
        &self,
        req: &UpdateZainboxRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.patch_json("/zainbox/update", req).await
    }

    /// `GET /zainbox/virtual-accounts/{zainboxCode}`
    pub async fn virtual_accounts(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/virtual-accounts/{zainbox_code}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/accounts/balance/{zainboxCode}`
    pub async fn accounts_balance(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/accounts/balance/{zainbox_code}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/transactions/{zainboxCode}`
    pub async fn transactions(
        &self,
        zainbox_code: &str,
        count: Option<&str>,
        txn_type: Option<&str>,
        account_number: Option<&str>,
        payment_channel: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/transactions/{zainbox_code}");
        let q = optional_query(&[
            ("count", count.map(str::to_string)),
            ("txnType", txn_type.map(str::to_string)),
            ("accountNumber", account_number.map(str::to_string)),
            ("paymentChannel", payment_channel.map(str::to_string)),
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
        ]);
        self.client.get_json_query(&path, &q).await
    }

    /// `GET /zainbox/transactions`
    pub async fn merchant_transactions(
        &self,
        count: Option<&str>,
        txn_type: Option<&str>,
        account_number: Option<&str>,
        payment_channel: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = optional_query(&[
            ("count", count.map(str::to_string)),
            ("txnType", txn_type.map(str::to_string)),
            ("accountNumber", account_number.map(str::to_string)),
            ("paymentChannel", payment_channel.map(str::to_string)),
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
        ]);
        self.client
            .get_json_query("/zainbox/transactions", &q)
            .await
    }

    /// `GET /zainbox/transfer/deposit/summary/{zainboxCode}`
    pub async fn deposit_summary(
        &self,
        zainbox_code: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/transfer/deposit/summary/{zainbox_code}");
        let q = optional_query(&[
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
        ]);
        self.client.get_json_query(&path, &q).await
    }

    /// `GET /zainbox/profile/{zainboxCode}`
    pub async fn profile(&self, zainbox_code: &str) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/profile/{zainbox_code}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/profile/v2/{zainboxCode}`
    pub async fn profile_v2(&self, zainbox_code: &str) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/profile/v2/{zainbox_code}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/billing/config/{zainboxCode}`
    pub async fn billing_config(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/billing/config/{zainbox_code}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/transactions/summary`
    pub async fn transactions_summary(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = optional_query(&[
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
        ]);
        self.client
            .get_json_query("/zainbox/transactions/summary", &q)
            .await
    }

    /// `GET /zainbox/card/transactions/{zainboxCode}`
    pub async fn card_transactions(
        &self,
        zainbox_code: &str,
        count: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        status: Option<&str>,
        email: Option<&str>,
        txn_ref: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/card/transactions/{zainbox_code}");
        let q = optional_query(&[
            ("count", count.map(str::to_string)),
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
            ("status", status.map(str::to_string)),
            ("email", email.map(str::to_string)),
            ("txnRef", txn_ref.map(str::to_string)),
        ]);
        self.client.get_json_query(&path, &q).await
    }

    /// `POST /zainbox/service-subscription/subscribe`
    pub async fn subscribe(
        &self,
        req: &ServiceSubscriptionRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/zainbox/service-subscription/subscribe", req)
            .await
    }

    /// `POST /zainbox/service-subscription/unsubscribe`
    pub async fn unsubscribe(
        &self,
        req: &ServiceSubscriptionRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/zainbox/service-subscription/unsubscribe", req)
            .await
    }

    /// `GET /zainbox/service-subscription/subscriptions/all/{zainbox_code}`
    pub async fn subscriptions_all(
        &self,
        zainbox_code: &str,
        service_name: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/service-subscription/subscriptions/all/{zainbox_code}");
        let q = optional_query(&[("serviceName", service_name.map(str::to_string))]);
        self.client.get_json_query(&path, &q).await
    }

    /// `GET /zainbox/service-subscription/subscriptions/active/{zainbox_code}`
    pub async fn subscriptions_active(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/service-subscription/subscriptions/active/{zainbox_code}");
        self.client.get_json(&path).await
    }
}
