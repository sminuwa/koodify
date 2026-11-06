# Koodify merchant webhook (Node.js)

Minimal Express receiver for checkout `callBackUrl`. Verifies `X-Koodify-Signature` on the **raw body bytes**, then appends the event to a log file.

```text
samples/node/webhook/koodify/
→ http://localhost:3091/webhooks/koodify/
```

## Why raw body matters

Koodify signs the exact bytes it POSTs. If you parse JSON first and then run:

```js
// WRONG — do not do this
crypto.createHmac("sha256", secret).update(JSON.stringify(req.body))
```

…verification will fail whenever key order or whitespace differs. Always HMAC the original buffer (e.g. `express.raw()`), then `JSON.parse`.

## Run

```bash
cd samples/node/webhook/koodify
npm install
MERCHANT_WEBHOOK_SECRET="your_secret" npm start
```

Checkout:

```js
callBackUrl: "https://school.example/webhooks/koodify"
// Koodify normalizes directory URLs to end with `/` before POSTing.
```

Local test with a known body:

```bash
BODY='{"event":"payment.success","status":"success","reference":"T1","description":"ok","currencyCode":"NGN"}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$MERCHANT_WEBHOOK_SECRET" | awk '{print $2}')
curl -sS -X POST "http://localhost:3091/webhooks/koodify/" \
  -H "Content-Type: application/json" \
  -H "X-Koodify-Event: payment.success" \
  -H "X-Koodify-Signature: $SIG" \
  --data-binary "$BODY"
```

## Files

| File | Role |
|------|------|
| `verify.js` | `verifyKoodifySignature(secret, rawBody, header)` |
| `server.js` | Express app using `express.raw()` |
| `verify.test.js` | `npm test` — proves raw vs re-serialized HMAC |

Secret: `MERCHANT_WEBHOOK_SECRET`, else `GATEWAY_API_KEY`. If unset, the demo logs a warning and skips the check (local only).

PHP twin: `samples/php/webhook/koodify/`. Full docs: `docs/inline-checkout.html#merchant-webhook`.
