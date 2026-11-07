use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;
use crate::types::query::optional_query;
use crate::types::virtual_account::{
    CreateDynamicVirtualAccountRequest, CreateVirtualAccountRequest, FundVirtualAccountRequest,
    UpdateVirtualAccountBvnRequest, UpdateVirtualAccountStatusRequest, VirtualAccountBalance,
    VirtualAccountData,
};

pub struct VirtualAccountApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl VirtualAccountApi<'_> {
    /// `POST /virtual-account/create/request`
    ///
    /// Parses the envelope as [`Value`] first so application-level failures
    /// (e.g. `data: {}` without `accountName`) are returned as
    /// `Ok(ApiResponse { code: … })` instead of a deserialize hard-fail.
    pub async fn create(
        &self,
        req: &CreateVirtualAccountRequest,
    ) -> Result<ApiResponse<VirtualAccountData>, ZainpayError> {
        let envelope: ApiResponse<Value> = self
            .client
            .post_json("/virtual-account/create/request", req)
            .await?;
        Ok(ApiResponse {
            code: envelope.code,
            status: envelope.status,
            description: envelope.description,
            data: match envelope.data {
                None | Some(Value::Null) => None,
                Some(value) => serde_json::from_value(value).ok(),
            },
        })
    }

    /// `POST /virtual-account/dynamic/create/request`
    pub async fn create_dynamic(
        &self,
        req: &CreateDynamicVirtualAccountRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/virtual-account/dynamic/create/request", req)
            .await
    }

    /// `GET /virtual-account/wallet/balance/v2/{accountNumber}`
    ///
    /// Live API uses this path; the older `/zainbox/virtual-account/...` prefix
    /// returns HTTP 405.
    pub async fn balance(
        &self,
        account_number: &str,
    ) -> Result<ApiResponse<VirtualAccountBalance>, ZainpayError> {
        let path = format!("/virtual-account/wallet/balance/v2/{account_number}");
        self.client.get_json(&path).await
    }

    /// `PATCH /virtual-account/change/account/status`
    pub async fn update_status(
        &self,
        req: &UpdateVirtualAccountStatusRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .patch_json("/virtual-account/change/account/status", req)
            .await
    }

    /// `POST /virtual-account/update/bvn`
    pub async fn update_bvn(
        &self,
        req: &UpdateVirtualAccountBvnRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/virtual-account/update/bvn", req)
            .await
    }

    /// `GET /virtual-account/wallet/transactions/{accountNumber}`
    pub async fn transactions(
        &self,
        account_number: &str,
        count: Option<&str>,
        txn_type: Option<&str>,
        payment_channel: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/virtual-account/wallet/transactions/{account_number}");
        let q = optional_query(&[
            ("count", count.map(str::to_string)),
            ("txnType", txn_type.map(str::to_string)),
            ("paymentChannel", payment_channel.map(str::to_string)),
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
        ]);
        self.client.get_json_query(&path, &q).await
    }

    /// `GET /virtual-account/dynamic/deposit/status/{txnRef}`
    pub async fn dynamic_deposit_status(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/virtual-account/dynamic/deposit/status/{txn_ref}");
        self.client.get_json(&path).await
    }

    /// Sandbox mock: `POST https://mock.zainpay.ng/virtual-account/static/fund`
    pub async fn fund_static(
        &self,
        req: &FundVirtualAccountRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_mock_json("/virtual-account/static/fund", req)
            .await
    }

    /// Sandbox mock: `POST https://mock.zainpay.ng/virtual-account/dynamic/fund`
    pub async fn fund_dynamic(
        &self,
        req: &FundVirtualAccountRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_mock_json("/virtual-account/dynamic/fund", req)
            .await
    }
}
