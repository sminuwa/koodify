# Koodify · Laravel app

Two portals in one Laravel sample, both calling **Koodify APIs**:

| Portal | URL | Who |
|--------|-----|-----|
| **Merchant** | `/login` · `/register` | Businesses (session `mt_…`) |
| **Admin** | `/admin/login` | Platform ops (`GATEWAY_API_KEY`) |

UI for admin comes from the Desktop `koodify-admin-portal` prototype (`public/admin/css`).

## Merchant flow

1. Register / login → `POST /api/v1/auth/register` · `login`
2. Dashboard → `GET /api/v1/auth/me`
3. KYC → `PUT /api/v1/auth/kyc`
4. Credentials → `POST /api/v1/auth/credentials/{env}/issue-secret`
5. Samples: checkout, VA, balance, webhooks

## Admin flow

1. Sign in at `/admin/login` with `KOODIFY_ADMIN_EMAIL` / `KOODIFY_ADMIN_PASSWORD`
2. Overview — merchant counts from `GET /api/v1/customers`
3. Merchants — list / review / **approve KYC** (`PATCH /kyc/{id}/status`) / retry provision
4. Other sidebar sections are UI placeholders for now

## Setup

```bash
cd samples/laravel-checkout
cp .env.example .env
php artisan key:generate
```

```env
KOODIFY_API_BASE=http://127.0.0.1:8080

# Admin portal
KOODIFY_GATEWAY_API_KEY=your_gateway_api_key
KOODIFY_ADMIN_EMAIL=admin@koodify.ng
KOODIFY_ADMIN_PASSWORD=Admin123!
```

On the gateway (optional local KYC auto-approve for merchants):

```env
MERCHANT_KYC_AUTO_APPROVE=true
```

```bash
cargo run -p koodify          # Terminal A
php artisan serve --port=8001 # Terminal B
```

- Merchant: http://127.0.0.1:8001/register  
- Admin: http://127.0.0.1:8001/admin/login  

## Notes

- Never put `sk_…` or `GATEWAY_API_KEY` in the browser.
- Admin login is app-local (.env); API calls use the gateway key as `X-Api-Key`.
- Merchant passwords live on Koodify (`customers.password_hash`).
