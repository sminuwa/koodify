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
    baseUrl: "http://localhost:8080", // or "/proxy/koodify" in the demo
  },
  function (response) {
    // { status, reference, description, payer?, customParam? }
    // payer: { email, phone, name? }
    // customParam: { matric, session, paymentType, studentId, … }
  },
  "YOUR_GATEWAY_API_KEY", // X-Api-Key — same as GATEWAY_API_KEY
  "sandbox"               // ignored when baseUrl overrides host
);
```

| Field | Required | Notes |
|-------|----------|--------|
| `amount` | yes | Naira string |
| `txnRef` | yes | Unique; used as payment log key |
| `mobileNumber` | yes | → stored `payer.phone` |
| `emailAddress` | yes | → stored `payer.email` |
| `zainboxCode` | yes | Merchant zainbox |
| `callBackUrl` | yes | Merchant webhook / return URL |
| `name` / `payer.name` | no | Koodify-only; merged into `payer` |
| `customParam` | no | Arbitrary key/values; stored + returned on callback (`customParams` alias) |
| `auth` | no | `"gateway"` → send `X-Api-Key` |
| `baseUrl` | no | Koodify origin (enables payer logging) |

## Logging & merge

Yes — every successful **init** through koodify writes an `operation_records` row keyed by `txnRef` (including `metadata.payer` and `metadata.customParam`). Status polls then reload that row and attach `data.payer` / `data.customParam`. The inline callback always echoes `customParam` from the checkout config as well.

## Build / demo

```bash
# terminal 1 — gateway
cargo run -p koodify

# terminal 2 — checkout demo
cd packages/koodify-inline
npm install
npm run demo
# http://localhost:5179  →  /proxy/koodify  →  :8080
```
