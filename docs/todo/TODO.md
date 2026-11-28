# Nexora Desktop — Atomic TODO Plan

Snapshot: `abyalax/Nexora-Desktop` at `281a75c` (shallow clone, 2026-11-22). Inputs: `docs/feature.md` (settings overview),
`docs/feature_checkpoint.md` (capability spec and status), `docs/architecture.md`, `AGENTS.md`, `design/nexora-v1.html`
(3,020 lines), and the Rust workspace (`app`, `core`, `platform`, `ui`, `guard`).

This file is a plan only. Nothing here has been implemented.

## 0. How to use this file

**Atomic task** = one commit, one behavior, test written first, ≤ ~150 changed lines, verifiable on its own. One PR may
carry several commits of the same page or capability.

**The two tracks.**

- **Rust (R)** is the product. A Rust box is done without the HTML box.
- **HTML (H)** is the design prototype. H boxes never block R boxes. Items that cannot be shown in a browser (boot method,
  tray, power plans, notifications, taskbar hiding) are marked **H: n/a**.

**Setting recipe.** Every setting in section 5 is one row with five boxes, in this order:

| Box | Step              | Done when                                                                                                                                                                                                                    |
| --- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Sp  | Spec              | Row exists in `docs/feature.md` with type, range and default                                                                                                                                                                 |
| Co  | Core              | Field in its section struct, `Default`, codec parse (clamp, garbage keeps old value), serialize only when ≠ default (ADR-0004). Tests: default, clamp low/high, garbage, round-trip, 0.4.0 golden files still byte-identical |
| UI  | Rust settings row | Row on the right page, label in English **and** Indonesian                                                                                                                                                                   |
| Fx  | Rust effect       | Consumer reads the value live (no restart). Pure logic → core test. Window/OS behavior → entry in `docs/qa/`                                                                                                                 |
| H   | HTML effect       | Handler registered in the HTML `effects` map (HTM-04)                                                                                                                                                                        |

Rule: **no UI row lands without its effect** (UI and Fx in the same PR). Core (Co) may land earlier. This is what
prevents the HTML problem: 167 of 184 HTML settings are saved but do nothing.

**Definition of done** for every task (from `AGENTS.md` §7):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo check --workspace
```

Plus: test seen failing first · new labels in EN + ID · `docs/feature_checkpoint.md` status updated · Windows-only
behavior has a QA entry · HTML change passes `node --test design/tests`.

**Legend for "Rust today"**: ✔ field exists in `Settings` · ◐ drafted only in an orphan file (not compiled) · ~ exists but
the model differs from the design · ✗ does not exist · ⊘ not in the spec / design-only.

**ID prefixes**: `ADR-` decisions · `SPEC-` documentation · `BASE-` baseline/CI · `ENG-` settings engine · `WIN-` settings
window · `HTM-` HTML foundations · `D-/F-/I-n.m` capability slices (spec IDs) · `GEN/STG/WAN/SCR/REC/QA-` cross-cutting.

---

## 1. What the audit found

These findings shape the order of work. Each one has a task below.

1. **A file is not compiled.** `crates/core/src/dock/behavior.rs` (331 lines: `DockConfig`, `IconEffect`, `HideEffect`, 6 tests)
   is not declared in `core/dock/mod.rs`. Adding `pub mod behavior;` fails: `crate::settings::parse_bool` is private to
   `codec.rs`. It already models 14 of the Dock settings (◐ rows below). → BASE-02
2. **Six app files are not declared.** `crates/app/src/dock/{popups,magnify,surfaces,items,geometry,prefs_sync}.rs`
   (885 lines) are not in `app/dock/mod.rs`. I could not build `app` here (Windows-only), so whether HEAD builds is
   unknown. → BASE-01, BASE-02
3. **Golden tests block new fields.** `Settings::serialize` is one 64-key `format!` that always writes every key, and
   `tests/settings_golden.rs` requires 0.4.0 files to round-trip **byte for byte**. Any new emitted key breaks them. → ADR-0004
4. **`Settings` is `Copy` and read by value.** `prefs::get()` returns `Settings` by value; text uses fixed-size `ShortText`.
   Six design settings are lists/free text (excluded programs, blacklists, deleted Start Menu programs) and cannot live
   there. → ADR-0005
5. **The checkpoint overstates step 6.** `feature_checkpoint.md` marks "Settings schema split" ✅, but only
   `DockSettings`, `MenubarSettings` and `IslandSettings` exist. `GeneralSettings`, `AppearanceSettings`,
   `StageSettings`, `InputSettings` (§5.4) do not. → ENG-03…06, SPEC-04
6. **The Settings window is a different shape.** Rust has 9 pages (Umum, Deck, Visor, Menu Bar, Tampilan, Scene, Screen
   Corners, Quick Search, Widget), custom-drawn, `page: u8`, with only `Toggle/Seg/Slider(u8)/Pick/Swatches/Thumbs/Radios/Text`
   rows. The spec has 20 pages in 4 groups and needs 6–10 option selects, sliders up to 1500, and a colour row. → ADR-0006, WIN-01…05
7. **The spec has holes.** `feature.md` lists Screen, Notifications, Time & Date, Audio, Display and Hardware Status in
   the sidebar but gives them no content. The design invents 27 settings there (plus 8 "Toothless eye" settings). → ADR-0007, SPEC-01
8. **Design ≠ core in 18 places** (ranges, option sets, defaults), e.g. `dock_size` clamps 32–72 vs design 24–96,
   `HotAction` has 7 values vs 10 in the spec, `notch_idle` has 3 styles vs 4, online lyrics default is on in core and
   the guide but off in the design. → SPEC-03
9. **Naming conflict.** `feature.md` says code uses *Finder* (`FinderSettings`); code uses `menubar` (`core/menubar/`,
   `MenubarSettings`); the checkpoint titles the section *Menubar*. → ADR-0003
10. **HTML bugs (verified in the file).** `:root { --accent: var(--accent) }` is a self-reference; 15 keys are duplicated
    between `shared` and `finder` (the `shared` copies have no UI); the font option `" system-ui"` has a leading space;
    Tailwind loads from a CDN so the prototype breaks offline; only 17 of 184 settings have a live effect. → HTM-01…06
11. **No safety net yet.** No `.github/`, no `rust-toolchain.toml`; `Cargo.toml` says edition 2021 while `AGENTS.md` says
    2024. Core tests do pass on Linux: in a standalone copy of `crates/core`, 262 unit tests (1 ignored) + 2 golden + 4
    section tests passed. → BASE-04…06
12. **Conflicts with product claims.** The spec's "Start as a Service" boot method conflicts with the README ("no background
    services", installer needs no administrator rights). → ADR-0007

---

## 2. Milestones

| M   | Goal                   | Exit criterion                                                                                                                      |
| --- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| M0  | Decisions and spec     | ADR-0003…0008 accepted; SPEC-01…05 merged                                                                                           |
| M1  | Foundations            | Windows baseline green; orphans resolved; CI on; settings engine + 4 section structs; new Settings controls; HTML schema test green |
| M2  | Settings window shape  | 20-page sidebar with groups; empty pages show only rows that already have effects                                                   |
| M3  | Dynamic Island         | I-14, idle/mode/material reconciliation; eye companion per ADR-0007                                                                 |
| M4  | Dock                   | D-1, D-3, D-5…D-17 slices                                                                                                           |
| M5  | Finder                 | F-1…F-18 slices (tray and notifications after their spikes)                                                                         |
| M6  | Shared pages           | General, Appearance surfaces, Stage Manager gaps, Window Animation, Screen (if accepted)                                            |
| M7  | Hardening              | Recovery tests, footprint check, i18n completeness, upgrade test, docs                                                              |
| M8  | HTML parity (optional) | Schema parity report Rust ↔ HTML                                                                                                    |

**Start here (first 10 tasks):** BASE-01 → BASE-02 → ADR-0003…0006 (one sitting) → SPEC-03 → BASE-04 → HTM-01/02/03 →
ENG-01 → ENG-03…06 → WIN-01/02/03/05 → I-14 as the first full vertical slice (smallest, proves the recipe in both tracks).

---

## M0 — Decisions and spec

**Status: done (documentation only; no Rust or HTML changed).** ADR-0003…0008 are in `docs/decisions/` (status
*accepted*, following the recommendations below; the items marked "Maintainer to confirm" in ADR-0007 are proposals).
SPEC-01…05 are merged into `docs/`. Code work that these decisions unlock is in M1 (ENG-03 does the Finder rename).

### Decisions (each becomes `docs/decisions/000x-*.md`)

- [x] **ADR-0003 Finder vs Menubar in code.** Options: (a) follow `feature.md`: rename `core/menubar`, `MenubarSettings`,
  `app/menubar` to `finder` using `pub use` aliases like ADR-0002; (b) change the rule in `feature.md` to *Menubar*.
  Recommendation: (a), before any `FinderSettings` field is added. UI strings stay *Menu Bar*. Blocks: ENG-03…, M5.
- [x] **ADR-0004 Settings persistence.** Keep flat `key=value`. Serialize new keys only when ≠ default (0.4.0 golden files
  stay byte-identical). Replace the 64-key `format!` and the giant `match` with a table-driven registry
  (key, type, range, default, owner). Add an optional `settings_version` line. Blocks: every Co box.
- [x] **ADR-0005 List-valued settings.** Options: (a) drop `Copy` from `Settings` (touches every `prefs::get()` call);
  (b) keep `Settings: Copy`, put lists in a separate `lists.cfg` behind a `ProgramLists` snapshot (`Arc`);
  (c) fixed arrays. Recommendation: (b). Blocks: WAN-04/05, DLP-05, DMN-09.
- [x] **ADR-0006 Settings window information architecture.** Rebuild the sidebar to the spec (System / Dock / Finder /
  Dynamic Island groups). Decide where the Rust-only pages live (Scene, Screen Corners, Quick Search, Widget). Update
  `feature.md` §1 to match. Blocks: WIN-05.
- [x] **ADR-0007 Scope of items outside the capability list.** Decide in/out for each; default recommendation in brackets.
  - Screen page: brightness overlay, night light, display target [add as capability `SCR`, or drop]
  - Toothless eye companion, 8 settings [decide; if in, add capability I-15 and a core timing model]
  - Screen mirroring toggle [drop]
  - "Start as a Service" boot method [drop; keep Registry + Scheduled Task]
  - Dock display option "All displays" [define: one Dock per display, or remove]
  - "Only display with…" Dock mode (text is cut off in the spec) [define or drop]
  - Hot-corner action "Advanced" [define or drop]
  - "Remove windows for excluded programs" (WAN-06) [define]
  - Battery "power plans": the design options (Best power efficiency / Balanced / Best performance) look like Windows 11
    power *modes*, not classic power *plans* [decide which API]
  - Feasibility spikes: tray icons (F-11) and notifications (F-12), see slices
- [x] **ADR-0008 Role of the HTML design.** Keep as visual/interaction reference. Schema in `design/settings-schema.js` is
  the single catalogue the HTML reads. H boxes are non-blocking. H effect only for visual/behavior capabilities.

### Spec work

- [x] **SPEC-01** Write the missing pages into `docs/feature.md` per ADR-0007: Screen, Notifications, Time & Date, Audio,
  Display, Hardware Status, Island Activities (27 settings have no spec row).
- [x] **SPEC-02** Create `docs/settings_catalog.md` (generated from §5; 175 in scope, 8 deferred, 1 dropped) from the tables in §5 (id, label, key, type, range, default, owner
  section, capability). Link it from `feature.md`.
- [x] **SPEC-03** Reconcile design vs core, recorded in `docs/spec_reconciliation.md` (the design-side edits are applied by HTM-01…). Proposed rule: for an **existing** setting, core's range/default wins and the
  design changes; for a **new** setting, the design wins. One commit per row:
  - [x] `dock_size` 32–72 vs 24–96 · [ ] `dock_gap` 0–24 vs 0–30 · [ ] `dock_margin` 0–60 vs 0–80
  - [x] `notch_round` 0–100 vs 0–48 · [ ] `blur` 0–100 vs 0–48 · [ ] `ui_scale` 100/125/150 vs 0.75–1.5 slider
  - [x] `accent` palette index vs hex · [ ] `autostart` bool vs 4 methods · [ ] `takeover` bool vs 3 states
  - [x] `hide_desktop_icons` bool vs 3 states · [ ] `stage_speed` percent vs 5 steps · [ ] `stage_size` card size vs icon size
  - [x] `above_fullscreen` ("always above full-screen") vs spec "automatically exit when playing full screen" (looks opposite)
  - [x] `notch_idle` 3 styles vs 4 · [ ] `Behavior` Fixed/Overlay/Smart vs spec Fixed/Smart/On demand
  - [x] `Material` Blur/Glass/Black vs Glass/Opaque/Translucent · [ ] `HotAction` 7 values vs 10
  - [x] defaults: `menubar` false vs `enableFinder` true · `Lang::Id` vs English · `lyrics_online` true vs false ·
    `notch_send` true vs false · `DockConfig` defaults (labels, separate, light intensity, delay) vs design
- [x] **SPEC-04** Re-verify every ✅/🟡/⬜ in `feature_checkpoint.md` (stale rows fixed and ⬜ rows spot-checked; the full ✅ audit needs Windows: BASE-01, BASE-12) (derived at commit `1e95f83`). Known stale: step 6;
  D-7 says Genie is ⬜ but `core/window_management/genie.rs` exists (is Funnel = Genie?); D-3 lock and D-14 effects are
  drafted in `behavior.rs`.
- [x] **SPEC-05** Fix doc drift: README "Modular" table paths vs the spec's sidebar, edition statement in `AGENTS.md`,
  `architecture.md` targets that do not exist (`taskbar/`, `thiserror`, `tracing`, CI).

---

## M1 — Foundations

### Baseline and CI

- [x] **BASE-01** On Windows 11, run the four DoD commands at HEAD. Record what fails. Fix only what blocks the build. All four passed on 2026-11-27; the initial test run found one missing English label, which was added before the passing rerun. Clippy reports existing `too_many_arguments` warnings.
- [x] **BASE-02** Resolve orphans:
  - [x] each of `app/dock/{popups,magnify,surfaces,items,geometry,prefs_sync}.rs`: wire into `mod.rs` or delete
  - [x] `core/dock/behavior.rs`: make `parse_bool` `pub(crate)`, add `pub mod behavior;`, run its 6 tests.
  - [x] ADR-0004 keeps `DockConfig` composed as Dock-domain behavior; `DockSettings` remains the persisted section projection.
- [x] **BASE-03** Orphan guard: a small script that fails CI when a `*.rs` file has no matching `mod` declaration. `python3.12 scripts/check_orphans.py` passed on 2026-11-27.
- [x] **BASE-04** `.github/workflows/ci.yml`: Ubuntu job `cargo test -p nexora-core`; Windows job fmt, clippy, test, check
  for the workspace; Node job `node --test design/tests`.
- [x] **BASE-05** Add `rust-toolchain.toml` (listed in `architecture.md` §2, missing).
- [x] **BASE-06** Edition: migrate to 2024 per `AGENTS.md`, or amend `AGENTS.md` to 2021 (decide, then do). ADR-0009 keeps 2021 through M7.
- [x] **BASE-07** Test helpers in `core/tests/`: one macro that generates the five Co tests (default, clamp low/high,
  garbage, round-trip) from a registry row, so a setting costs one line.
- [x] **BASE-08** Platform test convention: tests that need a live desktop use `#[ignore = "needs interactive session"]`;
  Windows runner runs `cargo test -p nexora-platform -- --ignored` on demand. Keep `media_fake.rs` style fakes for core-facing traits.
- [x] **BASE-09** `--settings-page <id>` flag next to `--open-settings`/`--exit-after`; smoke run per page must exit 0. Settings page IDs 0–5 all started, answered `state`/`wait_idle`, and exited through `nexora-ctl` on 2026-11-27.
- [x] **BASE-10** `docs/qa/TEMPLATE.md`: preconditions, steps, expected result, restore check (taskbar, work area).
- [x] **BASE-11** Run `tools/bench/m1-verify.ps1` for a baseline; record idle private memory in `docs/bench.md`.
  Optimized release with debug assertions for `nexora-ctl`: 22.36 MB private, after 30 s idle on the primary display.
  Re-run at the end of each milestone.

### Settings engine (core)

- [x] **ENG-01** Implement ADR-0004: `SETTINGS_REGISTRY` covers frozen legacy and extension rows; codec uses row callbacks and `settings_version`. New files write version 2; 0.4.0 fixtures remain byte-identical as version 1. Metadata/default/uniqueness, invalid-value, golden, extension and upgrade tests pass.
- [x] **ENG-02** Fixture policy: add `settings_0_5_0_*.cfg` per milestone; never edit existing fixtures. `settings_0_5_0_activities.cfg` is covered by `settings_extra` and section round-trip tests.
- [x] **ENG-03** `GeneralSettings` (`takeover, autostart, lang, ui_scale, above_fullscreen, anim`) + test like `settings_sections.rs`.
- [x] **ENG-04** `AppearanceSettings` (`theme, accent, blur, style, transparency, refraction, win_shadow, icon_*`) + test.
- [x] **ENG-05** `StageSettings` (`stage, stage_*, switcher, hide_desktop_icons`) + test.
- [x] **ENG-06** `InputSettings` (`hot, mac_shortcuts, clipboard_history`) + test.
- [x] **ENG-07** `SurfaceGlass { custom, blur, transparency, border }` + `resolve(global, surface)`; tests: custom off → global.
- [x] **ENG-08** `ProgramList` (ADR-0005): parse `;` lists, normalize, match by exe/name. Tests: case, spaces, empty, duplicates,
  trailing `;`, length cap.
- [x] **ENG-09** Setting dependencies (`customDockBlur` → enables its sliders; `iconEffect` → enables magnification only if it
  magnifies). Pure function + tests, used by the UI to grey rows.
- [x] **ENG-10** i18n test: every app catalogue label has Indonesian source text and an English translation.
- [x] **ENG-11** Limits for new text fields (greeting text uses fixed `ShortText`; decide the cap for new ones). `ShortText` caps UTF-8 text at 96 bytes; program lists separately cap entries in `settings/lists.rs`.

### Settings window (app/ui)

- [x] **WIN-01** Wide numeric slider row (min, max, step, unit; values to 1500 and fractional). Pure value↔position function in core, tested (`NumericSlider`); activation and preview delays use 0–1500 ms rows.
- [ ] **WIN-02** Select row for 6+ options (hot corner has 10, Dock display mode 8); keyboard navigation.
- [ ] **WIN-03** Colour row (hex or RGB) for accent and Dock light colour.
- [x] **WIN-04** List/text row for `ProgramList` values. Four `lists.cfg` collections have semicolon-separated text rows and save through `prefs::update_lists`.
- [ ] **WIN-05** Grouped sidebar with sub-pages (Dock ▸ 5, Finder ▸ 7, Dynamic Island ▸ 2); replace `page: u8` with a `Page` enum.
- [x] **WIN-06** Disabled-row state driven by ENG-09. Gated sliders dim and omit hit regions when disabled.
- [x] **WIN-07** Sidebar search (the HTML has one). Optional. Implemented by `page_matches`, which filters pages by their group and row labels.

### HTML foundations

- [x] **HTM-01** Move `defaultSettings` and `prefPages` into `design/settings-schema.js` (no DOM) so Node can import it.
- [x] **HTM-02** `design/tests/schema.test.mjs` (`node --test`): no key in two sections, every descriptor has a
  default and every default a descriptor, default inside range/options, option strings trimmed (fails today: `" system-ui"`),
  unique IDs.
- [x] **HTM-03** Fix the verified bugs: `--accent: var(--accent)`; the 15 duplicate `shared` keys; `" system-ui"`.
- [ ] **HTM-04** Replace the `if (key === …)` chain in `updateSetting` with an `effects['section.key']` map. Add a script that
  prints unwired keys (167 today); the count may only go down.
- [ ] **HTM-05** Stop loading Tailwind from a CDN (vendor it or precompile). Low priority.
- [ ] **HTM-06** Group setting cards by spec section instead of "every 6 fields".

---

## M2 — Settings catalogue (the atomic unit)

Generated from the design schema (`prefPages` + `defaultSettings`), then mapped by hand to the Rust code. Boxes are
**Sp Co UI Fx H**. Pre-ticked: Sp where the row is in `feature.md`, Co where the field exists, H where the HTML already
has a live effect. UI/Fx for ✔ rows are unticked on purpose: **BASE-12** audits them once and ticks what is true.
The `Rust today` column is my reading of field names; re-check when you start a row.

- [ ] **BASE-12** Audit pass over all ✔/~/◐ rows: confirm Co/UI/Fx against code and tick.

| Page                             | Prefix | Settings |      ✔ |      ◐ |      ~ |       ✗ |      ⊘ |
| -------------------------------- | ------ | -------: | -----: | -----: | -----: | ------: | -----: |
| System ▸ General                 | GEN    |       12 |      4 |      0 |      6 |       2 |      0 |
| System ▸ Appearance              | APP    |       33 |      0 |      0 |      0 |      33 |      0 |
| System ▸ System Icon Tray        | TRY    |        4 |      0 |      0 |      0 |       4 |      0 |
| System ▸ Screen                  | SCR    |        4 |      0 |      0 |      0 |       0 |      4 |
| System ▸ Stage Manager           | STG    |       15 |      7 |      0 |      3 |       5 |      0 |
| System ▸ Window Animation        | WAN    |        6 |      1 |      0 |      0 |       5 |      0 |
| Dock ▸ Look & Behavior           | DLB    |       10 |      4 |      4 |      0 |       2 |      0 |
| Dock ▸ Launchpad                 | DLP    |        7 |      0 |      0 |      0 |       7 |      0 |
| Dock ▸ Advanced                  | DAD    |       19 |      0 |      8 |      3 |       8 |      0 |
| Dock ▸ Minimize Effects          | DMN    |        9 |      0 |      0 |      1 |       8 |      0 |
| Dock ▸ Window Previews           | DPV    |        6 |      0 |      2 |      0 |       4 |      0 |
| Finder ▸ Look & Behavior         | FLB    |       14 |      0 |      0 |      0 |      14 |      0 |
| Finder ▸ Notifications           | FNO    |        3 |      0 |      0 |      0 |       3 |      0 |
| Finder ▸ Time & Date             | FTD    |        4 |      0 |      0 |      1 |       3 |      0 |
| Finder ▸ Audio                   | FAU    |        3 |      0 |      0 |      0 |       3 |      0 |
| Finder ▸ Display                 | FDI    |        2 |      0 |      0 |      0 |       1 |      1 |
| Finder ▸ Hardware Status         | FHW    |        3 |      0 |      0 |      0 |       3 |      0 |
| Finder ▸ Battery                 | FBA    |        6 |      0 |      0 |      1 |       5 |      0 |
| Dynamic Island ▸ Look & Behavior | ILB    |       17 |      6 |      0 |      3 |       0 |      8 |
| Dynamic Island ▸ Activities      | IAC    |        7 |      4 |      0 |      0 |       3 |      0 |
| **Total**                        |        |  **184** | **26** | **14** | **18** | **113** | **13** |

Reading: 26 settings are fully backed today, 14 only in the orphan file, 18 need a model change, 113 do not exist, 13 are
outside the spec. Types: 88 toggles, 53 sliders, 35 selects, 2 colours, 6 text.

#### System ▸ General (12)

| ID     | Setting                                        | Key · type · default                                                                                                     | Rust today                                                     | Cap | Sp Co UI Fx H |
| ------ | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | --- | ------------- |
| GEN-01 | Enable Finder                                  | `shared.enableFinder` · toggle · on                                                                                      | ✔ `menubar`                                                    | F-* | ☑☑☐☐☑         |
| GEN-02 | Enable Dock                                    | `shared.enableDock` · toggle · on                                                                                        | ✔ `dock_enabled`                                               | D-* | ☑☑☐☐☑         |
| GEN-03 | Automatically Exit When Playing On Full Screen | `shared.exitOnFullscreen` · toggle · on                                                                                  | ~ `above_fullscreen` (reads as the opposite behavior: confirm) | GEN | ☑☑☐☐☐         |
| GEN-04 | Global Interface Scale                         | `shared.globalScale` · range 0.75–1.5 · 1                                                                                | ~ `ui_scale` (steps 100/125/150; design is a .75–1.5 slider)   | GEN | ☑☑☐☐☑         |
| GEN-05 | Global Interface Blur Intensity                | `shared.globalBlur` · range 0–48 · 26                                                                                    | ~ `blur` (0–100; design 0–48)                                  | GEN | ☑☑☐☐☑         |
| GEN-06 | Appearance                                     | `shared.appearance` · select [light \| dark \| system] · dark                                                            | ✔ `theme` (Auto/Light/Dark)                                    | GEN | ☑☑☐☐☑         |
| GEN-07 | Accent Color                                   | `shared.accentColor` · color · #63c7dc                                                                                   | ~ `accent` (palette index; design is hex)                      | GEN | ☑☑☐☐☑         |
| GEN-08 | Start On Boot Method                           | `shared.startOnBoot` · select [Don't Start On Boot \| Registry Value \| Scheduled Task \| Start as a Service] · disabled | ~ `autostart` (bool; design has 4 methods)                     | GEN | ☑☑☐☐☐         |
| GEN-09 | Automatically hide the taskbar on boot         | `shared.hideTaskbarOnBoot` · select [none \| allDisplays \| dockDisplay] · none                                          | ~ `takeover` (bool; design has 3 states)                       | GEN | ☑☑☐☐☐         |
| GEN-10 | Display Language                               | `shared.displayLanguage` · select [English \| Indonesian] · English                                                      | ✔ `lang`                                                       | GEN | ☑☑☐☐☐         |
| GEN-11 | Display Font                                   | `shared.displayFont` · select (5 options) · Segoe UI                                                                     | ✗                                                              | GEN | ☑☑☐☐☐         |
| GEN-12 | Global Font Scaling                            | `shared.globalFontScale` · range 0.75–1.5 · 1                                                                            | ✗                                                              | GEN | ☑☑☐☐☐         |

#### System ▸ Appearance (33)

| ID     | Setting                                                           | Key · type · default                                 | Rust today | Cap  | Sp Co UI Fx H |
| ------ | ----------------------------------------------------------------- | ---------------------------------------------------- | ---------- | ---- | ------------- |
| APP-01 | Make Dock use custom blur and transparency values                 | `shared.customDockBlur` · toggle · off               | ✗          | D-15 | ☑☑☐☐☐         |
| APP-02 | Dock blur intensity                                               | `shared.dockBlur` · range 0–50 · 22                  | ✗          | D-15 | ☑☑☐☐☐         |
| APP-03 | Dock transparency                                                 | `shared.dockTransparency` · range 0–100 · 62         | ✗          | D-15 | ☑☑☐☐☐         |
| APP-04 | Make pinned icons reflect from the Dock                           | `shared.pinnedReflections` · toggle · off            | ✗          | D-15 | ☑☑☐☐☐         |
| APP-05 | Reflection blur strength                                          | `shared.reflectionBlur` · range 0–30 · 5             | ✗          | D-15 | ☑☑☐☐☐         |
| APP-06 | Reflection transparency                                           | `shared.reflectionTransparency` · range 0–100 · 35   | ✗          | D-15 | ☑☑☐☐☐         |
| APP-07 | Icon reflection distance                                          | `shared.reflectionDistance` · range 0–20 · 3         | ✗          | D-15 | ☑☑☐☐☐         |
| APP-08 | Gradient distance of icon reflection                              | `shared.reflectionGradientDistance` · range 0–30 · 8 | ✗          | D-15 | ☑☑☐☐☐         |
| APP-09 | Make Dock icon labels use custom blur and transparency values     | `shared.customIconLabels` · toggle · off             | ✗          | D-15 | ☑☑☐☐☐         |
| APP-10 | Label blur intensity                                              | `shared.labelBlur` · range 0–40 · 18                 | ✗          | D-15 | ☑☑☐☐☐         |
| APP-11 | Label transparency                                                | `shared.labelTransparency` · range 0–100 · 70        | ✗          | D-15 | ☑☑☐☐☐         |
| APP-12 | Label border transparency                                         | `shared.labelBorderTransparency` · range 0–100 · 12  | ✗          | D-15 | ☑☑☐☐☐         |
| APP-13 | Make context menus use custom blur and transparency values        | `shared.customContextMenus` · toggle · off           | ✗          | D-15 | ☑☑☐☐☐         |
| APP-14 | Context menu blur intensity                                       | `shared.contextBlur` · range 0–50 · 22               | ✗          | D-15 | ☑☑☐☐☐         |
| APP-15 | Context menu transparency                                         | `shared.contextTransparency` · range 0–100 · 85      | ✗          | D-15 | ☑☑☐☐☐         |
| APP-16 | Context menu border transparency                                  | `shared.contextBorder` · range 0–100 · 12            | ✗          | D-15 | ☑☑☐☐☐         |
| APP-17 | Make folder contents grid use custom blur and transparency values | `shared.customFolderGrid` · toggle · off             | ✗          | D-17 | ☑☑☐☐☐         |
| APP-18 | Folder grid blur intensity                                        | `shared.folderBlur` · range 0–50 · 22                | ✗          | D-17 | ☑☑☐☐☐         |
| APP-19 | Folder grid transparency                                          | `shared.folderTransparency` · range 0–100 · 85       | ✗          | D-17 | ☑☑☐☐☐         |
| APP-20 | Folder grid border transparency                                   | `shared.folderBorder` · range 0–100 · 12             | ✗          | D-17 | ☑☑☐☐☐         |
| APP-21 | Folder grid icon size                                             | `shared.folderIconSize` · range 24–96 · 48           | ✗          | D-17 | ☑☑☐☐☐         |
| APP-22 | Make Nexora menus use custom blur and transparency values         | `shared.customNexoraMenus` · toggle · off            | ✗          | D-15 | ☑☑☐☐☐         |
| APP-23 | Nexora menu blur intensity                                        | `shared.menuBlur` · range 0–50 · 26                  | ✗          | D-15 | ☑☑☐☐☐         |
| APP-24 | Nexora menu transparency                                          | `shared.menuTransparency` · range 0–100 · 90         | ✗          | D-15 | ☑☑☐☐☐         |
| APP-25 | Nexora menu border transparency                                   | `shared.menuBorder` · range 0–100 · 10               | ✗          | D-15 | ☑☑☐☐☐         |
| APP-26 | Make Finder use custom blur and transparency values               | `shared.customFinderBlur` · toggle · off             | ✗          | F-7  | ☑☑☐☐☐         |
| APP-27 | Finder blur intensity                                             | `shared.finderBlur` · range 0–50 · 26                | ✗          | F-7  | ☑☑☐☐☐         |
| APP-28 | Finder transparency                                               | `shared.finderTransparency` · range 0–100 · 88       | ✗          | F-7  | ☑☑☐☐☐         |
| APP-29 | Wallpaper tint intensity                                          | `shared.wallpaperTint` · range 0–100 · 0             | ✗          | F-7  | ☑☑☐☐☐         |
| APP-30 | Launchpad uses custom background blur                             | `shared.launchpadCustomBlur` · toggle · off          | ✗          | D-8  | ☑☑☐☐☐         |
| APP-31 | Display Launchpad above windows instead of hiding them            | `shared.launchpadAboveWindows` · toggle · off        | ✗          | D-8  | ☑☑☐☐☐         |
| APP-32 | Launchpad blurs the background instead of pre-rendering it        | `shared.launchpadBlurBackground` · toggle · on       | ✗          | D-8  | ☑☑☐☐☐         |
| APP-33 | Launchpad background blur intensity                               | `shared.launchpadBlur` · range 0–50 · 24             | ✗          | D-8  | ☑☑☐☐☐         |

#### System ▸ System Icon Tray (4) — H: n/a

| ID     | Setting                                      | Key · type · default                         | Rust today | Cap  | Sp Co UI Fx H |
| ------ | -------------------------------------------- | -------------------------------------------- | ---------- | ---- | ------------- |
| TRY-01 | Always display all tray icons in Finder      | `shared.trayAllFinder` · toggle · off        | ✗          | F-11 | ☑☑☐☐☐         |
| TRY-02 | Always display all tray icons in Dock        | `shared.trayAllDock` · toggle · off          | ✗          | F-11 | ☑☑☐☐☐         |
| TRY-03 | Display the system's own tray icon in Finder | `shared.traySystemFinder` · toggle · on      | ✗          | F-11 | ☑☑☐☐☐         |
| TRY-04 | Newly run tray icon display time (seconds)   | `shared.newTrayIconSeconds` · range 0–30 · 5 | ✗          | F-11 | ☑☑☐☐☐         |

#### System ▸ Screen (4) — not in `feature.md`, pending ADR-0007

| ID     | Setting                    | Key · type · default                                                                                 | Rust today | Cap | Sp Co UI Fx H |
| ------ | -------------------------- | ---------------------------------------------------------------------------------------------------- | ---------- | --- | ------------- |
| SCR-01 | Display brightness overlay | `shared.screenBrightness` · range 0–100 · 100                                                        | ⊘          | —   | ☐☑☐☐☐         |
| SCR-02 | Enable night light         | `shared.nightLight` · toggle · off                                                                   | ⊘          | —   | ☐☑☐☐☐         |
| SCR-03 | Night light intensity      | `shared.nightLightIntensity` · range 0–100 · 25                                                      | ⊘          | —   | ☐☑☐☐☐         |
| SCR-04 | Display target             | `shared.screenTarget` · select [All displays \| Primary display \| Active display] · Primary display | ⊘          | —   | ☐☑☐☐☐         |

#### System ▸ Stage Manager (15)

| ID     | Setting                                              | Key · type · default                                                                                     | Rust today                                                            | Cap | Sp Co UI Fx H |
| ------ | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | --- | ------------- |
| STG-01 | Show Stage Manager icon in Finder                    | `shared.stageIcon` · toggle · on                                                                         | ✗                                                                     | STG | ☑☑☐☐☐         |
| STG-02 | Show window title on mouseover                       | `shared.stageWindowTitle` · toggle · on                                                                  | ✔ `stage_titles`                                                      | STG | ☑☑☐☐☐         |
| STG-03 | Blur background window icons                         | `shared.stageBlurIcons` · toggle · on                                                                    | ✗                                                                     | STG | ☑☑☐☐☐         |
| STG-04 | Click on the desktop to hide all windows             | `shared.stageClickDesktopHide` · toggle · off                                                            | ✔ `stage_desktop_click`                                               | STG | ☑☑☐☐☐         |
| STG-05 | Automatically pause and play background window media | `shared.stageAutoPauseMedia` · toggle · on                                                               | ✔ `stage_media`                                                       | STG | ☑☑☐☐☐         |
| STG-06 | Show native window preview on hover                  | `shared.stageNativePreview` · toggle · on                                                                | ✗                                                                     | STG | ☑☑☐☐☐         |
| STG-07 | Show Desktop Icons                                   | `shared.stageDesktopIcons` · select [Always Hide Icon \| Auto Show Hide \| Always Show] · Auto Show Hide | ~ `hide_desktop_icons` (bool; design has 3 states)                    | STG | ☑☑☐☐☐         |
| STG-08 | Adjust Animation Speed                               | `shared.stageAnimationSpeed` · select (5 options) · 1.0x                                                 | ~ `stage_speed` (percent; design has 5 steps)                         | STG | ☑☑☐☐☐         |
| STG-09 | Open Stage Manager                                   | `shared.stageEnabled` · toggle · off                                                                     | ✔ `stage`                                                             | STG | ☑☑☐☐☐         |
| STG-10 | Window list screen position                          | `shared.stagePosition` · select [left \| right] · left                                                   | ✔ `stage_right`                                                       | STG | ☑☑☐☐☐         |
| STG-11 | Window List Show Hidden Mode                         | `shared.stageHiddenMode` · select [default \| alwaysShow \| alwaysHide] · default                        | ✗                                                                     | STG | ☑☑☐☐☐         |
| STG-12 | Keep Window Border                                   | `shared.stageKeepBorder` · toggle · off                                                                  | ✗                                                                     | STG | ☑☑☐☐☐         |
| STG-13 | Window list icon size                                | `shared.stageIconSize` · range 16–80 · 36                                                                | ~ `stage_size` (170, looks like card size; design is 16–80 icon size) | STG | ☑☑☐☐☐         |
| STG-14 | Window list icon spacing                             | `shared.stageIconSpacing` · range 0–30 · 8                                                               | ✔ `stage_gap`                                                         | STG | ☑☑☐☐☐         |
| STG-15 | Number of window list icons                          | `shared.stageIconCount` · range 1–20 · 8                                                                 | ✔ `stage_count`                                                       | STG | ☑☑☐☐☐         |

#### System ▸ Window Animation (6)

| ID     | Setting                                     | Key · type · default                                                             | Rust today                | Cap | Sp Co UI Fx H |
| ------ | ------------------------------------------- | -------------------------------------------------------------------------------- | ------------------------- | --- | ------------- |
| WAN-01 | Window opening animation                    | `shared.windowAnimation` · toggle · on                                           | ✔ `anim`                  | WAN | ☑☑☐☐☐         |
| WAN-02 | Window opening animation effect             | `shared.windowAnimationEffect` · select [scale \| fade \| slide \| zoom] · scale | ✗                         | WAN | ☑☑☐☐☐         |
| WAN-03 | Adjust animation speed                      | `shared.windowAnimationSpeed` · select (5 options) · 1.0x                        | ✗                         | WAN | ☑☑☐☐☐         |
| WAN-04 | Add excluded programs (semicolon separated) | `shared.excludedPrograms` · text · empty                                         | ✗ list value → ADR-0005   | WAN | ☑☑☐☐☐         |
| WAN-05 | Manage running program blacklist            | `shared.programBlacklist` · text · empty                                         | ✗ list value → ADR-0005   | WAN | ☑☑☐☐☐         |
| WAN-06 | Remove windows for excluded programs        | `shared.minimizeExcludedWindows` · toggle · on                                   | ✗ meaning unclear in spec | WAN | ☑☑☐☐☐         |

#### Dock ▸ Look & Behavior (10)

| ID     | Setting                         | Key · type · default                                                                                  | Rust today                                       | Cap  | Sp Co UI Fx H |
| ------ | ------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ---- | ------------- |
| DLB-01 | Icon size                       | `dock.iconSize` · range 24–96 · 48                                                                    | ✔ `dock_size` (clamp 32–72; design 24–96)        | D-5  | ☑☑☐☐☐         |
| DLB-02 | Icon effect                     | `dock.iconEffect` · select (6 options) · Scaling                                                      | ◐ `IconEffect` in orphan `core/dock/behavior.rs` | D-14 | ☑☑☐☐☐         |
| DLB-03 | Icon magnification              | `dock.iconMagnification` · range 1–2.5 · 1.4                                                          | ✔ `dock_zoom` (100–250 = 1.0–2.5×)               | D-5  | ☑☑☐☐☐         |
| DLB-04 | Icon light intensity            | `dock.iconLightIntensity` · range 0–100 · 35                                                          | ◐ `DockConfig.light_intensity`                   | D-14 | ☑☑☐☐☐         |
| DLB-05 | Icon background light intensity | `dock.iconBackgroundLight` · range 0–100 · 25                                                         | ◐ `DockConfig.bg_light_intensity`                | D-14 | ☑☑☐☐☐         |
| DLB-06 | Icon background light color     | `dock.iconBackgroundColor` · color · #63c7dc                                                          | ◐ `DockConfig.light_rgb` (design is hex)         | D-14 | ☑☑☐☐☐         |
| DLB-07 | Distance between icons          | `dock.iconSpacing` · range 0–30 · 8                                                                   | ✔ `dock_gap` (0–24; design 0–30)                 | D-5  | ☑☑☐☐☐         |
| DLB-08 | Dock distance from screen edge  | `dock.edgeDistance` · range 0–80 · 8                                                                  | ✔ `dock_margin` (0–60; design 0–80)              | D-5  | ☑☑☐☐☐         |
| DLB-09 | Position on screen              | `dock.position` · select [left \| bottom \| right \| top] · bottom                                    | ✗                                                | D-11 | ☑☑☐☐☐         |
| DLB-10 | Display on screen               | `dock.display` · select [Primary display \| All displays \| Display 1 \| Display 2] · Primary display | ✗                                                | D-12 | ☑☑☐☐☐         |

#### Dock ▸ Launchpad (7)

| ID     | Setting                                           | Key · type · default                                                                                           | Rust today              | Cap | Sp Co UI Fx H |
| ------ | ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ----------------------- | --- | ------------- |
| DLP-01 | Launchpad icon size                               | `dock.launchpadIconSize` · range 24–100 · 52                                                                   | ✗                       | D-8 | ☑☑☐☐☐         |
| DLP-02 | Icon horizontal spacing                           | `dock.launchpadHorizontalSpacing` · range 0–60 · 22                                                            | ✗                       | D-8 | ☑☑☐☐☐         |
| DLP-03 | Icon vertical spacing                             | `dock.launchpadVerticalSpacing` · range 0–60 · 24                                                              | ✗                       | D-8 | ☑☑☐☐☐         |
| DLP-04 | Choose which icons will be displayed by Launchpad | `dock.launchpadSources` · select [added \| addedStartMenu \| addedDesktop \| all] · added                      | ✗                       | D-8 | ☑☑☐☐☐         |
| DLP-05 | Edit deleted Start Menu programs                  | `dock.deletedStartMenuPrograms` · text · empty                                                                 | ✗ list value → ADR-0005 | D-8 | ☑☑☐☐☐         |
| DLP-06 | Select the screen to display Launchpad            | `dock.launchpadDisplay` · select [Primary display \| All displays \| Display 1 \| Display 2] · Primary display | ✗                       | D-8 | ☑☑☐☐☐         |
| DLP-07 | Follow Dock                                       | `dock.launchpadFollowDock` · toggle · on                                                                       | ✗                       | D-8 | ☑☑☐☐☐         |

#### Dock ▸ Advanced (19)

| ID     | Setting                                                               | Key · type · default                                                                                                                             | Rust today                                                                  | Cap  | Sp Co UI Fx H |
| ------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- | ---- | ------------- |
| DAD-01 | Scale all icons to macOS standards                                    | `dock.scaleMacIcons` · toggle · on                                                                                                               | ~ `dock_plates` (squircle plates: confirm same meaning)                     | D-9  | ☑☐☐☐☐         |
| DAD-02 | Double-click the folder icon to open in Finder                        | `dock.doubleClickFolderFinder` · toggle · on                                                                                                     | ✗                                                                           | D-17 | ☑☑☐☐☐         |
| DAD-03 | Hide the open folder button                                           | `dock.hideOpenFolderButton` · toggle · off                                                                                                       | ✗                                                                           | D-17 | ☑☑☐☐☐         |
| DAD-04 | Lock icon dragging                                                    | `dock.lockIconDragging` · toggle · off                                                                                                           | ◐ `DockConfig.lock_drag`                                                    | D-3  | ☑☑☐☐☐         |
| DAD-05 | Show icon labels                                                      | `dock.showIconLabels` · toggle · off                                                                                                             | ◐ `DockConfig.show_labels`                                                  | D-1  | ☑☑☐☐☐         |
| DAD-06 | Animate opening applications                                          | `dock.animateAppLaunch` · toggle · on                                                                                                            | ◐ `DockConfig.animate_launch`                                               | D-1  | ☑☑☐☐☐         |
| DAD-07 | Turn on window borders to prevent windows from hiding Dock and Finder | `dock.preventWindowsCoverDock` · toggle · on                                                                                                     | ✗                                                                           | D-10 | ☑☑☐☐☐         |
| DAD-08 | Show indicators under running programs                                | `dock.showRunningIndicators` · toggle · on                                                                                                       | ◐ `DockConfig.show_indicators`                                              | D-1  | ☑☑☐☐☐         |
| DAD-09 | Show icons of unpinned running programs                               | `dock.showUnpinnedRunning` · toggle · on                                                                                                         | ◐ `DockConfig.show_unpinned`                                                | D-1  | ☑☑☐☐☐         |
| DAD-10 | Separate unpinned running programs                                    | `dock.separateUnpinned` · toggle · off                                                                                                           | ◐ `DockConfig.separate_unpinned`                                            | D-1  | ☑☑☐☐☐         |
| DAD-11 | Bottom left hot-corner function                                       | `dock.bottomLeftHotCorner` · select (10 options) · Do nothing                                                                                    | ~ `hot[2]` (7 actions today; spec lists 10)                                 | D-16 | ☑☐☐☐☐         |
| DAD-12 | Dock display mode                                                     | `dock.displayMode` · select (8 options) · default                                                                                                | ~ `dock: Behavior` (Fixed/Overlay/Smart = 3 of 8 modes)                     | D-10 | ☑☐☐☐☐         |
| DAD-13 | Detection window                                                      | `dock.detectionWindow` · select [All Program Windows \| The Current Program Window] · All Program Windows                                        | ✗                                                                           | D-10 | ☑☑☐☐☐         |
| DAD-14 | Dock hidden effect                                                    | `dock.hiddenEffect` · select [Move \| Fade Away] · Move                                                                                          | ◐ `DockConfig.hide_effect`                                                  | D-10 | ☑☑☐☐☐         |
| DAD-15 | Dock activation trigger                                               | `dock.activationTrigger` · select [The edge of the screen \| Activation bar \| The edge of the screen & Activation bar] · The edge of the screen | ✗                                                                           | D-13 | ☑☑☐☐☐         |
| DAD-16 | Activation bar width                                                  | `dock.activationBarWidth` · range 1–100 · 4                                                                                                      | ✗                                                                           | D-13 | ☑☑☐☐☐         |
| DAD-17 | Activation bar height                                                 | `dock.activationBarHeight` · range 1–30 · 2                                                                                                      | ✗                                                                           | D-13 | ☑☑☐☐☐         |
| DAD-18 | Activation bar distance from screen edge                              | `dock.activationBarDistance` · range 0–40 · 0                                                                                                    | ✗                                                                           | D-13 | ☑☑☐☐☐         |
| DAD-19 | Mouse delay activates dock time (ms)                                  | `dock.mouseActivationDelay` · range 0–1500 · 150                                                                                                 | ◐ `DockConfig.activation_delay` (10 ms units, max 1000 ms; design max 1500) | D-13 | ☑☑☐☐☐         |

#### Dock ▸ Minimize Effects (9)

| ID     | Setting                                           | Key · type · default                                                                                                                                                                                  | Rust today                                  | Cap | Sp Co UI Fx H |
| ------ | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | --- | ------------- |
| DMN-01 | Use Nexora minimize effect                        | `dock.useNexoraMinimize` · toggle · on                                                                                                                                                                | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-02 | Don't animate minimizing windows                  | `dock.disableMinimizeAnimation` · toggle · off                                                                                                                                                        | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-03 | Hold down Shift to slow down an animation         | `dock.shiftSlowAnimation` · toggle · on                                                                                                                                                               | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-04 | Minimize windows into their Dock icon             | `dock.minimizeIntoIcon` · toggle · on                                                                                                                                                                 | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-05 | Dynamically display window content when minimized | `dock.dynamicMinimizeContent` · toggle · on                                                                                                                                                           | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-06 | Minimize window icon position                     | `dock.minimizedIconPosition` · select [Next to the icon on the right \| Independently displayed on the right \| Right side displayed independently using a separator] · Next to the icon on the right | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-07 | Minimize animation                                | `dock.minimizeAnimation` · select [Genie \| Scale \| Suck] · Genie                                                                                                                                    | ~ `min_effect` (0 Funnel, 1 Scale; no Suck) | D-7 | ☑☐☐☐☐         |
| DMN-08 | Minimize animation speed                          | `dock.minimizeAnimationSpeed` · select (5 options) · 1.0x                                                                                                                                             | ✗                                           | D-7 | ☑☑☐☐☐         |
| DMN-09 | Minimize animation blacklist                      | `dock.minimizeBlacklist` · text · empty                                                                                                                                                               | ✗ list value → ADR-0005                     | D-7 | ☑☑☐☐☐         |

#### Dock ▸ Window Previews (6)

| ID     | Setting                             | Key · type · default                     | Rust today                                              | Cap | Sp Co UI Fx H |
| ------ | ----------------------------------- | ---------------------------------------- | ------------------------------------------------------- | --- | ------------- |
| DPV-01 | Enable window previews              | `dock.windowPreviews` · toggle · on      | ◐ `DockConfig.preview_enabled`                          | D-6 | ☑☑☐☐☐         |
| DPV-02 | Window preview size                 | `dock.previewSize` · range 120–520 · 260 | ✗                                                       | D-6 | ☑☑☐☐☐         |
| DPV-03 | Preview delay on hover (ms)         | `dock.previewDelay` · range 0–1500 · 450 | ◐ `DockConfig.preview_delay` (max 1000 ms; design 1500) | D-6 | ☑☑☐☐☐         |
| DPV-04 | Application name                    | `dock.previewAppName` · toggle · on      | ✗                                                       | D-6 | ☑☑☐☐☐         |
| DPV-05 | Window control buttons              | `dock.previewControls` · toggle · on     | ✗                                                       | D-6 | ☑☑☐☐☐         |
| DPV-06 | Show native window preview on hover | `dock.previewNative` · toggle · on       | ✗                                                       | D-6 | ☑☑☐☐☐         |

#### Finder ▸ Look & Behavior (14)

| ID     | Setting                                                            | Key · type · default                              | Rust today | Cap  | Sp Co UI Fx H |
| ------ | ------------------------------------------------------------------ | ------------------------------------------------- | ---------- | ---- | ------------- |
| FLB-01 | Finder menu icon                                                   | `finder.menuIcon` · toggle · on                   | ✗          | F-1  | ☑☑☐☐☐         |
| FLB-02 | Auto-Hide Finder                                                   | `finder.autoHide` · toggle · off                  | ✗          | F-8  | ☑☑☐☐☐         |
| FLB-03 | Shut down confirmation box                                         | `finder.shutdownConfirmation` · toggle · on       | ✗          | F-9  | ☑☑☐☐☐         |
| FLB-04 | Maximized windows affect Finder appearance                         | `finder.maximizedAffectsAppearance` · toggle · on | ✗          | F-8  | ☑☑☐☐☐         |
| FLB-05 | Click the upper right corner to close the current maximized window | `finder.closeMaximizedOnCorner` · toggle · off    | ✗          | F-10 | ☑☑☐☐☐         |
| FLB-06 | Lock dragging tray icons                                           | `finder.lockTrayDragging` · toggle · off          | ✗          | F-11 | ☑☑☐☐☐         |
| FLB-07 | Show window options in Finder                                      | `finder.windowOptions` · toggle · on              | ✗          | F-3  | ☑☑☐☐☐         |
| FLB-08 | Show colored tray icons                                            | `finder.coloredTrayIcons` · toggle · on           | ✗          | F-11 | ☑☑☐☐☐         |
| FLB-09 | Show input method menu                                             | `finder.inputMethodMenu` · toggle · on            | ✗          | F-18 | ☑☑☐☐☐         |
| FLB-10 | Show search icon                                                   | `finder.searchIcon` · toggle · on                 | ✗          | F-18 | ☑☑☐☐☐         |
| FLB-11 | The account icon is displayed                                      | `finder.accountIcon` · toggle · on                | ✗          | F-18 | ☑☑☐☐☐         |
| FLB-12 | Displays the media playback icon                                   | `finder.mediaPlaybackIcon` · toggle · on          | ✗          | F-17 | ☑☑☐☐☐         |
| FLB-13 | Show music player lyrics menu                                      | `finder.lyricsMenu` · toggle · on                 | ✗          | F-17 | ☑☑☐☐☐         |
| FLB-14 | Show Color Player Lyrics                                           | `finder.colorPlayerLyrics` · toggle · on          | ✗          | F-17 | ☑☑☐☐☐         |

#### Finder ▸ Notifications (3) — not in `feature.md`, H: n/a

| ID     | Setting                                 | Key · type · default                           | Rust today | Cap  | Sp Co UI Fx H |
| ------ | --------------------------------------- | ---------------------------------------------- | ---------- | ---- | ------------- |
| FNO-01 | Show desktop notifications              | `finder.showNotifications` · toggle · on       | ✗          | F-12 | ☐☑☐☐☐         |
| FNO-02 | Notification sounds                     | `finder.notificationSounds` · toggle · on      | ✗          | F-12 | ☐☑☐☐☐         |
| FNO-03 | Notification display duration (seconds) | `finder.notificationDuration` · range 1–15 · 5 | ✗          | F-12 | ☐☑☐☐☐         |

#### Finder ▸ Time & Date (4) — not in `feature.md`

| ID     | Setting               | Key · type · default                                        | Rust today           | Cap | Sp Co UI Fx H |
| ------ | --------------------- | ----------------------------------------------------------- | -------------------- | --- | ------------- |
| FTD-01 | Show time in Menu Bar | `finder.showTime` · toggle · on                             | ✗                    | F-4 | ☐☑☐☐☐         |
| FTD-02 | Show date in Menu Bar | `finder.showDate` · toggle · on                             | ✗                    | F-4 | ☐☑☐☐☐         |
| FTD-03 | Time format           | `finder.timeFormat` · select [12-hour \| 24-hour] · 24-hour | ~ `clock_24h` (bool) | F-4 | ☐☑☐☐☐         |
| FTD-04 | Date format           | `finder.dateFormat` · select [Short \| Long \| ISO] · Short | ✗                    | F-4 | ☐☑☐☐☐         |

#### Finder ▸ Audio (3) — not in `feature.md`

| ID     | Setting                      | Key · type · default                     | Rust today | Cap  | Sp Co UI Fx H |
| ------ | ---------------------------- | ---------------------------------------- | ---------- | ---- | ------------- |
| FAU-01 | Show volume control          | `finder.showVolume` · toggle · on        | ✗          | F-13 | ☐☑☐☐☐         |
| FAU-02 | Show audio output selector   | `finder.showAudioOutput` · toggle · on   | ✗          | F-13 | ☐☑☐☐☐         |
| FAU-03 | Show media playback controls | `finder.showMediaControls` · toggle · on | ✗          | F-13 | ☐☑☐☐☐         |

#### Finder ▸ Display (2) — not in `feature.md`

| ID     | Setting                         | Key · type · default                       | Rust today | Cap  | Sp Co UI Fx H |
| ------ | ------------------------------- | ------------------------------------------ | ---------- | ---- | ------------- |
| FDI-01 | Show display brightness control | `finder.showBrightness` · toggle · on      | ✗          | F-14 | ☐☑☐☐☐         |
| FDI-02 | Show screen mirroring control   | `finder.showScreenMirroring` · toggle · on | ⊘          | —    | ☐☐☐☐☐         |

#### Finder ▸ Hardware Status (3) — not in `feature.md`

| ID     | Setting                  | Key · type · default                     | Rust today | Cap  | Sp Co UI Fx H |
| ------ | ------------------------ | ---------------------------------------- | ---------- | ---- | ------------- |
| FHW-01 | Show Wi-Fi status        | `finder.showWifi` · toggle · on          | ✗          | F-4  | ☐☑☐☐☐         |
| FHW-02 | Show Bluetooth status    | `finder.showBluetooth` · toggle · on     | ✗          | F-15 | ☐☑☐☐☐         |
| FHW-03 | Show device status icons | `finder.showHardwareIcons` · toggle · on | ✗          | F-15 | ☐☑☐☐☐         |

#### Finder ▸ Battery (6) — H: n/a for FBA-04…06

| ID     | Setting                                       | Key · type · default                                                                               | Rust today                                    | Cap  | Sp Co UI Fx H |
| ------ | --------------------------------------------- | -------------------------------------------------------------------------------------------------- | --------------------------------------------- | ---- | ------------- |
| FBA-01 | Show Battery Status menu in Finder            | `finder.showBatteryMenu` · toggle · on                                                             | ✗                                             | F-16 | ☑☑☐☐☐         |
| FBA-02 | Show Battery percentage in Finder status menu | `finder.batteryPercentage` · toggle · on                                                           | ~ `battery_pct` (3 modes; design is a toggle) | F-16 | ☑☐☐☐☐         |
| FBA-03 | Display the remaining battery time            | `finder.remainingBatteryTime` · toggle · on                                                        | ✗                                             | F-16 | ☑☑☐☐☐         |
| FBA-04 | Automatically switch power plans              | `finder.autoPowerPlans` · toggle · off                                                             | ✗                                             | F-16 | ☑☑☐☐☐         |
| FBA-05 | Power plan when plugged in                    | `finder.powerPlanPlugged` · select [balanced \| bestPerformance \| bestPowerEfficiency] · balanced | ✗                                             | F-16 | ☑☑☐☐☐         |
| FBA-06 | Power plan when using batteries               | `finder.powerPlanBattery` · select [balanced \| bestPerformance \| bestPowerEfficiency] · balanced | ✗                                             | F-16 | ☑☑☐☐☐         |

#### Dynamic Island ▸ Look & Behavior (17)

| ID     | Setting                      | Key · type · default                                                                       | Rust today                                                                 | Cap  | Sp Co UI Fx H |
| ------ | ---------------------------- | ------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ---- | ------------- |
| ILB-01 | Enable Dynamic Island        | `island.enabled` · toggle · on                                                             | ✔ `notch_enabled`                                                          | I-12 | ☑☑☐☐☑         |
| ILB-02 | Shape                        | `island.shape` · select [attached \| floating capsule] · attached                          | ✔ `notch_island`                                                           | I-2  | ☑☑☐☐☑         |
| ILB-03 | Display mode                 | `island.displayMode` · select [Fixed \| Smart \| On demand] · Smart                        | ~ `notch: Behavior` (Fixed/Overlay/Smart; spec says Fixed/Smart/On demand) | I-12 | ☑☑☐☐☐         |
| ILB-04 | Open on hover                | `island.openOnHover` · toggle · on                                                         | ✔ `notch_hover_open`                                                       | I-12 | ☑☑☐☐☑         |
| ILB-05 | Idle style                   | `island.idleStyle` · select [Minimal \| Toothless eyes \| Media \| Clock] · Toothless eyes | ~ `notch_idle` (0..2; design has 4 styles)                                 | I-3  | ☑☑☐☐☐         |
| ILB-06 | Greeting name                | `island.greetingName` · text · empty                                                       | ✔ `greet_name`                                                             | I-3  | ☑☑☐☐☐         |
| ILB-07 | Greeting text                | `island.greetingText` · text · "Hello!"                                                    | ✔ `greet_text`                                                             | I-3  | ☑☑☐☐☐         |
| ILB-08 | Material                     | `island.material` · select [Glass \| Opaque \| Translucent] · Glass                        | ~ `notch_material` (Blur/Glass/Black; design Glass/Opaque/Translucent)     | I-12 | ☑☑☐☐☐         |
| ILB-09 | Material rounding            | `island.rounding` · range 0–48 · 28                                                        | ✔ `notch_round` (0–100; design 0–48)                                       | I-12 | ☑☑☐☐☐         |
| ILB-10 | Show Toothless eye companion | `island.eyeEnabled` · toggle · on                                                          | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-11 | Eye color theme              | `island.eyeTheme` · select (6 options) · toothless                                         | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-12 | Eye shape                    | `island.eyeShape` · select (5 options) · toothless                                         | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-13 | Eye perception radius (px)   | `island.eyeRadius` · range 120–500 · 280                                                   | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-14 | Autonomous eye glances       | `island.eyeAutonomous` · toggle · on                                                       | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-15 | Autonomous glance chance (%) | `island.eyeGlanceChance` · range 0–100 · 78                                                | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-16 | Minimum glance interval (ms) | `island.eyeMinInterval` · range 1000–20000 · 5500                                          | ⊘                                                                          | —    | ☐☐☐☐☑         |
| ILB-17 | Maximum glance interval (ms) | `island.eyeMaxInterval` · range 2000–30000 · 11000                                         | ⊘                                                                          | —    | ☐☐☐☐☑         |

#### Dynamic Island ▸ Activities (7)

| ID     | Setting                  | Key · type · default                 | Rust today          | Cap  | Sp Co UI Fx H |
| ------ | ------------------------ | ------------------------------------ | ------------------- | ---- | ------------- |
| IAC-01 | Media and lyrics         | `island.mediaLyrics` · toggle · on   | ✗                   | I-14 | ☑☑☐☐☐         |
| IAC-02 | Compact lyrics           | `island.compactLyrics` · toggle · on | ✔ `lyrics_compact`  | I-5  | ☑☑☐☐☐         |
| IAC-03 | Online lyrics lookup     | `island.onlineLyrics` · toggle · off | ✔ `lyrics_online`   | I-5  | ☑☑☐☐☐         |
| IAC-04 | Downloads                | `island.downloads` · toggle · on     | ✔ `notch_downloads` | I-7  | ☑☑☐☐☐         |
| IAC-05 | File shelf               | `island.fileShelf` · toggle · on     | ✗                   | I-14 | ☑☑☐☐☐         |
| IAC-06 | Send to phone            | `island.sendToPhone` · toggle · off  | ✔ `notch_send`      | I-9  | ☑☑☐☐☐         |
| IAC-07 | Timer and calendar strip | `island.timerCalendar` · toggle · on | ✗                   | I-14 | ☑☑☐☐☐         |

---

#### Capability slices (behavior)

The catalogue covers settings plumbing. These slices cover the behavior behind them. Core logic and its test come first;
OS/window work follows; HTML last. Statuses come from `feature_checkpoint.md` and must be re-verified (SPEC-04).

## M3 — Dynamic Island

**I-14 Per-activity switches** (smallest slice; do it first, end to end, as the pattern for the rest)
- [x] I-14.1 core: `Activities { media, downloads, shelf, send, timer }` + `visible_tabs(flags)` and activity priority as pure
  functions. Tests: a disabled activity never appears; all off → idle only.
- [x] I-14.2 core: IAC-01, IAC-05, IAC-07 (Co boxes, recipe)
- [ ] I-14.3 app (code written, needs a Windows build + QA `docs/qa/island-activities.md`): tabs/zones hide; decide what a file drop does when the shelf is off (Send/Copy/Open still available?)
- [ ] I-14.4 UI rows (written for IAC-01/05/07 + display-mode segment; needs a Windows build) for the 7 Activities settings (UI + Fx together)
- [ ] I-14.5 H: tab visibility in the HTML island

**I-3 / I-12 Reconciliation** (needs SPEC-03)
- [x] I-3.1 idle styles: extend `notch_idle` from 3 to the agreed set; old values keep meaning (test: legacy 0/1/2)
- [x] I-12.1 display mode names ↔ `Behavior` ↔ `visibility::policy::Mode`: one mapping function, exhaustively tested
- [x] I-12.2 material names mapping, tested both directions
- [ ] I-12.3 acceptance: `step()` still exhaustively tested (unchanged, tests green); no wake-ups beyond the minute tick when idle (not measured: needs Windows, see BASE-11)

**I-15 Eye companion** (only if ADR-0007 accepts it)
- [ ] I-15.1 core: pure timing model (glance interval in [min,max], chance 0–1, seeded RNG for tests)
- [ ] I-15.2 core: theme and shape tables; tests for clamp and unknown names
- [ ] I-15.3 ui: draw; app: pointer tracking within `eyeRadius`
- [ ] I-15.4 settings rows ILB-10…17

**I-13 Notification surfacing**: after F-12.0 spike. Core queue model shared with F-12.

## M4 — Dock

**D-1 Running/pinned display options**
- [x] D-1.1 core: `show_indicators` toggles the indicator flag in `dock::entries`. Test.
- [x] D-1.2 core: `show_unpinned=false` excludes unpinned running apps; matrix test with `recent_apps`.
- [x] D-1.3 core: separator only when both groups are non-empty. Test.
- [x] D-1.4 ui: always-visible labels vs on hover (`app/dock/tooltip.rs`); Dock label visuals are setting-controlled and hover tooltips remain available.
- [x] D-1.5 app: launch bounce on/off (`dock/input.rs` reads `dock_animate_launch`).
- [x] D-1.6 H: HTML Dock label and launch animation settings are wired through `nxApplyDock` / `bounceAndOpen`.

**D-3 Lock reorder**
- [x] D-3.1 core: `reorder` refuses moves when locked; test `reorder_is_noop_when_locked`. Unpin by menu still works.
- [x] D-3.2 app: drag start is ignored when locked (`dock/input.rs`, `reorder::may_start_drag`). · [ ] QA and D-3.3 H.

**D-5 Size, gap, margin, zoom** (ranges per SPEC-03)
- [ ] D-5.1 widen/keep clamps in codec with tests · [ ] D-5.2 UI rows use WIN-01 sliders.

**D-6 Window previews settings**
- [x] D-6.1 core: size 120–520 step 10, delay 0–1500; codec tests. (`DockConfig` stores delay in 10 ms units up to 1000; widen.)
- [x] D-6.2 app: preview popup reads size/delay live (`dock/preview.rs`, `dock/input.rs`).
- [ ] D-6.3 ui: app name and window buttons toggles; D-6.2 Windows QA.
- [x] D-6.4 decision: "native preview" means a live DWM thumbnail for both Dock and STG-06 (`platform/wins/thumbnail.rs`).
- [ ] D-6.5 H.

**D-7 Minimize effects**
- [x] D-7.0 spec: ADR-0010 names effect 0 Genie (legacy Funnel), effect 1 Scale, effect 2 Suck; the Windows selector exposes all three and keeps the stored mapping.
- [x] D-7.1 core: effect enum {Funnel/Genie, Scale, Suck}; legacy `min_effect=0|1` still parses. Test.
- [x] D-7.2 core: Suck path math: endpoints, monotone, finite (no NaN). Tests.
- [x] D-7.3 core: speed multiplier 0.5–1.5 and Shift-slow factor → duration function. Tests.
- [x] D-7.4 core: blacklist match using `ProgramList` (ENG-08). Tests.
- [ ] D-7.5 app: "don't animate" and "use Nexora effect" off → plain minimize (QA).
- [x] D-7.6 core: minimized-item placement, 3 modes, with separator. Tests per mode.
- [ ] D-7.7 platform/ui: live window content (`platform/wins/thumbnail.rs`); toggle.
- [ ] D-7.8 "minimize into Dock icon" semantics (QA). · [ ] D-7.9 H.

**D-8 Launchpad page**
- [x] D-8.1 core: source filter (Added / +Start Menu / +Desktop / all). Tests.
- [x] D-8.2 platform: Start Menu and Desktop shortcut enumeration (`platform/apps.rs`, cached background load).
- [x] D-8.3 core: deleted-programs filter via `ProgramList`. Test.
- [x] D-8.4 core: grid from icon size + h/v spacing + screen size → columns/rows/pages. Tests.
- [ ] D-8.5 Launchpad display + Follow Dock via `shared::monitors::pick`.
- [ ] D-8.6 above windows vs hide; live blur vs pre-render (APP-30…33).
- [ ] D-8.7 H.

**D-9 Icon plates**: confirm `scaleMacIcons` = `dock_plates`; if yes alias, if no add. Test with `shared::iconplate`.

**D-10 Display behavior**
- [ ] D-10.1 characterization tests, then move Dock-specific rules from `app/dock/visibility.rs` to `core/dock/visibility.rs` (spec §2.4 marks this ⬜). No behavior change.
- [ ] D-10.2 spec: define "Only display with…".
- [ ] D-10.3 core: modes AlwaysShow, ForceVisible, DesktopOnly; on-disk `dock=` values stay stable. Table test: mode × inputs → visible.
- [x] D-10.4 core: detection scope (All windows / Current window) as an input to `overlapped`. Test with window lists.
- [ ] D-10.5 platform: overlap computation honoring scope (`wins/overlap.rs`).
- [ ] D-10.6 ui/app: hide effect Move vs Fade.
- [ ] D-10.7 "window borders keep windows from covering Dock/Finder": work-area reserve (QA: nothing left behind after switching).
- [ ] D-10.8 acceptance: mode switch at runtime, no restart. · [ ] D-10.9 H.

**D-11 Position (4 edges)**
- [x] D-11.1 core: geometry for left/right/top from (screen, edge, size, margin). Tests per edge. (`shared::geometry::Edge` exists.)
- [x] D-11.2 core: layout main axis (x or y). Test: same order, swapped axes.
- [x] D-11.3 core: magnification distance along the main axis. Test.
- [x] D-11.4 core: `position` codec; default bottom; old files unchanged.
- [ ] D-11.5 app: placement per edge + work-area reserve/release (QA).
- [ ] D-11.6 ui: vertical render (icons, indicators, separator, Trash).
- [ ] D-11.7 app: tooltip, preview, popup anchors flip per edge.
- [ ] D-11.8 app: minimize target from the new position. · [ ] D-11.9 H.

**D-12 Hosting display**
- [x] D-12.1 core: store a stable monitor id, not an index; `pick` falls back to primary when it is gone. Tests.
- [x] D-12.2 platform: monitor list with stable device id, bounds, work area, and primary status (`platform/monitors.rs`).
- [x] D-12.3 app: Dock rebuild is scheduled for `WM_DISPLAYCHANGE`/`WM_DPICHANGED` (`dock/wndproc.rs`). · [ ] hotplug QA.
- [ ] D-12.4 per ADR-0007: "All displays" semantics. · [ ] D-12.5 UI: select from detected displays (WIN-02).

**D-13 Activation bar**
- [x] D-13.1 core: hit-test rect (edge, width, height, offset). Tests per edge and an oversized offset.
- [x] D-13.2 core: delay state machine (enter → armed → reveal; leave cancels) as a pure `step()`. Tests like the Island's.
- [x] D-13.3 core: widen delay to 1500 ms; settings DAD-15…19.
- [ ] D-13.4 app: invisible hit window (layered, no-activate, topmost). · [ ] D-13.5 H.

**D-14 Icon effects and light**
- [ ] D-14.0 BASE-02 done first (`IconEffect` tests run).
- [x] D-14.1 core: scale curve per effect: None → 1.0, Select → 1.0, monotone falloff, symmetric. Tests.
- [x] D-14.2 core: UI dependency: zoom only matters when the effect magnifies (`effective_zoom`, exists in draft).
- [ ] D-14.3 ui: select plate. · [ ] D-14.4 ui: light glow (intensity, background intensity, RGB).
- [ ] D-14.5 rows DLB-02, 04, 05, 06 (needs WIN-03). · [ ] D-14.6 H.

**D-15 Surface glass and reflection**
- [ ] D-15.1 ENG-07 done. · one commit per surface, each with rows + consumer: [ ] Dock · [ ] labels · [ ] context menus ·
  [ ] Nexora menus
- [x] D-15.2 core: reflection params clamp/derive (distance, gradient). Tests.
- [ ] D-15.3 ui: reflection renderer (flipped icon, gradient mask, blur). Check footprint (BASE-11).
- [ ] D-15.4 H.

**D-16 Hot corners**
- [x] D-16.1 core: added display sleep, Start Menu, and Windows Widgets; legacy keys stay stable and `Mission` keeps the current Overview action.
- [x] D-16.2 platform/app: added display-off request and Start/Widgets actions in `hotcorners.rs`.
- [x] D-16.3 decision: retain all four configurable corners; existing settings and app already support four.

**D-17 Folder icons**
- [x] D-17.1 core: folder pin kind + grid layout (icon size 24–96). Tests: columns for a width.
- [ ] D-17.2 platform: list folder contents (entry cap, lazy icons).
- [ ] D-17.3 app: popup grid; double-click → Explorer; hide open-folder button. · [ ] D-17.4 H.

## M5 — Finder (after ADR-0003)

**Status: core slices done (`crates/core/src/finder/`); the platform, app and UI halves and both spikes are open and
listed in `windows_dependency.md` §5.** Core was built and tested on Linux (`cargo test -p nexora-core`); nothing here
was built or run on Windows.

- [x] F-1/F-3 core: menu item list omits hidden items (menu icon, window options). Layout tests. (`items::{Show, Available,
  build_with}`; `build` keeps the old behavior. Window options = the Window menu of the five static menus.)
- [x] **F-4 Time & Date**
  - [x] F-4.1 core: time format enum replaces `clock_24h`; legacy key still read. Test. (`clock::TimeFormat`; the key stays
    on disk so 0.4.0 golden files stay byte-identical, ADR-0004.)
  - [x] F-4.2 core: date formats Short/Long/ISO in English **and** Indonesian (acceptance criterion). Tests.
  - [x] F-4.3 core: show time/date toggles + truncation in `menubar/layout.rs`. Tests. (`clock::format_clock`,
    `layout::ellipsize`)
  - [x] F-4.4 core: Wi-Fi/Bluetooth/hardware visibility toggles. Layout tests.
  - [x] F-4.5 app: call `format_clock` and `build_with` with `Show::from_settings` (app layout now applies Finder clock and visibility settings).
- [ ] **F-7** Finder custom glass (APP-26…29): wire through `SurfaceGlass`.
- [ ] **F-8 Auto-hide and maximized appearance**
  - [x] F-8.1 core: map auto-hide to `Mode::AutoHide` with inputs. Test. (`appearance::{mode, Signals, transition}`)
  - [x] F-8.2 core: `bar_look(s, dark, maximized)`; tests for solid vs glass. (`appearance::bar_look`)
  - [ ] F-8.3 platform: maximized-window detection per monitor.
  - [ ] F-8.4 app: reveal on top-edge hover; work area released when hidden (QA; rule is `appearance::reserves_work_area`).
    · [ ] F-8.5 H.
- [ ] **F-9 Shut-down confirmation**
  - [x] F-9.1 core `needs_confirm(action, setting)`; decision: only Restart and Shut Down. Test. (`actions`)
  - [ ] F-9.2 app: themed dialog (Enter/Esc; `actions::dialog_result`). QA.
- [ ] **F-10 Corner close**
  - [x] F-10.1 core hit-test rect and exclusion list (never close desktop, shell, Nexora itself). Tests. (`actions::{corner_rect,
    in_corner, can_close}`)
  - [ ] F-10.2 platform posts `WM_CLOSE` to the maximized foreground window (QA).
- [ ] **F-11 Tray**
- [x] F-11.0 **spike** (no production code): Windows registry metadata check + documented API limits in ADR-0007; functional tray interaction remains gated.
  Output: evidence on Windows 11 build 26300 and a note with feasibility and approach. Gates F-11.5…7.
  - [x] F-11.1 core: tray model (id, order, visible, overflow); lock reorder. Tests. (`tray::Tray`)
  - [x] F-11.2 core: "show all" vs overflow rule. Test. · [x] F-11.3 core: "new icon shows N s" timer with injected clock.
    Test.
  - [x] F-11.4 core: coloured vs monochrome mapping. Tests.
  - [ ] F-11.5 platform: enumerate + change events. · [ ] F-11.6 app/ui: render, click forwarding, drag reorder.
  - [ ] F-11.7 rows TRY-01…04, FLB-06, FLB-08.
- [ ] **F-12 Notifications**
- [x] F-12.0 **spike**: Microsoft docs confirm listener consent and package-capability requirements; current unpackaged app needs an explicit packaging decision (ADR-0007).
  - [x] F-12.1 core: queue model (dedupe, TTL = duration setting). Tests. (`notifications::Queue`)
  - [ ] F-12.2 platform listener. · [ ] F-12.3 Finder menu. · [ ] F-12.4 hand over to I-13.
- [ ] **F-13 Audio**
  - [x] F-13.1 core device/volume/mute model (step clamp tests). (`audio`)
  - [ ] F-13.2 menu items on `platform/audio.rs`; rows FAU-01…03.
- [ ] **F-14 Display**: brightness menu on `platform/brightness.rs`; FDI-01. Screen mirroring per ADR-0007.
- [ ] **F-15 Hardware status**
  - [x] F-15.1 core: sampler interval, never faster than once per second; test. (`hardware`; CPU math reuses
    `widgets::cpu_percent`.)
  - [ ] F-15.2 CPU/memory item reusing the System Monitor in `core/panels/widgets.rs` (platform counters + render).
    Acceptance: no polling faster than once per second while idle (measure on Windows, QA-03).
- [ ] **F-16 Battery**
  - [ ] F-16.1 rows FBA-01…03; `battery_pct` 3 modes ↔ design toggle (SPEC-03).
  - [x] F-16.2 core: remaining-time text (h/min; English/Indonesian; unknown; charging). Tests. (`battery::remaining_text`)
  - [x] F-16.3 core: `plan_for(source, settings, current)` with hysteresis (no flapping). Tests. (`battery::{plan_for,
    AutoPlan}`)
  - [ ] F-16.4 platform: set power mode/plan (per ADR-0007) + AC/battery change events (QA).
- [ ] **F-17 Media in the bar**
  - [x] F-17.1 core width budgeting and ellipsis for title/lyrics (tests); plain vs coloured; reuses the Island's lyrics
    parser and `dynamic_island::tint`. (`media`)
  - [ ] F-17.2 app/ui: draw the item; rows FLB-12…14.
- [ ] **F-18 Input method, search, account**
  - [x] F-18.1 core: layout tests for the three toggles. (`items` tests)
  - [ ] F-18.2 platform keyboard layout list + switch; account name/avatar.

## M6 — Shared pages

Core logic for every page below is done and tested (Linux). What is left is OS or window work; it is listed in
`docs/todo/windows_dependency.md` §6 and needs a Windows build plus a QA entry (`docs/qa/m6-shared-pages.md`).

- [ ] **GEN Start on boot**
  - [x] core enum with migration (`autostart=true` → Registry): `start_method`, `Settings::set_start_method`, `reconcile`.
  - [x] platform Registry (`platform/startup.rs`).
  - [x] platform Scheduled Task creation/deletion and app selection of Registry vs Task; installer removes the task.
  - [x] ignored Scheduled Task create/delete test (`startup::scheduled_task_roundtrip`).
  - [ ] manual installer QA for Registry and Task methods. "Service" is dropped per ADR-0007.
- [ ] **GEN Hide taskbar on boot**
  - [x] core scope enum + selection: `window_management/taskbar.rs` (`Scope`, `to_hide`, `to_restore`). Tests: none, all, dock display, display gone, restore diff.
  - [ ] platform per-monitor taskbar hide (`platform/shell/taskbar.rs`); `nexora-guard` restores it (REC-01).
- [ ] **GEN Display font**
  - [x] core font list, fallback when not installed, text scale: `shared/font.rs` (`family`, `resolve`, `text_scale`). Tests: fallback chain, bundled Inter, clamp.
  - [ ] text rendering uses `resolve()` and `text_scale()` (live, no restart).
- [ ] **GEN Scale, accent, full-screen**: decisions are in `docs/spec_reconciliation.md` (rows 6, 7, 13); effect work is
  Windows-only (see windows_dependency).
- [ ] **STG gaps**
  - [x] core: `window_management/stage_options.rs` (`HiddenMode`, `strip_visible`, `desktop_icons_visible`,
    `icon_size`); animation speed steps already in `stage::SPEED_PRESETS`.
  - [ ] effects in `app/stage.rs`: icon in Finder (STG-01) · blur background icons · native preview · hidden mode ·
    keep border · desktop icons 3-state · animation speed.
- [ ] **WAN Window animation**
  - [x] core: effect enum (scale/fade/slide/zoom) + `frame()`, speed steps → duration (`animation::duration`),
    excluded/blacklist via `ProgramList` (`should_animate`), WAN-06 (`stage_options::left_out_of_stage`).
  - [ ] effects in `app/fx.rs` / `app/motion.rs`.
- [ ] **SCR Screen** (ADR-0007: in)
  - [x] core curves: `shared/screen.rs` (`dim_alpha`, `night_light`, `gamma_gain`, `targets`). Tests: monotone, clamp, off states, monitor choice.
  - [ ] overlay window (layered, click-through) or gamma ramp; gamma needs a guaranteed restore (REC-01).

## M7 — Hardening

**Status: core slices done (Linux); Windows halves are open and listed in `windows_dependency.md` §7.** Added:
`core/shared/features.rs` (REC-02), `core/tests/upgrade_0_4_0.rs` (QA-02), `core/tests/i18n_completeness.rs` (QA-04, with 169
English labels that were missing), QA files in `docs/qa/` (see `docs/qa/README.md`). Nothing here was built or run on Windows.

- [ ] **REC-01** Every change to a system surface has a restore path in `nexora-guard`: taskbar per display, work-area
  reserve, parked windows, cursors, gamma ramp (if used). Manual test: normal exit, panic, Ctrl-C, kill.
  - [x] checklist and result matrix: `docs/qa/rec-01-recovery.md`
  - [ ] taskbar per display, work-area reserve and gamma restored by `guard` · [ ] matrix run on Windows
- [ ] **REC-02** A feature that fails to start logs and the rest keeps running (spec rule 3). Test by forcing each feature to fail.
  - [x] core: `shared/features.rs` (`Startup`, `forced`, `START_ORDER`), 6 tests
  - [x] app: `bootstrap.rs` runs every feature through `Startup`; `NEXORA_FAIL_FEATURE` forces a failure (not built on Windows)
  - [x] Windows agent-control QA run: `docs/qa/rec-02-feature-failure.md` (all 15 forced features + `all`; visible behavior not asserted)
- [ ] **QA-01** One `docs/qa/<id>.md` per Windows-only behavior (use BASE-10).
  - [x] index and procedures for all listed M4/M5 behaviors: `docs/qa/README.md` (written; Windows execution remains open)
- [ ] **QA-02** Upgrade test: a real 0.4.0 `settings.cfg` loads in the new build, keeps all values, adds no keys until a
  new setting changes (ADR-0004), and a file saved by the new build is still readable by 0.4.0 semantics (unknown keys ignored).
  - [x] automated: `core/tests/upgrade_0_4_0.rs` · [ ] manual with a real file: `docs/qa/qa-02-upgrade.md`
- [ ] **QA-03** Footprint: repeat BASE-11 at each milestone; any new always-on work gets a note in `docs/decisions/`.
  - [x] procedure: `docs/qa/qa-03-footprint.md` · [x] M1 Windows measurement and `docs/bench.md`; idle wake-up trace and M7 remain open
- [x] **QA-04** i18n completeness (ENG-10) green for every page. (`core/tests/i18n_completeness.rs`; every `tr("…")` in
  `crates/app/src` is known to `shared::i18n`.)
- [ ] **QA-05** Update `feature_checkpoint.md`, `guides.md`, README modular table (SPEC-05).
  - [x] `feature_checkpoint.md` §8 · [ ] `guides.md` and README: no change needed yet; recheck after Windows runs
  - [ ] audit the ✅ rows against Windows behavior (BASE-12)

## M8 — HTML parity (optional)

- [x] **HTM-07** Script that diffs the Rust catalogue (`docs/settings_catalog.md`) against `design/settings-schema.js` and
  prints keys missing on either side. Run in CI as a warning, not a failure.
  Done: `scripts/schema_parity.mjs` (`--strict` exits 1 on differences), tests in `design/tests/schema-parity.test.mjs`, CI job
  `design` (warning only). Until HTM-01 creates `design/settings-schema.js` the script reads `defaultSettings` from
  `design/nexora-v1.html`. First report: 1 key only in the catalogue (`shared.aboveFullscreen`, GEN-03, design name is
  `shared.exitOnFullscreen`), 17 only in the HTML (`shared.exitOnFullscreen`, the other half of GEN-03; 14 duplicated
  `shared.*` keys that HTM-03 removes; and `shared.showScreenMirroring` plus `finder.showScreenMirroring`, dropped by
  ADR-0007 FDI-02). Node 22 needs `node --test "design/tests/*.test.mjs"`;
  a bare directory argument no longer works.

---

## 7. Test plan

| Layer           | What                                                                     | Where                       | Runs on                                            |
| --------------- | ------------------------------------------------------------------------ | --------------------------- | -------------------------------------------------- |
| L1 core unit    | Pure logic: codec, clamps, layout, policies, state machines              | `crates/core`               | Linux + Windows CI (works today)                   |
| L2 golden files | 0.4.0 fixtures byte-identical; per-milestone new fixtures                | `core/tests/`               | Linux + Windows CI                                 |
| L3 platform     | Real OS calls, marked `#[ignore]`                                        | `crates/platform`           | Windows, on demand                                 |
| L4 app smoke    | `nexora.exe --open-settings --settings-page <id> --exit-after 3` exits 0 | app                         | Windows (verify a CI runner has a desktop session) |
| L5 manual QA    | One checklist per window/OS behavior; includes restore check             | `docs/qa/`                  | Windows 11                                         |
| L6 HTML         | Schema invariants, unwired-key ratchet                                   | `design/tests/`             | `node --test`                                      |
| L7 footprint    | Idle private memory                                                      | `tools/bench/m1-verify.ps1` | Windows                                            |

Naming: tests are named after the behavior (`reorder_is_noop_when_locked`), not the function. Every pure function added in
a slice gets a test in the same commit.
