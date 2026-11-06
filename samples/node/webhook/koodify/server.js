/**
 * Demo merchant webhook for Koodify checkout `callBackUrl`.
 *
 *   npm install
 *   MERCHANT_WEBHOOK_SECRET=… node server.js
 *
 * Point checkout at: http://localhost:3091/webhooks/koodify/
 *
 * Uses express.raw() so HMAC runs on the original body bytes — the fix for
 * the common Node trap of verifying JSON.stringify(parsedBody).
 *
 * Docs: docs/inline-checkout.html#merchant-webhook
 * PHP twin: samples/php/webhook/koodify/
 */

const express = require("express");
const fs = require("fs");
const path = require("path");
const { verifyKoodifySignature } = require("./verify");

const PORT = Number(process.env.PORT || 3091);
const SECRET =
  process.env.MERCHANT_WEBHOOK_SECRET ||
  process.env.GATEWAY_API_KEY ||
  "";
const LOG_FILE = path.join(__dirname, "webhook-log.jsonl");

const app = express();

// Raw body required for signature verification. Do NOT use express.json() here.
app.post(
  ["/webhooks/koodify", "/webhooks/koodify/"],
  express.raw({ type: "*/*", limit: "1mb" }),
  (req, res) => {
    const rawBody = req.body; // Buffer
    if (!Buffer.isBuffer(rawBody) || rawBody.length === 0) {
      return res.status(400).type("text").send("empty body\n");
    }

    const signature = req.get("X-Koodify-Signature");
    if (SECRET) {
      if (!verifyKoodifySignature(SECRET, rawBody, signature)) {
        return res.status(401).type("text").send("invalid signature\n");
      }
    } else {
      console.warn(
        "[koodify] MERCHANT_WEBHOOK_SECRET / GATEWAY_API_KEY unset — skipping signature check (demo only)"
      );
    }

    let payload;
    try {
      payload = JSON.parse(rawBody.toString("utf8"));
    } catch {
      return res.status(400).type("text").send("invalid JSON\n");
    }

    // WRONG — never do this:
    // const bad = crypto.createHmac("sha256", SECRET).update(JSON.stringify(payload)).digest("hex");

    const entry = {
      receivedAt: new Date().toISOString(),
      reference: payload.reference ?? null,
      event: payload.event ?? null,
      payload,
    };

    try {
      fs.appendFileSync(LOG_FILE, `${JSON.stringify(entry)}\n`, { flag: "a" });
    } catch (err) {
      console.error("log write failed", err);
      return res.status(500).type("text").send("log write failed\n");
    }

    console.log(
      `ok event=${entry.event} reference=${entry.reference} bytes=${rawBody.length}`
    );
    return res.status(200).type("text").send("OK\n");
  }
);

app.get("/health", (_req, res) => res.type("text").send("ok\n"));

app.listen(PORT, () => {
  console.log(`Koodify webhook sample listening on http://localhost:${PORT}/webhooks/koodify/`);
  if (!SECRET) {
    console.warn("Set MERCHANT_WEBHOOK_SECRET to enforce X-Koodify-Signature checks.");
  }
});
