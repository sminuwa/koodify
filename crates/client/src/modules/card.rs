use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::card::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};
use crate::types::envelope::ApiResponse;

pub struct CardApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl CardApi<'_> {
    /// `POST /zainbox/card/initialize/payment`
    pub async fn initialize_payment(
        &self,
        req: &InitializeCardPaymentRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/zainbox/card/initialize/payment", req)
            .await
    }

    /// `POST /zainbox/card/recurring/purchase`
    pub async fn recurring_purchase(
        &self,
        req: &RecurringCardPurchaseRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/zainbox/card/recurring/purchase", req)
            .await
    }

    /// `POST /zainbox/card/initialize/payment/inline` — browser / InlineJS session.
    pub async fn initialize_payment_inline(
        &self,
        req: &InitializeCardPaymentRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .post_json("/zainbox/card/initialize/payment/inline", req)
            .await
    }

    /// `GET /zainbox/card/generate-temporary-virtual-account?bankType=&e=`
    ///
    /// `bank_type` is the raw bank key (e.g. `gtBank`); the client quotes it the
    /// way Zainpay's InlineJS does (`"gtBank"`).
    pub async fn generate_temporary_virtual_account(
        &self,
        bank_type: &str,
        session_e: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let quoted = format!("\"{bank_type}\"");
        self.client
            .get_json_query(
                "/zainbox/card/generate-temporary-virtual-account",
                &[("bankType", quoted.as_str()), ("e", session_e)],
            )
            .await
    }

    /// `GET /zainbox/card/temporary-virtual-account-payment-status?txnRef=`
    pub async fn temporary_virtual_account_payment_status(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client
            .get_json_query(
                "/zainbox/card/temporary-virtual-account-payment-status",
                &[("txnRef", txn_ref)],
            )
            .await
    }
}
