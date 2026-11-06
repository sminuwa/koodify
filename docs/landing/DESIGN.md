---
name: Koodify Landing
description: Midnight phosphor CRT terminal for payment infrastructure marketing
colors:
  crt-ground: "#050805"
  crt-void: "#020302"
  panel: "#08110c"
  panel-deep: "#050a07"
  phosphor: "#5dff88"
  phosphor-bright: "#9dffb0"
  phosphor-dim: "#3d9a5c"
  phosphor-string: "#7dff9a"
  line: "#2a6b3f"
  warn: "#c8ff6a"
  focus: "#b8ffc9"
  ink-black: "#000000"
  footer-ground: "#030503"
typography:
  display:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "clamp(4rem, 9vw, 7rem)"
    fontWeight: 400
    lineHeight: 1.15
    letterSpacing: "0.02em"
  headline:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "clamp(2.4rem, 5vw, 3.6rem)"
    fontWeight: 400
    lineHeight: 1.05
    letterSpacing: "normal"
  title:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "28px"
    fontWeight: 400
    lineHeight: 1.2
    letterSpacing: "normal"
  subtitle:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "24px"
    fontWeight: 400
    lineHeight: 1.25
    letterSpacing: "normal"
  body:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "20px"
    fontWeight: 400
    lineHeight: 1.55
    letterSpacing: "normal"
  lede:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "22px"
    fontWeight: 400
    lineHeight: 1.55
    letterSpacing: "normal"
  label:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "18px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.08em"
  code:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.55
    letterSpacing: "normal"
  caption:
    fontFamily: "VT323, Courier New, monospace"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.08em"
rounded:
  none: "0"
spacing:
  xs: "8px"
  sm: "12px"
  md: "18px"
  lg: "28px"
  xl: "36px"
  section: "88px"
  gutter: "20px"
  container: "1180px"
components:
  button-primary:
    backgroundColor: "{colors.phosphor}"
    textColor: "{colors.crt-ground}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "14px 22px"
  button-primary-hover:
    backgroundColor: "{colors.phosphor-bright}"
    textColor: "{colors.crt-ground}"
    rounded: "{rounded.none}"
    padding: "14px 22px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.phosphor}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "14px 22px"
  button-ghost-hover:
    backgroundColor: "color-mix(in srgb, #5dff88 12%, transparent)"
    textColor: "{colors.phosphor-bright}"
    rounded: "{rounded.none}"
    padding: "14px 22px"
  panel-terminal:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.phosphor}"
    rounded: "{rounded.none}"
    padding: "16px"
  code-window:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.phosphor}"
    rounded: "{rounded.none}"
    padding: "0"
  nav-link:
    backgroundColor: "transparent"
    textColor: "{colors.phosphor-dim}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
  nav-link-active:
    backgroundColor: "transparent"
    textColor: "{colors.phosphor-bright}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
  solution-row:
    backgroundColor: "transparent"
    textColor: "{colors.phosphor}"
    rounded: "{rounded.none}"
    padding: "24px 26px"
  cta-panel:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.phosphor}"
    rounded: "{rounded.none}"
    padding: "48px 40px"
---

# Design System: Koodify Landing

## Overview

**Creative North Star: "Phosphor Terminal (midnight CRT)"**

Koodify’s public landing reads as a live midnight operator session, not a fintech brochure. Near-black CRT ground, single self-hosted VT323 face, and phosphor green ink turn payment infrastructure into typed prompts, numbered commands, and scrolling event transcripts. Depth comes from 1px green rules, a fixed scanline veil, and hard black offset shadows on terminal panels—never soft SaaS elevation or rounded marketing cards.

The system is dense but legible: body sits at 20px monospace, ledes max around 62ch, and secondary copy dims to afterimage green. Bloom (soft green glow) is rare—reserved for the blinking cursor, the active nav phosphor dot, and the solid primary CTA. Interaction states print like log lines: uppercase nav indices (`01_HOME`), shell-style section commands (`$ cat platform.txt`), and illustrative event streams labeled as synthetic where a visitor could mistake them for live product data.

**Key Characteristics:**
- Single face: self-hosted VT323 with Courier New monospace fallback
- Phosphor triad (bright / ink / dim) on near-black CRT ground
- Square terminal panels with 1px `#2a6b3f` rules and hard black offsets (8–16px)
- Fixed scanline multiply veil over the full CRT frame
- Numbered nav (`01_HOME`…`06_CONTACT`) and shell command section labels
- Bloom only on cursor, active nav indicator, and primary CTA

## Colors

A single phosphor accent family on deep CRT neutrals—no second brand hue.

### Primary
- **Phosphor Ink** (`phosphor`): Default body text, borders on CTAs, meters, and selection fill. The working green of the terminal.
- **Phosphor Bright** (`phosphor-bright`): Headlines, active nav, prompt text, keywords, and hover lifts. The lit phosphor peak.
- **Phosphor Dim** (`phosphor-dim`): Status meta, inactive nav, notes, line numbers, and afterimage chrome. Recedes without leaving the green family.
- **Warn Chartreuse** (`warn`): Step heads (`[01] ACCEPT // COLLECT`) and numeric accents in code—alert ink inside the same CRT register, not a second brand color.

### Neutral
- **CRT Ground** (`crt-ground`): Theme color and CRT radial mid-field; also primary-button text.
- **CRT Void** (`crt-void`): Body canvas behind the CRT ellipse—slightly deeper than ground.
- **Panel** (`panel`): Terminal surfaces (hero aside, flow articles, code window, CTA).
- **Panel Deep** (`panel-deep`): Nested snippet wells inside flow articles.
- **Line Green** (`line`): All structural 1px rules—header, panels, strips, lists, footer.
- **Focus Mint** (`focus`): `:focus-visible` outline only.
- **Ink Black** (`ink-black`): Hard offset shadow pigment and inset meter framing.
- **Footer Ground** (`footer-ground`): Footer band, a hair darker than panel stack.

### Named Rules
**The Phosphor Triad Rule.** Text and chrome stay inside bright / ink / dim greens plus warn chartreuse. Do not introduce navy, purple, or warm SaaS accents.

**The Bloom Rarity Rule.** Soft green glow appears only on the cursor, the active nav phosphor square, and the primary CTA. Panels use hard black offsets, not diffuse glow.

## Typography

**Display Font:** VT323 (self-hosted `assets/fonts/VT323-Regular.ttf`, fallback Courier New, monospace)
**Body Font:** VT323 (same stack)
**Label/Mono Font:** VT323 (same stack—one voice)

**Character:** A single bitmap terminal face at all sizes. Hierarchy is size, brightness, and letter-spacing—never weight (everything ships at 400) or a second family.

### Hierarchy
- **Display** (400, `clamp(4rem, 9vw, 7rem)`, 1.15, tracking `0.02em`): Hero brand line (`KOODIFY / PAYMENT LAYER`); phosphor-bright with light text bloom.
- **Headline** (400, `clamp(2.4rem, 5vw, 3.6rem)`, 1.05): Section titles; often preceded by a dim `.cmd` shell line at 18px.
- **Title** (400, 24–28px): Flow / solution / trust headings in phosphor-bright.
- **Body** (400, 20px / 1.55; ledes 22px; mobile body 18px): Running copy; max ~48–62ch depending on column.
- **Label** (400, 16–18px, tracking `0.08em`–`0.14em`): Nav, status bar, strip, panel block titles, code chrome. Panel titles may use dashed underline rules.

### Named Rules
**The One Face Rule.** Never pair VT323 with a UI sans or display serif. Scale and phosphor level create hierarchy.

**The Log-Line Rule.** Nav indices, status chips, step heads, and event rows read as terminal output (`01_HOME`, `SYS:OK`, `[01] ACCEPT // COLLECT`)—not marketing eyebrows.

## Layout

Content locks to a centered wrap at `min(1180px, calc(100% - 40px))` (tightens to `min(100% - 28px, 540px)` under 700px). Sections pad `88px` vertically with `20px` horizontal; alt sections tint the panel color at 70% and add 1px line rules on both block edges.

First viewport is a two-column hero (`1.15fr / 0.85fr`, 36px gap) collapsing to one column at 900px. Platform flow articles are two-column rows inside square panels; developer and trust sections use ~equal two-column grids (48–56px gap). Solutions stack as a single bordered list with hairline dividers—no card grid. CTA is a full-width terminal panel inside the wrap. Sticky header stacks a status bar over numbered nav; under 700px nav collapses behind a square 1px-border menu toggle.

## Elevation & Depth

Hybrid CRT depth: tonal panel lift on void ground, a fixed scanline multiply veil, and hard rectangular black offsets on major panels. Soft bloom is not a general elevation tool.

### Shadow Vocabulary
- **Panel offset** (`box-shadow: 8px 12px 0 #000`): Hero terminal aside.
- **Code offset** (`box-shadow: 10px 14px 0 #000`): Developer code window.
- **CTA offset** (`box-shadow: 12px 16px 0 #000`): Contact command panel—largest hard shadow.
- **Primary bloom** (`box-shadow: 0 0 22px color-mix(in srgb, #5dff88 45%, transparent)`): Solid primary CTA only.
- **Cursor / active bloom** (`text-shadow` / `0 0 12px` phosphor mix): Blinking cursor and active nav square.

Scanlines: fixed full-viewport repeating 2px clear / 1px `rgba(0,0,0,0.18)` bands, `mix-blend-mode: multiply`, opacity reduced to 0.55 on small screens.

### Named Rules
**The Hard Offset Rule.** Terminal panels lift with opaque black offsets (8–16px), never large soft drop shadows. Soft glow stays on interactive phosphor only.

**The Veil Rule.** The scanline layer is always present over the CRT frame and never interactive (`pointer-events: none`).

## Shapes

Everything is square. No border-radius ships in the stylesheet (`rounded.none` = `0`). Form language is the 1px phosphor line, dashed inner rules under panel titles, and rectangular meters (12px tall) filled by a hard phosphor gradient stop. Nested blocks (panel-block, snippet, solution rows) share the same sharp silhouette—terminal panes and transcript wells, not cards.

### Named Rules
**The No-Radius Rule.** Do not round buttons, panels, inputs, or images. Square CRT geometry is the brand silhouette.

## Components

### Buttons
Command chips, not pills. 1px phosphor border, 14×22 padding, 20px type, tracking `0.06em`, 0.18s ease on color/shadow.
- **Primary:** Phosphor fill, CRT-ground text, bloom shadow; hover → phosphor-bright fill.
- **Ghost:** Transparent fill, phosphor text; hover → 12% phosphor wash and bright text.
- **Focus:** 2px `focus` outline, 3px offset (global `:focus-visible`).

### Cards / Containers
Not SaaS cards—terminal panels and transcript blocks.
- **Corner Style:** Square (0)
- **Background:** `panel` (snippets `panel-deep`)
- **Shadow Strategy:** Hard black offsets on hero panel, code window, CTA; none on flow/solution rows
- **Border:** 1px `line`
- **Internal Padding:** 16–26px typical; CTA 48×40 (32×22 on small screens)

### Navigation
Sticky header on 92% CRT-ground with 2px blur and bottom 1px line. Status bar prints `operator@koodify:~$` plus `SYS:OK`, clock, and signal bars. Links use `01_HOME`-style indices in dim ink; active/current link goes bright with a 10px phosphor square and bloom. Logo is the product PNG forced to phosphor via CSS filter. Mobile: hamburger is a square bordered control revealing a stacked link column.

### Signature: Hero prompt + event panel
Left: `>` prompt with typewriter line (~28ms/char, disabled under reduced motion), display headline, 34ch lede, primary/ghost CTA pair, dim note. Right: bordered hero panel stacking CORE STATUS / RECENT EVENTS / HEALTH meters; event log rotates illustrative lines every 4.2s when motion is allowed.

### Signature: Code window
Head/footer chrome in dim ink, line-numbered body, keyword bright / string `#7dff9a` / number warn. Hard 10×14 black offset.

### Signature: Flow steps & solution list
Flow: square panel rows with warn step heads, bright titles, snippet wells. Solutions: single outlined stack, row dividers only—no separate card shadows.

### Signature: Trust log & strip
Trust entries are divider-separated log blocks. Audience strip is a letterspaced dim row between 1px rules with the first token (`BUILT FOR`) in full phosphor.

## Do's and Don'ts

### Do:
- **Do** keep VT323 as the only face and express hierarchy with size + phosphor level.
- **Do** use 1px `line` rules and hard black offsets (8–16px) on major terminal panels.
- **Do** reserve bloom for cursor, active nav indicator, and primary CTA.
- **Do** label illustrative sessions/API samples as illustrative when they could be mistaken for live product.
- **Do** keep numbered nav and shell-style section commands as the wayfinding voice.

### Don't:
- **Don't** introduce rounded corners, soft multi-layer card shadows, or floating SaaS cards.
- **Don't** add a second typeface or non-phosphor accent family.
- **Don't** spray glow on every panel—hard offset is the panel lift; bloom is the live phosphor.
- **Don't** invent customer logos, compliance badges, or live metrics as factual proof.
- **Don't** replace log-line states with marketing eyebrows, pill chips, or badge clusters.
