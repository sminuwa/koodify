use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;

pub struct WebhookApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl WebhookApi<'_> {
    /// `GET /zainbox/repush/deposit/{txnRef}`
    pub async fn repush_deposit(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/repush/deposit/{txn_ref}");
        self.client.get_json(&path).await
    }

    /// `GET /zainbox/repush/card-payment/{txnRef}`
    pub async fn repush_card_payment(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/zainbox/repush/card-payment/{txn_ref}");
        self.client.get_json(&path).await
    }
}
