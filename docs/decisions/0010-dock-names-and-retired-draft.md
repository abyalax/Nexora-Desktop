# 0010 — Dock names, the plates alias, and the retired `DockConfig` draft

Status: accepted · Scope: `crates/core/src/dock/`, `crates/core/src/settings/`, `docs/settings_catalog.md`

## Context

Three small Dock questions were open in `TODO.md` (D-7.0, D-9, BASE-02) and one comment in `dock/minimize.rs` already
cites this ADR for the first.

## Decision

1. **Minimize effect name (D-7.0).** The old Settings window calls effect `0` *Funnel*; the spec calls it *Genie*. They
   are the same effect (`window_management::genie`). *Genie* is the name in all new labels. The stored `min_effect`
   values do not change: `0` Genie, `1` Scale, `2` Suck.
2. **Icon plates (D-9).** *Scale all icons to macOS standards* (`dock.scaleMacIcons`, DAD-01) is the existing
   `dock_plates` setting: both put an icon on a macOS-sized squircle plate. No new key is added. If Windows QA shows
   that they differ, split them with a new `Extra` row and leave `dock_plates` alone.
3. **`DockConfig` (BASE-02).** The draft struct in `dock/behavior.rs` is removed. Every field it carried is already an
   `Extra` row (ADR-0004: `dock_icon_effect`, `dock_icon_light_intensity`, `dock_hidden_effect`, `dock_show_labels`,
   and so on), and nothing outside its own tests used it. `behavior.rs` keeps the enums (`IconEffect`, `HideEffect`);
   `IconEffect::effective_zoom` replaces `DockConfig::effective_zoom`. The draft's on-disk keys (`dock_light`,
   `dock_wake_delay`, …) were never written by a release, so no file needs migrating.

## Consequences

* One place per setting: `Extra` rows. No second config struct to keep in step.
* Decision 2 rests on the meaning of the label, not on a Windows run; it is listed in `docs/todo/windows_dependency.md`.
