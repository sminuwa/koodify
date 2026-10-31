/**
 * Demo static server + proxies:
 * - /proxy/koodify/* → local koodify gateway (payer logging)
 * - /proxy/sandbox|live/* → Zainpay direct (legacy)
 */
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..", "demo");
const port = Number(process.env.PORT || 5179);

function loadEnvFile(filePath) {
  if (!fs.existsSync(filePath)) return;
  for (const line of fs.readFileSync(filePath, "utf8").split("\n")) {
    const t = line.trim();
    if (!t || t.startsWith("#")) continue;
    const i = t.indexOf("=");
    if (i < 0) continue;
    const k = t.slice(0, i).trim();
    let v = t.slice(i + 1).trim();
    if (
      (v.startsWith('"') && v.endsWith('"')) ||
      (v.startsWith("'") && v.endsWith("'"))
    ) {
      v = v.slice(1, -1);
    }
    if (!(k in process.env)) process.env[k] = v;
  }
}

loadEnvFile(path.join(__dirname, "..", "..", "..", ".env"));
loadEnvFile(path.join(__dirname, "..", ".env"));

const KoodifyUpstream =
  process.env.KOODIFY_BASE_URL ||
  `http://127.0.0.1:${(process.env.GATEWAY_BIND || "0.0.0.0:8080").split(":").pop()}`;
const GatewayKey = process.env.GATEWAY_API_KEY || "";

const HOSTS = {
  sandbox: "https://sandbox.zainpay.ng",
  live: "https://api.zainpay.ng",
};

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
};

function send(res, status, body, headers = {}) {
  res.writeHead(status, headers);
  res.end(body);
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on("data", (c) => chunks.push(c));
    req.on("end", () => resolve(Buffer.concat(chunks)));
    req.on("error", reject);
  });
}

async function forward(req, res, url, extraHeaders = {}) {
  const headers = { ...extraHeaders };
  for (const name of [
    "authorization",
    "content-type",
    "accept",
    "x-api-key",
  ]) {
    if (req.headers[name] && !headers[name] && !headers[name.toLowerCase()]) {
      headers[name] = req.headers[name];
    }
  }

  let body;
  if (req.method !== "GET" && req.method !== "HEAD") {
    body = await readBody(req);
  }

  let upstreamRes;
  try {
    upstreamRes = await fetch(url, { method: req.method, headers, body });
  } catch (err) {
    return send(
      res,
      502,
      JSON.stringify({
        code: "proxy_error",
        description: err.message || "Upstream unreachable",
        upstream: url,
      }),
      {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*",
      }
    );
  }

  const buf = Buffer.from(await upstreamRes.arrayBuffer());
  res.writeHead(upstreamRes.status, {
    "Content-Type":
      upstreamRes.headers.get("content-type") || "application/json",
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Headers":
      "authorization, content-type, accept, x-api-key",
    "Access-Control-Allow-Methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS",
  });
  res.end(buf);
}

function serveStatic(req, res) {
  let reqPath = decodeURIComponent((req.url || "/").split("?")[0]);
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
}

const server = http.createServer(async (req, res) => {
  if (req.method === "OPTIONS") {
    return send(res, 204, "", {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Headers":
        "authorization, content-type, accept, x-api-key",
      "Access-Control-Allow-Methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    });
  }

  if (req.url?.startsWith("/proxy/koodify")) {
    const targetPath = req.url.replace(/^\/proxy\/koodify/, "") || "/";
    const headers = {};
    if (GatewayKey) headers["X-Api-Key"] = GatewayKey;
    return forward(req, res, KoodifyUpstream.replace(/\/$/, "") + targetPath, headers);
  }

  if (req.url?.startsWith("/proxy/sandbox") || req.url?.startsWith("/proxy/live")) {
    const mode = req.url.startsWith("/proxy/live") ? "live" : "sandbox";
    const targetPath = req.url.replace(/^\/proxy\/(sandbox|live)/, "") || "/";
    return forward(req, res, HOSTS[mode] + targetPath);
  }

  if (req.method === "GET" || req.method === "HEAD") {
    return serveStatic(req, res);
  }

  send(res, 405, "Method not allowed");
});

server.listen(port, () => {
  console.log(`Koodify inline demo → http://localhost:${port}`);
  console.log(`Proxy koodify → ${KoodifyUpstream} (X-Api-Key ${GatewayKey ? "set" : "MISSING"})`);
  console.log(`Proxy sandbox → /proxy/sandbox/*`);
  console.log(`Proxy live    → /proxy/live/*`);
});
