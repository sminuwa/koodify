use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Match, Mock, MockServer, Request as WireRequest, ResponseTemplate};
use client::ZainpayClient;
use koodify::config::Config;
use koodify::{build_app_with, db};

fn sample_create_body() -> Value {
    json!({
        "address": "123 Main Street, Lagos",
        "bankType": "gtBank",
        "bvn": "12345678901",
        "dob": "15-06-1990",
        "email": "muhammad.abdullahi@example.com",
        "firstName": "Muhammad",
        "gender": "M",
        "mobileNumber": "08012345678",
        "state": "Lagos",
        "surname": "Abdullahi",
        "title": "Mr",
        "zainboxCode": "STORE_Xy9Kp2mNqRtVwZ1sL",
        "customerId": "cus_123",
        "storeId": "store_9"
    })
}

/// Ensures the outbound Zainpay body has only known fields (no custom metadata).
struct NoCustomMetadataBody;

impl Match for NoCustomMetadataBody {
    fn matches(&self, request: &WireRequest) -> bool {
        let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
            return false;
        };
        let Some(obj) = body.as_object() else {
            return false;
        };

        let forbidden = ["customerId", "storeId", "metadata", "tags"];
        if forbidden.iter().any(|k| obj.contains_key(*k)) {
            return false;
        }

        obj.get("bankType").and_then(|v| v.as_str()) == Some("gtBank")
            && obj.get("zainboxCode").and_then(|v| v.as_str())
                == Some("STORE_Xy9Kp2mNqRtVwZ1sL")
            && obj.get("firstName").and_then(|v| v.as_str()) == Some("Muhammad")
    }
}

async fn setup_pool() -> Option<PgPool> {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping: DATABASE_URL not set");
        return None;
    };

    let pool = db::connect(&url).await.expect("connect postgres");
    db::migrate(&pool).await.expect("migrate");

    // Isolate each run.
    sqlx::query("DELETE FROM virtual_accounts")
        .execute(&pool)
        .await
        .expect("cleanup");

    Some(pool)
}

#[tokio::test]
async fn create_strips_metadata_and_round_trips() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/virtual-account/create/request"))
        .and(header("Authorization", "Bearer zain-test-key"))
        .and(NoCustomMetadataBody)
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
        .expect(1)
        .mount(&server)
        .await;

    let config = Config::new(
        "127.0.0.1:0",
        "unused",
        server.uri(),
        "zain-test-key",
        "gateway-test-key",
    );

    let zainpay = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("zain-test-key")
        .build()
        .unwrap();

    let app = build_app_with(config, pool.clone(), zainpay);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/virtual-account/create/request")
                .header("content-type", "application/json")
                .header("x-api-key", "gateway-test-key")
                .body(Body::from(sample_create_body().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["code"], "00");
    assert_eq!(json["data"]["accountNumber"], "7760000001");
    assert_eq!(json["data"]["metadata"]["customerId"], "cus_123");
    assert_eq!(json["data"]["metadata"]["storeId"], "store_9");

    // Lookup from DB
    let lookup = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/virtual-accounts/7760000001")
                .header("x-api-key", "gateway-test-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(lookup.status(), StatusCode::OK);
    let lookup_body = axum::body::to_bytes(lookup.into_body(), usize::MAX)
        .await
        .unwrap();
    let stored: Value = serde_json::from_slice(&lookup_body).unwrap();
    assert_eq!(stored["accountNumber"], "7760000001");
    assert_eq!(stored["metadata"]["customerId"], "cus_123");
}

#[tokio::test]
async fn rejects_missing_api_key() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let server = MockServer::start().await;
    let config = Config::new(
        "127.0.0.1:0",
        "unused",
        server.uri(),
        "zain-test-key",
        "gateway-test-key",
    );
    let zainpay = ZainpayClient::builder()
        .base_url(server.uri())
        .api_key("zain-test-key")
        .build()
        .unwrap();
    let app = build_app_with(config, pool, zainpay);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/virtual-account/create/request")
                .header("content-type", "application/json")
                .body(Body::from(sample_create_body().to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
