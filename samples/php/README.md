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

## Log

Each POST appends one line to `webhook-log.jsonl`:

```json
{
  "receivedAt": "2026-11-01T22:00:00+00:00",
  "reference": "LIVE_…",
  "event": "payment.success",
  "payload": { "event": "payment.success", "amount": "500", "payer": {}, "customParam": {} }
}
```

```bash
tail -f /path/to/demo.koodify.ng/webhooks/koodify/webhook-log.jsonl | jq .
```

No webhook secret required for this demo — it only receipts and logs the transaction object Koodify posts.
