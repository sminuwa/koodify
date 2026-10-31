use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;
use crate::types::transfer::FundsTransferRequest;

pub struct TransferApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl TransferApi<'_> {
    /// `POST /bank/transfer/v2`
    ///
    /// Live API serves this under `/bank/transfer/v2` (docs URL field still
    /// shows `/zainbox/bank/transfer/v2`, which returns HTTP 405).
    pub async fn funds_transfer(
        &self,
        req: &FundsTransferRequest,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.post_json("/bank/transfer/v2", req).await
    }

    /// `GET /virtual-account/wallet/transaction/verify/{txnRef}`
    pub async fn verify(&self, txn_ref: &str) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/virtual-account/wallet/transaction/verify/{txn_ref}");
        self.client.get_json(&path).await
    }

    /// `GET /bank/transfer/network/status`
    pub async fn network_status(&self) -> Result<ApiResponse<Value>, ZainpayError> {
        self.client.get_json("/bank/transfer/network/status").await
    }
}
