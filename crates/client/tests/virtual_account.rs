use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use client::{CreateVirtualAccountRequest, ZainpayClient};

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
