# Koodify merchant webhook (Node)

```bash
cd samples/node/webhook/koodify
KOODIFY_SECRET_KEY="sk_live_…" npm start
```

Verifies `X-Koodify-Signature` with the same merchant secret used as `X-Api-Key`.

```bash
BODY='{"event":"payment.success","reference":"ORDER_1","status":"success"}'
SIG=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$KOODIFY_SECRET_KEY" | awk '{print $2}')
curl -sS -X POST "http://127.0.0.1:8787/webhooks/koodify" \
  -H "Content-Type: application/json" \
  -H "X-Koodify-Event: payment.success" \
  -H "X-Koodify-Signature: $SIG" \
  --data-binary "$BODY"
```

Secret: merchant `sk_…`. If unset, the demo logs a warning and skips the check (local only).
