# zainpay-rs

Reusable Rust integration for [Zainpay](https://docs.zainpay.ng/):

| Crate | Role |
|-------|------|
| `client` | Pure HTTP SDK covering all documented merchant API endpoints |
| `koodify` | Axum service that mirrors those paths, accepts custom `metadata`, persists it in Postgres |

## Modules (client)

- `zainbox` — create/list/update, accounts, balances, txns, profile, billing, subscriptions
- `virtual_accounts` — static/dynamic VA, balance, status/BVN, txns, sandbox funding
- `bank` — bank list, name enquiry
- `transfers` — funds transfer, verify, network status
- `settlements` — create/get/history, internal settlement account
- `reconciliation` — deposit verify, bank/card reconcile
- `cards` — initialize payment, recurring purchase
- `webhooks` — repush deposit / card-payment
- `composite` — convenience helpers that call **two** Zainpay endpoints as one response (exposed on koodify as plain `/api/v1/…` routes)

**Live API:** [https://api.koodify.ng](https://api.koodify.ng)

**API docs (HTML + Tailwind):** [docs/api.html](docs/api.html) — full gateway reference. Composite deep-dive: [docs/composite-endpoints.html](docs/composite-endpoints.html). Inline checkout: [docs/inline-checkout.html](docs/inline-checkout.html).

**Browser checkout (bank transfer):** [packages/koodify-inline](packages/koodify-inline) — `koodifyInitPayment` InlineJS-style widget (no cards). Hosted script: `https://js.koodify.ng/v1/koodify-inline.js` (defaults to `https://api.koodify.ng`). Merchants pass a publishable `publicKey` (`pk_live_…`); Koodify maps it to the zainbox. Docs: [docs/inline-checkout.html](docs/inline-checkout.html).

## Quick start

```bash
cp .env.example .env
# edit DATABASE_URL, ZAINPAY_API_KEY, GATEWAY_API_KEY

cargo run -p koodify
```

### Payer details on payments

Zainpay does not echo payer contact on status. On payment init (card / transfers / VA mutates via the gateway), koodify stores:

```json
{ "email": "…", "phone": "…", "name": "…" }
```

under `transactions` for that `txnRef` (with `zainbox_code` as business key when present), and returns it as `data.payer` on the init response and on later status lookups (`reconcile` card payment, deposit verify, dynamic deposit status, transfer verify, `GET /api/v1/deposit/{txnRef}`).

`email` / `phone` come from `emailAddress` / `mobileNumber`. Optional `name` (or `payer.name`) is koodify-only and is stripped before calling Zainpay.

### Metadata on mutating calls

Nested:

```json
{
  "zainpay": { "...": "official Zainpay fields" },
  "metadata": { "customerId": "cus_123" }
}
```

Or attach a `metadata` object next to the official fields. Custom fields are stored and returned in `data.metadata`; they are never forwarded to Zainpay.

Static VA create also accepts **flat** unknown keys (e.g. `customerId` at the top level).

### Local extras

- `GET /virtual-accounts/{accountNumber}` — DB lookup with metadata
- `GET /operations/{resourceKey}` / `GET /transactions/{resourceKey}` — transaction history by txnRef / accountNumber (`?zainboxCode=` optional)
- `GET /transactions/by-zainbox/{zainboxCode}` — transactions for a zainbox business key
- `POST /customers` / `GET /customers` — customer registry (`uuid` + integer `id` 1, 2, 3, …; optional `zainboxCode`)
- Seed (`cargo run -p koodify --example seed`) syncs live Zainpay zainboxes into `zainboxes`, creates one customer per box, and sets credential `privateKey` = zainbox `codeName`
- `PUT /kyc` — upsert KYC linked by integer `customerId` → `customers.id`
- `GET /kyc/by-customer/{customerId}` — fetch KYC for a customer
- `PUT /credentials` — upsert keys (`customerId`, `environment`: test|live, `publicKey`, `privateKey`)
- `GET /credentials/by-customer/{customerId}` — that customer's test + live credentials
- `POST /webhooks/zainpay` — inbound Zainpay webhook (HMAC `Zainpay-Signature`; set `ZAINPAY_WEBHOOK_SECRET`)
- Plain `/api/v1` convenience routes — e.g. `GET /api/v1/zainbox/{zainboxCode}`, `GET /api/v1/virtual-account/{accountNumber}`, `POST /api/v1/transfer`

Canonical bank/transfer paths match live API (`/bank/list`, `/bank/name-enquiry`, `/bank/transfer/v2`, `/virtual-account/wallet/balance/v2/...`). Legacy `/zainbox/bank/...` aliases remain on the gateway.

## Migrate & seed

```bash
# apply migrations
DATABASE_URL=postgres://postgres:postgres@localhost:5432/koodify \
  cargo run -p koodify --example migrate

# idempotent demo customers + KYC + test/live credentials
DATABASE_URL=postgres://postgres:postgres@localhost:5432/koodify \
  cargo run -p koodify --example seed
```

## Tests

```bash
cargo test -p client
cargo test -p koodify --lib

DATABASE_URL=postgres://sminuwa@localhost:5432/koodify \
  cargo test -p koodify --test gateway_va
```
