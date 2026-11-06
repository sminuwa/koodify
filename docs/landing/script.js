const header = document.getElementById("site-header");
const nav = document.querySelector(".nav");
const toggle = document.querySelector(".menu-toggle");
const year = document.getElementById("year");

const setHeaderState = () => {
  header?.classList.toggle("is-scrolled", window.scrollY > 12);
};

toggle?.addEventListener("click", () => {
  const open = nav.classList.toggle("is-open");
  toggle.setAttribute("aria-expanded", String(open));
});

document.querySelectorAll(".nav-links a, .nav-actions a").forEach((link) => {
  link.addEventListener("click", () => {
    nav?.classList.remove("is-open");
    toggle?.setAttribute("aria-expanded", "false");
  });
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && nav?.classList.contains("is-open")) {
    nav.classList.remove("is-open");
    toggle?.setAttribute("aria-expanded", "false");
    toggle?.focus();
  }
});

window.addEventListener("scroll", setHeaderState, { passive: true });
setHeaderState();

if (year) year.textContent = String(new Date().getFullYear());

const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const reveals = document.querySelectorAll(".reveal");

if (reduceMotion || !("IntersectionObserver" in window)) {
  reveals.forEach((el) => el.classList.add("is-visible"));
} else {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-visible");
        observer.unobserve(entry.target);
      });
    },
    { threshold: 0.12, rootMargin: "0px 0px -6% 0px" }
  );

  reveals.forEach((el) => observer.observe(el));
}

/** Hero payment story — plays once to explain checkout → route → code → success */
const stage = document.getElementById("hero-stage");
const checkout = stage?.querySelector(".hero-checkout");
const routeCard = stage?.querySelector(".hero-float-route");
const codeCard = stage?.querySelector(".hero-float-code");
const successCard = stage?.querySelector(".hero-float-success");
const bankName = document.getElementById("hero-bank-name");
const routeBadge = document.getElementById("hero-route-badge");
const progressLabel = document.getElementById("hero-progress-label");

const show = (el) => el?.classList.add("is-in");
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const settleHero = () => {
  stage?.setAttribute("data-story", "3");
  stage?.classList.add("is-complete");
  show(checkout);
  show(routeCard);
  show(codeCard);
  show(successCard);
  if (bankName) bankName.textContent = "Moniepoint";
  if (routeBadge) routeBadge.hidden = false;
  if (progressLabel) progressLabel.textContent = "Payment received";
};

const playHeroStory = async () => {
  if (!stage || !checkout) return;

  if (reduceMotion) {
    settleHero();
    return;
  }

  show(checkout);
  stage.setAttribute("data-story", "0");
  if (progressLabel) progressLabel.textContent = "Creating temporary account…";
  await wait(700);

  show(routeCard);
  stage.setAttribute("data-story", "1");
  if (bankName) bankName.textContent = "Moniepoint";
  if (routeBadge) routeBadge.hidden = false;
  if (progressLabel) progressLabel.textContent = "Listening for transfer…";
  await wait(900);

  show(codeCard);
  stage.setAttribute("data-story", "2");
  await wait(850);

  show(successCard);
  stage.setAttribute("data-story", "3");
  stage.classList.add("is-complete");
  if (progressLabel) progressLabel.textContent = "Payment received";
};

if (stage) {
  if (reduceMotion) {
    settleHero();
  } else if ("IntersectionObserver" in window) {
    const storyObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          storyObserver.unobserve(entry.target);
          playHeroStory();
        });
      },
      { threshold: 0.35 }
    );
    storyObserver.observe(stage);
  } else {
    playHeroStory();
  }
}
