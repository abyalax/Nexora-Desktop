# Design ↔ Core Reconciliation (SPEC-03)

Rule: for an **existing** setting (a field in the compiled `core::settings`), the range and default of `core` win and the
design changes. For a **new** setting, the design wins. An uncompiled draft (`core/dock/behavior.rs`) counts as new.
Where the stored value cannot express the spec (several states, several methods), the new key wins when present and the
legacy key is written for compatibility, derived from it (ADR-0004). Design changes are applied by the HTM tasks in
`TODO.md`; `settings_catalog.md` already shows the resolved values.

| #   | Setting                  | Core today                                           | Design / spec                         | Resolution                                                                                                                                        |
| --- | ------------------------ | ---------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `dock_size`              | 32–72, default 48                                    | 24–96, 48                             | 32–72, 48. Design range changes.                                                                                                                  |
| 2   | `dock_gap`               | 0–24, default 14                                     | 0–30, 8                               | 0–24, 14.                                                                                                                                         |
| 3   | `dock_margin`            | 0–60, default 8                                      | 0–80, 8                               | 0–60, 8.                                                                                                                                          |
| 4   | `notch_round`            | 0–100, default 50                                    | 0–48, 28                              | 0–100, 50.                                                                                                                                        |
| 5   | `blur`                   | 0–100, default 60                                    | 0–48, 26                              | 0–100, 60.                                                                                                                                        |
| 6   | `ui_scale`               | percent, clamp 75–200, default 100                   | slider 0.75–1.5, 1                    | Percent 75–200, 100. (The audit said 100/125/150 steps; the codec clamps 75–200.) UI may offer a slider.                                          |
| 7   | `accent`                 | palette index, default 0                             | hex `#63c7dc`                         | Keep the palette index; the UI row is the swatch palette. A free hex colour is deferred. (The Dock light colour DLB-06 is a new setting: hex.)     |
| 8   | `autostart`              | bool, default off                                    | 4 methods                             | New key `start_method` ∈ {none, registry, task}; *Start as a Service* is dropped (ADR-0007). Legacy `autostart` = `start_method ≠ none`.          |
| 9   | `takeover`               | bool, default **on**                                 | 3 states, default none                | New key `taskbar_scope` ∈ {none, all_displays, dock_display}, default `all_displays`. Legacy `takeover` = `scope ≠ none`.                          |
| 10  | `hide_desktop_icons`     | bool, default on                                     | 3 states, default Auto                | New key `desktop_icons` ∈ {hide, auto, show}, default `hide`. Legacy bool = `mode ≠ show`.                                                         |
| 11  | `stage_speed`            | percent 50–200, default 100                          | 5 steps 0.5×–1.5×                     | Store percent (50–200). The select offers the 5 presets 50/75/100/125/150.                                                                        |
| 12  | `stage_size`             | card size 70–260, default 170                        | icon size 16–80, 36                   | Keep card size 70–260, 170. The spec label *Window list icon size* maps to it; no separate icon-size field.                                       |
| 13  | `above_fullscreen`       | on = Dock/Island/Menu Bar stay above full-screen     | "Automatically exit on full screen"   | Opposite meanings. Keep core semantics. UI label: *Stay above full-screen windows*, default on (= design `exitOnFullscreen` off).                 |
| 14  | `notch_idle`             | 3 styles (0–2)                                       | 4 styles incl. Toothless eyes         | 3 styles. The eye style is out of scope (ADR-0007). Style names are set in the ILB-05 task.                                                       |
| 15  | Island display mode      | `Behavior` Fixed / Overlay / Smart                   | Fixed / Smart / On demand             | Keep the three core values and add **On demand** (new option). Dock modes follow ADR-0007 (7 modes) and D-10.                                      |
| 16  | `Material`               | Blur / Glass / Black                                 | Glass / Opaque / Translucent          | Keep core names. Opaque = Black, Translucent = Blur. Island default stays Black.                                                                  |
| 17  | `HotAction`              | 7: None, Mission, Launchpad, Desktop, Stage, Notifications, Lock | 10 incl. *Advanced*       | Keep the 7. *Show all windows* and *Open Mission Control* are both `Mission`. Add 3 new: sleep display, Start Menu, Widgets. *Advanced* dropped. Total 10. |
| 18  | default `menubar`        | off                                                  | `enableFinder` on                     | Core wins: off. Design default changes.                                                                                                           |
| 19  | default `lang`           | `Lang::Id`                                           | English                               | Core wins: Indonesian.                                                                                                                            |
| 20  | default `lyrics_online`  | on (also in `guides.md`)                             | off                                   | Core wins: on.                                                                                                                                    |
| 21  | default `notch_send`     | on                                                   | off                                   | Core wins: on.                                                                                                                                    |
| 22  | `DockConfig` defaults    | labels on, separate on, light 60, delay 20 (draft)   | labels off, separate off, light 35, delay 150 ms | Draft is uncompiled, so the design wins. Delay range 0–1500 ms; unit conversion is decided in BASE-02.                                    |

Found while reconciling (not in the original list; same rule):

| #   | Setting       | Core today                    | Design               | Resolution                                       |
| --- | ------------- | ----------------------------- | -------------------- | ------------------------------------------------ |
| 23  | `stage_gap`   | 0–60, default 20              | 0–30, 8              | 0–60, 20.                                        |
| 24  | `stage_count` | 2–10, default 6               | 1–20, 8              | 2–10, 6.                                         |
| 25  | `dock_zoom`   | 100–250 (%), default 200      | 1–2.5×, 1.4          | 100–250 %, 200 (2.0×).                           |
| 26  | `theme`       | Auto / Light / Dark, Auto     | light / dark / system, dark | Core names, default Auto (= system).      |
| 27  | `stage_desktop_click`, `stage_media` | on, off          | off, on              | Core wins: on, off.                              |
| 28  | `hide_desktop_icons` default | always hide (on)       | Auto Show Hide       | Covered by row 10: default `hide`.               |
