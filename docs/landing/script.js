const nav = document.querySelector('.nav');
const toggle = document.querySelector('.menu-toggle');
toggle?.addEventListener('click', () => {
  const open = nav.classList.toggle('open');
  toggle.setAttribute('aria-expanded', String(open));
});
document.querySelectorAll('.nav-links a').forEach((link) => {
  link.addEventListener('click', () => {
    nav.classList.remove('open');
    toggle?.setAttribute('aria-expanded', 'false');
  });
});

document.getElementById('year').textContent = String(new Date().getFullYear());

const clock = document.getElementById('clock');
const tick = () => {
  const now = new Date();
  clock.textContent = now.toTimeString().slice(0, 5);
};
tick();
setInterval(tick, 30_000);

const typed = document.getElementById('typed-line');
const message = 'payment infrastructure for products that move money';
const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

if (typed) {
  if (reduceMotion) {
    typed.textContent = message;
  } else {
    let i = 0;
    const typeNext = () => {
      typed.textContent = message.slice(0, i);
      i += 1;
      if (i <= message.length) {
        window.setTimeout(typeNext, 28);
      }
    };
    typeNext();
  }
}

const log = document.getElementById('event-log');
const lines = [
  'payment.success   KOOD-28491',
  'webhook.delivered attempt=1 verified=true',
  'route.selected    provider=A',
  'reconcile.matched ₦125,000',
  'payment.created   ref=order_29011',
  'webhook.retry     attempt=2 ok',
];
let lineIndex = 0;

if (log && !reduceMotion) {
  window.setInterval(() => {
    const stamp = new Date().toTimeString().slice(0, 5);
    const level = lineIndex % 3 === 1 ? 'OK  ' : 'INFO';
    const next = `${stamp} ${level} ${lines[lineIndex % lines.length]}`;
    const current = log.textContent.trim().split('\n');
    current.unshift(next);
    log.textContent = current.slice(0, 5).join('\n');
    lineIndex += 1;
  }, 4200);
}
