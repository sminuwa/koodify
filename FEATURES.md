# Koodify features

What Koodify adds on top of Zainpay — written so a non-engineer can skim it.
Each feature has **what it is**, a short **example**, and a simple **process flow**.
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

Docs: [docs/inline-checkout.html](docs/inline-checkout.html) · Script: `https://js.koodify.ng/v1/koodify-inline.js`

### Publishable public keys
**What it is:** A safe “shop ID” for the browser (`pk_live_…`). It tells Koodify *which business* is collecting money, without exposing secret API keys on the website.

**Example:** Like a storefront address on a map — public enough to find the shop, not the keys to the safe.

**Process flow:**
1. Seed / admin assigns a `pk_live_…` to a zainbox in Koodify’s database.
2. Merchant puts that key in their website checkout config.
3. On pay, the popup sends the public key to Koodify.
4. Koodify looks up the matching zainbox and uses its secret Zainpay identity server-side.
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
2. Gateway picks the current preferred bank from `VA_BANK_CANDIDATES`.
3. Tries to create the temporary account with that bank.
4. On failure → tries the next candidate; records success/speed for each attempt.
5. Preferred bank sticks for a while (`VA_BANK_STICKY_SECS`), then can be re-scored.
6. If merchant *does* pass `bankType`, that bank is used once (no auto-switch) — override path.

```env
VA_BANK_CANDIDATES=gtBank,moniepoint
VA_BANK_STICKY_SECS=600
```

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

### Merchant notice email
**What it is:** Same success can email the *business* (zainbox notification address) that money arrived.

**Example:** Bursar gets “Payment delivered”; parent gets their own receipt.

**Process flow:**
1. Same success trigger as payer receipt.
2. Separate once-per-txn claim for merchant notice.
3. Resolve zainbox → read `email_notification`.
4. If missing → skip; if present → send merchant template with payer + settlement details.

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

### Zainpay API mirror
**What it is:** Koodify exposes Zainpay-style payment APIs, plus storage and extras.  
Live: [https://api.koodify.ng](https://api.koodify.ng)

**Example:** Your app talks to Koodify and gets Zainpay’s power plus memory (payer, metadata, receipts).

**Process flow:**
1. Client calls a Koodify path (often with secret key).
2. Koodify may strip metadata / enrich the body.
3. Request is forwarded to Zainpay with server credentials.
4. Response returns; Koodify may persist and enrich before answering you.

### Client SDK (`client` crate)
**What it is:** A Rust toolkit for calling Zainpay cleanly (boxes, accounts, banks, transfers, cards, webhooks, etc.).

**Example:** A background job lists banks or verifies a transfer without hand-rolled HTTP.

**Process flow:**
1. Engineer builds a `ZainpayClient` with base URL + API key.
2. Calls a module method (e.g. create VA, transfer, webhook verify).
3. SDK sends HTTP, parses the standard envelope (`code` / `data`).
4. Caller handles success or error.

### Composite endpoints
**What it is:** One Koodify request that runs two Zainpay calls and returns one combined answer.

**Example:** “Show this zainbox” returns profile *and* balances in one trip.  
Docs: [docs/composite-endpoints.html](docs/composite-endpoints.html)

**Process flow:**
1. Client calls e.g. `GET /api/v1/zainbox/{code}` with secret key.
2. Koodify calls Zainpay endpoint A, then endpoint B.
3. Results are merged into one JSON response.
4. Client renders a dashboard from a single round-trip.

### Inbound webhooks
**What it is:** Zainpay pings Koodify when a deposit lands. Koodify verifies the signature, stores the event, and can email receipts.

**Example:** Parent pays at 2:01; by 2:02 both emails can fire even if the browser tab closed.

**Process flow:**
1. Zainpay POSTs to `/webhooks/zainpay` with a signature header.
2. Koodify verifies HMAC using `ZAINPAY_WEBHOOK_SECRET` (required).
3. Event is persisted.
4. On deposit success → trigger payer/merchant email flow (same as status success).
5. Invalid signature → rejected.

### Local ops lookups
**What it is:** Admin “look up this account / reference / zainbox history” against Koodify’s database. Health: `GET /health`.

**Example:** Ops pastes an account number and sees the VA plus custom metadata.

**Process flow:**
1. Authenticated request with resource key or account number.
2. Koodify reads Postgres (not a live Zainpay round-trip for the local lookup).
3. Returns stored VA / transaction rows with metadata.
4. `/health` simply answers that the service is up.

### Canonical + legacy paths
**What it is:** Newer Zainpay-style URLs and older `/zainbox/bank/…` aliases so old integrations keep working.

**Example:** An old script on a legacy path still works while new code uses the modern path.

**Process flow:**
1. Request hits either canonical or legacy route.
2. Both map to the same underlying handler / Zainpay call.
3. Response shape stays consistent for the client.

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

### Customers
**What it is:** Merchant/customer registry (id, contacts, optional zainbox link).

**Example:** Seed creates one customer per zainbox so credentials and KYC have a home.

**Process flow:**
1. Create/update customer via admin API (secret key) or seed.
2. Record gets UUID + integer id.
3. Optional `zainboxCode` links them to a box.
4. Credentials and KYC attach to that customer id.

### KYC
**What it is:** Identity profile (name, BVN, address, …) linked to a customer.

**Example:** Compliance stores KYC against customer `#42`.

**Process flow:**
1. Admin upserts KYC for a `customerId`.
2. Fields saved with status (e.g. active / pending).
3. Later fetch by KYC id or by customer.
4. Status can be patched without rewriting the whole profile.

### Credentials
**What it is:** Test and live key pairs per customer (public + private).

**Example:** After seed, each school has a browser public key and a private key matching the Zainpay zainbox id.

**Process flow:**
1. Upsert credentials for `(customerId, environment)`.
2. Public key used in browser checkout; private key stays server-side / for ops.
3. Lookup by customer (both envs) or by customer + environment.
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

## Zainbox Payment Split

**Status:** Shipped (v1) — validate at init, run after deposit success, internal VA/ISA → secondary ISA

**What it is:** After a customer pays the full amount into one **primary** zainbox, Koodify automatically redistributes value to **secondary** zainboxes by share (percent and/or fixed amount). Developers declare *shares and zainbox codes* — not external bank accounts. Moves are **internal** (zainbox/VA → zainbox/VA), not NIP payouts to third-party banks.

**Example:** A school collects ₦10,000. Plan: ₦2,000 to tutor, ₦1,000 to agent, **percent: 100** on primary (= ₦7,000 remainder). Parent pays once; Koodify settles the boxes after success.

**Why this shape:**
- Checkout stays simple: one deposit, one DVA, Adaptive Bank/DVA Routing unchanged.
- No bank account numbers / bank codes in the browser.
- Matches Zainpay pricing: transfers between virtual accounts across zainboxes are treated as free (vs ~₦25 external).
- Safer than post-collection external payouts for v1.

**Not in v1:**
- Splitting into three customer deposits.
- Sending money to arbitrary external bank accounts from checkout.
- Replacing Zainpay’s scheduled settlement (`POST /zainbox/settlement`) — that remains a separate merchant config for bank payout schedules.

#### Checkout / init shape

Prefer a saved plan for production; allow inline shares for demos.

```js
// Option A — saved plan (recommended)
koodifyInitPayment({
  publicKey: "pk_live_…",
  amount: "10000",
  txnRef: "ORDER_889",
  email: "parent@school.com",
  splitPlanId: "split_school_fees_v1",
});

// Option B — inline shares (demo / simple merchants)
koodifyInitPayment({
  publicKey: "pk_live_…",
  amount: "10000",
  txnRef: "ORDER_889",
  paymentSplit: {
    // Optional; default = zainbox resolved from publicKey
    primaryZainbox: "school_main_box",
    shares: [
      { zainboxCode: "tutor_box", amount: "2000" },
      { zainboxCode: "agent_box", amount: "1000" },
      { zainboxCode: "school_main_box", percent: 100 }, // 100% of remainder
    ],
  },
});
```

Share fields (per leg):
| Field | Meaning |
|-------|---------|
| `zainboxCode` | Destination (or primary keep) box |
| `amount` | Fixed Naira amount (taken first from the deposit) |
| `percent` | Share of **whatever remains after all fixed `amount` legs** (must total **100%** of that remainder). Use `percent: 100` on primary to keep the rest. |

A leg uses **either** `percent` **or** `amount`, not both.

#### Validation rules (reject at init if broken)

1. At least one share; max N legs (suggested cap: **10**).
2. Every `zainboxCode` exists in Koodify’s zainboxes cache and is **active** (v1 ACL).
3. `primaryZainbox` (if set) must match the publicKey’s box; that box must appear in the plan as collector; customer VA is always on **primary only**.
4. Fixed `amount` legs are allocated first; `percent` legs split the **remainder** and must sum to 100%.
5. Flooring leftover kobo in the remainder pool goes to the primary percent leg (or first percent leg). Over-allocate fixed amounts → reject.
6. No zero/negative shares; no duplicate destination boxes in one plan (or merge duplicates — prefer **reject duplicates** for clarity).
7. Inline `paymentSplit` is stored on the txn like `customParam` (Koodify-only; not sent to Zainpay create-VA).

#### Fee policy

- Split math uses the **amount the customer was asked to pay** (checkout amount).
- Zainpay deposit fees (if any) are **not** auto-deducted from secondary legs in v1 — document that primary may absorb fee variance.
- Internal VA→VA legs: expect **no** Koodify markup in v1; rely on Zainpay’s free internal pricing.

#### Rounding

- Work in **kobo** (integer).
- Fixed amounts first; then `percent` of the remainder (floor); leftover kobo → primary percent leg.
- Fixed `amount` legs must be whole Naira/kobo-safe strings; reject fractions that don’t map cleanly.

#### Runtime process flow

1. Merchant starts checkout with `splitPlanId` or `paymentSplit`.
2. Koodify validates the plan → stores it on the transaction log for that `txnRef`.
3. Customer pays full amount into the **primary** zainbox DVA (same path as today).
4. On success (status poll **or** deposit webhook) → Koodify claims “split not run yet” for that `txnRef` (idempotent, same pattern as receipt claims).
5. Resolve plan → compute kobo legs → for each **secondary** leg, initiate an **internal** funds transfer from primary source VA / wallet to that zainbox’s destination VA (exact source field confirmed in sandbox proof).
6. Verify each leg; persist status per leg (`pending` / `success` / `failed`).
7. Primary keep is implicit (no transfer to self required).
8. Receipts / merchant notice / status enrichment may include a `split` summary (who got what).
9. Failed secondary leg → mark plan `partial`; retry policy: bounded automatic retries, then ops-visible failed legs (no silent drop).

#### Failure & safety rules

| Situation | Behavior |
|-----------|----------|
| Invalid plan at init | Reject checkout start; no VA created |
| Deposit never succeeds | No split runs |
| Split already claimed for `txnRef` | No second run |
| One secondary transfer fails | Other succeeded legs stay; failed leg retried / flagged `partial` |
| Destination zainbox deactivated after init | Fail that leg; do not invent an external bank fallback |
| Insufficient balance on primary at split time | Fail remaining legs; alert ops (should be rare right after deposit) |
| Direct Zainpay popup mode (`auth: "zainpay"`) | Split **off** — Koodify must own the post-success path |
| **Primary zainbox has auto settlement enabled** | Zainpay rejects transfers: `code=04` — *Funds Transfer is Disabled for Auto Settlement Enabled Zainbox* |

**Hard constraint (confirmed live):** the **primary / source** zainbox must have **auto settlement disabled** (`settlement.status = false`, or no active settlement schedule) for Koodify to move funds to secondary boxes. Scheduled settlement on secondaries is fine.

Recommended layout:
- **Collector box** (checkout `publicKey`) — settlement **off**, used only to receive + split  
- **Secondary boxes** — receive ISA credits; may use T1/T7 settlement to external banks  

#### Endpoint proof (2026-10-31)

Confirmed on live Zainpay via `crates/koodify/examples/va_to_va_proof.rs`:

| Item | Result |
|------|--------|
| Endpoint | `POST /bank/transfer/v2` |
| Verify | `GET /virtual-account/wallet/transaction/verify/{txnRef}` |
| Path tested | Primary **gtBank VA** → secondary **ZainBank ISA** (cross-zainbox) |
| Fee | `txnFee: "0"` (internal / free) |
| Result | Immediate `code=00`, `status=success` (not only queued) |
| Balances | Source −100 kobo, dest +100 kobo |

Notes for implementation:
- Each zainbox has an **Internal Settlement Account** (`bankType: ZainBank`, bank code `090976`) from `GET /zainbox/internal-settlement-account/{code}` (also appears in VA list).
- Deposit VAs use issuer codes (e.g. gtBank `000013`); list field is `bankAccount` + `bankName` as the NUBAN code.
- Prefer debiting the **funded deposit VA** (or ISA if auto-swept) and crediting the secondary **ISA** (or a known VA).
- Name enquiry on ISA may return invalid — skip NE for ZainBank legs.
- Re-run proof: `PROOF_EXECUTE=1 PRIMARY_ZAINBOX=… SECONDARY_ZAINBOX=… cargo run -p koodify --example va_to_va_proof`

#### Shipped surface

| Piece | Where |
|-------|--------|
| Init validation + schedule | `POST …/initialize/payment/inline` (+ authenticated initialize) |
| Saved plans | `PUT/GET /api/v1/split-plans` (secret key) |
| Run status | `GET /api/v1/payment-splits/{txnRef}` + `data.paymentSplit` on status poll |
| Trigger | Status poll success **or** `deposit.success` webhook (idempotent claim) |
| DB | Migration `015_payment_splits.sql` |
| Proof harness | `cargo run -p koodify --example va_to_va_proof` |

#### Process flow (layman)

1. Parent pays ₦10,000 once into the school’s main box account.
2. Money lands in the primary zainbox.
3. Koodify reads the share plan for that payment.
4. It moves ₦2,000 to the tutor box and ₦1,000 to the agent box (inside Zainpay).
5. School main box keeps ₦7,000.
6. Status / admin lookup shows the split summary.

---

## Env-gated / optional (plain English)

| Feature | When it turns on |
|---------|------------------|
| Receipt & merchant emails | You set a Resend API key |
| Inbound webhooks | You set the Zainpay webhook secret |
| Multi-bank Adaptive Bank/DVA Routing | You list more than one bank in `VA_BANK_CANDIDATES` |
| Custom params / payer name / metadata | You send those fields in the request |
| Merchant email destination | The zainbox has an `email_notification` address |
| Direct Zainpay popup mode | You set `auth: "zainpay"` and supply an API key |
| Live-test local server port | Optional `LIVE_TEST_PORT` |
| Zainbox Payment Split | Checkout sends `paymentSplit` or `splitPlanId` (gateway mode) |
