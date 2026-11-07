use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use client::{
    CreateDynamicVirtualAccountRequest, CreateVirtualAccountRequest, ZainpayClient,
};

fn sample_request() -> CreateVirtualAccountRequest {
    CreateVirtualAccountRequest {
        address: "123 Main Street, Lagos".into(),
        bank_type: "gtBank".into(),
        bvn: "12345678901".into(),
        dob: "15-06-1990".into(),
        email: "muhammad.abdullahi@example.com".into(),
        first_name: "Muhammad".into(),
        gender: "M".into(),
        mobile_number: "08012345678".into(),
        state: "Lagos".into(),
        surname: "Abdullahi".into(),
        title: "Mr".into(),
        zainbox_code: "STORE_Xy9Kp2mNqRtVwZ1sL".into(),
        middle_name: None,
    }
}

#[tokio::test]
async fn create_virtual_account_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/virtual-account/create/request"))
        .and(header("Authorization", "Bearer test-key"))
        .and(header("Content-Type", "application/json"))
        .and(body_partial_json(json!({
            "bankType": "gtBank",
            "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL",
            "firstName": "Muhammad",
            "surname": "Abdullahi",
            "email": "muhammad.abdullahi@example.com"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "data": {
                "accountName": "Muhammad Ahmad Abdullahi",
                "accountNumber": "7760000001",
                "accountType": "",
                "bankName": "gtBank",
                "email": "muhammad.abdullahi@example.com"
            },
            "description": "successful",
            "status": "200 OK"
        })))
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client
        .virtual_accounts()
        .create(&sample_request())
        .await
        .unwrap();

    assert!(response.is_success());
    let data = response.data.unwrap();
    assert_eq!(data.account_number, "7760000001");
    assert_eq!(data.bank_name, "gtBank");
}

#[tokio::test]
async fn create_virtual_account_application_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/virtual-account/create/request"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "02",
            "data": null,
            "description": "invalid bvn",
            "status": "400 BAD_REQUEST"
        })))
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client
        .virtual_accounts()
        .create(&sample_request())
        .await
        .unwrap();

    assert!(!response.is_success());
    assert_eq!(response.code, "02");
    assert!(response.data.is_none());
}

#[tokio::test]
async fn create_virtual_account_error_with_empty_data_object() {
    // Live Zainpay often returns `data: {}` on failure — must not 502 at Koodify.
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/virtual-account/create/request"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "99",
            "data": {},
            "description": "Unable to create virtual account",
            "status": "400 BAD_REQUEST"
        })))
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client
        .virtual_accounts()
        .create(&sample_request())
        .await
        .unwrap();

    assert!(!response.is_success());
    assert_eq!(response.code, "99");
    assert_eq!(response.description, "Unable to create virtual account");
}

#[tokio::test]
async fn create_dynamic_virtual_account_sends_numeric_duration() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/virtual-account/dynamic/create/request"))
        .and(header("Authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "bankType": "gtBank",
            "email": "ada@example.com",
            "amount": "50000",
            "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL",
            "txnRef": "ORDER_12345_DVA",
            "duration": 900,
            "accountName": "Zainpay Checkout",
            "callBackUrl": "https://example.com/webhook/zainpay"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "description": "successful",
            "status": "200 OK",
            "data": {
                "accountNumber": "7760000001",
                "accountName": "Zainpay Checkout",
                "txnRef": "ORDER_12345_DVA"
            }
        })))
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let req = CreateDynamicVirtualAccountRequest {
        bank_type: "gtBank".into(),
        email: "ada@example.com".into(),
        amount: "50000".into(),
        zainbox_code: "STORE_Xy9Kp2mNqRtVwZ1sL".into(),
        txn_ref: "ORDER_12345_DVA".into(),
        duration: "900".into(),
        account_name: "Zainpay Checkout".into(),
        call_back_url: "https://example.com/webhook/zainpay".into(),
    };

    let response = client.virtual_accounts().create_dynamic(&req).await.unwrap();
    assert!(response.is_success());
}

#[test]
fn dynamic_request_serializes_duration_as_number() {
    let req = CreateDynamicVirtualAccountRequest {
        bank_type: "gtBank".into(),
        email: "ada@example.com".into(),
        amount: "50000".into(),
        zainbox_code: "box".into(),
        txn_ref: "ref".into(),
        duration: "900".into(),
        account_name: "Zainpay Checkout".into(),
        call_back_url: "https://example.com/hook".into(),
    };
    let value = serde_json::to_value(&req).unwrap();
    assert!(value["duration"].is_number());
    assert_eq!(value["duration"], 900);
    assert!(value["amount"].is_string());
    assert_eq!(value["amount"], "50000");
}

#[tokio::test]
async fn balance_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/virtual-account/wallet/balance/v2/7760000001"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": "00",
            "description": "successful",
            "status": "200 OK",
            "data": {
                "accountNumber": "7760000001",
                "balance": "150000"
            }
        })))
        .mount(&server)
        .await;

    let client = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("test-key")
        .build()
        .unwrap();

    let response = client
        .virtual_accounts()
        .balance("7760000001")
        .await
        .unwrap();

    assert!(response.is_success());
    let data = response.data.unwrap();
    assert_eq!(data.account_number.as_deref(), Some("7760000001"));
}
