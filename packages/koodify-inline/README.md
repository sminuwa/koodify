# @koodify/inline

Bank-transfer-only checkout widget. Opens a Koodify sheet and collects payment via a one-time virtual account.

**HTML docs:** [docs/inline-checkout.html](../../docs/inline-checkout.html)

Prefer routing through the **koodify gateway** so payer contact is logged on init and returned as `data.payer` / callback `payer` on status.

## Checkout payload

```js
koodifyInitPayment(
  {
    // required
    amount: "500",                 // whole Naira preferred ("500" not "500.00")
    txnRef: "ORDER_" + Date.now(), // unique per attempt
    mobileNumber: "08012345678",
    zainboxCode: "YOUR_ZAINBOX_CODE",
    emailAddress: "customer@example.com",
    callBackUrl: "https://example.com/webhook",

    // recommended (stored by koodify; not sent to Zainpay)
    name: "Ada Lovelace",
    // or: payer: { name: "Ada Lovelace" },

    // optional merchant key/values — stored by koodify, echoed on callback
    customParam: {
      matric: "ND/12/2211",
      session: "2026",
      paymentType: "School Fees",
      studentId: "STU001",
    },

    // optional
    currencyCode: "NGN",
    logoUrl: "https://example.com/logo.png",
    bankType: "gtBank",
    allowRecurringPayment: false,

    // talk to koodify (not Zainpay directly)
    auth: "gateway",
    baseUrl: "https://api.koodify.ng", // local: "http://localhost:8080" or "/proxy/koodify"
  },
  function (response) {
    // { status, reference, description, payer?, customParam? }
    // payer: { email, phone, name? }
    // customParam: { matric, session, paymentType, studentId, … }
  },
  "YOUR_GATEWAY_PUBLIC_KEY", // publishable X-Api-Key (safe in browser)
  "live"
);
```

| Field | Required | Notes |
|-------|----------|--------|
| `amount` | yes | Naira string |
| `txnRef` | yes | Unique; used as payment log key |
| `mobileNumber` | yes | → stored `payer.phone` |
| `emailAddress` | yes | → stored `payer.email` |
| `zainboxCode` | yes | Merchant box (seeded; treat as the merchant private id) |
| `callBackUrl` | yes | Merchant webhook / return URL |
| `name` / `payer.name` | no | Koodify-only; merged into `payer` |
| `customParam` | no | Arbitrary key/values; stored + returned on callback (`customParams` alias) |
| `auth` | no | `"gateway"` / `"koodify"` for hosted API |
| `baseUrl` | no | Koodify origin — production `https://api.koodify.ng` |

Checkout needs **both**:
- **Public key** — `GATEWAY_PUBLIC_KEY` (falls back to `GATEWAY_API_KEY` if unset); sent as `X-Api-Key`
- **zainboxCode** — which merchant box (must be seeded + active)

Keep `GATEWAY_API_KEY` secret for admin routes. Zainpay’s private key never leaves the server.

## Logging & merge

Yes — every successful **init** through koodify writes a `transactions` row keyed by `txnRef` + `zainbox_code` (including `metadata.payer` and `metadata.customParam`). Status polls then reload that row and attach `data.payer` / `data.customParam`. The inline callback always echoes `customParam` from the checkout config as well.

## Build / demo

```bash
# terminal 1 — gateway
cargo run -p koodify

# terminal 2 — local checkout demo (proxies to local gateway)
cd packages/koodify-inline
npm install
npm run demo
# http://localhost:5179  →  /proxy/koodify  →  :8080
```

## Live test (hosted API)

Pure HTML/JS harness under `live-test/` against **https://api.koodify.ng**.
Paste `GATEWAY_API_KEY` and zainbox code in the form (values stick in `localStorage`).

```bash
cd packages/koodify-inline
npm run build          # copies dist → live-test/koodify-inline.js
npm run test:live      # optional static server
# open http://localhost:5180
# or open live-test/index.html after build
```
