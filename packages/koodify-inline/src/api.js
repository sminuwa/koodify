const HOSTS = {
  koodify: "https://api.koodify.ng",
  zainpaySandbox: "https://sandbox.zainpay.ng",
  zainpayLive: "https://api.zainpay.ng",
};

/**
 * Resolve API origin.
 * Hosted CDN checkout defaults to api.koodify.ng — merchants need not pass baseUrl.
 * Override for local/demo (`http://localhost:8080`, `/proxy/koodify`) or direct Zainpay.
 *
 * @param {"sandbox"|"live"} mode
 * @param {string} [override]
 * @param {boolean} [useGateway=true]
 */
export function resolveBaseUrl(mode, override, useGateway = true) {
  if (override && String(override).trim()) {
    return String(override).replace(/\/$/, "");
  }
  if (useGateway) {
    return HOSTS.koodify;
  }
  return mode === "live" ? HOSTS.zainpayLive : HOSTS.zainpaySandbox;
}

/**
 * Gateway is the default. Pass auth: "zainpay" | "direct" for raw Zainpay.
 * @param {{ auth?: string, baseUrl?: string }} config
 */
export function isGatewayCheckout(config) {
  const auth = String(config?.auth ?? "gateway").toLowerCase();
  if (auth === "zainpay" || auth === "direct") return false;
  if (auth === "gateway" || auth === "koodify" || auth === "") return true;
  const base = String(config?.baseUrl || "");
  return /koodify|localhost:8080|127\.0\.0\.1:8080|\/proxy\/koodify/i.test(base);
}

/**
 * @param {{ auth?: string, baseUrl?: string }} config
 * @param {string} apiKey
 */
export function buildAuthHeaders(config, apiKey) {
  const headers = {
    Accept: "application/json",
    "Content-Type": "application/json",
  };
  const key = String(apiKey || "").trim();
  if (!key) return headers;

  // Gateway checkout is publicKey-based — never put a secret in the browser.
  if (isGatewayCheckout(config)) {
    return headers;
  }
  headers.Authorization = `Bearer ${key}`;
  return headers;
}

export function explainNetworkError(err, _mode) {
  const msg = err?.message || String(err);
  if (/failed to fetch|networkerror|load failed/i.test(msg)) {
    return "Browser blocked the API call (CORS). Check api.koodify.ng CORS, or set baseUrl for local/proxy.";
  }
  return msg;
}

function describeApiBody(body) {
  if (!body) return "";
  if (typeof body.description === "string" && body.description.trim()) {
    return body.description;
  }
  if (typeof body.message === "string" && body.message.trim()) {
    return body.message;
  }
  if (typeof body === "object") {
    const parts = [];
    for (const [field, val] of Object.entries(body)) {
      if (Array.isArray(val) && val[0]?.code) {
        parts.push(`${field}: ${val[0].code}`);
      }
    }
    if (parts.length) return parts.join("; ");
  }
  return "";
}

function extractSessionKey(data) {
  if (!data) return "";
  if (typeof data === "string") {
    try {
      const url = new URL(data);
      return url.searchParams.get("e") || "";
    } catch {
      return data;
    }
  }
  if (typeof data === "object") {
    return (
      data.e ||
      data.sessionKey ||
      data.token ||
      extractSessionKey(data.value) ||
      ""
    );
  }
  return "";
}

async function parseJson(res) {
  const text = await res.text();
  let body = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = { description: text || res.statusText };
  }
  return body;
}

export async function initializeInlinePayment({
  baseUrl,
  apiKey,
  config,
  payload,
}) {
  let res;
  try {
    res = await fetch(`${baseUrl}/zainbox/card/initialize/payment/inline`, {
      method: "POST",
      headers: buildAuthHeaders(config, apiKey),
      body: JSON.stringify(payload),
    });
  } catch (err) {
    const e = new Error(err.message || "Failed to fetch");
    e.cause = err;
    e.network = true;
    throw e;
  }
  const body = await parseJson(res);
  const code = body?.code != null ? String(body.code) : "";
  const description = describeApiBody(body);

  if (!res.ok || (code && code !== "00" && code !== "04")) {
    const duplicate =
      code === "04" || /duplicate\s+txn\s+reference/i.test(description);
    if (!duplicate) {
      const err = new Error(
        description || `Payment initialization failed (${res.status})`
      );
      err.code = code;
      err.body = body;
      throw err;
    }
  }

  const sessionKey = extractSessionKey(body?.data);
  return {
    sessionKey,
    body,
    duplicate: code === "04",
    payer: body?.data?.payer || body?.data?.metadata?.payer || null,
    customParam:
      body?.data?.customParam ||
      body?.data?.metadata?.customParam ||
      body?.data?.customParams ||
      body?.data?.metadata?.customParams ||
      null,
  };
}

export async function generateTemporaryVa({
  baseUrl,
  apiKey,
  config,
  sessionKey,
  bankType,
}) {
  // Gateway auto-routes when bankType is omitted. Direct Zainpay still needs a bank.
  const params = new URLSearchParams();
  params.set("e", sessionKey);
  if (bankType != null && String(bankType).trim() !== "") {
    params.set("bankType", `"${String(bankType).trim()}"`);
  }
  const url = `${baseUrl}/zainbox/card/generate-temporary-virtual-account?${params.toString()}`;
  const res = await fetch(url, {
    headers: buildAuthHeaders(config, apiKey),
  });
  const body = await parseJson(res);
  if (!res.ok || String(body?.code) !== "00" || !body?.data) {
    const err = new Error(
      body?.description || "Could not create transfer account"
    );
    err.code = body?.code;
    err.body = body;
    throw err;
  }
  const data = body.data;
  const kobo = parseFloat(data.amount);
  const naira = Number.isFinite(kobo) ? kobo / 100 : NaN;
  return {
    bankType: data.bankType || bankType || "",
    accountNumber: String(data.accountNumber || ""),
    accountName: String(data.accountName || ""),
    amountNaira: Number.isInteger(naira)
      ? String(naira)
      : Number.isFinite(naira)
        ? naira.toFixed(2)
        : String(data.amount ?? ""),
    paymentStatus: data.paymentStatus || "pending",
    txnRef: data.txnRef || "",
    timeToLive: Number(data.timeToLive) || 1800,
    email: data.email || "",
    callBackUrl: data.callBackUrl || "",
    payer: data.payer || null,
  };
}

export async function fetchVaPaymentStatus({
  baseUrl,
  apiKey,
  config,
  txnRef,
}) {
  const url = `${baseUrl}/zainbox/card/temporary-virtual-account-payment-status?txnRef=${encodeURIComponent(txnRef)}`;
  const res = await fetch(url, {
    headers: buildAuthHeaders(config, apiKey),
  });
  const body = await parseJson(res);
  const status = String(body?.data?.paymentStatus || "").toLowerCase();
  return {
    status,
    callBackUrl: body?.data?.callBackUrl || "",
    description: body?.description || "",
    payer: body?.data?.payer || body?.data?.metadata?.payer || null,
    customParam:
      body?.data?.customParam ||
      body?.data?.metadata?.customParam ||
      body?.data?.customParams ||
      body?.data?.metadata?.customParams ||
      null,
    body,
  };
}
