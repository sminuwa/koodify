/**
 * Minimal Koodify merchant webhook receiver (Node).
 *
 *   KOODIFY_SECRET_KEY=sk_live_… node server.js
 *
 * Verifies X-Koodify-Signature with the same sk_… used as X-Api-Key.
 */

const http = require("http");
const { verifyKoodifySignature } = require("./verify");

const PORT = Number(process.env.PORT || 8787);
const SECRET =
  process.env.KOODIFY_SECRET_KEY ||
  process.env.MERCHANT_WEBHOOK_SECRET ||
  process.env.GATEWAY_API_KEY ||
  "";

const server = http.createServer((req, res) => {
  if (req.method === "GET" && req.url === "/health") {
    res.writeHead(200, { "Content-Type": "text/plain" });
    res.end("ok");
    return;
  }

  if (req.method !== "POST" || !req.url.startsWith("/webhooks/koodify")) {
    res.writeHead(404, { "Content-Type": "text/plain" });
    res.end("not found");
    return;
  }

  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    const rawBody = Buffer.concat(chunks);
    const signature = req.headers["x-koodify-signature"];
    const event = req.headers["x-koodify-event"] || "";

    if (!SECRET) {
      console.warn(
        "[koodify] KOODIFY_SECRET_KEY unset — skipping signature check (demo only)"
      );
    } else if (!verifyKoodifySignature(SECRET, rawBody, signature)) {
      console.warn("[koodify] invalid signature");
      res.writeHead(401, { "Content-Type": "text/plain" });
      res.end("invalid signature");
      return;
    }

    let payload = {};
    try {
      payload = JSON.parse(rawBody.toString("utf8"));
    } catch {
      payload = { raw: rawBody.toString("utf8") };
    }

    console.log("[koodify] webhook", { event, reference: payload.reference, status: payload.status });
    res.writeHead(200, { "Content-Type": "text/plain" });
    res.end("ok");
  });
});

server.listen(PORT, () => {
  console.log(`Koodify webhook listening on :${PORT}/webhooks/koodify`);
  if (!SECRET) {
    console.warn("Set KOODIFY_SECRET_KEY (sk_…) to enforce X-Koodify-Signature checks.");
  }
});
