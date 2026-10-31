/**
 * Attach window.koodifyInitPayment from the IIFE export,
 * and copy the bundle into demo/ (serve only hosts that folder).
 */
const fs = require("fs");
const path = require("path");

const out = path.join(__dirname, "..", "dist", "koodify-inline.js");
const demoOut = path.join(__dirname, "..", "demo", "koodify-inline.js");
let code = fs.readFileSync(out, "utf8");

const bridge = `
;(function (root) {
  var api = root.KoodifyInline;
  if (!api) return;
  root.koodifyInitPayment = api.koodifyInitPayment;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }
})(typeof window !== "undefined" ? window : globalThis);
`;

if (!code.includes("koodifyInitPayment = api.koodifyInitPayment")) {
  code += bridge;
  fs.writeFileSync(out, code);
}

fs.copyFileSync(out, demoOut);
console.log("built", out, "(" + code.length + " bytes)");
console.log("copied", demoOut);
