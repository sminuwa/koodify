//! Convenience APIs that call two Zainpay endpoints and return one envelope.

use serde_json::Value;

use crate::client::ZainpayClient;
use crate::error::ZainpayError;
use crate::types::composite::{
    composite_ok, DynamicDepositCheck, MerchantActivity, NameResolve, PayoutPrep,
    SettlementContext, SettlementCreateWithHistory, SubscriptionsBundle, TransferAndVerify,
    VirtualAccountWallet, ZainboxActivity, ZainboxProfileBilling, ZainboxSnapshot,
    ZainboxWalletView,
};
use crate::types::envelope::ApiResponse;
use crate::types::settlement::CreateSettlementRequest;
use crate::types::transfer::FundsTransferRequest;

pub struct CompositeApi<'a> {
    pub(crate) client: &'a ZainpayClient,
}

impl CompositeApi<'_> {
    /// Profile + account balances for a zainbox.
    ///
    /// Upstream: `GET /zainbox/profile/{code}` + `GET /zainbox/accounts/balance/{code}`
    pub async fn zainbox_snapshot(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<ZainboxSnapshot>, ZainpayError> {
        let profile = self.client.zainbox().profile(zainbox_code).await?;
        let balances = self.client.zainbox().accounts_balance(zainbox_code).await?;
        Ok(composite_ok(
            "composite: zainbox profile + balances",
            ZainboxSnapshot { profile, balances },
        ))
    }

    /// Virtual accounts list + balances for a zainbox.
    ///
    /// Upstream: `GET /zainbox/virtual-accounts/{code}` + `GET /zainbox/accounts/balance/{code}`
    pub async fn zainbox_wallet_view(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<ZainboxWalletView>, ZainpayError> {
        let virtual_accounts = self.client.zainbox().virtual_accounts(zainbox_code).await?;
        let balances = self.client.zainbox().accounts_balance(zainbox_code).await?;
        Ok(composite_ok(
            "composite: zainbox virtual accounts + balances",
            ZainboxWalletView {
                virtual_accounts,
                balances,
            },
        ))
    }

    /// Transactions + deposit summary for a zainbox.
    ///
    /// Upstream: `GET /zainbox/transactions/{code}` + `GET /zainbox/transfer/deposit/summary/{code}`
    pub async fn zainbox_activity(
        &self,
        zainbox_code: &str,
        count: Option<&str>,
        txn_type: Option<&str>,
        account_number: Option<&str>,
        payment_channel: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<ZainboxActivity>, ZainpayError> {
        let transactions = self
            .client
            .zainbox()
            .transactions(
                zainbox_code,
                count,
                txn_type,
                account_number,
                payment_channel,
                date_from,
                date_to,
            )
            .await?;
        let deposit_summary = self
            .client
            .zainbox()
            .deposit_summary(zainbox_code, date_from, date_to)
            .await?;
        Ok(composite_ok(
            "composite: zainbox transactions + deposit summary",
            ZainboxActivity {
                transactions,
                deposit_summary,
            },
        ))
    }

    /// Settlement config + internal settlement account.
    ///
    /// Upstream: `GET /zainbox/settlement?zainboxCode=` + `GET /zainbox/internal-settlement-account/{code}`
    ///
    /// Note: live `GET /zainbox/settlement` may return HTTP 405; the nested
    /// `settlement` field will then not be populated (this method errors).
    pub async fn settlement_context(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<SettlementContext>, ZainpayError> {
        let settlement = self.client.settlements().get(zainbox_code).await?;
        let internal_account = self
            .client
            .settlements()
            .internal_account(zainbox_code)
            .await?;
        Ok(composite_ok(
            "composite: settlement config + internal account",
            SettlementContext {
                settlement,
                internal_account,
            },
        ))
    }

    /// VA balance + transactions for one account number.
    ///
    /// Upstream: `GET /virtual-account/wallet/balance/v2/{acct}` +
    /// `GET /virtual-account/wallet/transactions/{acct}`
    pub async fn virtual_account_wallet(
        &self,
        account_number: &str,
        count: Option<&str>,
        txn_type: Option<&str>,
        payment_channel: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<ApiResponse<VirtualAccountWallet>, ZainpayError> {
        let balance_typed = self
            .client
            .virtual_accounts()
            .balance(account_number)
            .await?;
        let balance = ApiResponse {
            code: balance_typed.code,
            status: balance_typed.status,
            description: balance_typed.description,
            data: balance_typed
                .data
                .map(|d| serde_json::to_value(d).unwrap_or(Value::Null)),
        };
        let transactions = self
            .client
            .virtual_accounts()
            .transactions(
                account_number,
                count,
                txn_type,
                payment_channel,
                date_from,
                date_to,
            )
            .await?;
        Ok(composite_ok(
            "composite: virtual-account balance + transactions",
            VirtualAccountWallet {
                balance,
                transactions,
            },
        ))
    }

    /// Bank list + name enquiry.
    ///
    /// Upstream: `GET /bank/list` + `GET /bank/name-enquiry`
    pub async fn name_resolve(
        &self,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ApiResponse<NameResolve>, ZainpayError> {
        let banks = self.client.bank().list().await?;
        let name_enquiry = self
            .client
            .bank()
            .name_enquiry(bank_code, account_number)
            .await?;
        Ok(composite_ok(
            "composite: bank list + name enquiry",
            NameResolve {
                banks,
                name_enquiry,
            },
        ))
    }

    /// All + active service subscriptions for a zainbox.
    ///
    /// Upstream: `…/subscriptions/all/{code}` + `…/subscriptions/active/{code}`
    pub async fn subscriptions_bundle(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<SubscriptionsBundle>, ZainpayError> {
        let all = self
            .client
            .zainbox()
            .subscriptions_all(zainbox_code, None)
            .await?;
        let active = self
            .client
            .zainbox()
            .subscriptions_active(zainbox_code)
            .await?;
        Ok(composite_ok(
            "composite: subscriptions all + active",
            SubscriptionsBundle { all, active },
        ))
    }

    /// Profile + billing config for a zainbox.
    ///
    /// Upstream: `GET /zainbox/profile/{code}` + `GET /zainbox/billing/config/{code}`
    pub async fn zainbox_profile_billing(
        &self,
        zainbox_code: &str,
    ) -> Result<ApiResponse<ZainboxProfileBilling>, ZainpayError> {
        let profile = self.client.zainbox().profile(zainbox_code).await?;
        let billing_config = self.client.zainbox().billing_config(zainbox_code).await?;
        Ok(composite_ok(
            "composite: zainbox profile + billing config",
            ZainboxProfileBilling {
                profile,
                billing_config,
            },
        ))
    }

    /// Merchant transaction summary + list.
    ///
    /// Upstream: `GET /zainbox/transactions/summary` + `GET /zainbox/transactions`
    pub async fn merchant_activity(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        count: Option<&str>,
        txn_type: Option<&str>,
        account_number: Option<&str>,
        payment_channel: Option<&str>,
    ) -> Result<ApiResponse<MerchantActivity>, ZainpayError> {
        let summary = self
            .client
            .zainbox()
            .transactions_summary(date_from, date_to)
            .await?;
        let transactions = self
            .client
            .zainbox()
            .merchant_transactions(
                count,
                txn_type,
                account_number,
                payment_channel,
                date_from,
                date_to,
            )
            .await?;
        Ok(composite_ok(
            "composite: merchant transactions summary + list",
            MerchantActivity {
                summary,
                transactions,
            },
        ))
    }

    /// Network status + name enquiry (payout preflight).
    ///
    /// Upstream: `GET /bank/transfer/network/status` + `GET /bank/name-enquiry`
    pub async fn payout_prep(
        &self,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ApiResponse<PayoutPrep>, ZainpayError> {
        let network_status = self.client.transfers().network_status().await?;
        let name_enquiry = self
            .client
            .bank()
            .name_enquiry(bank_code, account_number)
            .await?;
        Ok(composite_ok(
            "composite: transfer network status + name enquiry",
            PayoutPrep {
                network_status,
                name_enquiry,
            },
        ))
    }

    /// Funds transfer then verify by the request `txnRef`.
    ///
    /// Upstream: `POST /bank/transfer/v2` + `GET …/transaction/verify/{txnRef}`
    pub async fn transfer_and_verify(
        &self,
        req: &FundsTransferRequest,
    ) -> Result<ApiResponse<TransferAndVerify>, ZainpayError> {
        let transfer = self.client.transfers().funds_transfer(req).await?;
        let verify = self.client.transfers().verify(&req.txn_ref).await?;
        Ok(composite_ok(
            "composite: funds transfer + verify",
            TransferAndVerify { transfer, verify },
        ))
    }

    /// Dynamic deposit status + deposit verify for one txn ref.
    ///
    /// Upstream: `GET …/dynamic/deposit/status/{txnRef}` +
    /// `GET …/deposit/verify/v2/{txnRef}`
    pub async fn dynamic_deposit_check(
        &self,
        txn_ref: &str,
    ) -> Result<ApiResponse<DynamicDepositCheck>, ZainpayError> {
        let deposit_status = self
            .client
            .virtual_accounts()
            .dynamic_deposit_status(txn_ref)
            .await?;
        let deposit_verify = self.client.reconciliation().verify_deposit(txn_ref).await?;
        Ok(composite_ok(
            "composite: dynamic deposit status + deposit verify",
            DynamicDepositCheck {
                deposit_status,
                deposit_verify,
            },
        ))
    }

    /// Create settlement then fetch history for the same zainbox.
    ///
    /// Upstream: `POST /zainbox/settlement` + `GET /zainbox/settlement/history/{code}`
    pub async fn settlement_create_with_history(
        &self,
        req: &CreateSettlementRequest,
        count: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        status: Option<&str>,
    ) -> Result<ApiResponse<SettlementCreateWithHistory>, ZainpayError> {
        let create = self.client.settlements().create(req).await?;
        let history = self
            .client
            .settlements()
            .history(&req.zainbox_code, count, date_from, date_to, status)
            .await?;
        Ok(composite_ok(
            "composite: settlement create + history",
            SettlementCreateWithHistory { create, history },
        ))
    }
}
