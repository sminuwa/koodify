# Koodify merchant webhook (PHP demo)

Minimal receiver for checkout `callBackUrl`: accept Koodify’s `payment.success` JSON and append it to a log file.

```text
samples/php/webhook/koodify/index.php
→ https://demo.koodify.ng/webhooks/koodify/
```

## Deploy

```bash
mkdir -p /path/to/demo.koodify.ng/webhooks/koodify
cp samples/php/webhook/koodify/index.php \
   /path/to/demo.koodify.ng/webhooks/koodify/index.php
chmod 775 /path/to/demo.koodify.ng/webhooks/koodify
```

Checkout:

```js
callBackUrl: "https://demo.koodify.ng/webhooks/koodify"
// Koodify normalizes directory URLs to end with `/` before POSTing.
```

## Koodify POST body

Headers: `Content-Type: application/json`, `X-Koodify-Event: payment.success`, `X-Koodify-Signature` (HMAC-SHA256 hex).

```json
{
  "event": "payment.success",
  "status": "success",
  "reference": "LIVE_…",
  "description": "Payment was successful",
  "amount": "500",
  "currencyCode": "NGN",
  "zainboxCode": "main_box",
  "payer": {
    "email": "customer@example.com",
    "phone": "08012345678",
    "name": "Ada Lovelace"
  },
  "customParam": {
    "matric": "ND/12/2211",
    "session": "2026"
  },
  "paymentSplit": {
    "txnRef": "LIVE_…",
    "status": "success",
    "legs": []
  }
}
```

`amount` / `zainboxCode` / `payer` / `customParam` / `paymentSplit` appear only when Koodify has them. Full docs: `docs/inline-checkout.html#merchant-webhook`.

## Log

Each POST appends one line to `webhook-log.jsonl`:

```json
{
  "receivedAt": "2026-11-01T22:00:00+00:00",
  "reference": "LIVE_…",
  "event": "payment.success",
  "payload": { "…full body above…" }
}
```

```bash
tail -f /path/to/demo.koodify.ng/webhooks/koodify/webhook-log.jsonl | jq .
```

No webhook secret required for this demo — it only receipts and logs the transaction object Koodify posts.

For signature verification on **raw body** bytes (including the Node anti-`JSON.stringify` pattern), see `samples/node/webhook/koodify/`.
