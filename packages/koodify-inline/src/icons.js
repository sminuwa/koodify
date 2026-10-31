/** Inline SVGs for the checkout sheet (currentColor). */

export const iconClose = `
<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
  <path d="M3 3l8 8M11 3L3 11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export const iconCard = `
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="3" y="6" width="18" height="12" rx="2.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M3 10h18" stroke="currentColor" stroke-width="1.8"/>
  <path d="M7 15h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export const iconPerson = `
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="8" r="3.5" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 19c1.5-3.2 4-4.8 7-4.8s5.5 1.6 7 4.8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export const iconDoc = `
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M7 3.5h7l4 4V20a1.5 1.5 0 0 1-1.5 1.5h-9.5A1.5 1.5 0 0 1 5.5 20V5A1.5 1.5 0 0 1 7 3.5z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M14 3.5V8h4.5" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
</svg>`;

export const iconCopy = `
<svg width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="8" y="8" width="11" height="11" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M5 15V6a2 2 0 0 1 2-2h9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export const iconClock = `
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.8"/>
  <path d="M12 8v4.5l3 1.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`;

export const iconWarn = `
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M12 4l9 16H3L12 4z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
  <path d="M12 10v4M12 17h.01" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export const iconRefresh = `
<svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M20 12a8 8 0 1 1-2.3-5.6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
  <path d="M20 5v5h-5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`;

export const iconArrow = `
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M5 12h14M13 6l6 6-6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`;

export const iconBolt = `
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M13 2L4 14h7l-1 8 10-14h-7l0-6z" fill="currentColor"/>
</svg>`;

export const iconShield = `
<svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <path d="M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6l7-3z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
</svg>`;

export const iconLock = `
<svg width="12" height="12" viewBox="0 0 24 24" fill="none" aria-hidden="true">
  <rect x="5" y="11" width="14" height="10" rx="2" stroke="currentColor" stroke-width="1.8"/>
  <path d="M8 11V8a4 4 0 1 1 8 0v3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>`;

export function bankMark(bankType, bankName) {
  const key = String(bankType || "").toLowerCase();
  if (key === "gtbank" || /gt\s*bank/i.test(bankName)) {
    return `<span class="kd-bank-mark kd-bank-gt" aria-hidden="true">GT</span>`;
  }
  if (key === "moniepoint" || /moniepoint/i.test(bankName)) {
    return `<span class="kd-bank-mark" aria-hidden="true">M</span>`;
  }
  const letter = (bankName || "B").trim().charAt(0).toUpperCase() || "B";
  return `<span class="kd-bank-mark" aria-hidden="true">${letter}</span>`;
}

export function bankFullName(bankType, bankName) {
  const key = String(bankType || "").toLowerCase();
  if (key === "gtbank" || /gt\s*bank/i.test(bankName)) {
    return "Guaranty Trust Bank";
  }
  if (key === "moniepoint" || /moniepoint/i.test(bankName)) {
    return "Moniepoint";
  }
  return bankName || "Bank";
}

/** Decorative SVG scene for the artwork pane */
export const artScene = `
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
</svg>`;
