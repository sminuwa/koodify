use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::envelope::ApiResponse;

pub struct ReconciliationApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl ReconciliationApi<'_> {
    /// `GET /virtual-account/wallet/deposit/verify/v2/{txnRef}`
    pub async fn verify_deposit(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let path = format!("/virtual-account/wallet/deposit/verify/v2/{txn_ref}");
        self.client.get_json(&path).await
    }

    /// `GET /virtual-account/wallet/transaction/reconcile/bank-deposit`
    pub async fn reconcile_bank_deposit(
        &self,
        bank_type: &str,
        verification_type: &str,
        account_number: &str,
        session_id: Option<&str>,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let mut q = vec![
            ("bankType", bank_type.to_string()),
            ("verificationType", verification_type.to_string()),
            ("accountNumber", account_number.to_string()),
        ];
        if let Some(session_id) = session_id {
            q.push(("sessionId", session_id.to_string()));
        }
        self.client
            .get_json_query(
                "/virtual-account/wallet/transaction/reconcile/bank-deposit",
                &q,
            )
            .await
    }

    /// `GET /virtual-account/wallet/transaction/reconcile/card-payment`
    pub async fn reconcile_card_payment(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<Value>, ZainpayError> {
        let q = [("txnRef", txn_ref.to_string())];
        self.client
            .get_json_query(
                "/virtual-account/wallet/transaction/reconcile/card-payment",
                &q,
            )
            .await
    }
}
