# Koodify · Laravel sample

Small Laravel app with:

1. **Inline checkout** — browser `koodifyInitPayment` + webhook receiver  
2. **Virtual account create** — server `POST /api/v1/va/create` with `sk_…`

Merchants only need two keys:

| Key | Where | Use |
|-----|--------|-----|
| `pk_…` | Browser | Inline checkout |
| `sk_…` | Server | API calls (`X-Api-Key`) **and** webhook HMAC (`X-Koodify-Signature`) |

## Routes

| Route | Purpose |
|-------|---------|
| `GET /` | Inline checkout page |
| `GET /virtual-accounts/create` | Form to create a static VA |
| `POST /virtual-accounts` | Calls Koodify VA create |
| `POST /webhooks/koodify` | Merchant webhook (CSRF exempt; verify with `sk_…`) |

## Setup

```bash
cd samples/laravel-checkout
cp .env.example .env
php artisan key:generate
```

```env
KOODIFY_PUBLIC_KEY=pk_live_…
KOODIFY_SECRET_KEY=sk_live_…
```

```bash
php artisan serve
```

- Checkout: `http://127.0.0.1:8000/`  
- VA create: `http://127.0.0.1:8000/virtual-accounts/create`

## Webhooks locally

Tunnel the app, then set:

```env
APP_URL=https://YOUR_SUBDOMAIN.ngrok-free.app
KOODIFY_CALLBACK_URL=https://YOUR_SUBDOMAIN.ngrok-free.app/webhooks/koodify
```

Verify signatures with the same `sk_…`:

```php
hash_hmac('sha256', $rawBody, $secretKey) === $signatureHeader
```

```bash
tail -f storage/logs/laravel.log
```

## Notes

- Never put `sk_…` in the browser.
- VA create omits BVN / DOB / gender — Koodify fills `VA_DEFAULT_*` before calling Zainpay.
- Docs: https://developer.koodify.ng
