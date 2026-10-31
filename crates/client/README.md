# client

Reusable async Rust HTTP client for the [Zainpay](https://docs.zainpay.ng/) payment API.

This crate talks to Zainpay only — it does **not** store custom metadata. Use
`koodify` when you need to persist extra fields alongside Zainpay responses.

## Example

```rust
use client::{CreateVirtualAccountRequest, Environment, ZainpayClient};

# async fn example() -> Result<(), client::ZainpayError> {
let client = ZainpayClient::new(Environment::Sandbox, "your-private-key");

let response = client
    .virtual_accounts()
    .create(&CreateVirtualAccountRequest {
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
    })
    .await?;

assert_eq!(response.code, "00");
# Ok(())
# }
```

## Composite helpers

Call two Zainpay endpoints as one envelope via `client.composite()` (mirrored on koodify as plain `/api/v1/…` routes). Full gateway docs: [`docs/api.html`](../../docs/api.html). Composite deep-dive: [`docs/composite-endpoints.html`](../../docs/composite-endpoints.html).

```rust
# async fn example(client: &client::ZainpayClient) -> Result<(), client::ZainpayError> {
let snap = client.composite().zainbox_snapshot("charg_0atrmzL5qsMFKGYPrahj").await?;
assert_eq!(snap.code, "00");
# let _ = snap;
# Ok(())
# }
```

## Webhook signatures

```rust
use client::{verify_webhook_signature, WebhookEvent};

# fn example(raw_body: &[u8], signature: &str, secret: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
assert!(verify_webhook_signature(secret, raw_body, signature));
let event: WebhookEvent = serde_json::from_slice(raw_body)?;
# let _ = event;
# Ok(())
# }
```
