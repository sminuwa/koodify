# Koodify features

What Koodify adds on top of Zainpay — written so a non-engineer can skim it.
Each feature has **what it is**, a short **example**, a simple **process flow**, and (where useful) a **code** snippet for implementers.
Technical how-tos live under `docs/`.

---

## Checkout

### Inline bank-transfer checkout
**What it is:** A ready-made payment popup for websites. The customer sees account details, pays by bank transfer, and the page learns when money arrives.

**Example:** A school website shows “Pay fees”. Parent clicks it, sees a temporary account, transfers ₦50,000, and the site confirms payment without staff checking a bank statement.

**Process flow:**
1. Merchant site loads the Koodify script and calls `koodifyInitPayment` with amount, reference, contact details, and public key.
2. Popup opens → Koodify starts a payment session with Zainpay.
3. Koodify creates a one-time virtual account and shows bank + account number + amount.
4. Popup listens (about every 3 seconds) for the transfer; customer can also tap “Check status”.
5. When paid → short success screen → callback returns result to the merchant site.

```html
<script src="https://js.koodify.ng/v1/koodify-inline.js"></script>
<script>
  koodifyInitPayment(
    {
      publicKey: "pk_live_…",
      amount: "50000",
      txnRef: "FEES_" + Date.now(),
      mobileNumber: "08012345678",
      emailAddress: "parent@example.com",
      callBackUrl: "https://school.example/paid",
      name: "Ada Lovelace",
    },
    function (response) {
      // { status, reference, description, payer?, customParam? }
      console.log(response.status, response.reference);
    }
  );
</script>
```

Docs: [docs/inline-checkout.html](docs/inline-checkout.html) · Script: `https://js.koodify.ng/v1/koodify-inline.js`

### Publishable public keys
**What it is:** A safe “shop ID” for the browser (`pk_live_…`). It tells Koodify *which business* is collecting money, without exposing secret API keys on the website. Source of truth is `zainboxes.public_key` (credentials mirror it for dashboards).

**Example:** Like a storefront address on a map — public enough to find the shop, not the keys to the safe.

**Process flow:**
1. Seed, KYC approval provision, or admin assigns a `pk_live_…` to a zainbox in Koodify’s database.
2. Merchant puts that key in their website checkout config (same as demo.koodify.ng inline samples).
3. On pay, the popup sends the public key to Koodify.
4. Koodify looks up the matching zainbox and uses its secret Zainpay identity (`zainbox_code`) server-side.
5. Unknown or inactive keys are rejected.

### Open checkout routes
**What it is:** The payment popup can start a payment and check status without the merchant’s secret key. Secret keys are only for admin-style actions.

**Example:** The school website opens checkout with just its public key. Changing KYC still needs the secret admin key.

**Process flow:**
1. Browser calls open routes: initialize → create temporary VA → poll status.
2. Identity comes from `publicKey` (then session key / `txnRef`).
3. Admin routes (customers, KYC, credentials, most mirror APIs) require `GATEWAY_API_KEY`.
4. Secret key never needs to sit in the website’s JavaScript.

### Adaptive Bank/DVA Routing
**What it is:** Koodify chooses *which bank* issues the temporary / dynamic virtual account (DVA). Merchants don’t hard-code GTBank vs Moniepoint. If one bank struggles, Koodify switches.

**Example:** GTBank VA creation lags at lunch → new checkouts briefly get Moniepoint accounts → GTBank recovers later.

**Process flow:**
1. Checkout omits `bankType` (recommended).
2. Gateway picks the current preferred bank from `VA_BANK_CANDIDATES` (default when unset: `gtBank` only).
3. Tries to create the temporary account with that bank.
4. On failure → tries the next candidate; records success/speed for each attempt.
5. Preferred bank sticks for a while (`VA_BANK_STICKY_SECS`), then can be re-scored.
6. If merchant *does* pass `bankType`, that bank is used once (no auto-switch) — override path.
7. At startup, keys in `VA_BANK_CANDIDATES` that are not on Koodify’s known-good list (`gtBank`, `wemaBank`, `polaris`, `fcmb`, `zainMFB`, `moniepoint`) log a warning only — they are still tried.

```env
VA_BANK_CANDIDATES=gtBank,moniepoint
VA_BANK_STICKY_SECS=600
```

**Planned (Zainpay):** When Zainpay exposes an “available temporary/dynamic VA banks” API, the `client` crate will fetch/cache that list. Routing will validate (and preferably populate) candidates from it; `VA_BANK_CANDIDATES` becomes preference order / override, not the catalog. Until then env + the local known list are the stopgap (`bank_router::warn_if_experimental`).

### Gateway vs direct Zainpay mode
**What it is:** By default the popup talks to Koodify (recommended). Advanced users can point it straight at Zainpay.

**Example:** Most merchants use Koodify as a smart middleman; a developer can flip to Zainpay-only for raw testing.

**Process flow (gateway — default):**
1. Popup → `api.koodify.ng` with public key.
2. Koodify handles routing, storage, receipts, bank choice.

**Process flow (direct):**
1. Config sets `auth: "zainpay"` + merchant API key.
2. Popup → Zainpay hosts directly.
3. Merchant must supply a bank type (defaults to `gtBank`); Koodify extras (receipts, smart routing) are skipped.

```js
// Default — omit auth / baseUrl in production
koodifyInitPayment({ publicKey: "pk_live_…", /* … */ }, cb);

// Direct Zainpay (no Koodify extras)
koodifyInitPayment({
  auth: "zainpay",
  publicKey: "/* Zainpay merchant key */",
  bankType: "gtBank",
  /* … */
}, cb);
```

### Checkout UX extras
**What it is:** Polish around the popup: logo, reused-reference errors, short success screen.

**Example:** After a successful transfer, parent sees confirmation for a few seconds, then returns to the school site.

**Process flow:**
1. Popup renders (optional custom `logoUrl`).
2. If `txnRef` was already used → clear error, ask for a fresh reference.
3. On success → confirmation UI (~10 seconds) → close → fire merchant callback.

---

## Payments intelligence

### Payer enrichment
**What it is:** Zainpay often doesn’t echo email/phone later. Koodify remembers them at start and re-attaches them on status checks.

**Example:** Checkout collects `parent@email.com` and `080…`. Later “paid” responses still include who paid.

**Process flow:**
1. Initialize payment includes email / phone / optional name.
2. Koodify stores payer under that `txnRef` (and zainbox when known).
3. Name (if any) is stripped before talking to Zainpay.
4. Later status / reconcile / deposit lookup → Koodify merges `payer` back into the response.
5. Receipts and callbacks can use that same payer info.

### Custom params
**What it is:** Extra labels on a payment (student ID, order ID, class). Stored and returned by Koodify; Zainpay never sees them.

**Example:** School sends `{ "matric": "ND/12/2211", "session": "2026" }` and gets them back on success.

**Process flow:**
1. Merchant includes `customParam` / `customParams` in checkout or init.
2. Koodify stores the object with the transaction.
3. Fields are not forwarded to Zainpay.
4. Status, callback, and emails can include the same object again.

```js
koodifyInitPayment({
  /* required fields… */
  customParam: {
    matric: "ND/12/2211",
    session: "2026",
    paymentType: "School Fees",
  },
}, function (response) {
  // echoed on success when you sent it
  console.log(response.customParam);
});
```

### Metadata on mutating calls
**What it is:** General notes on API actions (customer IDs, store IDs). Saved by Koodify, not sent to Zainpay.

**Example:** Create a VA tagged `{ "customerId": "cus_123" }`; later lookup still shows the tag.

**Process flow:**
1. Client sends official Zainpay fields plus `metadata` (or flat extra keys on some creates).
2. Koodify splits “Zainpay fields” vs “your notes”.
3. Only Zainpay fields go upstream.
4. Response comes back with your notes in `data.metadata`.
5. Local DB keeps them for later lookups.

### Transaction logging
**What it is:** A diary of payment-related API calls keyed by references like `txnRef`.

**Example:** Support asks about `ORDER_889` → you see initialize, VA create, and status checks.

**Process flow:**
1. A mutating call hits the gateway.
2. Koodify records operation name, resource key, request/response, zainbox when known.
3. Enrichment (payer, customParam, email-sent flags) lives in the same log.
4. Ops queries by reference or zainbox to reconstruct the story.

### Activity events (trigger audit)
**What it is:** Append-only timeline that records *who* fired payment success — Zainpay webhook (Path 1), status poll, or reconciler (Path 2). Use this to confirm whether Zainpay is delivering webhooks.

**Example:** Email arrived ~60s after pay with no `inbound.webhook.received` row → Path 2 only; Zainpay never knocked.

**Process flow:**
1. Inbound Zainpay POST → `inbound.webhook.received` (or `duplicate` / `unconfirmed`).
2. Success side effects → `payment.success.side_effects` with `source`: `zainpay_webhook` | `status_poll` | `reconciler`.
3. Outbound merchant/customer POST → `merchant.webhook.attempt` then `delivered` / `failed` / `skipped` (URL + HTTP status in payload).
4. Look up: `GET /api/v1/activity/{txnRef}` (secret `X-Api-Key`).

```bash
curl "https://api.koodify.ng/api/v1/activity/LIVE_your_txn" \
  -H "X-Api-Key: $GATEWAY_API_KEY"
```

**DB:** migration `017_activity_events.sql`.

### Zainbox Payment Split
**What it is:** After the customer pays the full amount into one **primary** zainbox, Koodify redistributes value to **secondary** zainboxes. Merchants declare *zainbox codes + shares* (not external bank accounts). Moves are **internal** VA/ISA → ISA (typically free on Zainpay).

**Example:** Checkout for ₦1,000 with shares `₦200` + `₦50` + `percent: 100` on primary → primary keeps ₦750; the other boxes receive ₦200 and ₦50 after payment succeeds.

**Process flow:**
1. Create a reusable plan under `ApiKeyAuth` (`PUT /api/v1/split-plans`), **or** send ad-hoc `paymentSplit` on the authenticated initialize route. Browser checkout (`koodifyInitPayment` / open inline) accepts **`splitPlanId` only**.
2. Koodify validates the plan (active zainboxes, math) and stores a pending run for that `txnRef` (init fails closed if persist fails).
3. Customer pays once into the primary box’s temporary account (same as normal checkout). Open checkout requires `publicKey`.
4. On explicit payment success (status string `success`/`paid`/… — not API `code=00` alone) via webhook (preferred) or status poll → Koodify claims the run once, pins the deposit source account, then transfers each secondary leg with a stable idempotent `transfer_txn_ref`.
5. Primary keep needs no transfer. Status / admin APIs show `data.paymentSplit` with per-leg results.
6. Ops can force a pending/failed/partial run: `POST /api/v1/payment-splits/{txnRef}` (secret `X-Api-Key`). Force-run may use a funded-VA fallback when source was never recorded. A `running` claim expires after **10 minutes** (`claimed_at` lease) so a crashed worker can be reclaimed automatically or via force-run.

**Share rules:**
| Field | Meaning |
|-------|---------|
| `amount` | Fixed Naira — taken **first** from the deposit |
| `percent` | Share of **whatever remains** after fixed amounts (must total **100%** of that remainder). Use `percent: 100` on primary to keep the rest |
| Either `amount` **or** `percent` per leg — not both |

**Requirements / limits:**
- Gateway mode only (`auth: "gateway"`). Direct Zainpay popup does not run splits.
- **Primary (collector) zainbox must have auto settlement OFF.** If settlement is enabled, Zainpay returns `code=04` (*Funds Transfer is Disabled for Auto Settlement Enabled Zainbox*). Disable via `POST /zainbox/settlement` with `"status": false` (boolean). Secondaries may keep T1/T7 settlement.
- Recommended layout: collector box (settlement off) → split → secondary boxes (settlement on if desired).
- All `zainboxCode`s must exist and be **active** in Koodify’s zainboxes cache (seed/sync).
- Max 10 shares; no duplicate destinations; invalid plan → checkout init rejected.
- Deposit fees (if any) are not auto-deducted from secondary legs; primary absorbs fee variance in v1 (underpayment > ₦10 vs checkout refuses the split).
- Auto-run requires a pinned deposit source (webhook/status `accountNumber`); it will not debit an arbitrary funded VA on the primary box.

#### Code — authenticated ad-hoc shares (ApiKeyAuth)

```bash
curl -X POST "https://api.koodify.ng/zainbox/card/initialize/payment" \
  -H "Content-Type: application/json" \
  -H "X-Api-Key: $GATEWAY_API_KEY" \
  -d '{
    "zainboxCode": "main_box",
    "amount": "1000",
    "txnRef": "ORDER_889",
    "mobileNumber": "08012345678",
    "emailAddress": "payer@example.com",
    "callBackUrl": "https://merchant.example/paid",
    "paymentSplit": {
      "shares": [
        { "zainboxCode": "tutor_box", "amount": "200" },
        { "zainboxCode": "agent_box", "amount": "50" },
        { "zainboxCode": "main_box", "percent": 100 }
      ]
    }
  }'
```

Percent-only example (70 / 20 / 10 of the full deposit):

```json
"paymentSplit": {
  "shares": [
    { "zainboxCode": "main_box", "percent": 70 },
    { "zainboxCode": "tutor_box", "percent": 20 },
    { "zainboxCode": "agent_box", "percent": 10 }
  ]
}
```

#### Code — disable auto-settlement on the collector

```bash
curl -X POST "https://api.koodify.ng/zainbox/settlement" \
  -H "Content-Type: application/json" \
  -H "X-Api-Key: $GATEWAY_API_KEY" \
  -d '{
    "name": "Collector — settlement off",
    "scheduleType": "T1",
    "schedulePeriod": "1",
    "zainboxCode": "main_box",
    "status": false,
    "settlementAccountList": [
      {
        "bankCode": "000",
        "accountNumber": "0123456789",
        "percentage": "100"
      }
    ]
  }'
```

(`status` must be the JSON boolean `false`, not the string `"false"`.)

#### Code — reusable plan + checkout by id

```bash
curl -X PUT "https://api.koodify.ng/api/v1/split-plans" \
  -H "Content-Type: application/json" \
  -H "X-Api-Key: $GATEWAY_API_KEY" \
  -d '{
    "id": "split_school_fees_v1",
    "name": "School fees — tutor + agent",
    "primaryZainboxCode": "main_box",
    "isActive": true,
    "shares": [
      { "zainboxCode": "tutor_box", "amount": "200" },
      { "zainboxCode": "agent_box", "amount": "50" },
      { "zainboxCode": "main_box", "percent": 100 }
    ]
  }'
```

```js
koodifyInitPayment({
  publicKey: "pk_live_…",
  amount: "1000",
  txnRef: "ORDER_890",
  mobileNumber: "08012345678",
  emailAddress: "payer@example.com",
  callBackUrl: "https://merchant.example/paid",
  splitPlanId: "split_school_fees_v1",
}, cb);
```

#### Code — status / force-run

Payment status (and admin GET) include `data.paymentSplit` after the run is stored:

```json
{
  "code": "00",
  "data": {
    "paymentSplit": {
      "txnRef": "ORDER_889",
      "primaryZainboxCode": "main_box",
      "checkoutAmount": "1000",
      "status": "success",
      "legs": [
        {
          "index": 0,
          "zainboxCode": "tutor_box",
          "role": "transfer",
          "amountKobo": 20000,
          "amount": "200",
          "status": "success"
        },
        {
          "index": 1,
          "zainboxCode": "agent_box",
          "role": "transfer",
          "amountKobo": 5000,
          "amount": "50",
          "status": "success"
        },
        {
          "index": 2,
          "zainboxCode": "main_box",
          "role": "keep",
          "amountKobo": 75000,
          "percent": 100,
          "status": "success"
        }
      ]
    }
  }
}
```

```bash
# Inspect run
curl "https://api.koodify.ng/api/v1/payment-splits/ORDER_889" \
  -H "X-Api-Key: $GATEWAY_API_KEY"

# Force-run pending / retry failed or partial
curl -X POST "https://api.koodify.ng/api/v1/payment-splits/ORDER_889" \
  -H "X-Api-Key: $GATEWAY_API_KEY"
# alias: POST …/payment-splits/ORDER_889/run
```

**Admin / ops APIs** (secret `X-Api-Key` = `GATEWAY_API_KEY`):

| Method | Path | Purpose |
|--------|------|---------|
| `PUT` | `/api/v1/split-plans` | Create/update a reusable plan |
| `GET` | `/api/v1/split-plans?primaryZainboxCode=` | List plans for a box (`primaryZainbox` alias accepted) |
| `GET` | `/api/v1/split-plans/{id}` | Get one plan |
| `GET` | `/api/v1/payment-splits/{txnRef}` | Run status + legs |
| `POST` | `/api/v1/payment-splits/{txnRef}` | Force-run pending / retry failed or partial |
| `POST` | `/api/v1/payment-splits/{txnRef}/run` | Same as force-run (alias) |

**DB:** migration `015_payment_splits.sql` (`split_plans`, `payment_splits`, `payment_split_legs`).

---

## Notifications

### Payer receipt email
**What it is:** After a successful transfer, email the customer a branded receipt (Resend). Once per payment.

**Example:** Parent pays → inbox gets “Payment received · NGN 50,000”. Needs `RESEND_API_KEY`.

**Process flow:**
1. Payment becomes successful (status poll *or* deposit webhook).
2. Koodify claims “receipt not sent yet” for that `txnRef` (prevents duplicates).
3. Loads payer email, amount, bank/sender details, custom params.
4. Renders receipt HTML and sends via Resend.
5. If no Resend key or no payer email → skip quietly.

```env
RESEND_API_KEY=re_…
RESEND_FROM=Koodify <receipts@yourdomain.com>
```

### Merchant notice email
**What it is:** Same success can email the *business* (zainbox notification address) that money arrived.

**Example:** Bursar gets “Payment delivered”; parent gets their own receipt.

**Process flow:**
1. Same success trigger as payer receipt.
2. Separate once-per-txn claim for merchant notice.
3. Resolve zainbox → read `email_notification`.
4. If missing → skip; if present → send merchant template with payer + settlement details.

### Merchant outbound webhook (`callBackUrl`)
**What it is:** On payment success, Koodify POSTs a signed JSON payload to the merchant’s server URL — even if the browser tab closed and the JS callback never ran. Merchants never receive raw Zainpay webhook bodies; they get Koodify’s enriched event (`payer`, `customParam`, `paymentSplit`, …).

**Example:** Parent pays and closes the phone; the school backend still gets `payment.success` and marks fees paid.

**Architecture (two inbound paths → one customer event):**
1. **Path 1 — Zainpay push:** Every zainbox’s Zainpay webhook URL should point at Koodify (`POST /webhooks/zainpay`). Koodify verifies, de-dupes, enriches, then POSTs the custom customer event.
2. **Path 2 — Koodify reconcile:** A background sweeper polls Zainpay for recent unpaid checkouts that never got a `deposit.success` webhook, and on success runs the **same** side effects (so a missed Zainpay delivery still notifies the customer once).

**Process flow (customer delivery):**
1. Checkout stores `callBackUrl` on init (required by the inline widget). Fallback: zainbox `callback_url` from seed.
2. Success trigger (inbound webhook, browser status poll, *or* reconciler) → shared side effects (receipts / split / merchant POST).
3. Koodify claims once per `txnRef` (`metadata.merchantWebhookSent`).
4. Builds JSON (`event`, `status`, `reference`, `amount`, `payer`, `customParam`, `paymentSplit` when present).
5. Normalizes directory-style callback URLs to end with `/` (skips file-looking paths) and follows 301/302 while **keeping POST** (avoids nginx trailing-slash → HTTP 405).
6. POSTs with `X-Koodify-Signature` = hex HMAC-SHA256 of the raw body; also sends `X-Koodify-Event: payment.success`.
7. Up to 3 attempts with short backoff; non-2xx is logged. Signing secret: `MERCHANT_WEBHOOK_SECRET`, else `GATEWAY_API_KEY`.

```js
// Checkout — callBackUrl must be an absolute http(s) endpoint that accepts POST
koodifyInitPayment({
  /* … */
  callBackUrl: "https://school.example/webhooks/koodify",
  // Koodify POSTs to …/webhooks/koodify/ (trailing slash added when missing)
}, browserCb); // browserCb still fires when the tab stays open
```

```bash
# Merchant verifies (pseudo)
# signature = HMAC_SHA256_hex(MERCHANT_WEBHOOK_SECRET or GATEWAY_API_KEY, raw_body)
# compare to header X-Koodify-Signature
```

```http
POST /webhooks/koodify/ HTTP/1.1
Content-Type: application/json
User-Agent: Koodify-Webhook/1.0
X-Koodify-Event: payment.success
X-Koodify-Signature: <hex hmac-sha256 of raw body>
```

```json
{
  "event": "payment.success",
  "status": "success",
  "reference": "ORDER_889",
  "description": "Payment was successful",
  "amount": "1000",
  "currencyCode": "NGN",
  "zainboxCode": "main_box",
  "payer": {
    "email": "parent@example.com",
    "phone": "08012345678",
    "name": "Ada Lovelace"
  },
  "customParam": {
    "matric": "ND/12/2211",
    "session": "2026",
    "paymentType": "School Fees",
    "studentId": "STU001"
  },
  "paymentSplit": {
    "txnRef": "ORDER_889",
    "primaryZainboxCode": "main_box",
    "checkoutAmount": "1000",
    "totalKobo": 100000,
    "status": "success",
    "sourceAccount": "0123456789",
    "sourceBankCode": "058",
    "error": null,
    "legs": [
      {
        "index": 0,
        "zainboxCode": "tutor_box",
        "role": "secondary",
        "amountKobo": 20000,
        "percent": null,
        "amount": "200",
        "status": "success",
        "destAccount": "9876543210",
        "transferTxnRef": "split_ORDER_889_0",
        "paymentRef": "…",
        "error": null,
        "attempts": 1
      }
    ]
  }
}
```

Always present: `event`, `status`, `reference`, `description`, `currencyCode`. Optional when known: `amount`, `zainboxCode`, `payer`, `customParam`, `paymentSplit`.

**Notes:** Use a real API route, not a marketing page URL. The browser JS callback and this HTTP POST are independent. Direct Zainpay popup mode does not get this delivery. Point all zainbox webhook addresses at Koodify so Path 1 works. HTML docs: `docs/inline-checkout.html#merchant-webhook`. PHP sample: `samples/php/webhook/koodify/`.

### Pending checkout reconciler
**What it is:** Koodify’s own safety net: periodically ask Zainpay “is this pending checkout paid?” and, if yes and Path 1 never fired, push the same custom customer webhook / receipts / split.

**Example:** Zainpay webhook is delayed; within ~1–2 minutes Koodify notices the payment is successful and notifies the school anyway (once).

**Process flow:**
1. Background loop (or `POST /api/v1/reconcile/pending` with secret key) lists recent `card.initialize*` txns.
2. Skips txns that already have inbound `deposit.success`, `merchantWebhookSent`, or `paymentSuccessHandled`.
3. Respects grace period (let webhook/status win first) and per-txn poll cooldown.
4. Polls Zainpay temporary VA payment status.
5. On success → same shared side effects as Path 1 / status poll.

```env
RECONCILE_ENABLED=true
RECONCILE_INTERVAL_SECS=60
RECONCILE_LOOKBACK_HOURS=48
RECONCILE_GRACE_SECS=45
RECONCILE_POLL_COOLDOWN_SECS=120
RECONCILE_BATCH_SIZE=40
```

```bash
curl -X POST "https://api.koodify.ng/api/v1/reconcile/pending" \
  -H "X-Api-Key: $GATEWAY_API_KEY"
```

### Branded templates
**What it is:** Ready-made email layouts (logo, amount panel, details) for payer and merchant.

**Example:** Both emails look like product mail, not a plain text alert.

**Process flow:**
1. Templates live under `crates/koodify/templates/emails/`.
2. At send time, Koodify fills amount, reference, rows, greeting.
3. Logo is attached inline; HTML/CSS are injected.
4. Compact mobile-friendly layout is applied via stylesheet rules.

---

## Gateway platform

### Stable Koodify API surface (`/api/v1`)
**What it is:** The merchant-facing contract. Developers integrate against short, stable Koodify paths under `/api/v1/…`. Under the hood, the gateway (via the `client` crate) maps each call to one or more **current** Zainpay HTTP endpoints. If Zainpay renames or moves an upstream path, Koodify updates that mapping only — **integrators do not change their code**.

**Why it exists:** Zainpay’s own URLs are long, versioned, and can change (`/virtual-account/wallet/balance/v2/…`, `/virtual-account/create/request`, …). Binding merchants directly to those paths couples their apps to Zainpay’s churn. Koodify paths are the insulation layer.

**Example:** A school app calls `POST /api/v1/va/create` and `GET /api/v1/va/balance/{accountNumber}` forever. Today those talk to Zainpay’s create/balance URLs; if Zainpay ships a `v3` balance path, only Koodify’s client mapping and docs’ “Talks to” line change.

**Process flow:**
1. Merchant app calls a **canonical** Koodify route (e.g. `/api/v1/va/balance/{n}`), not the raw Zainpay path.
2. Gateway authenticates (`X-Api-Key` / public key as appropriate), may peel metadata / enrich.
3. `client` crate issues the mapped Zainpay request(s) with server credentials.
4. Koodify may persist, enrich, or combine (composites), then returns the response.
5. Docs always show **Koodify path → talks to → Zainpay path(s)** so developers understand the hop without depending on the upstream URL.

**Naming rules (house style):**
- Prefer `/api/v1/{resource}/{action}` — short nouns (`va`, `dva`, `zainbox`, `merchant`, `banks`, `transfers`).
- Name routes for **what they return**, not an internal nickname (e.g. merchant transactions composite is `/api/v1/merchant/transactions`, not `…/activity`).
- Keep `/api/v1/activity*` for the real Koodify `activity_events` timeline (local, not a Zainpay mirror).
- Zainpay-shaped gateway paths (and older aliases) may keep working for compatibility, but **new integrations should use `/api/v1`**.

**Docs:** Cluster endpoints by resource; each entry lists the upstream Zainpay path(s). Full catalogue: [docs/api.html](docs/api.html) · composites: [docs/composite-endpoints.html](docs/composite-endpoints.html).

```text
Merchant  →  GET /api/v1/va/balance/{n}          ← stable (do not change for Zainpay churn)
                ↓
            client maps to
                ↓
Zainpay   →  GET /virtual-account/wallet/balance/v2/{n}   ← may change; only Koodify updates
```

**Shipped stable wrappers:**

*Composites / renames*
| Koodify | Talks to (Zainpay) |
|---------|--------------------|
| `GET /api/v1/merchant/transactions` | summary + list (alias: `…/activity`) |
| `GET /api/v1/zainbox/{code}/transactions` | box ledger + deposit summary (alias: `…/activity`) |
| `GET /api/v1/va/{n}` | VA balance + transactions |
| `GET /api/v1/dva/verify/{txnRef}` | DVA status + deposit verify (alias: `…/deposit/{txnRef}`) |

*VA / DVA*
| Koodify | Talks to |
|---------|----------|
| `POST /api/v1/va/create` | `POST /virtual-account/create/request` |
| `GET /api/v1/va/balance/{n}` | `…/wallet/balance/v2/{n}` |
| `GET /api/v1/va/transactions/{n}` | `…/wallet/transactions/{n}` |
| `PATCH /api/v1/va/status` · `POST /api/v1/va/bvn` · `POST /api/v1/va/fund` | matching VA paths |
| `POST /api/v1/dva/create` | `POST /virtual-account/dynamic/create/request` |
| `GET /api/v1/dva/deposit/{txnRef}` · `POST /api/v1/dva/fund` | dynamic status / mock fund |

*Zainbox / merchant*
| Koodify | Talks to |
|---------|----------|
| `GET /api/v1/zainboxes` | `GET /zainbox/list` |
| `POST /api/v1/zainbox` · `PATCH /api/v1/zainbox` | create / update |
| `GET /api/v1/zainbox/{code}/vas` · `/balances` · `/profile` · `/ledger` · `/deposit-summary` · `/card-transactions` | matching `/zainbox/…` paths |
| `POST /api/v1/zainbox/subscribe` · `/unsubscribe` | service-subscription |
| `GET /api/v1/merchant/transactions/summary` · `/list` | summary / merchant transactions |

*Bank / transfer / settlement / checkout*
| Koodify | Talks to |
|---------|----------|
| `GET /api/v1/banks` · `/banks/enquiry` · `/banks/resolve` | list / name-enquiry / composite |
| `POST /api/v1/transfers` · `GET …/verify` · `…/network` | transfer v2 / verify / network |
| `POST\|GET /api/v1/settlements` · `/history/{code}` · `/internal/{code}` | settlement family |
| `POST /api/v1/checkout/initialize` · `/initialize/inline` · `GET /account` · `/status` · `POST /recurring` | card / TVA checkout family |
| `GET /api/v1/reconcile/deposit/{txnRef}` · `/bank-deposit` · `/card-payment` | reconcile family |
| `GET /api/v1/webhooks/repush/deposit\|card/{txnRef}` | webhook repush |

Live API host: [https://api.koodify.ng](https://api.koodify.ng)

### Zainpay API mirror (compatibility paths)
**What it is:** Koodify also exposes many Zainpay-shaped URLs (same path shape as upstream) for drop-in proxying, plus storage and extras. Prefer the **stable `/api/v1` surface** above for new work.

**Example:** An old script still posts to `/virtual-account/create/request` on the gateway; new code should use `/api/v1/va/create` once that wrapper ships.

**Process flow:**
1. Client calls a Koodify path (often with secret key) — either `/api/v1/…` or a Zainpay-shaped alias.
2. Koodify may strip metadata / enrich the body.
3. Request is forwarded to Zainpay with server credentials (path taken from the client mapping, not hard-wired into the merchant’s integration).
4. Response returns; Koodify may persist and enrich before answering you.

### Client SDK (`client` crate)
**What it is:** A Rust toolkit for calling Zainpay cleanly (boxes, accounts, banks, transfers, cards, webhooks, etc.).

**Example:** A background job lists banks or verifies a transfer without hand-rolled HTTP.

**Process flow:**
1. Engineer builds a `ZainpayClient` with base URL + API key.
2. Calls a module method (e.g. create VA, transfer, webhook verify).
3. SDK sends HTTP, parses the standard envelope (`code` / `data`).
4. Caller handles success or error.

```rust
use client::{Environment, ZainpayClient};

let zainpay = ZainpayClient::new(Environment::Sandbox, api_key);
let banks = zainpay.bank().list().await?;
```

### Composite endpoints
**What it is:** One Koodify request that runs two Zainpay calls and returns one combined answer.

**Example:** “Show this zainbox” returns profile *and* balances in one trip.  
Docs: [docs/composite-endpoints.html](docs/composite-endpoints.html)

**Process flow:**
1. Client calls e.g. `GET /api/v1/zainbox/{code}` with secret key.
2. Koodify calls Zainpay endpoint A, then endpoint B.
3. Results are merged into one JSON response.
4. Client renders a dashboard from a single round-trip.

```bash
curl "https://api.koodify.ng/api/v1/zainbox/main_box" \
  -H "X-Api-Key: $GATEWAY_API_KEY"
```

### Inbound webhooks
**What it is:** Zainpay pings Koodify when a deposit lands. Koodify verifies the signature, stores the event, and runs enriched side effects (emails, split, customer webhook). This is Path 1 — **every** zainbox’s Zainpay `callbackUrl` must be Koodify.

**Example:** Parent pays at 2:01; by 2:02 emails + merchant `payment.success` can fire even if the browser tab closed.

**Canonical URL:** `{PUBLIC_BASE_URL}/webhooks/zainpay` (e.g. `https://api.koodify.ng/webhooks/zainpay`).

**Wiring onto zainboxes:**
1. Set `PUBLIC_BASE_URL=https://api.koodify.ng` in `/opt/koodify/.env` and restart.
2. Create/update via Koodify already forces `callbackUrl` to that inbound URL.
3. Backfill existing boxes once:

```bash
curl -X POST "https://api.koodify.ng/api/v1/zainboxes/sync-webhook-callbacks" \
  -H "X-Api-Key: $GATEWAY_API_KEY"
```

**Process flow:**
1. Zainpay POSTs plain JSON to open `POST /webhooks/zainpay` (no gateway API key, HMAC not required).
2. Koodify de-dupes (`webhook_deliveries`), logs `inbound.webhook.received`, persists the event.
3. Extracts `txnRef` → confirms payment with Zainpay (temp-VA status and/or deposit verify).
4. If confirmed → shared success side effects (receipts / split / merchant POST).
5. Always returns HTTP 200 for valid JSON so Zainpay stops retrying; Path 2 covers delayed confirms.

**Note:** Zainbox `callbackUrl` on Zainpay is Path 1 (Koodify). Merchant delivery uses checkout `callBackUrl` (Path outbound) — do not point merchants at `/webhooks/zainpay`.

### Local ops lookups
**What it is:** Admin “look up this account / reference / zainbox history” against Koodify’s database. Health: `GET /health`.

**Example:** Ops pastes an account number and sees the VA plus custom metadata.

**Process flow:**
1. Authenticated request with resource key or account number.
2. Koodify reads Postgres (not a live Zainpay round-trip for the local lookup).
3. Returns stored VA / transaction rows with metadata.
4. `/health` simply answers that the service is up.

### Canonical + legacy paths
**What it is:** Layered aliases so old integrations keep working while the stable `/api/v1` surface grows: (1) preferred Koodify `/api/v1/…`, (2) Zainpay-shaped gateway mirrors, (3) older quirks like `/zainbox/bank/…`.

**Example:** An old script on a legacy path still works; new merchants integrate only against `/api/v1` so a future Zainpay path change never reaches them.

**Process flow:**
1. Request hits canonical `/api/v1`, a Zainpay-shaped mirror, or a legacy alias.
2. All map to the same handler / `client` call (and thus the current Zainpay upstream URL).
3. Response shape stays consistent for the client; only the documented “Talks to” upstream may change over time.

---

## Merchant data & seeding

### Zainboxes cache
**What it is:** Local copy of each merchant “box” (name, notification email, active flag, public key).

**Example:** Checkout looks up `pk_live_…` → finds “Acme School Fees” and its bursar email.

**Process flow:**
1. Seed (or sync) pulls boxes from Zainpay into Postgres.
2. Public key is assigned/preserved on the row.
3. Checkout / emails resolve business identity from this cache.
4. Inactive boxes are refused for checkout.

### Customers (merchants)
**What it is:** Merchant registry (id, contacts, optional zainbox link) used for register → KYC → dashboard.

**Example:** A school registers as a customer (`status: pending`, no zainbox yet). After KYC approval they receive a zainbox + `pk_live_…` and can embed that key like the demo inline checkout samples.

**Process flow:**
1. `POST /customers` (or `/api/v1/customers`) creates the merchant (UUID + integer id). API create/update rejects duplicate emails; seed may still share notification emails across boxes.
2. Optional `zainboxCode` may be set by seed/admin; self-register leaves it empty until KYC approval.
3. `GET /customers/{id}` returns dashboard detail: profile, `hasPassword`, assigned zainbox, publishable `publicKey`, KYC summary, credentials.
4. Lookups: `GET /customers?email=`, `GET /customers/by-uuid/{uuid}`, `GET /customers/{id}/zainbox`.
5. `PATCH /customers/{id}` for partial profile edits; `POST /customers/{id}/provision` retries zainbox create/sync.
6. Credentials and KYC attach to that customer id. Login auth UI comes later (`password_hash` column is ready, never exposed).

### KYC
**What it is:** Identity profile (name, BVN, address, …) linked to a customer. Approving KYC provisions the merchant zainbox.

**Example:** Compliance sets KYC for customer `#42` to `approved` → Koodify creates a Zainpay zainbox named after their business (`displayName`), mints `pk_live_…`, links the customer, and mirrors credentials.

**Process flow:**
1. Admin upserts KYC for a `customerId` (typically `pending` while under review).
2. Fields saved with status (`pending` / `approved` / `rejected`; legacy `active` still accepted and does **not** auto-provision).
3. Later fetch by KYC id or by customer.
4. `PATCH /kyc/{id}/status` with `approved` (or upsert with that status) triggers merchant provisioning.
5. Response may include a `provision` object (`zainboxCode`, `publicKey`). On Zainpay failure, retry `POST /customers/{id}/provision`.

### Credentials
**What it is:** Test and live key pairs per customer. Publishable key mirrors the assigned zainbox; “private key” is the zainbox `codeName` (Bearer id).

**Example:** After seed or KYC approval, each school has a browser public key (`pk_live_…`) and a server-side zainbox code matching Zainpay.

**Process flow:**
1. Upsert credentials for `(customerId, environment)` — or let provisioning sync them from the zainbox.
2. Checkout resolves identity from `zainboxes.public_key` only; credentials mirror that key for dashboard/API compatibility.
3. `zainboxCode` stays server-side / for ops (reveal with `?revealZainboxCode=true`).
4. Status can disable a key pair without deleting history.

### Virtual accounts table
**What it is:** Memory of VAs Koodify created, including your tags.

**Example:** Permanent VA for a landlord; later fetch by account number still shows `tenantId`.

**Process flow:**
1. Create VA through the gateway (with optional metadata).
2. Upstream success → row saved locally.
3. Later `GET /virtual-accounts/{accountNumber}` returns stored data + metadata.
4. Balance/status can still be checked via mirrored Zainpay routes when needed.

### Seeding
**What it is:** One-command sync from live Zainpay: import boxes, mint public keys, create customers and credentials.

**Example:** New environment → run seed → every live zainbox is checkout-ready with `pk_live_…`.

**Process flow:**
1. Run `cargo run -p koodify --example seed` with `ZAINPAY_API_KEY`.
2. Fetch live zainboxes from Zainpay.
3. Upsert into local `zainboxes` (preserve existing public keys when possible).
4. Ensure each box has a `pk_live_…`.
5. Create/update one customer per box + test/live credentials.
6. Print public keys for merchants to embed.

```bash
export DATABASE_URL=postgres://…
export ZAINPAY_API_KEY=…
cargo run -p koodify --example seed
```

### Migrations
**What it is:** Ordered database schema updates.

**Example:** Deploy → migrate → new columns (public keys, email flags) exist before traffic.

**Process flow:**
1. Run migrate example against `DATABASE_URL`.
2. Pending SQL files apply in number order.
3. Schema matches what the new code expects.
4. App starts serving with the updated tables.

---

## Developer tooling

### Inline package
**What it is:** Source for the browser checkout script, with local demo and live-test harness.

**Example:** Engineer runs `npm run demo`, clicks Pay against a laptop gateway, then ships the CDN build.

**Process flow:**
1. Edit `packages/koodify-inline/src`.
2. `npm run build` → `dist/koodify-inline.js` (+ demo/live-test copies).
3. `npm run demo` for local proxy testing, or `npm run test:live` against hosted API.
4. Upload dist to `js.koodify.ng/v1/` on release.

### HTML docs
API reference, inline checkout guide, and composite guide under `docs/`.

**Example:** New integrator opens `docs/inline-checkout.html` and copies the snippet.

**Process flow:**
1. Open the relevant HTML doc in a browser.
2. Follow install / quickstart / parameter tables.
3. Wire the site → test → go live.

### Tests
Automated checks for the Rust client and gateway logic.

**Example:** Before release, `cargo test -p koodify --lib` confirms Adaptive Bank/DVA Routing and receipt HTML.

**Process flow:**
1. Developer or CI runs the test suite.
2. Unit/integration tests exercise routers, mail templates, metadata, etc.
3. Failures block bad changes; green → safer deploy.

---

## Schema naming (house style)

| Concept | Canonical DB column | Notes |
|---------|---------------------|--------|
| Checkout / payment id | `txn_ref` | Matches Zainpay `txnRef`. Used on `activity_events`, `payment_splits`, `payment_split_legs`, `webhook_deliveries`. |
| Polymorphic ops id | `resource_key` | **Only** on `transactions` — may be `txnRef` *or* `accountNumber`. |
| Zainbox business id | `zainbox_code` | On `zainboxes` (PK), `customers`, `credentials`, legs, logs, etc. Zainpay wire name remains `codeName`. |
| Split owner box | `primary_zainbox_code` | Role-qualified; not bare `primary_zainbox`. |
| Split leg transfer id | `transfer_txn_ref` | Child transfer’s own ref — not the checkout `txn_ref`. |

`credentials.zainbox_code` is the Zainpay Bearer identity (formerly misnamed `private_key`). API: `zainboxCode`; reveal with `?revealZainboxCode=true`.

Migration: `018_naming_consistency.sql`.

---

## Env-gated / optional (plain English)

| Feature | When it turns on |
|---------|------------------|
| Receipt & merchant emails | You set a Resend API key |
| Merchant outbound webhook | Checkout `callBackUrl` (or zainbox `callback_url`) is an absolute http(s) URL; signed with `MERCHANT_WEBHOOK_SECRET` or `GATEWAY_API_KEY` |
| Pending checkout reconciler | On by default (`RECONCILE_ENABLED`); disable with `false`. Force: `POST /api/v1/reconcile/pending` |
| Activity events | Always on after migrate `017`; query `GET /api/v1/activity/{txnRef}` |
| Inbound webhooks | Set `ZAINPAY_WEBHOOK_SECRET` + `PUBLIC_BASE_URL`; sync zainbox callbacks to `/webhooks/zainpay` |
| Multi-bank Adaptive Bank/DVA Routing | You list more than one bank in `VA_BANK_CANDIDATES` (today’s catalog source; later → Zainpay available-banks API via `client`) |
| Custom params / payer name / metadata | You send those fields in the request |
| Merchant email destination | The zainbox has an `email_notification` address |
| Direct Zainpay popup mode | You set `auth: "zainpay"` and supply an API key |
| Live-test local server port | Optional `LIVE_TEST_PORT` |
| Zainbox Payment Split | Gateway: `splitPlanId` on open checkout, or `paymentSplit`/`splitPlanId` on ApiKeyAuth init; primary settlement must be off |
