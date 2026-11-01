var KoodifyInline=(()=>{var R=Object.defineProperty;var le=Object.getOwnPropertyDescriptor;var pe=Object.getOwnPropertyNames;var ke=Object.prototype.hasOwnProperty;var ue=(e,t)=>{for(var r in t)R(e,r,{get:t[r],enumerable:!0})},fe=(e,t,r,p)=>{if(t&&typeof t=="object"||typeof t=="function")for(let k of pe(t))!ke.call(e,k)&&k!==r&&R(e,k,{get:()=>t[k],enumerable:!(p=le(t,k))||p.enumerable});return e};var me=e=>fe(R({},"__esModule",{value:!0}),e);var Ie={};ue(Ie,{default:()=>Pe,koodifyInitPayment:()=>ce});var Y={koodify:"https://api.koodify.ng",zainpaySandbox:"https://sandbox.zainpay.ng",zainpayLive:"https://api.zainpay.ng"};function X(e,t,r=!0){return t&&String(t).trim()?String(t).replace(/\/$/,""):r?Y.koodify:e==="live"?Y.zainpayLive:Y.zainpaySandbox}function B(e){let t=String(e?.auth??"gateway").toLowerCase();if(t==="zainpay"||t==="direct")return!1;if(t==="gateway"||t==="koodify"||t==="")return!0;let r=String(e?.baseUrl||"");return/koodify|localhost:8080|127\.0\.0\.1:8080|\/proxy\/koodify/i.test(r)}function j(e,t){let r={Accept:"application/json","Content-Type":"application/json"},p=String(t||"").trim();return!p||B(e)||(r.Authorization=`Bearer ${p}`),r}function O(e,t){let r=e?.message||String(e);return/failed to fetch|networkerror|load failed/i.test(r)?"Browser blocked the API call (CORS). Check api.koodify.ng CORS, or set baseUrl for local/proxy.":r}function he(e){if(!e)return"";if(typeof e.description=="string"&&e.description.trim())return e.description;if(typeof e.message=="string"&&e.message.trim())return e.message;if(typeof e=="object"){let t=[];for(let[r,p]of Object.entries(e))Array.isArray(p)&&p[0]?.code&&t.push(`${r}: ${p[0].code}`);if(t.length)return t.join("; ")}return""}function _(e){if(!e)return"";if(typeof e=="string")try{return new URL(e).searchParams.get("e")||""}catch{return e}return typeof e=="object"&&(e.e||e.sessionKey||e.token||_(e.value))||""}async function A(e){let t=await e.text(),r=null;try{r=t?JSON.parse(t):null}catch{r={description:t||e.statusText}}return r}async function J({baseUrl:e,apiKey:t,config:r,payload:p}){let k;try{k=await fetch(`${e}/zainbox/card/initialize/payment/inline`,{method:"POST",headers:j(r,t),body:JSON.stringify(p)})}catch(o){let f=new Error(o.message||"Failed to fetch");throw f.cause=o,f.network=!0,f}let s=await A(k),l=s?.code!=null?String(s.code):"",h=he(s);if((!k.ok||l&&l!=="00"&&l!=="04")&&!(l==="04"||/duplicate\s+txn\s+reference/i.test(h))){let f=new Error(h||`Payment initialization failed (${k.status})`);throw f.code=l,f.body=s,f}return{sessionKey:_(s?.data),body:s,duplicate:l==="04",payer:s?.data?.payer||s?.data?.metadata?.payer||null,customParam:s?.data?.customParam||s?.data?.metadata?.customParam||s?.data?.customParams||s?.data?.metadata?.customParams||null}}async function V({baseUrl:e,apiKey:t,config:r,sessionKey:p,bankType:k}){let s=new URLSearchParams;s.set("e",p),k!=null&&String(k).trim()!==""&&s.set("bankType",`"${String(k).trim()}"`);let l=`${e}/zainbox/card/generate-temporary-virtual-account?${s.toString()}`,h=await fetch(l,{headers:j(r,t)}),d=await A(h);if(!h.ok||String(d?.code)!=="00"||!d?.data){let v=new Error(d?.description||"Could not create transfer account");throw v.code=d?.code,v.body=d,v}let o=d.data,f=parseFloat(o.amount),m=Number.isFinite(f)?f/100:NaN;return{bankType:o.bankType||k||"",accountNumber:String(o.accountNumber||""),accountName:String(o.accountName||""),amountNaira:Number.isInteger(m)?String(m):Number.isFinite(m)?m.toFixed(2):String(o.amount??""),paymentStatus:o.paymentStatus||"pending",txnRef:o.txnRef||"",timeToLive:Number(o.timeToLive)||1800,email:o.email||"",callBackUrl:o.callBackUrl||"",payer:o.payer||null}}async function Q({baseUrl:e,apiKey:t,config:r,txnRef:p}){let k=`${e}/zainbox/card/temporary-virtual-account-payment-status?txnRef=${encodeURIComponent(p)}`,s=await fetch(k,{headers:j(r,t)}),l=await A(s);return{status:String(l?.data?.paymentStatus||"").toLowerCase(),callBackUrl:l?.data?.callBackUrl||"",description:l?.description||"",payer:l?.data?.payer||l?.data?.metadata?.payer||null,customParam:l?.data?.customParam||l?.data?.metadata?.customParam||l?.data?.customParams||l?.data?.metadata?.customParams||null,body:l}}var H=`@import url("https://fonts.googleapis.com/css2?family=DM+Sans:opsz,wght@9..40,400;9..40,500;9..40,600;9..40,700;9..40,800&display=swap");

.kd-root {
  --kd-ink: #0a2342;
  --kd-ink-soft: #1b3d66;
  --kd-label: #6b7f99;
  --kd-muted: #5b7190;
  --kd-line: #e4ebf5;
  --kd-surface: #ffffff;
  --kd-blue: #1a6dff;
  --kd-blue-deep: #0f57d9;
  --kd-blue-soft: #e8f0ff;
  --kd-warn-bg: #fff4e5;
  --kd-warn: #a86a12;
  --kd-ok: #16a34a;
  --kd-danger: #c03a32;
  --kd-shadow: 0 24px 64px rgba(8, 30, 70, 0.28);
  --kd-radius: 14px;
  --kd-font: "DM Sans", system-ui, sans-serif;
  --kd-brand: "DM Sans", system-ui, sans-serif;
  --kd-figure: "DM Sans", system-ui, sans-serif;
  all: initial;
  font-family: var(--kd-font);
}

.kd-root *,
.kd-root *::before,
.kd-root *::after {
  box-sizing: border-box;
}

.kd-overlay {
  position: fixed;
  inset: 0;
  z-index: 2147483000;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0;
  background: rgba(6, 16, 36, 0.55);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  animation: kd-fade-in 220ms ease;
}

@media (min-width: 560px) {
  .kd-overlay {
    align-items: center;
    padding: 24px;
  }
}

.kd-sheet {
  width: min(440px, 100%);
  max-height: min(94vh, 720px);
  overflow: auto;
  background: #fff;
  border-radius: var(--kd-radius) var(--kd-radius) 0 0;
  box-shadow: var(--kd-shadow);
  border-bottom: none;
  animation: kd-rise 360ms cubic-bezier(0.22, 1, 0.36, 1);
  font-family: var(--kd-font);
  color: var(--kd-ink);
}

@media (min-width: 560px) {
  .kd-sheet {
    border-radius: var(--kd-radius);
    border-bottom: 1px solid var(--kd-line);
  }
}

.kd-progress {
  position: sticky;
  top: 0;
  z-index: 3;
  width: 100%;
  height: 3px;
  overflow: hidden;
  background: rgba(26, 109, 255, 0.12);
  opacity: 0;
  visibility: hidden;
}

.kd-progress.is-active {
  opacity: 1;
  visibility: visible;
}

.kd-progress-bar {
  position: absolute;
  inset: 0 auto 0 0;
  width: 38%;
  background: var(--kd-blue);
  transform: translateX(-120%);
}

.kd-progress.is-active .kd-progress-bar {
  animation: kd-progress-sweep 1.35s ease-in-out infinite;
}

.kd-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 18px 0;
}

.kd-brand {
  min-width: 0;
}

.kd-brand-logo {
  display: block;
  width: auto;
  height: 28px;
  max-width: min(160px, 55vw);
  object-fit: contain;
  object-position: left center;
}

.kd-brand-sub {
  margin-top: 4px;
  font-size: 12px;
  font-weight: 600;
  color: var(--kd-muted);
}

.kd-close {
  appearance: none;
  border: 0;
  background: #e8eef8;
  color: var(--kd-ink-soft);
  width: 32px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  transition:
    background 200ms cubic-bezier(0.22, 1, 0.36, 1),
    color 200ms cubic-bezier(0.22, 1, 0.36, 1),
    transform 200ms cubic-bezier(0.22, 1, 0.36, 1);
}

.kd-close:hover {
  background: #d9e3f2;
  color: var(--kd-ink);
}

.kd-close:active {
  transform: scale(0.96);
}

.kd-body {
  padding: 12px 18px 8px;
}

.kd-foot {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 18px 16px;
  color: var(--kd-muted);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.kd-foot svg {
  width: 11px;
  height: 11px;
  flex-shrink: 0;
  opacity: 0.75;
}

.kd-foot strong {
  font-weight: 800;
  color: var(--kd-ink-soft);
}

/* Amount hero */
.kd-hero {
  position: relative;
  margin-bottom: 10px;
  padding: 14px 14px 12px;
  border-radius: 12px;
  overflow: hidden;
  background:
    radial-gradient(80% 120% at 100% 0%, rgba(80, 150, 255, 0.22), transparent 55%),
    linear-gradient(135deg, #eef4ff 0%, #f7faff 55%, #e8f0ff 100%);
  border: 1px solid #d9e6fb;
}

.kd-hero-label {
  margin: 0 0 4px;
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--kd-muted);
}

.kd-amount {
  margin: 0;
  font-family: var(--kd-figure);
  font-size: clamp(28px, 7vw, 34px);
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1;
  font-variant-numeric: tabular-nums;
  color: var(--kd-ink);
}

.kd-email-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  padding: 5px 10px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.85);
  border: 1px solid #d7e4fb;
  color: var(--kd-ink-soft);
  font-size: 12px;
  font-weight: 650;
  max-width: 100%;
  position: relative;
  z-index: 1;
}

.kd-email-chip svg {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  color: var(--kd-blue);
}

.kd-email-chip span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.kd-hero-copy {
  position: relative;
  z-index: 1;
}

.kd-hero-art {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 58%;
  height: 100%;
  pointer-events: none;
  opacity: 0.28;
  filter: saturate(0.85);
  mask-image: linear-gradient(90deg, transparent 0%, #000 38%, #000 100%);
  -webkit-mask-image: linear-gradient(90deg, transparent 0%, #000 38%, #000 100%);
}

.kd-hero-art .kd-art-scene {
  position: absolute;
  right: -12px;
  bottom: -28px;
  width: 168px;
  height: auto;
  display: block;
}

.kd-warn {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px 12px;
  margin-bottom: 10px;
  border-radius: 10px;
  background: var(--kd-warn-bg);
  color: var(--kd-warn);
  border: 1px solid rgba(168, 106, 18, 0.16);
}

.kd-warn svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.kd-warn p {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.4;
  font-weight: 600;
}

.kd-warn strong {
  font-weight: 800;
  color: #7a4e0d;
}

/* Details card */
.kd-panel {
  background: #fff;
  border: 1px solid var(--kd-line);
  border-radius: 10px;
  overflow: hidden;
}

.kd-bank-head {
  display: grid;
  grid-template-columns: 38px 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--kd-line);
  background: #fbfcff;
}

.kd-bank-mark {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 800;
  color: #fff;
  background: #64748b;
}

.kd-bank-gt {
  background: #e87511;
}

.kd-bank-title {
  margin: 0;
  font-family: var(--kd-figure);
  font-size: 14px;
  font-weight: 800;
  color: var(--kd-ink);
  letter-spacing: -0.02em;
}

.kd-bank-sub {
  margin: 1px 0 0;
  font-size: 11px;
  font-weight: 600;
  color: var(--kd-muted);
}

.kd-bank-tag {
  padding: 4px 8px;
  border-radius: 6px;
  background: var(--kd-blue-soft);
  color: var(--kd-blue-deep);
  font-size: 11px;
  font-weight: 750;
}

.kd-row {
  display: grid;
  grid-template-columns: 30px 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  border-top: 1px solid var(--kd-line);
  min-width: 0;
}

.kd-bank-head + .kd-row {
  border-top: 0;
}

.kd-row-icon {
  width: 30px;
  height: 30px;
  border-radius: 7px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: #f1f5fb;
  color: var(--kd-ink-soft);
  line-height: 0;
}

.kd-row-icon svg {
  display: block;
  width: 14px;
  height: 14px;
}

.kd-row-main {
  min-width: 0;
}

.kd-row-label {
  margin: 0;
  font-size: 10px;
  font-weight: 750;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--kd-label);
}

.kd-row-value {
  margin: 2px 0 0;
  font-family: var(--kd-figure);
  font-size: 14px;
  font-weight: 700;
  color: var(--kd-ink);
  word-break: break-word;
}

.kd-row-value.kd-mono {
  font-size: 12.5px;
  font-weight: 650;
  letter-spacing: 0.01em;
  font-variant-numeric: tabular-nums;
  color: var(--kd-ink-soft);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kd-acct-number {
  margin: 2px 0 0;
  font-family: var(--kd-figure);
  font-size: 20px;
  font-weight: 800;
  letter-spacing: 0.05em;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--kd-ink);
}

.kd-copy {
  appearance: none;
  border: 1.5px solid rgba(26, 109, 255, 0.3);
  background: #f5f9ff;
  color: var(--kd-blue);
  border-radius: 7px;
  padding: 5px 9px;
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--kd-font);
  font-size: 11px;
  font-weight: 750;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition:
    background 180ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.kd-copy:hover {
  background: var(--kd-blue-soft);
  border-color: var(--kd-blue);
}

.kd-copy:active {
  transform: scale(0.97);
}

.kd-copy.is-done {
  background: rgba(22, 163, 74, 0.1);
  border-color: rgba(22, 163, 74, 0.35);
  color: var(--kd-ok);
}

/* Meta: expiry + listening */
.kd-meta {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
  margin-top: 10px;
  align-items: stretch;
}

.kd-expiry {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border-radius: 8px;
  background: #fff4e5;
  color: #a86a12;
  font-size: 12px;
  font-weight: 650;
  white-space: nowrap;
}

.kd-expiry strong {
  font-family: var(--kd-figure);
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

.kd-listen {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 8px 10px;
  border-radius: 8px;
  background: #f0f6ff;
  border: 1px solid #d7e4fb;
  color: var(--kd-ink);
}

.kd-listen-radar {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: var(--kd-blue);
  box-shadow: 0 0 0 0 rgba(26, 109, 255, 0.45);
  animation: kd-radar 1.6s ease-out infinite;
  flex-shrink: 0;
}

.kd-listen-title {
  margin: 0;
  font-size: 12px;
  font-weight: 750;
  color: var(--kd-ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 1;
}

.kd-listen-dots {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
}

.kd-listen-dots i {
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: var(--kd-blue);
  opacity: 0.35;
  animation: kd-dot 1.1s ease-in-out infinite;
}

.kd-listen-dots i:nth-child(2) {
  animation-delay: 0.15s;
}

.kd-listen-dots i:nth-child(3) {
  animation-delay: 0.3s;
}

.kd-listen:not(.is-active) .kd-listen-radar,
.kd-listen:not(.is-active) .kd-listen-dots {
  animation: none;
  opacity: 0.4;
}

.kd-actions {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 8px;
  margin-top: 12px;
  align-items: stretch;
}

.kd-actions-stack {
  grid-template-columns: 1.4fr 1fr;
}

.kd-actions-stack:has(> :only-child) {
  grid-template-columns: 1fr;
}

.kd-btn {
  appearance: none;
  border: 1.5px solid transparent;
  border-radius: 8px;
  padding: 12px 14px;
  min-height: 46px;
  font-family: var(--kd-font);
  font-size: 14px;
  font-weight: 750;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition:
    background 220ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 220ms cubic-bezier(0.22, 1, 0.36, 1),
    transform 220ms cubic-bezier(0.22, 1, 0.36, 1),
    opacity 180ms ease;
}

.kd-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.kd-btn-primary {
  background: var(--kd-blue);
  color: #fff;
  border-color: var(--kd-blue);
  box-shadow: 0 8px 18px rgba(26, 109, 255, 0.22);
}

.kd-btn-primary:hover:not(:disabled) {
  background: var(--kd-blue-deep);
  border-color: var(--kd-blue-deep);
  box-shadow: 0 10px 22px rgba(15, 87, 217, 0.28);
  transform: translateY(-1px);
}

.kd-btn-primary:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: 0 4px 12px rgba(15, 87, 217, 0.2);
}

.kd-btn-ghost {
  background: #fff;
  color: var(--kd-ink-soft);
  border-color: #c9d7ec;
  box-shadow: none;
}

.kd-btn-ghost:hover:not(:disabled) {
  background: #f5f8fd;
  border-color: #aebdcf;
  color: var(--kd-ink);
  transform: translateY(-1px);
}

.kd-btn-ghost:active:not(:disabled) {
  transform: translateY(0);
  background: #eef2f8;
}

.kd-alert {
  margin: 10px 0 0;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.4;
  font-weight: 650;
}

.kd-alert-error {
  background: rgba(192, 58, 50, 0.1);
  color: var(--kd-danger);
  border: 1px solid rgba(192, 58, 50, 0.16);
}

.kd-center {
  padding: 40px 8px 16px;
  text-align: center;
}

.kd-spinner {
  width: 34px;
  height: 34px;
  margin: 0 auto 14px;
  border-radius: 999px;
  border: 3px solid rgba(10, 35, 66, 0.1);
  border-top-color: var(--kd-blue);
  animation: kd-spin 700ms linear infinite;
}

.kd-center h3 {
  font-family: var(--kd-figure);
  font-size: 20px;
  margin: 0 0 8px;
  letter-spacing: -0.03em;
  font-weight: 800;
}

.kd-center p {
  margin: 0;
  color: var(--kd-muted);
  font-size: 14px;
  line-height: 1.45;
}

.kd-success-hint {
  margin-top: 8px !important;
  font-size: 12px !important;
  font-weight: 600;
  opacity: 0.9;
}

.kd-success-mark {
  width: 56px;
  height: 56px;
  margin: 0 auto 14px;
  border-radius: 999px;
  background: rgba(22, 163, 74, 0.12);
  color: var(--kd-ok);
  display: grid;
  place-items: center;
  font-size: 24px;
  font-weight: 800;
}

@keyframes kd-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes kd-rise {
  from {
    opacity: 0;
    transform: translateY(22px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes kd-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes kd-progress-sweep {
  0% {
    transform: translateX(-120%);
  }
  100% {
    transform: translateX(320%);
  }
}

@keyframes kd-radar {
  0% {
    box-shadow: 0 0 0 0 rgba(26, 109, 255, 0.45);
  }
  70% {
    box-shadow: 0 0 0 12px rgba(26, 109, 255, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(26, 109, 255, 0);
  }
}

@keyframes kd-dot {
  0%,
  80%,
  100% {
    opacity: 0.3;
    transform: translateY(0);
  }
  40% {
    opacity: 1;
    transform: translateY(-2px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .kd-overlay,
  .kd-sheet,
  .kd-spinner,
  .kd-progress-bar,
  .kd-listen-radar,
  .kd-listen-dots i,
  .kd-btn {
    animation: none !important;
    transition: none !important;
  }
}
`;var Z="data:image/png;base64,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";var F=`
<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
  <path d="M3 3l8 8M11 3L3 11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,K=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="3" y="6" width="18" height="12" rx="2.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M3 10h18" stroke="currentColor" stroke-width="1.8"/>
  <path d="M7 15h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,E=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="8" r="3.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 19c1.5-3.2 4-4.8 7-4.8s5.5 1.6 7 4.8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,q=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M7 3.5h7l4 4V20a1.5 1.5 0 0 1-1.5 1.5h-9.5A1.5 1.5 0 0 1 5.5 20V5A1.5 1.5 0 0 1 7 3.5z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M14 3.5V8h4.5" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
</svg>`,D=`
<svg width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="8" y="8" width="11" height="11" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 15V6a2 2 0 0 1 2-2h9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,G=`
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.8"/>
  <path d="M12 8v4.5l3 1.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`,ee=`
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M12 4l9 16H3L12 4z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M12 10v4M12 17h.01" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,te=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M20 12a8 8 0 1 1-2.3-5.6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
  <path d="M20 5v5h-5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`;var re=`
<svg width="12" height="12" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="5" y="11" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M8 11V8a4 4 0 1 1 8 0v3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;function ae(e,t){let r=String(e||"").toLowerCase();return r==="gtbank"||/gt\s*bank/i.test(t)?'<span class="kd-bank-mark kd-bank-gt" aria-hidden="true">GT</span>':r==="moniepoint"||/moniepoint/i.test(t)?'<span class="kd-bank-mark" aria-hidden="true">M</span>':`<span class="kd-bank-mark" aria-hidden="true">${(t||"B").trim().charAt(0).toUpperCase()||"B"}</span>`}function oe(e,t){let r=String(e||"").toLowerCase();return r==="gtbank"||/gt\s*bank/i.test(t)?"Guaranty Trust Bank":r==="moniepoint"||/moniepoint/i.test(t)?"Moniepoint":t||"Bank"}var ie=`
<svg class="kd-art-scene" viewBox="0 0 360 280" fill="none" aria-hidden="true">
  <path class="kd-art-path" d="M40 220 C120 180, 180 80, 300 40" stroke="rgba(255,255,255,0.35)" stroke-width="2" stroke-dasharray="6 8"/>
  <circle cx="300" cy="40" r="10" fill="rgba(255,255,255,0.9)"/>
  <path d="M294 40l8-3 1 8-4-3-5-2z" fill="#1a6dff"/>
  <g transform="translate(48 138)">
    <rect x="0" y="48" width="140" height="78" rx="10" fill="#0b3d9e"/>
    <rect x="10" y="20" width="120" height="36" rx="6" fill="#1557c7"/>
    <polygon points="70,0 140,22 0,22" fill="#1e6cf0"/>
    <rect x="22" y="58" width="18" height="48" rx="3" fill="#9ec2ff"/>
    <rect x="50" y="58" width="18" height="48" rx="3" fill="#9ec2ff"/>
    <rect x="78" y="58" width="18" height="48" rx="3" fill="#9ec2ff"/>
    <rect x="106" y="58" width="18" height="48" rx="3" fill="#9ec2ff"/>
    <rect x="8" y="118" width="124" height="10" rx="3" fill="#072f7a"/>
  </g>
  <g transform="translate(198 150)">
    <rect x="0" y="10" width="100" height="120" rx="12" fill="#ffffff"/>
    <rect x="14" y="28" width="72" height="8" rx="4" fill="#d7e4ff"/>
    <rect x="14" y="46" width="56" height="8" rx="4" fill="#d7e4ff"/>
    <rect x="14" y="64" width="64" height="8" rx="4" fill="#d7e4ff"/>
    <rect x="14" y="82" width="48" height="8" rx="4" fill="#d7e4ff"/>
    <circle cx="78" cy="108" r="18" fill="#22c55e"/>
    <path d="M70 108l5 5 11-12" stroke="#fff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  </g>
</svg>`;var xe={gtBank:"GTBank",moniepoint:"Moniepoint",wemaBank:"Wema Bank",polaris:"Polaris Bank",fcmb:"FCMB",zainMFB:"Zain MFB"};function ve(e,t){let r=Number(t);return Number.isFinite(r)?`${e} ${r.toLocaleString("en-NG",{minimumFractionDigits:Number.isInteger(r)?0:2,maximumFractionDigits:2})}`:`${e} ${t}`}function we(e,t){let r=Number(t);return Number.isFinite(r)?`${e} ${r.toLocaleString("en-NG",{minimumFractionDigits:2,maximumFractionDigits:2})}`:`${e} ${t}`}function ne(e){let t=Math.max(0,Math.floor(e)),r=Math.floor(t/60),p=t%60;return`${r}:${String(p).padStart(2,"0")}`}function ye(){if(document.querySelector("style[data-koodify-inline]"))return;let e=document.createElement("style");e.setAttribute("data-koodify-inline","true"),e.textContent=H,document.head.appendChild(e)}function Le(e){let t=document.createElement("template");return t.innerHTML=e.trim(),t.content.firstElementChild}function W(e){return`<button type="button" class="kd-copy" ${e} aria-label="Copy">
    ${D}<span>Copy</span>
  </button>`}function ze(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function se({amount:e,currency:t,email:r,logoUrl:p,onClose:k}){ye();let s=ve(t,e),l=we(t,e),h=ze(r),d=Le(`
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
                src="${p||Z}"
                alt="Koodify"
                width="132"
                height="36"
              />
              <div class="kd-brand-sub">Bank Transfer Checkout</div>
            </div>
            <button type="button" class="kd-close" aria-label="Close" data-kd-close>${F}</button>
          </header>
          <div class="kd-body" data-kd-body></div>
          <footer class="kd-foot">
            ${re}
            <span>Powered by <strong>Koodify</strong></span>
          </footer>
        </div>
      </div>
    </div>
  `),o=d.querySelector("[data-kd-body]"),f=d.querySelector("[data-kd-progress]"),m=!1,v=!1,L=n=>{m||(m=!0,d.remove(),document.querySelector("style[data-koodify-inline]")?.remove(),k?.(n))},M=()=>L(v?"success":"cancelled");d.querySelector("[data-kd-close]").addEventListener("click",()=>{M()}),d.querySelector(".kd-overlay").addEventListener("click",n=>{n.target.classList.contains("kd-overlay")&&M()});let w=n=>{n.key==="Escape"&&M()};document.addEventListener("keydown",w);let T=()=>{document.removeEventListener("keydown",w),m||(m=!0,d.remove())};document.body.appendChild(d);let c=n=>{f&&f.classList.toggle("is-active",!!n)};return{destroy:T,close:L,setLoading(n){c(!0),o.innerHTML=`
        <div class="kd-center">
          <div class="kd-spinner" aria-hidden="true"></div>
          <h3>Setting up transfer</h3>
          <p>${n||"Creating a one-time account for this payment\u2026"}</p>
        </div>
      `},setError(n,{retryable:b=!1,onRetry:y}={}){c(!1),o.innerHTML=`
        <div class="kd-center">
          <h3>Couldn\u2019t start payment</h3>
          <p>${n||"Something went wrong."}</p>
          <div class="kd-actions kd-actions-stack">
            ${b?'<button type="button" class="kd-btn kd-btn-primary" data-kd-retry>Try again</button>':""}
            <button type="button" class="kd-btn kd-btn-ghost" data-kd-dismiss>Close</button>
          </div>
        </div>
      `,o.querySelector("[data-kd-dismiss]")?.addEventListener("click",()=>L("failed")),o.querySelector("[data-kd-retry]")?.addEventListener("click",()=>y?.())},setTransfer(n,b){c(!0);let y=xe[n.bankType]||n.bankType||"Bank",S=oe(n.bankType,y);o.innerHTML=`
        <div class="kd-hero">
          <div class="kd-hero-copy">
            <p class="kd-hero-label">Amount due</p>
            <h2 class="kd-amount">${s}</h2>
            ${h?`<div class="kd-email-chip">${E}<span>${h}</span></div>`:""}
          </div>
          <div class="kd-hero-art" aria-hidden="true">${ie}</div>
        </div>

        <div class="kd-warn">
          ${ee}
          <p>Transfer exactly <strong>${l}</strong>. Do not include any extra amount to avoid delay.</p>
        </div>

        <div class="kd-panel">
          <div class="kd-bank-head">
            ${ae(n.bankType,y)}
            <div class="kd-bank-head-text">
              <p class="kd-bank-title">${S}</p>
              <p class="kd-bank-sub">${y}</p>
            </div>
            <span class="kd-bank-tag">Bank</span>
          </div>

          <div class="kd-row">
            <span class="kd-row-icon">${K}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account number</p>
              <p class="kd-acct-number" data-kd-acct>${n.accountNumber}</p>
            </div>
            ${W("data-kd-copy-acct")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${E}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account name</p>
              <p class="kd-row-value" data-kd-name>${n.accountName||"\u2014"}</p>
            </div>
            ${W("data-kd-copy-name")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${q}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Payment reference</p>
              <p class="kd-row-value kd-mono" data-kd-ref>${n.txnRef||"\u2014"}</p>
            </div>
            ${W("data-kd-copy-ref")}
          </div>
        </div>

        <div class="kd-meta">
          <div class="kd-expiry">
            ${G}
            <span>Expires in <strong data-kd-ttl>\u2014</strong></span>
          </div>
          <div class="kd-listen is-active" data-kd-status-chip>
            <span class="kd-listen-radar" aria-hidden="true"></span>
            <p class="kd-listen-title" data-kd-status>Listening for payment\u2026</p>
            <span class="kd-listen-dots" aria-hidden="true"><i></i><i></i><i></i></span>
          </div>
        </div>

        <div class="kd-alert kd-alert-error" data-kd-err style="display:none"></div>

        <div class="kd-actions">
          <button type="button" class="kd-btn kd-btn-primary" data-kd-confirm>
            ${te}<span>Check Payment Status</span>
          </button>
          <button type="button" class="kd-btn kd-btn-ghost" data-kd-cancel>Cancel</button>
        </div>
      `;let x=(g,a)=>{let i=o.querySelector(g),u=o.querySelector(a)?.textContent?.trim()||"";i?.addEventListener("click",async()=>{try{await navigator.clipboard.writeText(u)}catch{let U=document.createElement("textarea");U.value=u,document.body.appendChild(U),U.select(),document.execCommand("copy"),U.remove()}let z=i.querySelector("span");z&&(z.textContent="Copied"),i.classList.add("is-done"),setTimeout(()=>{z&&(z.textContent="Copy"),i.classList.remove("is-done")},1400)})};x("[data-kd-copy-acct]","[data-kd-acct]"),x("[data-kd-copy-name]","[data-kd-name]"),x("[data-kd-copy-ref]","[data-kd-ref]"),o.querySelector("[data-kd-cancel]")?.addEventListener("click",()=>L("cancelled")),o.querySelector("[data-kd-confirm]")?.addEventListener("click",()=>b?.onCheckNow?.());let $=Number(n.timeToLive)||1800,I=o.querySelector("[data-kd-ttl]");I.textContent=ne($);let N=setInterval(()=>{$-=1,I.textContent=ne($),$<=0&&(clearInterval(N),b?.onExpired?.())},1e3);return{stopTimer:()=>clearInterval(N),setStatus(g){let a=o.querySelector("[data-kd-status]"),i=o.querySelector("[data-kd-status-chip]");a&&(a.textContent=g);let u=/listen|check|wait|pending|not yet/i.test(String(g||""));i&&i.classList.toggle("is-active",u),c(u)},setListening(g){let a=o.querySelector("[data-kd-confirm]"),i=o.querySelector("[data-kd-status-chip]");a&&(a.disabled=!1),i&&i.classList.toggle("is-active",!!g),c(!!g)},showError(g){let a=o.querySelector("[data-kd-err]");a&&(a.style.display=g?"block":"none",a.textContent=g||"")}}},setSuccess(n){v=!0,c(!1),o.innerHTML=`
        <div class="kd-center">
          <div class="kd-success-mark" aria-hidden="true">\u2713</div>
          <h3>Payment confirmed</h3>
          <p>Transfer received${n?` for <strong>${n}</strong>`:""}.</p>
          <p class="kd-success-hint">This window closes automatically in 10 seconds.</p>
          <div class="kd-actions kd-actions-stack">
            <button type="button" class="kd-btn kd-btn-primary" data-kd-done>Done</button>
          </div>
        </div>
      `,o.querySelector("[data-kd-done]")?.addEventListener("click",()=>L("success"))}}}var de=["amount","txnRef","mobileNumber","emailAddress","callBackUrl"];function Se(e){let t=e?.publicKey??e?.public_key??e?.koodifyKey??"";return t!=null&&String(t).trim()!==""?String(t).trim():""}function $e(e){return!e||typeof e!="object"?de.slice():de.filter(t=>{let r=e[t];return r==null||String(r).trim()===""})}function Te(e){let t=parseFloat(e);return Number.isFinite(t)?Number.isInteger(t)?String(t):t.toFixed(2):String(e)}function C(e,t){try{typeof e=="function"&&e(t)}catch{}}function Me(e){let t=e?.customParam??e?.customParams;return!t||typeof t!="object"||Array.isArray(t)?null:{...t}}function ce(e,t,r,p){return new Promise((k,s)=>{let l=$e(e);if(l.length){let a={status:"failed",reference:e?.txnRef||"",description:`Missing required field(s): ${l.join(", ")}`};C(t,a),s(a);return}let h=Se(e);if(!h){let a={status:"failed",reference:e?.txnRef||"",description:"Missing publicKey (publishable pk_live_\u2026 from Koodify)"};C(t,a),s(a);return}let d=Me(e),o=e.currencyCode||"NGN",f=p==="live"?"live":"sandbox",m=B(e),v=X(f,e.baseUrl,m);if(!m&&(!r||String(r).trim()==="")){let a={status:"failed",reference:e.txnRef,description:"Missing API key for direct Zainpay calls"};C(t,a),s(a);return}let L=e.bankType!=null&&String(e.bankType).trim()!==""?String(e.bankType).trim():"",M=m?L:L||"gtBank",w=!1,T=!1,c=null,P=[],n=null,b=null,y=(a,i)=>{w||(w=!0,S(),c?.stopTimer?.(),b&&!a.payer&&(a.payer=b),d&&!a.customParam&&(a.customParam=d),C(t,a),i?k(a):s(a))},S=()=>{P.forEach(a=>clearTimeout(a)),P=[],n!=null&&(clearTimeout(n),n=null)},x=se({amount:e.amount,currency:o,email:e.emailAddress,logoUrl:e.logoUrl,onClose:a=>{if(w)return;if(a==="success"){y({status:"success",reference:e.txnRef,description:"Payment was successful"},!0),x.destroy();return}let i=a==="cancelled"?"cancelled":"failed";y({status:i,reference:e.txnRef,description:i==="cancelled"?"Payment cancelled":"Payment failed"},!1),x.destroy()}});async function $(){x.setLoading();try{let a={amount:Te(e.amount),txnRef:e.txnRef,mobileNumber:e.mobileNumber,emailAddress:e.emailAddress,callBackUrl:e.callBackUrl,currencyCode:o,logoUrl:e.logoUrl,paymentChannels:["bank_transfer"],allowRecurringPayment:!!e.allowRecurringPayment};if(m?a.publicKey=h:a.zainboxCode=e.zainboxCode||h,e.name&&(a.name=e.name),e.payer&&(a.payer=e.payer),d&&(a.customParam=d),m){let z=e.splitPlanId||e.split_plan_id;z!=null&&String(z).trim()!==""&&(a.splitPlanId=String(z).trim())}let i=await J({baseUrl:v,apiKey:r,config:e,payload:a});if(i.payer&&(b=i.payer),i.customParam&&typeof i.customParam=="object"&&(d={...d||{},...i.customParam}),!i.sessionKey&&!i.duplicate)throw new Error("Payment session was not returned by the API");if(!i.sessionKey)throw new Error("Duplicate transaction reference \u2014 provide a fresh txnRef");let u=await V({baseUrl:v,apiKey:r,config:e,sessionKey:i.sessionKey,bankType:M});u.payer&&(b=u.payer),c=x.setTransfer(u,{onExpired:()=>{S(),c?.showError("This account expired. Close and start a new payment."),c?.setListening(!1)},onCheckNow:()=>g(u,{manual:!0})}),N(u)}catch(a){let i=a?.network?O(a,f):a.message||"Payment initialization failed";x.setError(i,{retryable:!0,onRetry:()=>$()})}}let I=3e3;function N(a){S(),c?.showError(""),c?.setListening(!0),c?.setStatus("Listening\u2026");let i=async()=>{w||T||(await g(a,{manual:!1}),!(w||T)&&P.push(setTimeout(i,I)))};i()}async function g(a,{manual:i}){if(!(w||T)){i&&(c?.showError(""),c?.setStatus("Checking\u2026"));try{let u=await Q({baseUrl:v,apiKey:r,config:e,txnRef:a.txnRef||e.txnRef});if(u.payer&&(b=u.payer),u.customParam&&typeof u.customParam=="object"&&(d={...d||{},...u.customParam}),u.status==="success"){T=!0,S(),c?.stopTimer?.(),c?.setListening(!1),x.setSuccess(a.txnRef||e.txnRef),n=setTimeout(()=>{n=null,y({status:"success",reference:a.txnRef||e.txnRef,description:"Payment was successful",payer:b||void 0,customParam:d||void 0},!0),x.destroy()},1e4);return}if(u.status==="failed"){S(),c?.setListening(!1),c?.setStatus("Failed"),c?.showError(u.description||"Payment failed. Try again or contact support.");return}c?.setStatus(i?"Not yet":"Listening\u2026"),i&&c?.showError("No payment yet \u2014 keep this open; we\u2019ll keep checking automatically.")}catch{i&&c?.showError("Couldn\u2019t reach status API. We\u2019ll keep retrying automatically."),c?.setStatus("Listening\u2026")}}}$()})}var Pe={koodifyInitPayment:ce};return me(Ie);})();

;(function (root) {
  var api = root.KoodifyInline;
  if (!api) return;
  root.koodifyInitPayment = api.koodifyInitPayment;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }
})(typeof window !== "undefined" ? window : globalThis);
