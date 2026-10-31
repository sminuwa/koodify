/**
 * Tiny static file server for live-test/ (pure HTML/JS page).
 * No .env — enter GATEWAY_API_KEY in the form.
 */
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..", "live-test");
const distJs = path.join(__dirname, "..", "dist", "koodify-inline.js");
const port = Number(process.env.LIVE_TEST_PORT || process.env.PORT || 5180);

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};

function send(res, status, body, headers = {}) {
  res.writeHead(status, headers);
  res.end(body);
}

if (!fs.existsSync(distJs)) {
  throw new Error("Missing dist/koodify-inline.js — run npm run build first");
}
fs.copyFileSync(distJs, path.join(root, "koodify-inline.js"));

const server = http.createServer((req, res) => {
  const url = (req.url || "/").split("?")[0];

  if (req.method !== "GET" && req.method !== "HEAD") {
    return send(res, 405, "Method not allowed");
  }

  let reqPath = decodeURIComponent(url);
  if (reqPath === "/") reqPath = "/index.html";
  const file = path.normalize(path.join(root, reqPath));
  if (!file.startsWith(root)) {
    return send(res, 403, "Forbidden");
  }
  if (!fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    return send(res, 404, "Not found");
  }
  const ext = path.extname(file);
  send(res, 200, fs.readFileSync(file), {
    "Content-Type": MIME[ext] || "application/octet-stream",
    "Cache-Control": "no-store",
  });
});

server.listen(port, () => {
  console.log(`Koodify live test → http://localhost:${port}`);
  console.log(`Open that URL and paste your GATEWAY_API_KEY in the form.`);
});
