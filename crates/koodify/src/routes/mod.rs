pub mod activity;
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
        // ── Stable Koodify `/api/v1` surface (insulates merchants from Zainpay path churn)
        // Composites (one Koodify call → two Zainpay upstreams)
        .route(
            "/api/v1/zainbox/{zainbox_code}",
            get(composite::zainbox_snapshot),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/wallet",
            get(composite::zainbox_wallet_view),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/transactions",
            get(composite::zainbox_activity),
        )
        // Deprecated alias — prefer `/transactions`
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
            "/api/v1/merchant/transactions",
            get(composite::merchant_activity),
        )
        // Deprecated alias — prefer `/merchant/transactions`
        .route(
            "/api/v1/merchant/activity",
            get(composite::merchant_activity),
        )
        .route("/api/v1/bank/resolve", get(composite::name_resolve))
        .route("/api/v1/payout/prep", get(composite::payout_prep))
        .route("/api/v1/transfer", post(composite::transfer_and_verify))
        .route(
            "/api/v1/settlement",
            post(composite::settlement_create_with_history),
        )
        // Static VA — talks to `/virtual-account/…` upstreams
        .route("/api/v1/va/create", post(virtual_account::create))
        .route(
            "/api/v1/va/balance/{account_number}",
            get(virtual_account::balance),
        )
        .route(
            "/api/v1/va/transactions/{account_number}",
            get(virtual_account::transactions),
        )
        .route(
            "/api/v1/va/{account_number}",
            get(composite::virtual_account_wallet),
        )
        .route("/api/v1/va/status", patch(virtual_account::update_status))
        .route("/api/v1/va/bvn", post(virtual_account::update_bvn))
        .route("/api/v1/va/fund", post(virtual_account::fund_static))
        // Deprecated composite alias — prefer `/api/v1/va/{accountNumber}`
        .route(
            "/api/v1/virtual-account/{account_number}",
            get(composite::virtual_account_wallet),
        )
        // Dynamic VA (DVA)
        .route(
            "/api/v1/dva/create",
            post(virtual_account::create_dynamic),
        )
        .route(
            "/api/v1/dva/deposit/{txn_ref}",
            get(virtual_account::dynamic_deposit_status),
        )
        .route(
            "/api/v1/dva/verify/{txn_ref}",
            get(composite::dynamic_deposit_check),
        )
        .route("/api/v1/dva/fund", post(virtual_account::fund_dynamic))
        // Deprecated composite alias — prefer `/api/v1/dva/verify/{txnRef}`
        .route(
            "/api/v1/deposit/{txn_ref}",
            get(composite::dynamic_deposit_check),
        )
        // Transfer verify (single upstream)
        .route(
            "/api/v1/transfers/{txn_ref}/verify",
            get(transfer::verify),
        )
        .route(
            "/api/v1/transfers/network",
            get(transfer::network_status),
        )
        .route("/api/v1/banks", get(bank::list))
        .route("/api/v1/banks/resolve", get(composite::name_resolve))
        .route("/api/v1/banks/enquiry", get(bank::name_enquiry))
        // Zainbox admin (single upstream wrappers)
        .route("/api/v1/zainboxes", get(zainbox::list))
        .route(
            "/api/v1/zainbox",
            post(zainbox::create).patch(zainbox::update),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/vas",
            get(zainbox::virtual_accounts),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/balances",
            get(zainbox::accounts_balance),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/profile",
            get(zainbox::profile_v2),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/profile/v1",
            get(zainbox::profile),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/deposit-summary",
            get(zainbox::deposit_summary),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/card-transactions",
            get(zainbox::card_transactions),
        )
        .route(
            "/api/v1/zainbox/{zainbox_code}/ledger",
            get(zainbox::transactions),
        )
        .route(
            "/api/v1/zainbox/subscribe",
            post(zainbox::subscribe),
        )
        .route(
            "/api/v1/zainbox/unsubscribe",
            post(zainbox::unsubscribe),
        )
        .route(
            "/api/v1/merchant/transactions/summary",
            get(zainbox::transactions_summary),
        )
        .route(
            "/api/v1/merchant/transactions/list",
            get(zainbox::merchant_transactions),
        )
        // Transfers / settlements / reconcile (single upstream)
        .route("/api/v1/transfers", post(transfer::funds_transfer))
        .route(
            "/api/v1/settlements",
            post(settlement::create).get(settlement::get),
        )
        .route(
            "/api/v1/settlements/history/{zainbox_code}",
            get(settlement::history),
        )
        .route(
            "/api/v1/settlements/internal/{zainbox_code}",
            get(settlement::internal_account),
        )
        .route(
            "/api/v1/reconcile/deposit/{txn_ref}",
            get(reconciliation::verify_deposit),
        )
        .route(
            "/api/v1/reconcile/bank-deposit",
            get(reconciliation::reconcile_bank_deposit),
        )
        .route(
            "/api/v1/reconcile/card-payment",
            get(reconciliation::reconcile_card_payment),
        )
        // Checkout (card / temporary VA) — stable names for inline + server flows
        .route(
            "/api/v1/checkout/initialize",
            post(card::initialize_payment),
        )
        .route(
            "/api/v1/checkout/initialize/inline",
            post(card::initialize_payment_inline),
        )
        .route(
            "/api/v1/checkout/account",
            get(card::generate_temporary_virtual_account),
        )
        .route(
            "/api/v1/checkout/status",
            get(card::temporary_virtual_account_payment_status),
        )
        .route(
            "/api/v1/checkout/recurring",
            post(card::recurring_purchase),
        )
        .route(
            "/api/v1/webhooks/repush/deposit/{txn_ref}",
            get(webhook::repush_deposit),
        )
        .route(
            "/api/v1/webhooks/repush/card/{txn_ref}",
            get(webhook::repush_card_payment),
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
        .route("/api/v1/activity", get(activity::recent))
        .route(
            "/api/v1/activity/{txn_ref}",
            get(activity::by_txn_ref),
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
        // Customers / merchants (`uuid` + integer `id` = 1, 2, 3, …)
        // Stable aliases under `/api/v1/customers` for future merchant UI.
        .route("/customers", post(customers::create).get(customers::list))
        .route(
            "/api/v1/customers",
            post(customers::create).get(customers::list),
        )
        .route(
            "/customers/by-uuid/{uuid}",
            get(customers::by_uuid),
        )
        .route(
            "/api/v1/customers/by-uuid/{uuid}",
            get(customers::by_uuid),
        )
        .route(
            "/customers/{id}",
            get(customers::get_one)
                .put(customers::update)
                .patch(customers::patch),
        )
        .route(
            "/api/v1/customers/{id}",
            get(customers::get_one)
                .put(customers::update)
                .patch(customers::patch),
        )
        .route(
            "/customers/{id}/zainbox",
            get(customers::zainbox),
        )
        .route(
            "/api/v1/customers/{id}/zainbox",
            get(customers::zainbox),
        )
        .route(
            "/customers/{id}/provision",
            post(customers::provision),
        )
        .route(
            "/api/v1/customers/{id}/provision",
            post(customers::provision),
        )
        .route(
            "/customers/{id}/status",
            patch(customers::update_status),
        )
        .route(
            "/api/v1/customers/{id}/status",
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
        .route(
            "/credentials/{id}/rotate-secret",
            post(credentials::rotate_secret),
        )
        .route(
            "/api/v1/credentials/{id}/rotate-secret",
            post(credentials::rotate_secret),
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
            "/api/v1/zainboxes/sync-webhook-callbacks",
            post(zainbox::sync_webhook_callbacks),
        )
        .route(
            "/zainbox/repush/deposit/{txn_ref}",
            get(webhook::repush_deposit),
        )
        .route(
            "/zainbox/repush/card-payment/{txn_ref}",
            get(webhook::repush_card_payment),
        )
}
