# Koodify: closing Zainpay DX gaps — and what we add on top

**Audience:** product, partners, and engineers evaluating Koodify as a Zainpay wrapper.  
**Scope:** (1) ten documentation-led integration challenges from the Zainpay Developer Experience assessment, and how Koodify addresses each; (2) product features Koodify adds that Zainpay alone does not provide.

Living feature inventory (skim-friendly): [`FEATURES.md`](../FEATURES.md).  
HTML API / checkout docs: [`docs/api.html`](api.html), [`docs/inline-checkout.html`](inline-checkout.html).

---

## 1. Executive summary

Zainpay is the payment rail (virtual accounts, transfers, settlements, webhooks). Integrating against Zainpay docs directly exposes merchants to conflicting instructions, ambiguous statuses, opaque error codes, bank-specific VA issuance, and unclear idempotency / versioning.

**Koodify** sits in front of Zainpay as a gateway:

| Layer | Role |
|-------|------|
| Merchant app / browser | Talks to **Koodify** (`/api/v1`, inline checkout, merchant webhooks) |
| Koodify | Normalizes status & errors, routes banks, signs one webhook contract, retries safely, enriches payments |
| Zainpay | Issues VAs, moves money, settles |

Merchants should treat Zainpay as an implementation detail. New integrations use **`https://api.koodify.ng/api/v1/…`** and the inline script — not raw Zainpay paths or HMAC rules.

---

## 2. Challenge → how Koodify addresses it

The ten items below come from the *Zainpay Developer Experience — Visual Issue Pack*. For each: the pain, Koodify’s approach, and a concrete example.

### At a glance

| # | Challenge | Koodify outcome |
|---|-----------|-----------------|
| 01 | Conflicting webhook signing instructions | One outbound HMAC contract for merchants |
| 02 | Node samples that HMAC re-serialized JSON | Raw-body sample + docs anti-pattern |
| 03 | Transfer vs deposit event name mismatch | Outbound event is always `payment.success` |
| 04 | “Txn not found” ambiguous vs pending | Canonical `data.koodify.paymentStatus` |
| 05 | Field names / shapes vary | Stable camelCase merchant contract + `data.koodify` |
| 06 | Opaque application code `04` | `data.koodify.error` with actionable hints |
| 07 | Unclear request idempotency | `Idempotency-Key` on transfer POSTs |
| 08 | No clear versioning / deprecation policy | `/api/v1` policy + Sunset headers |
| 09 | SDK links lack onboarding | Inline checkout, FEATURES, PHP/Node samples |
| 10 | `bankType` leaks into every integration | Adaptive Bank/DVA Routing (omit bank) |

---

### 01 — Webhook signing instructions conflict

**Challenge:** Zainpay documentation pages disagree on which credential / format to use for HMAC verification of the same webhook process. Integrators guess wrong and “signature mismatch” loops burn days.

**How Koodify addresses it:** Merchants **never verify Zainpay’s inbound HMAC**. Every zainbox’s Zainpay `callbackUrl` points at Koodify (`POST /webhooks/zainpay`). Koodify confirms the payment with Zainpay APIs, then POSTs a **single** merchant event signed as:

- Header: `X-Koodify-Signature`
- Algorithm: HMAC-SHA256 hex over the **raw body**
- Secret: `MERCHANT_WEBHOOK_SECRET`, else `GATEWAY_API_KEY`

**Example:**

```http
POST /webhooks/koodify/ HTTP/1.1
Content-Type: application/json
X-Koodify-Event: payment.success
X-Koodify-Signature: <hex hmac-sha256 of raw body>

{"event":"payment.success","status":"success","reference":"ORDER_889",…}
```

School backend verifies only against Koodify’s secret — one rule, one sample set (PHP + Node).

---

### 02 — Node.js examples rebuild the request body

**Challenge:** Common Node samples parse JSON, then sign/verify `JSON.stringify(parsed)`. Key order and whitespace change → valid deliveries fail verification.

**How Koodify addresses it:** Document and ship the correct pattern: verify **raw bytes**, then parse.

**Example (Node):**

```js
// Correct — express.raw() Buffer
app.post("/webhooks/koodify", express.raw({ type: "*/*" }), (req, res) => {
  const rawBody = req.body; // Buffer
  const ok = verifyKoodifySignature(secret, rawBody, req.get("X-Koodify-Signature"));
  const payload = JSON.parse(rawBody.toString("utf8"));
  // …
});

// WRONG — never do this
// crypto.createHmac("sha256", secret).update(JSON.stringify(req.body))
```

Samples: `samples/node/webhook/koodify/`, `samples/php/webhook/koodify/`. Docs: `docs/inline-checkout.html#merchant-webhook`.

---

### 03 — Transfer webhook event name does not match

**Challenge:** Docs describe `transfer.success` while sample payloads say `deposit.success` (or the reverse). Handlers miss events.

**How Koodify addresses it:** Merchant outbound webhooks always use:

- `X-Koodify-Event: payment.success`
- Body `"event": "payment.success"`

Inbound Zainpay names stay internal. Path 1 (webhook) and Path 2 (reconciler) both emit the same customer event.

**Example:** Merchant code only needs:

```js
if (req.get("X-Koodify-Event") === "payment.success") {
  markFeesPaid(req.body.reference);
}
```

---

### 04 — Transaction-not-found response is ambiguous

**Challenge:** Zainpay’s `code=04` / “Txn not found” is documented as possibly pending, failed, or genuinely missing. Merchants cannot safely decide whether to fulfill or wait.

**How Koodify addresses it:** Status and verify responses attach a canonical block:

```json
"data": {
  "paymentStatus": "pending",
  "koodify": {
    "paymentStatus": "pending",
    "isPaid": false,
    "hint": "Awaiting the expected deposit. Do not fulfill yet.",
    "upstream": { "code": "00", "description": "DVA status", "paymentStatus": "pending" }
  }
}
```

Canonical values: `pending` · `success` · `failed` · `not_found` · `mismatch` · `expired` · `unknown`.

Fulfillment (receipts, splits, merchant webhook) runs **only** when `koodify.isPaid === true`. `code=00` alone is never treated as paid. “Txn not found” maps to `not_found`, not pending.

**Example:** Checkout expected ₦200; status still pending → `isPaid: false`. Ops ask “is it paid?” → read `data.koodify.paymentStatus`, not raw `code`.

---

### 05 — Field names and response shapes vary

**Challenge:** Equivalent fields use different casing or nesting across Zainpay endpoint examples (`txnRef` vs `txn_ref`, status in different places).

**How Koodify addresses it:**

- Merchant webhook and enrichment use stable **camelCase** (`txnRef`, `zainboxCode`, `customParam`, `paymentSplit`).
- Inbound extractors accept common aliases.
- Canonical payment / error info lives under **`data.koodify`** on status and verify paths.
- New work uses the stable `/api/v1` surface (not ad-hoc Zainpay mirrors).

**Example:** Merchant webhook always exposes `reference`, `payer`, `customParam` when known — independent of which Zainpay field carried the deposit.

---

### 06 — Generic error code gives too little guidance

**Challenge:** Application `code=04` (and similar) group many failures into one bucket. Integrators see “application error” with no next step.

**How Koodify addresses it:** Failed envelopes (not `00` / `21`) gain:

```json
"data": {
  "koodify": {
    "error": {
      "koodifyCode": "transfer_disabled_auto_settlement",
      "message": "Funds Transfer is Disabled for Auto Settlement Enabled Zainbox",
      "hint": "Disable auto settlement on the primary zainbox (POST /api/v1/settlements with status:false), then retry…",
      "upstreamCode": "04",
      "upstreamDescription": "Funds Transfer is Disabled for Auto Settlement Enabled Zainbox"
    }
  }
}
```

Description heuristics catch auto-settlement, txn not found, duplicate refs, expiry; otherwise the code table supplies a hint. Client `describe_code` aligns with Zainpay’s published status codes.

---

### 07 — Request idempotency is not clearly specified

**Challenge:** Docs discuss webhook dedupe but not safe retries for money-moving POSTs. Timeouts cause double transfers.

**How Koodify addresses it:**

- **Transfers:** optional `Idempotency-Key` (or `X-Idempotency-Key`) on `POST /api/v1/transfers` and `POST /api/v1/transfer`.
  - Same key + same body → replay stored JSON (24h).
  - Same key + different body → HTTP 409.
  - In-flight → 409; stale pending claims can reclaim after ~2 minutes.
- **Payment splits:** stable per-leg `transfer_txn_ref` + verify-before-resend (already idempotent).

**Example:**

```bash
curl -X POST "https://api.koodify.ng/api/v1/transfers" \
  -H "X-Api-Key: $GATEWAY_API_KEY" \
  -H "Idempotency-Key: payout_ORDER_889_1" \
  -H "Content-Type: application/json" \
  -d '{ … same body … }'
# Retry after timeout → same success payload; no second debit
```

Migration: `020_idempotency_keys.sql`.

---

### 08 — API versioning and deprecation policy is not evident

**Challenge:** Zainpay exposes versioned / changing paths (`…/balance/v2/…`) without a clear merchant-facing compatibility policy.

**How Koodify addresses it:**

1. **`/api/v1` is the stable contract.** Additive changes only.
2. **Breaking changes** go to `/api/v2` (or a new major) — not silent breaks on `/api/v1`.
3. **Deprecated aliases** keep working until **Sunset 2027-05-01**, with response headers:
   - `Deprecation: true`
   - `Sunset: Sat, 01 May 2027 00:00:00 GMT`
   - `Link: <successor>; rel="successor-version"`
   - `X-Koodify-Successor: …`
4. Zainpay upstream churn is absorbed inside Koodify’s client mapping.

| Deprecated | Use instead |
|------------|-------------|
| `GET …/zainbox/{code}/activity` | `…/zainbox/{code}/transactions` |
| `GET …/merchant/activity` | `…/merchant/transactions` |
| `GET …/virtual-account/{n}` | `…/va/{n}` |
| `GET …/deposit/{txnRef}` | `…/dva/verify/{txnRef}` |

`GET /api/v1/activity/{txnRef}` is **not** deprecated — it is Koodify’s local payment timeline.

---

### 09 — SDK links lack practical onboarding context

**Challenge:** Language / platform links without a clear “first successful payment” path.

**How Koodify addresses it:** A guided merchant path:

1. Publishable `publicKey` (`pk_live_…`)
2. Drop-in script: `https://js.koodify.ng/v1/koodify-inline.js`
3. Docs: `docs/inline-checkout.html`, `FEATURES.md`
4. Webhook samples: PHP + Node
5. Optional Rust `client` crate for server jobs

**Example:** School site embeds inline checkout with `publicKey` + `callBackUrl` — no Zainpay secret in the browser, no bank hard-coding.

---

### 10 — Bank-specific selection can leak into integrations

**Challenge:** VA flows expose `bankType` and docs suggest “try another bank” when issuance fails. Apps hard-code GTBank / Moniepoint and break when one issuer is unhealthy.

**How Koodify addresses it:** **Adaptive Bank/DVA Routing**

- Recommended: **omit `bankType`**
- Koodify picks from `VA_BANK_CANDIDATES`, fails over on errors, keeps a sticky preferred bank
- Explicit `bankType` = one-shot override (no auto-switch)

**Example:**

```js
koodifyInitPayment({
  publicKey: "pk_live_…",
  amount: "50000",
  txnRef: "FEES_" + Date.now(),
  // bankType omitted — Koodify chooses / fails over
  callBackUrl: "https://school.example/webhooks/koodify",
}, cb);
```

```env
VA_BANK_CANDIDATES=gtBank,moniepoint
VA_BANK_STICKY_SECS=600
```

---

## 3. Additional features Koodify adds (not in Zainpay)

These are **product capabilities** on top of wrapping and DX insulation. Zainpay remains the rail; Koodify owns the merchant experience around it.

### 3.1 Checkout product

| Feature | What it gives you |
|---------|-------------------|
| **Inline bank-transfer checkout** | Hosted popup script: show account + amount, poll until paid, browser callback |
| **Publishable public keys** | `pk_live_…` in the browser; secrets stay on the server |
| **Open checkout routes** | Init / temp VA / status without `GATEWAY_API_KEY` |
| **Checkout UX extras** | Clear reused-`txnRef` errors, short success screen, optional logo |
| **Gateway vs direct mode** | Default through Koodify; optional raw Zainpay for debugging |

### 3.2 Payments intelligence

| Feature | What it gives you |
|---------|-------------------|
| **Payer enrichment** | Store email / phone / name at init; re-attach on status (Zainpay often does not echo them) |
| **Custom params** | Merchant metadata (`matric`, `session`, …) stored and returned; not forwarded to Zainpay |
| **Metadata on mutating calls** | Arbitrary notes persisted with the operation |
| **Transaction logging** | Diary of API calls by `txnRef` / resource |
| **Activity events** | Audit of *who* fired success: Zainpay webhook, status poll, or reconciler |
| **Zainbox Payment Split** | After collect on primary box, redistribute to secondary zainboxes (idempotent internal transfers) |

### 3.3 Notifications & reliability

| Feature | What it gives you |
|---------|-------------------|
| **Payer receipt email** | Branded Resend receipt once per successful payment |
| **Merchant notice email** | Ops / bursar notified on the same success |
| **Merchant outbound webhook** | Signed `payment.success` to `callBackUrl` even if the browser closed |
| **Pending checkout reconciler** | Path 2: if Zainpay webhook never arrives, Koodify polls and still delivers once |
| **Branded email templates** | Product-looking HTML for receipts / notices |

### 3.4 Gateway platform & ops

| Feature | What it gives you |
|---------|-------------------|
| **Stable `/api/v1` surface** | Short routes that map to current Zainpay URLs behind the scenes |
| **Composite endpoints** | One Koodify call → two Zainpay calls (e.g. DVA status + deposit verify) |
| **Inbound webhook hub** | Central `POST /webhooks/zainpay` + callback sync for all zainboxes |
| **Local ops lookups** | DB look-ups by reference / account; `GET /health` |
| **Zainpay-shaped mirrors** | Compatibility paths for older scripts (prefer `/api/v1` for new work) |
| **Rust client crate** | Typed Zainpay HTTP toolkit used by the gateway |

### 3.5 Merchant data & onboarding

| Feature | What it gives you |
|---------|-------------------|
| **Zainboxes cache** | Local box identity, notification email, public key, active flag |
| **Customers registry** | Merchant profiles for register → KYC → dashboard |
| **KYC workflow** | Profiles and approval hooks before provisioning |
| **Credentials** | Test/live key material tied to customers / boxes |
| **Virtual accounts table** | Local VA rows with metadata |
| **Seeding & migrations** | Postgres schema + seed from Zainpay inventory |

### 3.6 Developer tooling

| Feature | What it gives you |
|---------|-------------------|
| **`koodify-inline` package** | Source + live-test HTML for the checkout widget |
| **HTML docs** | Tailwind API / composite / inline docs under `docs/` |
| **Samples** | PHP + Node merchant webhook receivers |
| **Automated tests** | Unit coverage for status, errors, splits, routing, idempotency headers |

---

## 4. Reference architecture (mental model)

```text
┌─────────────────────┐     publicKey / callBackUrl      ┌──────────────────────┐
│  Merchant website   │ ────────────────────────────────►│  Koodify inline JS    │
│  + backend webhook  │◄──── payment.success (signed) ───│  + api.koodify.ng     │
└─────────────────────┘                                  └──────────┬───────────┘
                                                                    │
                         enrich · route bank · split · receipts · reconcile
                                                                    │
                                                                    ▼
                                                         ┌──────────────────────┐
                                                         │       Zainpay        │
                                                         │  VA · transfer · WH  │
                                                         └──────────────────────┘
```

**Two paths to the same customer outcome**

1. **Path 1:** Zainpay deposit webhook → Koodify → merchant webhook / emails / split  
2. **Path 2:** Reconciler or browser status poll confirms paid → same side effects (once)

---

## 5. What Koodify deliberately does *not* claim

- **Cannot block a wrong-amount NIP credit before it hits the bank.** Detection and fulfillment gating happen after Zainpay status/webhooks.
- **No merchant-facing Zainpay “reverse payment” API** is documented; DVA mismatch auto-refund (when Zainpay applies it) is upstream. Temp-VA wrong amounts may need bank/Zainpay ops (as seen in live tests).
- **Direct Zainpay mode** on the inline widget skips Koodify extras (routing, receipts, enriched webhooks).

---

## 6. Quick start for a new merchant

1. Obtain a publishable `pk_live_…` for your zainbox (checkout).  
2. Obtain a merchant secret `sk_live_…` for server APIs (wallet, ledger, settlements, billing) — **not** the platform `GATEWAY_API_KEY`.  
3. Embed `https://js.koodify.ng/v1/koodify-inline.js` and call `koodifyInitPayment` (**omit `bankType`**).  
4. Set `callBackUrl` to your HTTPS webhook; verify `X-Koodify-Signature` on **raw body**.  
5. Fulfill only when status shows `data.koodify.isPaid === true` (or you received `payment.success`).  
6. Call privileged reads with `X-Api-Key: sk_live_…` (scoped to your zainbox).  
7. Prefer canonical `/api/v1/…` paths; ignore deprecated aliases.

---

## 7. Related documents

| Doc | Purpose |
|-----|---------|
| [`FEATURES.md`](../FEATURES.md) | Full feature catalogue (what / example / flow) |
| [`docs/inline-checkout.html`](inline-checkout.html) | Checkout + merchant webhook contract |
| [`docs/api.html`](api.html) | Gateway API reference |
| [`samples/node/README.md`](../samples/node/README.md) | Raw-body HMAC Node sample |
| [`samples/php/README.md`](../samples/php/README.md) | PHP webhook receiver |
| Visual Issue Pack | Original 10 DX illustrations (external pack) |

---

*Document version: aligned with Koodify gateway work through payment-status normalization, API error enrichment, transfer idempotency, Node webhook sample, and `/api/v1` deprecation policy (Sunset 2027-05-01).*
