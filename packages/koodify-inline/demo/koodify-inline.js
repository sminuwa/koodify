var KoodifyInline=(()=>{var E=Object.defineProperty;var de=Object.getOwnPropertyDescriptor;var ce=Object.getOwnPropertyNames;var le=Object.prototype.hasOwnProperty;var pe=(e,t)=>{for(var r in t)E(e,r,{get:t[r],enumerable:!0})},ke=(e,t,r,l)=>{if(t&&typeof t=="object"||typeof t=="function")for(let p of ce(t))!le.call(e,p)&&p!==r&&E(e,p,{get:()=>t[p],enumerable:!(l=de(t,p))||l.enumerable});return e};var fe=e=>ke(E({},"__esModule",{value:!0}),e);var Le={};pe(Le,{default:()=>Ne,koodifyInitPayment:()=>se});var ue={sandbox:"https://sandbox.zainpay.ng",live:"https://api.zainpay.ng"};function X(e,t){return t&&String(t).trim()?String(t).replace(/\/$/,""):ue[e==="live"?"live":"sandbox"]}function q(e,t){let r={Accept:"application/json","Content-Type":"application/json"},l=String(t||"").trim();if(!l)return r;let p=String(e?.auth||"").toLowerCase(),c=String(e?.baseUrl||"");return p==="gateway"||p==="koodify"||/koodify|localhost:8080|127\.0\.0\.1:8080|\/proxy\/koodify/i.test(c)?r["X-Api-Key"]=l:r.Authorization=`Bearer ${l}`,r}function D(e,t){let r=e?.message||String(e);return/failed to fetch|networkerror|load failed/i.test(r)?t==="live"?"Browser blocked the API call (CORS). Point baseUrl at koodify or /proxy/koodify.":"Network error. Check connectivity, baseUrl, and API key.":r}function me(e){if(!e)return"";if(typeof e.description=="string"&&e.description.trim())return e.description;if(typeof e.message=="string"&&e.message.trim())return e.message;if(typeof e=="object"){let t=[];for(let[r,l]of Object.entries(e))Array.isArray(l)&&l[0]?.code&&t.push(`${r}: ${l[0].code}`);if(t.length)return t.join("; ")}return""}function O(e){if(!e)return"";if(typeof e=="string")try{return new URL(e).searchParams.get("e")||""}catch{return e}return typeof e=="object"&&(e.e||e.sessionKey||e.token||O(e.value))||""}async function A(e){let t=await e.text(),r=null;try{r=t?JSON.parse(t):null}catch{r={description:t||e.statusText}}return r}async function U({baseUrl:e,apiKey:t,config:r,payload:l}){let p;try{p=await fetch(`${e}/zainbox/card/initialize/payment/inline`,{method:"POST",headers:q(r,t),body:JSON.stringify(l)})}catch(a){let m=new Error(a.message||"Failed to fetch");throw m.cause=a,m.network=!0,m}let c=await A(p),d=c?.code!=null?String(c.code):"",f=me(c);if((!p.ok||d&&d!=="00"&&d!=="04")&&!(d==="04"||/duplicate\s+txn\s+reference/i.test(f))){let m=new Error(f||`Payment initialization failed (${p.status})`);throw m.code=d,m.body=c,m}return{sessionKey:O(c?.data),body:c,duplicate:d==="04",payer:c?.data?.payer||c?.data?.metadata?.payer||null,customParam:c?.data?.customParam||c?.data?.metadata?.customParam||c?.data?.customParams||c?.data?.metadata?.customParams||null}}async function I({baseUrl:e,apiKey:t,config:r,sessionKey:l,bankType:p}){let c=encodeURIComponent(`"${p}"`),d=`${e}/zainbox/card/generate-temporary-virtual-account?bankType=${c}&e=${encodeURIComponent(l)}`,f=await fetch(d,{headers:q(r,t)}),u=await A(f);if(!f.ok||String(u?.code)!=="00"||!u?.data){let w=new Error(u?.description||"Could not create transfer account");throw w.code=u?.code,w.body=u,w}let a=u.data,m=parseFloat(a.amount),b=Number.isFinite(m)?m/100:NaN;return{bankType:a.bankType||p,accountNumber:String(a.accountNumber||""),accountName:String(a.accountName||""),amountNaira:Number.isInteger(b)?String(b):Number.isFinite(b)?b.toFixed(2):String(a.amount??""),paymentStatus:a.paymentStatus||"pending",txnRef:a.txnRef||"",timeToLive:Number(a.timeToLive)||1800,email:a.email||"",callBackUrl:a.callBackUrl||"",payer:a.payer||null}}async function K({baseUrl:e,apiKey:t,config:r,txnRef:l}){let p=`${e}/zainbox/card/temporary-virtual-account-payment-status?txnRef=${encodeURIComponent(l)}`,c=await fetch(p,{headers:q(r,t)}),d=await A(c);return{status:String(d?.data?.paymentStatus||"").toLowerCase(),callBackUrl:d?.data?.callBackUrl||"",description:d?.description||"",payer:d?.data?.payer||d?.data?.metadata?.payer||null,customParam:d?.data?.customParam||d?.data?.metadata?.customParam||d?.data?.customParams||d?.data?.metadata?.customParams||null,body:d}}var j=`@import url("https://fonts.googleapis.com/css2?family=DM+Sans:opsz,wght@9..40,400;9..40,500;9..40,600;9..40,700;9..40,800&display=swap");

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
`;var H="data:image/png;base64,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";var V=`
<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
  <path d="M3 3l8 8M11 3L3 11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,Y=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="3" y="6" width="18" height="12" rx="2.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M3 10h18" stroke="currentColor" stroke-width="1.8"/>
  <path d="M7 15h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,T=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="8" r="3.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 19c1.5-3.2 4-4.8 7-4.8s5.5 1.6 7 4.8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,Z=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M7 3.5h7l4 4V20a1.5 1.5 0 0 1-1.5 1.5h-9.5A1.5 1.5 0 0 1 5.5 20V5A1.5 1.5 0 0 1 7 3.5z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M14 3.5V8h4.5" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
</svg>`,_=`
<svg width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="8" y="8" width="11" height="11" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 15V6a2 2 0 0 1 2-2h9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,W=`
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.8"/>
  <path d="M12 8v4.5l3 1.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`,J=`
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M12 4l9 16H3L12 4z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M12 10v4M12 17h.01" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`,Q=`
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M20 12a8 8 0 1 1-2.3-5.6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
  <path d="M20 5v5h-5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`;var ee=`
<svg width="12" height="12" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="5" y="11" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M8 11V8a4 4 0 1 1 8 0v3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;function te(e,t){return String(e).toLowerCase()==="gtbank"||/gt\s*bank/i.test(t)?'<span class="kd-bank-mark kd-bank-gt" aria-hidden="true">GT</span>':`<span class="kd-bank-mark" aria-hidden="true">${(t||"B").trim().charAt(0).toUpperCase()||"B"}</span>`}function re(e,t){return String(e).toLowerCase()==="gtbank"||/gt\s*bank/i.test(t)?"Guaranty Trust Bank":t||"Bank"}var ae=`
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
</svg>`;var ge={gtBank:"GTBank"};function be(e,t){let r=Number(t);return Number.isFinite(r)?`${e} ${r.toLocaleString("en-NG",{minimumFractionDigits:Number.isInteger(r)?0:2,maximumFractionDigits:2})}`:`${e} ${t}`}function ye(e,t){let r=Number(t);return Number.isFinite(r)?`${e} ${r.toLocaleString("en-NG",{minimumFractionDigits:2,maximumFractionDigits:2})}`:`${e} ${t}`}function ie(e){let t=Math.max(0,Math.floor(e)),r=Math.floor(t/60),l=t%60;return`${r}:${String(l).padStart(2,"0")}`}function ve(){if(document.querySelector("style[data-koodify-inline]"))return;let e=document.createElement("style");e.setAttribute("data-koodify-inline","true"),e.textContent=j,document.head.appendChild(e)}function we(e){let t=document.createElement("template");return t.innerHTML=e.trim(),t.content.firstElementChild}function M(e){return`<button type="button" class="kd-copy" ${e} aria-label="Copy">
    ${_}<span>Copy</span>
  </button>`}function Se(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function oe({amount:e,currency:t,email:r,logoUrl:l,onClose:p}){ve();let c=be(t,e),d=ye(t,e),f=Se(r),u=we(`
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
                src="${l||H}"
                alt="Koodify"
                width="132"
                height="36"
              />
              <div class="kd-brand-sub">Bank Transfer Checkout</div>
            </div>
            <button type="button" class="kd-close" aria-label="Close" data-kd-close>${V}</button>
          </header>
          <div class="kd-body" data-kd-body></div>
          <footer class="kd-foot">
            ${ee}
            <span>Powered by <strong>Koodify</strong></span>
          </footer>
        </div>
      </div>
    </div>
  `),a=u.querySelector("[data-kd-body]"),m=u.querySelector("[data-kd-progress]"),b=!1,w=!1,x=n=>{b||(b=!0,u.remove(),document.querySelector("style[data-koodify-inline]")?.remove(),p?.(n))},S=()=>x(w?"success":"cancelled");u.querySelector("[data-kd-close]").addEventListener("click",()=>{S()}),u.querySelector(".kd-overlay").addEventListener("click",n=>{n.target.classList.contains("kd-overlay")&&S()});let k=n=>{n.key==="Escape"&&S()};document.addEventListener("keydown",k);let z=()=>{document.removeEventListener("keydown",k),b||(b=!0,u.remove())};document.body.appendChild(u);let g=n=>{m&&m.classList.toggle("is-active",!!n)};return{destroy:z,close:x,setLoading(n){g(!0),a.innerHTML=`
        <div class="kd-center">
          <div class="kd-spinner" aria-hidden="true"></div>
          <h3>Setting up transfer</h3>
          <p>${n||"Creating a one-time account for this payment\u2026"}</p>
        </div>
      `},setError(n,{retryable:y=!1,onRetry:h}={}){g(!1),a.innerHTML=`
        <div class="kd-center">
          <h3>Couldn\u2019t start payment</h3>
          <p>${n||"Something went wrong."}</p>
          <div class="kd-actions kd-actions-stack">
            ${y?'<button type="button" class="kd-btn kd-btn-primary" data-kd-retry>Try again</button>':""}
            <button type="button" class="kd-btn kd-btn-ghost" data-kd-dismiss>Close</button>
          </div>
        </div>
      `,a.querySelector("[data-kd-dismiss]")?.addEventListener("click",()=>x("failed")),a.querySelector("[data-kd-retry]")?.addEventListener("click",()=>h?.())},setTransfer(n,y){g(!0);let h=ge[n.bankType]||n.bankType||"Bank",R=re(n.bankType,h);a.innerHTML=`
        <div class="kd-hero">
          <div class="kd-hero-copy">
            <p class="kd-hero-label">Amount due</p>
            <h2 class="kd-amount">${c}</h2>
            ${f?`<div class="kd-email-chip">${T}<span>${f}</span></div>`:""}
          </div>
          <div class="kd-hero-art" aria-hidden="true">${ae}</div>
        </div>

        <div class="kd-warn">
          ${J}
          <p>Transfer exactly <strong>${d}</strong>. Do not include any extra amount to avoid delay.</p>
        </div>

        <div class="kd-panel">
          <div class="kd-bank-head">
            ${te(n.bankType,h)}
            <div class="kd-bank-head-text">
              <p class="kd-bank-title">${R}</p>
              <p class="kd-bank-sub">${h}</p>
            </div>
            <span class="kd-bank-tag">Bank</span>
          </div>

          <div class="kd-row">
            <span class="kd-row-icon">${Y}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account number</p>
              <p class="kd-acct-number" data-kd-acct>${n.accountNumber}</p>
            </div>
            ${M("data-kd-copy-acct")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${T}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Account name</p>
              <p class="kd-row-value" data-kd-name>${n.accountName||"\u2014"}</p>
            </div>
            ${M("data-kd-copy-name")}
          </div>
          <div class="kd-row">
            <span class="kd-row-icon">${Z}</span>
            <div class="kd-row-main">
              <p class="kd-row-label">Payment reference</p>
              <p class="kd-row-value kd-mono" data-kd-ref>${n.txnRef||"\u2014"}</p>
            </div>
            ${M("data-kd-copy-ref")}
          </div>
        </div>

        <div class="kd-meta">
          <div class="kd-expiry">
            ${W}
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
            ${Q}<span>Check Payment Status</span>
          </button>
          <button type="button" class="kd-btn kd-btn-ghost" data-kd-cancel>Cancel</button>
        </div>
      `;let N=(o,s)=>{let v=a.querySelector(o),$=a.querySelector(s)?.textContent?.trim()||"";v?.addEventListener("click",async()=>{try{await navigator.clipboard.writeText($)}catch{let B=document.createElement("textarea");B.value=$,document.body.appendChild(B),B.select(),document.execCommand("copy"),B.remove()}let F=v.querySelector("span");F&&(F.textContent="Copied"),v.classList.add("is-done"),setTimeout(()=>{F&&(F.textContent="Copy"),v.classList.remove("is-done")},1400)})};N("[data-kd-copy-acct]","[data-kd-acct]"),N("[data-kd-copy-name]","[data-kd-name]"),N("[data-kd-copy-ref]","[data-kd-ref]"),a.querySelector("[data-kd-cancel]")?.addEventListener("click",()=>x("cancelled")),a.querySelector("[data-kd-confirm]")?.addEventListener("click",()=>y?.onCheckNow?.());let P=Number(n.timeToLive)||1800,L=a.querySelector("[data-kd-ttl]");L.textContent=ie(P);let i=setInterval(()=>{P-=1,L.textContent=ie(P),P<=0&&(clearInterval(i),y?.onExpired?.())},1e3);return{stopTimer:()=>clearInterval(i),setStatus(o){let s=a.querySelector("[data-kd-status]"),v=a.querySelector("[data-kd-status-chip]");s&&(s.textContent=o);let $=/listen|check|wait|pending|not yet/i.test(String(o||""));v&&v.classList.toggle("is-active",$),g($)},setListening(o){let s=a.querySelector("[data-kd-confirm]"),v=a.querySelector("[data-kd-status-chip]");s&&(s.disabled=!1),v&&v.classList.toggle("is-active",!!o),g(!!o)},showError(o){let s=a.querySelector("[data-kd-err]");s&&(s.style.display=o?"block":"none",s.textContent=o||"")}}},setSuccess(n){w=!0,g(!1),a.innerHTML=`
        <div class="kd-center">
          <div class="kd-success-mark" aria-hidden="true">\u2713</div>
          <h3>Payment confirmed</h3>
          <p>Transfer received${n?` for <strong>${n}</strong>`:""}.</p>
          <p class="kd-success-hint">This window closes automatically in 10 seconds.</p>
          <div class="kd-actions kd-actions-stack">
            <button type="button" class="kd-btn kd-btn-primary" data-kd-done>Done</button>
          </div>
        </div>
      `,a.querySelector("[data-kd-done]")?.addEventListener("click",()=>x("success"))}}}var ne=["amount","txnRef","mobileNumber","zainboxCode","emailAddress","callBackUrl"];function Ce(e){return!e||typeof e!="object"?ne.slice():ne.filter(t=>{let r=e[t];return r==null||String(r).trim()===""})}function Pe(e){let t=parseFloat(e);return Number.isFinite(t)?Number.isInteger(t)?String(t):t.toFixed(2):String(e)}function G(e,t){try{typeof e=="function"&&e(t)}catch{}}function ze(e){let t=e?.customParam??e?.customParams;return!t||typeof t!="object"||Array.isArray(t)?null:{...t}}function se(e,t,r,l){return new Promise((p,c)=>{let d=Ce(e);if(d.length){let i={status:"failed",reference:e?.txnRef||"",description:`Missing required field(s): ${d.join(", ")}`};G(t,i),c(i);return}let f=ze(e),u=e.currencyCode||"NGN",a=l==="live"?"live":"sandbox",m=X(a,e.baseUrl);if(!/\/proxy\/koodify/i.test(m)&&(!r||String(r).trim()==="")){let i={status:"failed",reference:e.txnRef,description:"Missing apiKey (gateway X-Api-Key or Zainpay public key)"};G(t,i),c(i);return}let w=e.bankType||"gtBank",x=!1,S=!1,k=null,z=[],g=null,C=null,n=(i,o)=>{x||(x=!0,y(),k?.stopTimer?.(),C&&!i.payer&&(i.payer=C),f&&!i.customParam&&(i.customParam=f),G(t,i),o?p(i):c(i))},y=()=>{z.forEach(i=>clearTimeout(i)),z=[],g!=null&&(clearTimeout(g),g=null)},h=oe({amount:e.amount,currency:u,email:e.emailAddress,logoUrl:e.logoUrl,onClose:i=>{if(x)return;if(i==="success"){n({status:"success",reference:e.txnRef,description:"Payment was successful"},!0),h.destroy();return}let o=i==="cancelled"?"cancelled":"failed";n({status:o,reference:e.txnRef,description:o==="cancelled"?"Payment cancelled":"Payment failed"},!1),h.destroy()}});async function R(){h.setLoading();try{let i={amount:Pe(e.amount),txnRef:e.txnRef,mobileNumber:e.mobileNumber,zainboxCode:e.zainboxCode,emailAddress:e.emailAddress,callBackUrl:e.callBackUrl,currencyCode:u,logoUrl:e.logoUrl,paymentChannels:["bank_transfer"],allowRecurringPayment:!!e.allowRecurringPayment};e.name&&(i.name=e.name),e.payer&&(i.payer=e.payer),f&&(i.customParam=f);let o=await U({baseUrl:m,apiKey:r,config:e,payload:i});if(o.payer&&(C=o.payer),o.customParam&&typeof o.customParam=="object"&&(f={...f||{},...o.customParam}),!o.sessionKey&&!o.duplicate)throw new Error("Payment session was not returned by the API");if(!o.sessionKey)throw new Error("Duplicate transaction reference \u2014 provide a fresh txnRef");let s=await I({baseUrl:m,apiKey:r,config:e,sessionKey:o.sessionKey,bankType:w});s.payer&&(C=s.payer),k=h.setTransfer(s,{onExpired:()=>{y(),k?.showError("This account expired. Close and start a new payment."),k?.setListening(!1)},onCheckNow:()=>L(s,{manual:!0})}),P(s)}catch(i){let o=i?.network?D(i,a):i.message||"Payment initialization failed";h.setError(o,{retryable:!0,onRetry:()=>R()})}}let N=3e3;function P(i){y(),k?.showError(""),k?.setListening(!0),k?.setStatus("Listening\u2026");let o=async()=>{x||S||(await L(i,{manual:!1}),!(x||S)&&z.push(setTimeout(o,N)))};o()}async function L(i,{manual:o}){if(!(x||S)){o&&(k?.showError(""),k?.setStatus("Checking\u2026"));try{let s=await K({baseUrl:m,apiKey:r,config:e,txnRef:i.txnRef||e.txnRef});if(s.payer&&(C=s.payer),s.customParam&&typeof s.customParam=="object"&&(f={...f||{},...s.customParam}),s.status==="success"){S=!0,y(),k?.stopTimer?.(),k?.setListening(!1),h.setSuccess(i.txnRef||e.txnRef),g=setTimeout(()=>{g=null,n({status:"success",reference:i.txnRef||e.txnRef,description:"Payment was successful",payer:C||void 0,customParam:f||void 0},!0),h.destroy()},1e4);return}if(s.status==="failed"){y(),k?.setListening(!1),k?.setStatus("Failed"),k?.showError(s.description||"Payment failed. Try again or contact support.");return}k?.setStatus(o?"Not yet":"Listening\u2026"),o&&k?.showError("No payment yet \u2014 keep this open; we\u2019ll keep checking automatically.")}catch{o&&k?.showError("Couldn\u2019t reach status API. We\u2019ll keep retrying automatically."),k?.setStatus("Listening\u2026")}}}R()})}var Ne={koodifyInitPayment:se};return fe(Le);})();

;(function (root) {
  var api = root.KoodifyInline;
  if (!api) return;
  root.koodifyInitPayment = api.koodifyInitPayment;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }
})(typeof window !== "undefined" ? window : globalThis);
