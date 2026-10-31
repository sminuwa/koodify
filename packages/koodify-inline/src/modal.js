import cssText from "./styles.css";
import koodifyLogoUrl from "../assets/koodify-logo.png";
import {
  iconClose,
  iconCard,
  iconPerson,
  iconDoc,
  iconCopy,
  iconClock,
  iconWarn,
  iconRefresh,
  iconLock,
  bankMark,
  bankFullName,
  artScene,
} from "./icons.js";

const BANK_LABELS = {
  gtBank: "GTBank",
  moniepoint: "Moniepoint",
  wemaBank: "Wema Bank",
  polaris: "Polaris Bank",
  fcmb: "FCMB",
  zainMFB: "Zain MFB",
};

function formatMoney(currency, amount) {
  const n = Number(amount);
  if (!Number.isFinite(n)) return `${currency} ${amount}`;
  return `${currency} ${n.toLocaleString("en-NG", {
    minimumFractionDigits: Number.isInteger(n) ? 0 : 2,
    maximumFractionDigits: 2,
  })}`;
}

function formatExact(currency, amount) {
  const n = Number(amount);
  if (!Number.isFinite(n)) return `${currency} ${amount}`;
  return `${currency} ${n.toLocaleString("en-NG", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

function formatCountdown(seconds) {
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${String(r).padStart(2, "0")}`;
}

function ensureStyles() {
  if (document.querySelector("style[data-koodify-inline]")) return;
  const style = document.createElement("style");
  style.setAttribute("data-koodify-inline", "true");
  style.textContent = cssText;
  document.head.appendChild(style);
}

function el(html) {
  const t = document.createElement("template");
  t.innerHTML = html.trim();
  return t.content.firstElementChild;
}

function copyButton(attr) {
  return `<button type="button" class="kd-copy" ${attr} aria-label="Copy">
    ${iconCopy}<span>Copy</span>
  </button>`;
}

function escapeHtml(value) {
  return String(value || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function createCheckoutModal({
  amount,
  currency,
  email,
  logoUrl,
  onClose,
}) {
  ensureStyles();

  const money = formatMoney(currency, amount);
  const exact = formatExact(currency, amount);
  const safeEmail = escapeHtml(email);

  const root = el(`
    <div class="kd-root" role="dialog" aria-modal="true" aria-label="Koodify bank transfer">
      <div class="kd-overlay">
        <div class="kd-sheet">
          <div class="kd-progress is-active" data-kd-progress role="progressbar" aria-label="Waiting for payment">
            <span class="kd-progress-bar"></span>
          </div>
          <header class="kd-top">
            <div class="kd-brand">
              <img
                class="kd-brand-logo"
                src="${logoUrl || koodifyLogoUrl}"
                alt="Koodify"
                width="132"
                height="36"
              />
              <div class="kd-brand-sub">Bank Transfer Checkout</div>
            </div>
            <button type="button" class="kd-close" aria-label="Close" data-kd-close>${iconClose}</button>
          </header>
          <div class="kd-body" data-kd-body></div>
          <footer class="kd-foot">
            ${iconLock}
            <span>Powered by <strong>Koodify</strong></span>
          </footer>
        </div>
      </div>
    </div>
  `);

  const body = root.querySelector("[data-kd-body]");
  const progress = root.querySelector("[data-kd-progress]");

  let closed = false;
  let successPending = false;
  const close = (reason) => {
    if (closed) return;
    closed = true;
    root.remove();
    document.querySelector("style[data-koodify-inline]")?.remove();
    onClose?.(reason);
  };
  const dismiss = () => close(successPending ? "success" : "cancelled");

  root.querySelector("[data-kd-close]").addEventListener("click", () => {
    dismiss();
  });
  root.querySelector(".kd-overlay").addEventListener("click", (e) => {
    if (e.target.classList.contains("kd-overlay")) dismiss();
  });
  const onKey = (e) => {
    if (e.key === "Escape") dismiss();
  };
  document.addEventListener("keydown", onKey);

  const destroy = () => {
    document.removeEventListener("keydown", onKey);
    if (!closed) {
      closed = true;
      root.remove();
    }
  };

  document.body.appendChild(root);

  const setProgress = (active) => {
    if (progress) progress.classList.toggle("is-active", !!active);
  };

  const api = {
    destroy,
    close,
    setLoading(message) {
      setProgress(true);
      body.innerHTML = `
        <div class="kd-center">
          <div class="kd-spinner" aria-hidden="true"></div>
          <h3>Setting up transfer</h3>
          <p>${message || "Creating a one-time account for this payment…"}</p>
        </div>
      `;
    },
    setError(message, { retryable = false, onRetry } = {}) {
      setProgress(false);
      body.innerHTML = `
        <div class="kd-center">
          <h3>Couldn’t start payment</h3>
          <p>${message || "Something went wrong."}</p>
          <div class="kd-actions kd-actions-stack">
            ${
              retryable
                ? `<button type="button" class="kd-btn kd-btn-primary" data-kd-retry>Try again</button>`
                : ""
            }
            <button type="button" class="kd-btn kd-btn-ghost" data-kd-dismiss>Close</button>
          </div>
        </div>
      `;
      body.querySelector("[data-kd-dismiss]")?.addEventListener("click", () =>
        close("failed")
      );
      body.querySelector("[data-kd-retry]")?.addEventListener("click", () =>
        onRetry?.()
      );
    },
    setTransfer(details, handlers) {
      setProgress(true);
      const bankShort =
        BANK_LABELS[details.bankType] || details.bankType || "Bank";
      const bankLong = bankFullName(details.bankType, bankShort);

      body.innerHTML = `
        <div class="kd-hero">
          <div class="kd-hero-copy">
            <p class="kd-hero-label">Amount due</p>
            <h2 class="kd-amount">${money}</h2>
            ${
              safeEmail
                ? `<div class="kd-email-chip">${iconPerson}<span>${safeEmail}</span></div>`
                : ""
            }
          </div>
          <div class="kd-hero-art" aria-hidden="true">${artScene}</div>
        </div>

        <div class="kd-warn">
          ${iconWarn}
          <p>Transfer exactly <strong>${exact}</strong>. Do not include any extra amount to avoid delay.</p>
        </div>

        <div class="kd-panel">
          <div class="kd-bank-head">
            ${bankMark(details.bankType, bankShort)}
            <div class="kd-bank-head-text">
              <p class="kd-bank-title">${bankLong}</p>
              <p class="kd-bank-sub">${bankShort}</p>
            </div>
            <span class="kd-bank-tag">Bank</span>
          </div>

          <div class="kd-row">
            <span class="kd-row-icon">${iconCard}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account number</p>
              <p class="kd-acct-number" data-kd-acct>${details.accountNumber}</p>
            </div>
            ${copyButton("data-kd-copy-acct")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${iconPerson}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account name</p>
              <p class="kd-row-value" data-kd-name>${details.accountName || "—"}</p>
            </div>
            ${copyButton("data-kd-copy-name")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${iconDoc}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Payment reference</p>
              <p class="kd-row-value kd-mono" data-kd-ref>${details.txnRef || "—"}</p>
            </div>
            ${copyButton("data-kd-copy-ref")}
          </div>
        </div>

        <div class="kd-meta">
          <div class="kd-expiry">
            ${iconClock}
            <span>Expires in <strong data-kd-ttl>—</strong></span>
          </div>
          <div class="kd-listen is-active" data-kd-status-chip>
            <span class="kd-listen-radar" aria-hidden="true"></span>
            <p class="kd-listen-title" data-kd-status>Listening for payment…</p>
            <span class="kd-listen-dots" aria-hidden="true"><i></i><i></i><i></i></span>
          </div>
        </div>

        <div class="kd-alert kd-alert-error" data-kd-err style="display:none"></div>

        <div class="kd-actions">
          <button type="button" class="kd-btn kd-btn-primary" data-kd-confirm>
            ${iconRefresh}<span>Check Payment Status</span>
          </button>
          <button type="button" class="kd-btn kd-btn-ghost" data-kd-cancel>Cancel</button>
        </div>
      `;

      const wireCopy = (btnSel, valueSel) => {
        const btn = body.querySelector(btnSel);
        const value = body.querySelector(valueSel)?.textContent?.trim() || "";
        btn?.addEventListener("click", async () => {
          try {
            await navigator.clipboard.writeText(value);
          } catch {
            const ta = document.createElement("textarea");
            ta.value = value;
            document.body.appendChild(ta);
            ta.select();
            document.execCommand("copy");
            ta.remove();
          }
          const label = btn.querySelector("span");
          if (label) label.textContent = "Copied";
          btn.classList.add("is-done");
          setTimeout(() => {
            if (label) label.textContent = "Copy";
            btn.classList.remove("is-done");
          }, 1400);
        });
      };
      wireCopy("[data-kd-copy-acct]", "[data-kd-acct]");
      wireCopy("[data-kd-copy-name]", "[data-kd-name]");
      wireCopy("[data-kd-copy-ref]", "[data-kd-ref]");

      body.querySelector("[data-kd-cancel]")?.addEventListener("click", () =>
        close("cancelled")
      );
      body.querySelector("[data-kd-confirm]")?.addEventListener("click", () =>
        handlers?.onCheckNow?.()
      );

      let remaining = Number(details.timeToLive) || 1800;
      const ttlEl = body.querySelector("[data-kd-ttl]");
      ttlEl.textContent = formatCountdown(remaining);
      const timer = setInterval(() => {
        remaining -= 1;
        ttlEl.textContent = formatCountdown(remaining);
        if (remaining <= 0) {
          clearInterval(timer);
          handlers?.onExpired?.();
        }
      }, 1000);

      return {
        stopTimer: () => clearInterval(timer),
        setStatus(text) {
          const s = body.querySelector("[data-kd-status]");
          const chip = body.querySelector("[data-kd-status-chip]");
          if (s) s.textContent = text;
          const waiting = /listen|check|wait|pending|not yet/i.test(
            String(text || "")
          );
          if (chip) chip.classList.toggle("is-active", waiting);
          setProgress(waiting);
        },
        setListening(active) {
          const btn = body.querySelector("[data-kd-confirm]");
          const chip = body.querySelector("[data-kd-status-chip]");
          if (btn) btn.disabled = false;
          if (chip) chip.classList.toggle("is-active", !!active);
          setProgress(!!active);
        },
        showError(message) {
          const err = body.querySelector("[data-kd-err]");
          if (!err) return;
          err.style.display = message ? "block" : "none";
          err.textContent = message || "";
        },
      };
    },
    setSuccess(reference) {
      successPending = true;
      setProgress(false);
      body.innerHTML = `
        <div class="kd-center">
          <div class="kd-success-mark" aria-hidden="true">✓</div>
          <h3>Payment confirmed</h3>
          <p>Transfer received${reference ? ` for <strong>${reference}</strong>` : ""}.</p>
          <p class="kd-success-hint">This window closes automatically in 10 seconds.</p>
          <div class="kd-actions kd-actions-stack">
            <button type="button" class="kd-btn kd-btn-primary" data-kd-done>Done</button>
          </div>
        </div>
      `;
      body.querySelector("[data-kd-done]")?.addEventListener("click", () =>
        close("success")
      );
    },
  };

  return api;
}
