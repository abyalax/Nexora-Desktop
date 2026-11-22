# 0008 — Role of the HTML design

Status: accepted · Scope: `design/`

## Context

`design/nexora-v1.html` (3 020 lines) is a prototype. It shows 184 settings, but only 17 have a live effect, it loads
Tailwind from a CDN (breaks offline), and it contains known bugs (self-referencing `--accent`, 15 duplicated keys,
a leading space in a font option). Treating it as a second product would double the work.

## Decision

* The HTML is a **visual and interaction reference**, not a product. The Rust app is the product.
* The settings catalogue lives in one file, `design/settings-schema.js`; the HTML reads it. It must match
  `docs/settings_catalog.md` (checked by `node --test design/tests`, see HTM-01…).
* **H boxes never block R boxes.** A Rust task is done without its HTML box.
* An HTML effect is written only for visual or behavior capabilities. Items that a browser cannot show (boot method,
  tray, power modes, notifications, taskbar hiding) are marked **H: n/a**.
* When core and design disagree for an existing setting, core wins and the design changes; for a new setting the
  design wins (SPEC-03).

## Consequences

* M8 (HTML parity) stays optional and ends with a schema parity report Rust ↔ HTML.
