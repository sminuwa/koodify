pub mod bank;
pub mod card;
pub mod composite;
pub mod credentials;
pub mod customers;
pub mod helpers;
pub mod kyc;
pub mod operations;
pub mod payment_split;
pub mod reconcile;
pub mod reconciliation;
pub mod settlement;
pub mod transfer;
pub mod virtual_account;
pub mod webhook;
pub mod zainbox;

use axum::routing::{get, patch, post, put};
use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        // Plain /api/v1 convenience (two Zainpay calls → one response)
        .route(
            "/api/v1/zainbox/{zainbox_code}",
            get(composite::zainbox_snapshot),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/wallet",
            get(composite::zainbox_wallet_view),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/activity",
            get(composite::zainbox_activity),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/settlement",
            get(composite::settlement_context),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/billing",
            get(composite::zainbox_profile_billing),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/subscriptions",
            get(composite::subscriptions_bundle),
        )
        .route(
            "/api/v1/virtual-account/{account_number}",
            get(composite::virtual_account_wallet),
        )
        .route("/api/v1/bank/resolve", get(composite::name_resolve))
        .route(
            "/api/v1/merchant/activity",
            get(composite::merchant_activity),
        )
        .route("/api/v1/payout/prep", get(composite::payout_prep))
        .route("/api/v1/transfer", post(composite::transfer_and_verify))
        .route(
            "/api/v1/deposit/{txn_ref}",
            get(composite::dynamic_deposit_check),
        )
        .route(
            "/api/v1/settlement",
            post(composite::settlement_create_with_history),
        )
        // Zainbox Payment Split (admin)
        .route(
            "/api/v1/split-plans",
            put(payment_split::upsert_plan).get(payment_split::list_plans),
        )
        .route("/api/v1/split-plans/{id}", get(payment_split::get_plan))
        .route(
            "/api/v1/payment-splits/{txn_ref}",
            get(payment_split::get_payment_split).post(payment_split::run_payment_split),
        )
        .route(
            "/api/v1/payment-splits/{txn_ref}/run",
            post(payment_split::run_payment_split),
        )
        .route(
            "/api/v1/reconcile/pending",
            post(reconcile::run_pending),
        )
        // Local lookups
        .route(
            "/virtual-accounts/{account_number}",
            get(virtual_account::get_one),
        )
        .route(
            "/virtual-accounts/{account_number}/balance",
            get(virtual_account::balance),
        )
        .route(
            "/operations/{resource_key}",
            get(operations::by_resource_key),
        )
        .route(
            "/transactions/{resource_key}",
            get(operations::by_resource_key),
        )
        .route(
            "/transactions/by-zainbox/{zainbox_code}",
            get(operations::by_zainbox_code),
        )
        // Customers (`uuid` + integer `id` = 1, 2, 3, …)
        .route("/customers", post(customers::create).get(customers::list))
        .route(
            "/customers/{id}",
            get(customers::get_one).put(customers::update),
        )
        .route(
            "/customers/{id}/status",
            patch(customers::update_status),
        )
        // KYC profiles (FK → customers.id)
        .route("/kyc", put(kyc::upsert).get(kyc::list))
        .route(
            "/kyc/by-customer/{customer_id}",
            get(kyc::by_customer),
        )
        .route("/kyc/{id}", get(kyc::get_one))
        .route("/kyc/{id}/status", patch(kyc::update_status))
        // Credentials: each customer has test + live (publicKey / privateKey)
        .route(
            "/credentials",
            put(credentials::upsert).get(credentials::list),
        )
        .route(
            "/credentials/by-customer/{customer_id}",
            get(credentials::by_customer),
        )
        .route(
            "/credentials/by-customer/{customer_id}/{environment}",
            get(credentials::by_customer_env),
        )
        .route("/credentials/{id}", get(credentials::get_one))
        .route(
            "/credentials/{id}/status",
            patch(credentials::update_status),
        )
        // Zainbox
        .route("/zainbox/create/request", post(zainbox::create))
        .route("/zainbox/list", get(zainbox::list))
        .route("/zainbox/update", patch(zainbox::update))
        .route(
            "/zainbox/virtual-accounts/{zainbox_code}",
            get(zainbox::virtual_accounts),
        )
        .route(
            "/zainbox/accounts/balance/{zainbox_code}",
            get(zainbox::accounts_balance),
        )
        .route("/zainbox/transactions", get(zainbox::merchant_transactions))
        .route(
            "/zainbox/transactions/{zainbox_code}",
            get(zainbox::transactions),
        )
        .route(
            "/zainbox/transfer/deposit/summary/{zainbox_code}",
            get(zainbox::deposit_summary),
        )
        .route("/zainbox/profile/{zainbox_code}", get(zainbox::profile))
        .route("/zainbox/profile/v2/{zainbox_code}", get(zainbox::profile_v2))
        .route(
            "/zainbox/billing/config/{zainbox_code}",
            get(zainbox::billing_config),
        )
        .route(
            "/zainbox/transactions/summary",
            get(zainbox::transactions_summary),
        )
        .route(
            "/zainbox/card/transactions/{zainbox_code}",
            get(zainbox::card_transactions),
        )
        .route(
            "/zainbox/service-subscription/subscribe",
            post(zainbox::subscribe),
        )
        .route(
            "/zainbox/service-subscription/unsubscribe",
            post(zainbox::unsubscribe),
        )
        .route(
            "/zainbox/service-subscription/subscriptions/all/{zainbox_code}",
            get(zainbox::subscriptions_all),
        )
        .route(
            "/zainbox/service-subscription/subscriptions/active/{zainbox_code}",
            get(zainbox::subscriptions_active),
        )
        // Virtual accounts
        .route(
            "/virtual-account/create/request",
            post(virtual_account::create),
        )
        .route(
            "/virtual-account/dynamic/create/request",
            post(virtual_account::create_dynamic),
        )
        .route(
            "/virtual-account/wallet/balance/v2/{account_number}",
            get(virtual_account::balance),
        )
        .route(
            "/zainbox/virtual-account/wallet/balance/v2/{account_number}",
            get(virtual_account::balance),
        )
        .route(
            "/virtual-account/change/account/status",
            patch(virtual_account::update_status),
        )
        .route(
            "/virtual-account/update/bvn",
            post(virtual_account::update_bvn),
        )
        .route(
            "/virtual-account/wallet/transactions/{account_number}",
            get(virtual_account::transactions),
        )
        .route(
            "/virtual-account/dynamic/deposit/status/{txn_ref}",
            get(virtual_account::dynamic_deposit_status),
        )
        .route(
            "/virtual-account/static/fund",
            post(virtual_account::fund_static),
        )
        .route(
            "/virtual-account/dynamic/fund",
            post(virtual_account::fund_dynamic),
        )
        // Bank (canonical live paths + legacy /zainbox/bank/* aliases)
        .route("/bank/list", get(bank::list))
        .route("/bank/name-enquiry", get(bank::name_enquiry))
        .route("/zainbox/bank/list", get(bank::list))
        .route("/zainbox/bank/name-enquiry", get(bank::name_enquiry))
        // Transfers
        .route("/bank/transfer/v2", post(transfer::funds_transfer))
        .route("/zainbox/bank/transfer/v2", post(transfer::funds_transfer))
        .route(
            "/virtual-account/wallet/transaction/verify/{txn_ref}",
            get(transfer::verify),
        )
        .route("/bank/transfer/network/status", get(transfer::network_status))
        // Settlements
        .route(
            "/zainbox/settlement",
            post(settlement::create).get(settlement::get),
        )
        .route(
            "/zainbox/settlement/history/{zainbox_code}",
            get(settlement::history),
        )
        .route(
            "/zainbox/internal-settlement-account/{zainbox_code}",
            get(settlement::internal_account),
        )
        // Reconciliation
        .route(
            "/virtual-account/wallet/deposit/verify/v2/{txn_ref}",
            get(reconciliation::verify_deposit),
        )
        .route(
            "/virtual-account/wallet/transaction/reconcile/bank-deposit",
            get(reconciliation::reconcile_bank_deposit),
        )
        .route(
            "/virtual-account/wallet/transaction/reconcile/card-payment",
            get(reconciliation::reconcile_card_payment),
        )
        // Cards
        .route(
            "/zainbox/card/initialize/payment",
            post(card::initialize_payment),
        )
        .route(
            "/zainbox/card/initialize/payment/inline",
            post(card::initialize_payment_inline),
        )
        .route(
            "/zainbox/card/generate-temporary-virtual-account",
            get(card::generate_temporary_virtual_account),
        )
        .route(
            "/zainbox/card/temporary-virtual-account-payment-status",
            get(card::temporary_virtual_account_payment_status),
        )
        .route(
            "/zainbox/card/recurring/purchase",
            post(card::recurring_purchase),
        )
        // Webhooks
        .route("/webhooks/zainpay", post(webhook::inbound))
        .route(
            "/zainbox/repush/deposit/{txn_ref}",
            get(webhook::repush_deposit),
        )
        .route(
            "/zainbox/repush/card-payment/{txn_ref}",
            get(webhook::repush_card_payment),
        )
}
