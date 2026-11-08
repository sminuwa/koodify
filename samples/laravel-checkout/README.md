# Koodify · Laravel app

Two portals in one Laravel sample, both calling **Koodify APIs**:

| Portal | URL | Who |
|--------|-----|-----|
| **Merchant** | `/login` · `/register` | Businesses (session `mt_…`) |
| **Admin** | `/admin/auth` | Platform ops (`at_…` from gateway `users`) — same UI shell as merchant |

UI for admin comes from the Desktop `koodify-admin-portal` prototype (`public/admin/css`).

## Merchant flow

1. Register / login → `POST /api/v1/auth/register` · `login`
2. Dashboard → `GET /api/v1/auth/me`
3. KYC / compliance → merchant portal
4. Credentials → `POST /api/v1/auth/credentials/{env}/issue-secret`
5. Samples: checkout, VA, balance, webhooks

## Admin flow

1. Sign in at `/admin/auth` with gateway admin username/password (`users` table; bootstrap via `ADMIN_USERNAME` / `ADMIN_PASSWORD` on the gateway when empty)
2. Overview — pending KYC queue from `GET /api/v1/admin/dashboard`
3. Merchants — list / per-section compliance approve / **Approve KYC & provision** (`POST …/kyc/approve` emails merchant + provisions zainbox + `pk_…`)
4. Merchant mints `sk_…` from Credentials after approval
5. Other sidebar sections are UI placeholders for now

## Setup

```bash
cd samples/laravel-checkout
cp .env.example .env
php artisan key:generate
```

```env
KOODIFY_API_BASE=http://127.0.0.1:8080

# Admin portal login hints (auth is against the gateway)
KOODIFY_ADMIN_USERNAME=admin
KOODIFY_ADMIN_PASSWORD=Admin123!

# Optional X-Api-Key fallback
# KOODIFY_GATEWAY_API_KEY=your_gateway_api_key
```

On the gateway:

```env
ADMIN_USERNAME=admin
ADMIN_PASSWORD=Admin123!
# Optional local KYC auto-approve for merchants:
# MERCHANT_KYC_AUTO_APPROVE=true
```

```bash
cargo run -p koodify          # Terminal A
php artisan serve --port=8001 # Terminal B
```

- Merchant: http://127.0.0.1:8001/register  
- Admin: http://127.0.0.1:8001/admin/auth  

## Notes

- Never put `sk_…` or `GATEWAY_API_KEY` in the browser.
- Admin portal sessions use Bearer `at_…` from `POST /api/v1/admin/auth/login`.
- KYC approve provisions the zainbox and publishable key; secrets stay merchant self-serve.
