/**
 * Verify Koodify outbound webhook signature.
 *
 * CRITICAL: pass the *raw request bytes* (Buffer / Uint8Array / string as
 * received on the wire). Never HMAC(JSON.stringify(parsedObject)) — key order
 * and whitespace will not match what Koodify signed.
 *
 * Signature header: X-Koodify-Signature (hex HMAC-SHA256)
 * Secret: merchant sk_… (same as X-Api-Key)
 */


const crypto = require("crypto");

/**
 * @param {string} secret
 * @param {Buffer|string|Uint8Array} rawBody
 * @param {string|undefined|null} signatureHeader
 * @returns {boolean}
 */
function verifyKoodifySignature(secret, rawBody, signatureHeader) {
  if (!secret || rawBody == null || !signatureHeader) {
    return false;
  }
  const provided = String(signatureHeader).trim().toLowerCase().replace(/^sha256=/, "");
  if (!/^[0-9a-f]+$/.test(provided)) {
    return false;
  }

  const expected = crypto
    .createHmac("sha256", secret)
    .update(rawBody)
    .digest("hex");

  const a = Buffer.from(expected, "utf8");
  const b = Buffer.from(provided, "utf8");
  if (a.length !== b.length) {
    return false;
  }
  return crypto.timingSafeEqual(a, b);
}

module.exports = { verifyKoodifySignature };
