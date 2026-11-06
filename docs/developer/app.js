const examples = {
  nodejs: {
    label: "NODE.JS · ZAINBOX WALLET",
    code: `const zainboxCode = process.env.KOODIFY_ZAINBOX_CODE;
const response = await fetch(
  \`https://api.koodify.ng/api/v1/zainbox/\${zainboxCode}/wallet\`,
  {
    headers: {
      "X-Api-Key": process.env.KOODIFY_SECRET_KEY // sk_live_…
    }
  }
);

const data = await response.json();
// data.data.balances / data.data.virtualAccounts`,
  },
  php: {
    label: "PHP · ZAINBOX WALLET",
    code: `$zainboxCode = getenv("KOODIFY_ZAINBOX_CODE");
$ch = curl_init("https://api.koodify.ng/api/v1/zainbox/" . rawurlencode($zainboxCode) . "/wallet");
curl_setopt_array($ch, [
  CURLOPT_RETURNTRANSFER => true,
  CURLOPT_HTTPHEADER => [
    "X-Api-Key: " . getenv("KOODIFY_SECRET_KEY"), // sk_live_…
  ],
]);
$response = curl_exec($ch);
$data = json_decode($response, true);`,
  },
  python: {
    label: "PYTHON · ZAINBOX WALLET",
    code: `import os, requests

zainbox_code = os.environ["KOODIFY_ZAINBOX_CODE"]
response = requests.get(
    f"https://api.koodify.ng/api/v1/zainbox/{zainbox_code}/wallet",
    headers={
        "X-Api-Key": os.environ["KOODIFY_SECRET_KEY"],  # sk_live_…
    },
    timeout=30,
)
data = response.json()`,
  },
  rust: {
    label: "RUST · ZAINBOX WALLET",
    code: `let zainbox_code = std::env::var("KOODIFY_ZAINBOX_CODE")?;
let response = reqwest::Client::new()
    .get(format!(
        "https://api.koodify.ng/api/v1/zainbox/{zainbox_code}/wallet"
    ))
    .header("X-Api-Key", std::env::var("KOODIFY_SECRET_KEY")?) // sk_live_…
    .send()
    .await?;

let data: serde_json::Value = response.json().await?;`,
  },
};

const tabs = [...document.querySelectorAll("#languageTabs .tab[data-lang]")];
const snippetLabel = document.getElementById("snippetLabel");
const snippetCode = document.getElementById("snippetCode");

tabs.forEach((tab) => {
  const initiallySelected = tab.classList.contains("active");
  tab.setAttribute("aria-selected", String(initiallySelected));
  tab.tabIndex = initiallySelected ? 0 : -1;

  tab.addEventListener("click", () => {
    const example = examples[tab.dataset.lang];
    if (!example) return;

    tabs.forEach((item) => {
      const selected = item === tab;
      item.classList.toggle("active", selected);
      item.setAttribute("aria-selected", String(selected));
      item.tabIndex = selected ? 0 : -1;
    });

    snippetLabel.textContent = example.label;
    snippetCode.textContent = example.code;
    snippetCode.setAttribute("aria-label", `${tab.textContent.trim()} code example`);
  });

  tab.addEventListener("keydown", (event) => {
    if (!["ArrowRight", "ArrowLeft", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const index = tabs.indexOf(tab);
    const nextIndex = event.key === "Home"
      ? 0
      : event.key === "End"
        ? tabs.length - 1
        : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
    tabs[nextIndex].focus();
    tabs[nextIndex].click();
  });
});

async function copyText(text) {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (_) {}
  }

  const field = document.createElement("textarea");
  field.value = text;
  field.setAttribute("readonly", "");
  field.style.position = "fixed";
  field.style.opacity = "0";
  document.body.appendChild(field);
  field.select();
  let copied = false;
  try {
    copied = document.execCommand("copy");
  } catch (_) {}
  field.remove();
  return copied;
}

document.querySelectorAll(".copy").forEach((button) => {
  button.type = "button";
  button.setAttribute("aria-live", "polite");
  button.addEventListener("click", async () => {
    const snippet = button.closest(".snippet")?.querySelector("pre");
    if (!snippet) return;

    const originalLabel = button.dataset.originalLabel || button.textContent.trim();
    button.dataset.originalLabel = originalLabel;
    const copied = await copyText(snippet.innerText || snippet.textContent);
    button.textContent = copied ? "Copied" : "Copy failed";
    button.classList.toggle("copy-error", !copied);
    window.clearTimeout(button.copyResetTimer);
    button.copyResetTimer = window.setTimeout(() => {
      button.textContent = originalLabel;
      button.classList.remove("copy-error");
    }, 1800);
  });
});

const sidebar = document.getElementById("docsNavigation");
const searchInput = document.getElementById("docsSearch");
const searchStatus = document.getElementById("searchStatus");
const navItems = [...sidebar.querySelectorAll(".side-link")];
const navGroups = [...sidebar.querySelectorAll(".side-group")];
const rightNavItems = [...document.querySelectorAll(".rightnav a")];
const rightNav = document.querySelector(".rightnav");
const articleSections = [...document.querySelectorAll(".article section[id]")];
const mobileJump = document.getElementById("mobileJump");

function filterDocs() {
  const query = searchInput.value.trim().toLocaleLowerCase();

  articleSections.forEach((section) => {
    section.hidden = Boolean(query) && !section.textContent.toLocaleLowerCase().includes(query);
  });

  navItems.forEach((link) => {
    const target = document.querySelector(link.getAttribute("href"));
    const isNestedApiLink = link.closest(".side-sub");
    const searchableText = `${link.textContent} ${isNestedApiLink ? "" : target?.textContent || ""}`;
    link.hidden = Boolean(query) && !searchableText.toLocaleLowerCase().includes(query);
  });

  rightNavItems.forEach((link) => {
    const target = document.querySelector(link.getAttribute("href"));
    const searchableText = `${link.textContent} ${target?.textContent || ""}`;
    link.hidden = Boolean(query) && !searchableText.toLocaleLowerCase().includes(query);
  });
  rightNav.hidden = Boolean(query) && !rightNavItems.some((link) => !link.hidden);

  navGroups.forEach((group) => {
    group.hidden = Boolean(query) && !group.querySelector(".side-link:not([hidden])");
  });

  const matches = [...new Set(navItems.filter((link) => !link.hidden).map((link) => link.hash))];
  searchStatus.textContent = query
    ? matches.length
      ? `${matches.length} matching ${matches.length === 1 ? "topic" : "topics"}. Press Enter to open the first result.`
      : "No matches. Try another search term."
    : "";

  [...mobileJump.options].forEach((option) => {
    if (!option.value) return;
    const target = document.querySelector(option.value);
    const text = `${option.textContent} ${target?.textContent || ""}`.toLocaleLowerCase();
    option.disabled = Boolean(query) && !text.includes(query);
  });
  mobileJump.disabled = Boolean(query) && ![...mobileJump.options].some((option) => option.value && !option.disabled);
  if (mobileJump.options[mobileJump.selectedIndex]?.disabled) mobileJump.value = "";

  if (query) {
    const firstMatch = navItems.find((link) => !link.hidden);
    const target = firstMatch && document.querySelector(firstMatch.getAttribute("href"));
    if (target) setCurrentSection(target);
  }
}

searchInput.addEventListener("input", filterDocs);
searchInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    const firstResult = navItems.find((link) => !link.hidden);
    if (firstResult) firstResult.click();
  }
});

document.addEventListener("keydown", (event) => {
  const target = event.target;
  const typing = target instanceof HTMLElement &&
    (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT", "BUTTON"].includes(target.tagName));

  if (event.key === "/" && !typing) {
    event.preventDefault();
    searchInput.focus();
  }

  if (event.key === "Escape" && document.activeElement === searchInput) {
    searchInput.value = "";
    filterDocs();
    searchInput.blur();
  }
});

mobileJump.addEventListener("change", () => {
  if (mobileJump.value) {
    window.location.hash = mobileJump.value;
    mobileJump.blur();
  }
});

const allSectionLinks = [...document.querySelectorAll('.side-link[href^="#"], .rightnav a[href^="#"]')];
const sectionTargets = [...new Set(allSectionLinks.map((link) => document.querySelector(link.getAttribute("href"))).filter(Boolean))];
const crumbCurrent = document.querySelector(".crumb span");
const crumbLabels = {
  overview: "Getting started",
  quickstart: "Getting started",
  authentication: "Getting started",
  environments: "Getting started",
  checkout: "API Guides",
  "virtual-accounts": "API Guides",
  "payment-splits": "API Guides",
  webhooks: "API Guides",
  "api-guides": "API Guides",
  api: "API reference",
  "api-va-create": "API reference",
  "api-checkout": "API reference",
  "api-transfer": "API reference",
  "api-activity": "API reference",
  "api-merchant-txns": "API reference",
  errors: "Resources",
  security: "Resources",
  "api-zainbox-wallet": "API reference",
};

function setCurrentSection(section) {
  if (!section?.id) return;
  const hash = `#${section.id}`;
  allSectionLinks.forEach((link) => {
    const active = link.hash === hash;
    link.classList.toggle("active", active);
    if (active) link.setAttribute("aria-current", "location");
    else link.removeAttribute("aria-current");
  });

  const matchingMobileOption = [...mobileJump.options].find((option) => option.value === hash);
  if (matchingMobileOption) mobileJump.value = hash;
  if (crumbCurrent) crumbCurrent.textContent = crumbLabels[section.id] || "Documentation";
}

if ("IntersectionObserver" in window) {
  const intersections = new Map();
  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => intersections.set(entry.target, entry.isIntersecting));
    const visible = [...intersections]
      .filter(([, isIntersecting]) => isIntersecting)
      .map(([target]) => target)
      .sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top);
    if (visible.length) setCurrentSection(visible[visible.length - 1]);
  }, { rootMargin: "-18% 0px -72% 0px" });

  sectionTargets.forEach((target) => observer.observe(target));
} else {
  sectionTargets.forEach((target) => {
    const link = document.querySelector(`.side-link[href="#${target.id}"]`);
    if (link) link.addEventListener("click", () => setCurrentSection(target));
  });
}

const initialTarget = location.hash ? document.querySelector(location.hash) : null;
setCurrentSection(initialTarget?.closest(".article") ? initialTarget : document.getElementById("overview"));
filterDocs();

const readProgress = document.getElementById("readProgress");

function updateReadProgress() {
  if (!readProgress) return;
  const scrollTop = window.scrollY || document.documentElement.scrollTop;
  const height = document.documentElement.scrollHeight - window.innerHeight;
  const ratio = height > 0 ? Math.min(1, Math.max(0, scrollTop / height)) : 0;
  readProgress.style.width = `${(ratio * 100).toFixed(2)}%`;
}

window.addEventListener("scroll", updateReadProgress, { passive: true });
window.addEventListener("resize", updateReadProgress, { passive: true });
updateReadProgress();
