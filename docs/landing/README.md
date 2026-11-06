# Koodify landing page

A responsive, static landing page for Koodify, using the supplied Koodify logo asset.

## Files
- `index.html` — page structure and copy
- `styles.css` — responsive visual system and components
- `script.js` — mobile navigation and current footer year
- `assets/koodify-logo.png` — light-background logo (from logo pack)
- `assets/koodify-logo-white.png` — dark-background logo (header/footer)
- `assets/koodify-icon.png` — app mark used in hero mock
- `assets/brand/` — full Koodify logo pack (logo, icon, favicon, app-icons)
- `assets/features/` — v1 feature artworks used on the Platform section

## Preview locally
Open `index.html` in a browser, or run a local server from this directory:

```bash
python3 -m http.server 8000
```

Then visit `http://localhost:8000`.

## Before publishing
1. Replace the illustrative API snippet with the exact public Koodify SDK/API syntax.
2. Confirm the production dashboard, documentation, sign-in and sales-contact URLs; current CTAs point to the contact section or a mailto address.
3. Confirm which product capabilities are generally available before making public claims. The dashboard and payment UI are illustrative visual mockups, not live product data.
4. Add verified compliance/security statements, customer logos and testimonials only when approved.

## In this repo

Open `docs/landing/index.html`, or serve with:

```bash
python3 -m http.server 8000 --directory docs/landing
```
