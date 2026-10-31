//! Reusable HTTP client for the Zainpay payment API.
//!
//! This crate has no database dependency. Use it from CLIs, workers, or the
//! `koodify` service which adds custom metadata persistence.

mod client;
mod error;
pub mod modules;
pub mod types;
mod webhook_verify;

pub use client::{Environment, ZainpayClient, ZainpayClientBuilder};
pub use error::ZainpayError;
pub use modules::bank::BankApi;
pub use modules::card::CardApi;
pub use modules::composite::CompositeApi;
pub use modules::reconciliation::ReconciliationApi;
pub use modules::settlement::SettlementApi;
pub use modules::transfer::TransferApi;
pub use modules::virtual_account::VirtualAccountApi;
pub use modules::webhook::WebhookApi;
pub use modules::zainbox::ZainboxApi;
pub use types::card::{InitializeCardPaymentRequest, RecurringCardPurchaseRequest};
pub use types::codes::{describe_code, is_success_code, SUCCESS as CODE_SUCCESS};
pub use types::composite::{
    DynamicDepositCheck, MerchantActivity, NameResolve, PayoutPrep, SettlementContext,
    SettlementCreateWithHistory, SubscriptionsBundle, TransferAndVerify, VirtualAccountWallet,
    ZainboxActivity, ZainboxProfileBilling, ZainboxSnapshot, ZainboxWalletView,
};
pub use types::envelope::ApiResponse;
pub use types::settlement::{CreateSettlementRequest, SettlementAccountInput};
pub use types::transfer::FundsTransferRequest;
pub use types::virtual_account::{
    CreateDynamicVirtualAccountRequest, CreateVirtualAccountRequest, FundVirtualAccountRequest,
    UpdateVirtualAccountBvnRequest, UpdateVirtualAccountStatusRequest, VirtualAccountBalance,
    VirtualAccountData,
};
pub use types::webhook::{WebhookAmount, WebhookData, WebhookEvent, WebhookEventType};
pub use types::zainbox::{
    CreateZainboxRequest, ServiceSubscriptionRequest, UpdateZainboxRequest,
};
pub use webhook_verify::{sign_webhook_body, verify_webhook_signature};
