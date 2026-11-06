# Koodify landing page

Static marketing site for Koodify, redesigned as a midnight phosphor-terminal experience.

## Files
- `index.html` — page structure and copy
- `styles.css` — phosphor CRT visual system
- `script.js` — nav, clock, typed prompt, illustrative event log
- `assets/koodify-logo.png` — brand logo
- `assets/fonts/VT323-Regular.ttf` — self-hosted terminal face
- `PRODUCT.md` / `DESIGN.md` — product truth and design system

## Preview locally

From the repo root:

```bash
python3 -m http.server 8000 --directory docs/landing
```

Visit `http://localhost:8000`, or open `docs/landing/index.html` directly.

## Before publishing
1. Replace the illustrative API snippet with the exact public Koodify SDK/API syntax.
2. Confirm production dashboard, docs, sign-in, and sales URLs; CTAs currently use `#` anchors or `hello@koodify.ng`.
3. Confirm generally-available capabilities before hardening claims. Dashboard/event panels are illustrative.
4. Add verified compliance marks, logos, and testimonials only when approved.
