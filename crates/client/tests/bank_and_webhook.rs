use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use client::{
    sign_webhook_body, verify_webhook_signature, WebhookEvent, WebhookEventType, ZainpayClient,
};

#[tokio::test]
async fn bank_list_uses_live_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/bank/list"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": [{ "code": "000013", "name": "GTBANK" }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client.bank().list().await.unwrap();
    assert!(response.is_success());
}

#[tokio::test]
async fn name_enquiry_uses_live_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/bank/name-enquiry"))
        .and(query_param("bankCode", "000013"))
        .and(query_param("accountNumber", "5006079921"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": {
                "accountName": "ZNPAY SUNUSI INUWA",
                "accountNumber": "5006079921",
                "bankCode": "000013"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client
        .bank()
        .name_enquiry("000013", "5006079921")
        .await
        .unwrap();
    assert!(response.is_success());
}

#[tokio::test]
async fn funds_transfer_uses_live_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/bank/transfer/v2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "status": "200 OK",
            "description": "successful",
            "data": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let req = client::FundsTransferRequest {
        destination_account_number: "0123456789".into(),
        destination_bank_code: "000014".into(),
        source_account_number: "5006079921".into(),
        source_bank_code: "000013".into(),
        amount: "10000".into(),
        txn_ref: "TXN_TEST_001".into(),
        zainbox_code: "charg_test".into(),
        call_back_url: None,
        narration: None,
    };

    let response = client.transfers().funds_transfer(&req).await.unwrap();
    assert!(response.is_success());
}

#[test]
fn parses_deposit_webhook_and_verifies_signature() {
    let body = json!({
        "data": {
            "depositedAmount": "50000",
            "txnChargesAmount": "750",
            "amountAfterCharges": "49250",
            "bankName": "gtBank",
            "beneficiaryAccountName": "Muhammad Ahmad Abdullahi",
            "beneficiaryAccountNumber": "7760000001",
            "paymentRef": "bOQtDmSgmiaZpXC6PiAR",
            "txnRef": "ORDER_12345_DEP",
            "txnType": "deposit",
            "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL"
        },
        "event": "deposit.success"
    });
    let raw = serde_json::to_vec(&body).unwrap();
    let secret = b"webhook-secret";
    let sig = sign_webhook_body(secret, &raw);
    assert!(verify_webhook_signature(secret, &raw, &sig));

    let event: WebhookEvent = serde_json::from_slice(&raw).unwrap();
    assert_eq!(event.event, WebhookEventType::DepositSuccess);
    assert_eq!(event.data.idempotency_key(), Some("ORDER_12345_DEP"));
}
