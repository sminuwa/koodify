# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users
Merchants and their engineers integrating Koodify into a product or backend—first-time integrators and experienced payment teams.

## Product Purpose
Public merchant integration documentation for Koodify: authenticate with `pk_…` / `sk_…`, embed inline checkout, verify webhooks, use payment splits, and call stable `/api/v1` routes. Success means a merchant can go live without seeing Koodify-internal ops, platform secrets, or upstream provider details.

## Positioning
Merchant-facing integration docs only. Product marketing and platform feature storytelling live on the main website—not here.

## Operating Context
Static documentation site (`index.html`, `styles.css`, `app.js`). Readers arrive from onboarding or dashboards, copy samples, and ship checkout + webhooks.

## Capabilities and Constraints
- Illustrative API paths and payloads; confirm against the live API before production.
- Do not document platform-only credentials, rotate/admin endpoints, or internal infrastructure.
- Stack: plain HTML/CSS/JS with search, language tabs, copy buttons, and scroll-spy.

## Brand Commitments
- Product name: Koodify (lowercase wordmark in chrome).
- Audience: merchants integrating payments.

## Product Principles
1. Merchant integration clarity beats platform storytelling.
2. Wayfinding is the product: search, sidebar, and on-this-page stay obvious.
3. Code and endpoints are first-class content.
4. Security guidance stays practical (keys in browser vs server, webhook signature).
5. Examples stay honest about being illustrative.

## Accessibility & Inclusion
Keyboard navigation, skip link, focus visibility, and search/`/` shortcut must remain intact.
