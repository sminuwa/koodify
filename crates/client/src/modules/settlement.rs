use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;
use crate::types::query::optional_query;
use crate::types::settlement::CreateSettlementRequest;

pub struct SettlementApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl SettlementApi<'_> {
    /// `POST /zainbox/settlement`
    pub async fn create(
        &self,
        req: &CreateSettlementRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.post_json("/zainbox/settlement", req).await
    }

    /// `GET /zainbox/settlement?zainboxCode=`
    ///
    /// Documented in Zainpay docs; as of 2026-10 live `api.zainpay.ng` returns
    /// HTTP 405 for this GET (POST create and history still work).
    pub async fn get(&self, zainbox_code: &str) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = [("zainboxCode", zainbox_code.to_string())];
        self.client
            .get_json_query("/zainbox/settlement", &q)
            .await
    }

    /// `GET /zainbox/settlement/history/{zainboxCode}`
    pub async fn history(
        &self,
        zainbox_code: &str,
        count: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        status: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/settlement/history/{zainbox_code}");
        let q = optional_query(&[
            ("count", count.map(str::to_string)),
            ("dateFrom", date_from.map(str::to_string)),
            ("dateTo", date_to.map(str::to_string)),
            ("status", status.map(str::to_string)),
        ]);
        self.client.get_json_query(&path, &q).await
    }

    /// `GET /zainbox/internal-settlement-account/{zainboxCode}`
    pub async fn internal_account(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/internal-settlement-account/{zainbox_code}");
        self.client.get_json(&path).await
    }
}
