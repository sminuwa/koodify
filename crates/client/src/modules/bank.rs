use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;

pub struct BankApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl BankApi<'_> {
    /// `GET /bank/list`
    ///
    /// Live API serves this under `/bank/list` (docs URL field still shows
    /// `/zainbox/bank/list`, which returns HTTP 405).
    pub async fn list(&self) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.get_json("/bank/list").await
    }

    /// `GET /bank/name-enquiry`
    pub async fn name_enquiry(
        &self,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = [
            ("bankCode", bank_code.to_string()),
            ("accountNumber", account_number.to_string()),
        ];
        self.client
            .get_json_query("/bank/name-enquiry", &q)
            .await
    }
}
