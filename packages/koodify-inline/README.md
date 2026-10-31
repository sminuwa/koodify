# @koodify/inline

Bank-transfer-only checkout widget. Opens a Koodify sheet and collects payment via a one-time virtual account.

**Hosted script (production):**

```html
<script src="https://js.koodify.ng/v1/koodify-inline.js"></script>
```

**HTML docs:** [docs/inline-checkout.html](../../docs/inline-checkout.html) — includes internal notes on **Listening…** polling (3s HTTP status checks, capacity table).

Prefer routing through the **koodify gateway** so payer contact is logged on init and returned as `data.payer` / callback `payer` on status.

## Checkout payload

```js
koodifyInitPayment(
  {
    // required
    amount: "500",                 // whole Naira preferred ("500" not "500.00")
    txnRef: "ORDER_" + Date.now(), // unique per attempt
    mobileNumber: "08012345678",
    publicKey: "pk_live_…",       // from seed/dashboard — maps to zainbox server-side
    emailAddress: "customer@example.com",
    callBackUrl: "https://example.com/webhook",

    // recommended (stored by koodify; not sent to Zainpay)
    name: "Ada Lovelace",
    // or: payer: { name: "Ada Lovelace" },

    // optional value-add — omit entirely if unused; any keys/values you want
    customParam: {
      matric: "ND/12/2211",
      session: "2026",
      paymentType: "School Fees",
      studentId: "STU001",
    },

    // optional — Zainbox Payment Split (gateway mode only)
    // paymentSplit: {
    //   shares: [
    //     { zainboxCode: "primary_box", percent: 70 },
    //     { zainboxCode: "tutor_box", amount: "2000" },
    //     { zainboxCode: "agent_box", amount: "1000" },
    //   ],
    // },
    // or: splitPlanId: "split_school_fees_v1",

    // optional
    currencyCode: "NGN",
    logoUrl: "https://example.com/logo.png",
    // bankType: omit on gateway — Koodify auto-picks; override only if needed
    allowRecurringPayment: false,

    // baseUrl / auth default from the CDN script — omit in production
    // baseUrl: "http://localhost:8080", // local override only
  },
  function (response) {
    // { status, reference, description, payer?, customParam? }
    // payer: { email, phone, name? }
    // customParam only present when you sent it
  }
);
```

| Field | Required | Notes |
|-------|----------|--------|
| `amount` | yes | Naira string |
| `txnRef` | yes | Unique; used as payment log key |
| `mobileNumber` | yes | → stored `payer.phone` |
| `emailAddress` | yes | → stored `payer.email` |
| `publicKey` | yes | Publishable `pk_live_…`; tied to zainbox in Koodify DB |
| `callBackUrl` | yes | Merchant webhook / return URL |
| `name` / `payer.name` | no | Koodify-only; merged into `payer` |
| `customParam` | **no** | Optional. Arbitrary merchant fields; omit if unused. Alias `customParams` |
| `paymentSplit` / `splitPlanId` | no | After deposit, Koodify redistributes to secondary zainboxes (gateway only). See `FEATURES.md` |
| `auth` | no | Defaults to `"gateway"`. Use `"zainpay"` only for direct Zainpay |
| `baseUrl` | no | Defaults to `https://api.koodify.ng`. Override for local/proxy only |

Checkout needs an active **`publicKey`**. Koodify resolves it to the merchant zainbox before calling Zainpay. Keep `GATEWAY_API_KEY` and Zainpay’s private key on the server only.

## Logging & merge

Every successful **init** through koodify writes a `transactions` row keyed by `txnRef` + `zainbox_code` (with `metadata.payer`, and `metadata.customParam` when provided). Status polls reload that row and attach `data.payer` / `data.customParam` when stored. The inline callback echoes `customParam` only if it was sent.

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
Paste a seeded zainbox code in the form (values stick in `localStorage`).

```bash
cd packages/koodify-inline
npm run build          # copies dist → live-test/koodify-inline.js
npm run test:live      # optional static server
# open http://localhost:5180
# or open live-test/index.html after build
```
