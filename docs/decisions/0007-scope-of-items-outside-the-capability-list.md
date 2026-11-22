# 0007 — Scope of items outside the capability list

Status: accepted · Scope: `docs/feature.md`, `docs/settings_catalog.md`, capability slices in `TODO.md`

## Context

The design prototype and `feature.md` contain items that have no capability ID, are cut off, conflict with product
claims, or cannot be shown to be feasible. Each one is decided here so no row is built on a guess.

## Decision

| Item                                                  | Decision | Notes                                                                                                                                            |
| ----------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Screen page (brightness overlay, night light, target) | **In**   | New capability `SCR`. Page has 4 settings: brightness overlay, night light, night light intensity, display target (SCR-01…04).                                                            |
| Toothless eye companion (8 settings, ILB-10…17)       | **Out for now** | Design-only. No I-15, no core timing model. Revisit after M3. Rows stay `⊘` and are not built in Rust.                                      |
| Screen mirroring toggle (FDI-02)                      | **Drop** | Remove from spec and design schema.                                                                                                              |
| "Start as a Service" boot method                      | **Drop** | Conflicts with README ("no background services", installer needs no administrator rights). Keep *Don't start*, *Registry Value*, *Scheduled Task*. |
| Dock display option "All displays"                    | **Drop** | The display list shows *Primary display* and each detected display (D-12, `shared::monitors::pick`). One Dock per display is not planned.        |
| Dock display mode "Only display with…"                | **Drop** | The spec text is cut off and no behavior is defined. Dock modes become 7.                                                                         |
| Hot-corner action "Advanced"                          | **Drop** | No behavior defined. `HotAction` gains the three missing real actions only (see SPEC-03).                                                        |
| WAN-06 "Remove windows for excluded programs"         | **Define** | Toggle. When on, windows of programs in the *excluded programs* list are not animated and are left out of the Stage Manager window list and the switcher. Maintainer to confirm before WAN-06 starts. |
| Battery "power plans" (FBA-04…06)                     | **Decide: power modes** | The design options (Best power efficiency / Balanced / Best performance) are Windows 11 power *modes* (overlay schemes), not classic plans. UI label becomes *Power mode*. Exact API to be confirmed in the FBA spike. |
| Tray icons (F-11) and notifications (F-12)            | **Spike first** | One time-boxed spike each, result recorded as an addendum to this ADR, before any of their rows start. Rows stay blocked until then.        |

## Consequences

* SPEC-01 writes the pages marked **In** into `feature.md`; dropped items are removed from spec and design schema.
* Settings count in `TODO.md` §5 drops by the dropped rows (FDI-02, the "All displays" option, etc.) when SPEC-02
  regenerates the catalog.
