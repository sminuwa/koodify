import {
  resolveBaseUrl,
  isGatewayCheckout,
  explainNetworkError,
  initializeInlinePayment,
  generateTemporaryVa,
  fetchVaPaymentStatus,
} from "./api.js";
import { createCheckoutModal } from "./modal.js";

const REQUIRED = [
  "amount",
  "txnRef",
  "mobileNumber",
  "emailAddress",
  "callBackUrl",
];

/** Publishable merchant key required for gateway checkout. */
function resolvePublicKey(config) {
  const pk =
    config?.publicKey ??
    config?.public_key ??
    config?.koodifyKey ??
    "";
  if (pk != null && String(pk).trim() !== "") return String(pk).trim();
  return "";
}

function missingFields(config) {
  if (!config || typeof config !== "object") return REQUIRED.slice();
  return REQUIRED.filter((k) => {
    const v = config[k];
    return v == null || String(v).trim() === "";
  });
}

function formatAmount(amount) {
  const n = parseFloat(amount);
  if (!Number.isFinite(n)) return String(amount);
  return Number.isInteger(n) ? String(n) : n.toFixed(2);
}

function emit(cb, payload) {
  try {
    if (typeof cb === "function") cb(payload);
  } catch {
    /* merchant callback errors should not break teardown */
  }
}

/** Accept `customParam` or `customParams` as a plain object of merchant key/values. */
function normalizeCustomParam(config) {
  const raw = config?.customParam ?? config?.customParams;
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
  return { ...raw };
}

/**
 * Open Koodify bank-transfer checkout.
 *
 * Defaults (CDN build): talks to https://api.koodify.ng —
 * merchants pass a publishable `publicKey` (`pk_live_…`) tied to their zainbox
 * in Koodify. Optional `baseUrl` for local/proxy; `auth: "zainpay"` for direct Zainpay.
 *
 * @param {object} paymentConfig
 * @param {function} callback - ({ status, reference, description, payer?, customParam? }) => void
 * @param {string} [apiKey] - optional; ignored for koodify gateway checkout
 * @param {"sandbox"|"live"} [mode="sandbox"] - only used for direct Zainpay hosts
 * @returns {Promise<object>}
 */
export function koodifyInitPayment(paymentConfig, callback, apiKey, mode) {
  return new Promise((resolve, reject) => {
    const missing = missingFields(paymentConfig);
    if (missing.length) {
      const payload = {
        status: "failed",
        reference: paymentConfig?.txnRef || "",
        description: `Missing required field(s): ${missing.join(", ")}`,
      };
      emit(callback, payload);
      reject(payload);
      return;
    }
    const publicKey = resolvePublicKey(paymentConfig);
    if (!publicKey) {
      const payload = {
        status: "failed",
        reference: paymentConfig?.txnRef || "",
        description: "Missing publicKey (publishable pk_live_… from Koodify)",
      };
      emit(callback, payload);
      reject(payload);
      return;
    }
    let customParam = normalizeCustomParam(paymentConfig);
    const currency = paymentConfig.currencyCode || "NGN";
    const resolvedMode = mode === "live" ? "live" : "sandbox";
    const useGateway = isGatewayCheckout(paymentConfig);
    const baseUrl = resolveBaseUrl(
      resolvedMode,
      paymentConfig.baseUrl,
      useGateway
    );
    // Direct Zainpay still needs a Bearer key; gateway uses publicKey → zainbox.
    if (!useGateway && (!apiKey || String(apiKey).trim() === "")) {
      const payload = {
        status: "failed",
        reference: paymentConfig.txnRef,
        description: "Missing API key for direct Zainpay calls",
      };
      emit(callback, payload);
      reject(payload);
      return;
    }
    // Gateway: omit bankType so Koodify auto-routes. Direct Zainpay still needs a bank key.
    const explicitBank =
      paymentConfig.bankType != null && String(paymentConfig.bankType).trim() !== ""
        ? String(paymentConfig.bankType).trim()
        : "";
    const bankType = useGateway
      ? explicitBank
      : explicitBank || "gtBank";
    let settled = false;
    let succeeded = false;
    let transferView = null;
    let pollTimers = [];
    let successCloseTimer = null;
    let lastPayer = null;

    const finish = (payload, ok) => {
      if (settled) return;
      settled = true;
      clearPolls();
      transferView?.stopTimer?.();
      if (lastPayer && !payload.payer) payload.payer = lastPayer;
      if (customParam && !payload.customParam) payload.customParam = customParam;
      emit(callback, payload);
      if (ok) resolve(payload);
      else reject(payload);
    };

    const clearPolls = () => {
      pollTimers.forEach((id) => clearTimeout(id));
      pollTimers = [];
      if (successCloseTimer != null) {
        clearTimeout(successCloseTimer);
        successCloseTimer = null;
      }
    };

    const modal = createCheckoutModal({
      amount: paymentConfig.amount,
      currency,
      email: paymentConfig.emailAddress,
      logoUrl: paymentConfig.logoUrl,
      onClose: (reason) => {
        if (settled) return;
        if (reason === "success") {
          finish(
            {
              status: "success",
              reference: paymentConfig.txnRef,
              description: "Payment was successful",
            },
            true
          );
          modal.destroy();
          return;
        }
        const status = reason === "cancelled" ? "cancelled" : "failed";
        finish(
          {
            status,
            reference: paymentConfig.txnRef,
            description:
              status === "cancelled"
                ? "Payment cancelled"
                : "Payment failed",
          },
          false
        );
        modal.destroy();
      },
    });

    async function start() {
      modal.setLoading();
      try {
        const payload = {
          amount: formatAmount(paymentConfig.amount),
          txnRef: paymentConfig.txnRef,
          mobileNumber: paymentConfig.mobileNumber,
          emailAddress: paymentConfig.emailAddress,
          callBackUrl: paymentConfig.callBackUrl,
          currencyCode: currency,
          logoUrl: paymentConfig.logoUrl,
          paymentChannels: ["bank_transfer"],
          allowRecurringPayment: !!paymentConfig.allowRecurringPayment,
        };
        if (useGateway) {
          // Koodify resolves publicKey → zainbox server-side (zainboxCode alone is rejected).
          payload.publicKey = publicKey;
        } else {
          // Direct Zainpay still needs the real zainbox code.
          payload.zainboxCode =
            paymentConfig.zainboxCode || publicKey;
        }
        if (paymentConfig.name) payload.name = paymentConfig.name;
        if (paymentConfig.payer) payload.payer = paymentConfig.payer;
        if (customParam) payload.customParam = customParam;
        // Zainbox Payment Split — gateway-only; open checkout accepts splitPlanId only
        // (ad-hoc paymentSplit.shares must be created server-side via ApiKeyAuth).
        if (useGateway) {
          const planId =
            paymentConfig.splitPlanId || paymentConfig.split_plan_id;
          if (planId != null && String(planId).trim() !== "") {
            payload.splitPlanId = String(planId).trim();
          }
        }

        const init = await initializeInlinePayment({
          baseUrl,
          apiKey,
          config: paymentConfig,
          payload,
        });
        if (init.payer) lastPayer = init.payer;
        if (init.customParam && typeof init.customParam === "object") {
          customParam = { ...(customParam || {}), ...init.customParam };
        }

        if (!init.sessionKey && !init.duplicate) {
          throw new Error("Payment session was not returned by the API");
        }

        // If duplicate and no session key, still try bankType flow may fail —
        // surface a clear error.
        if (!init.sessionKey) {
          throw new Error(
            "Duplicate transaction reference — provide a fresh txnRef"
          );
        }

        const va = await generateTemporaryVa({
          baseUrl,
          apiKey,
          config: paymentConfig,
          sessionKey: init.sessionKey,
          bankType,
        });
        if (va.payer) lastPayer = va.payer;

        transferView = modal.setTransfer(va, {
          onExpired: () => {
            clearPolls();
            transferView?.showError(
              "This account expired. Close and start a new payment."
            );
            transferView?.setListening(false);
          },
          onCheckNow: () => checkPaymentOnce(va, { manual: true }),
        });

        // Auto-detect transfer — no button click required
        startAutoPoll(va);
      } catch (err) {
        const message = err?.network
          ? explainNetworkError(err, resolvedMode)
          : err.message || "Payment initialization failed";
        modal.setError(message, {
          retryable: true,
          onRetry: () => start(),
        });
      }
    }

    const POLL_MS = 3000;

    function startAutoPoll(va) {
      clearPolls();
      transferView?.showError("");
      transferView?.setListening(true);
      transferView?.setStatus("Listening…");

      const loop = async () => {
        if (settled || succeeded) return;
        await checkPaymentOnce(va, { manual: false });
        if (settled || succeeded) return;
        pollTimers.push(setTimeout(loop, POLL_MS));
      };
      loop();
    }

    async function checkPaymentOnce(va, { manual }) {
      if (settled || succeeded) return;
      if (manual) {
        transferView?.showError("");
        transferView?.setStatus("Checking…");
      }

      try {
        const result = await fetchVaPaymentStatus({
          baseUrl,
          apiKey,
          config: paymentConfig,
          txnRef: va.txnRef || paymentConfig.txnRef,
        });
        if (result.payer) lastPayer = result.payer;
        if (result.customParam && typeof result.customParam === "object") {
          customParam = { ...(customParam || {}), ...result.customParam };
        }
        if (result.status === "success") {
          succeeded = true;
          clearPolls();
          transferView?.stopTimer?.();
          transferView?.setListening(false);
          modal.setSuccess(va.txnRef || paymentConfig.txnRef);
          // Keep this timer out of pollTimers so polling cleanup cannot reset it.
          successCloseTimer = setTimeout(() => {
            successCloseTimer = null;
            finish(
              {
                status: "success",
                reference: va.txnRef || paymentConfig.txnRef,
                description: "Payment was successful",
                payer: lastPayer || undefined,
                customParam: customParam || undefined,
              },
              true
            );
            modal.destroy();
          }, 10000);
          return;
        }
        if (result.status === "failed") {
          clearPolls();
          transferView?.setListening(false);
          transferView?.setStatus("Failed");
          transferView?.showError(
            result.description ||
              "Payment failed. Try again or contact support."
          );
          return;
        }
        transferView?.setStatus(manual ? "Not yet" : "Listening…");
        if (manual) {
          transferView?.showError(
            "No payment yet — keep this open; we’ll keep checking automatically."
          );
        }
      } catch {
        if (manual) {
          transferView?.showError(
            "Couldn’t reach status API. We’ll keep retrying automatically."
          );
        }
        transferView?.setStatus("Listening…");
      }
    }

    start();
  });
}

export default { koodifyInitPayment };
