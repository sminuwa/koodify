/**
 * Attach window.koodifyInitPayment from the IIFE export,
 * and copy the bundle into demo/ + live-test/.
 */
const fs = require("fs");
const path = require("path");

const out = path.join(__dirname, "..", "dist", "koodify-inline.js");
const demoOut = path.join(__dirname, "..", "demo", "koodify-inline.js");
const liveOut = path.join(__dirname, "..", "live-test", "koodify-inline.js");
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
fs.mkdirSync(path.dirname(liveOut), { recursive: true });
fs.copyFileSync(out, liveOut);
console.log("built", out, "(" + code.length + " bytes)");
console.log("copied", demoOut);
console.log("copied", liveOut);
