const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const crypto = require("crypto");
const { verifyKoodifySignature } = require("./verify");

describe("verifyKoodifySignature", () => {
  const secret = "test_secret";
  const raw = Buffer.from('{"event":"payment.success","reference":"T1"}', "utf8");

  it("accepts HMAC of raw bytes", () => {
    const sig = crypto.createHmac("sha256", secret).update(raw).digest("hex");
    assert.equal(verifyKoodifySignature(secret, raw, sig), true);
    assert.equal(verifyKoodifySignature(secret, raw, `sha256=${sig}`), true);
  });

  it("rejects HMAC of JSON.stringify(parsed) when spacing differs", () => {
    const parsed = JSON.parse(raw.toString("utf8"));
    // Re-serialized object — different bytes than Koodify signed.
    const rebuilt = JSON.stringify(parsed);
    const wrongSig = crypto.createHmac("sha256", secret).update(rebuilt).digest("hex");
    // Only fails when rebuilt !== raw; force a spaced variant:
    const spaced = Buffer.from('{ "event": "payment.success", "reference": "T1" }', "utf8");
    const spacedSig = crypto.createHmac("sha256", secret).update(spaced).digest("hex");
    assert.equal(verifyKoodifySignature(secret, raw, spacedSig), false);
    assert.ok(wrongSig); // silence unused if rebuilt matched raw by chance
  });

  it("rejects wrong secret / tampered body", () => {
    const sig = crypto.createHmac("sha256", secret).update(raw).digest("hex");
    assert.equal(verifyKoodifySignature("other", raw, sig), false);
    assert.equal(verifyKoodifySignature(secret, Buffer.from("{}"), sig), false);
  });
});
