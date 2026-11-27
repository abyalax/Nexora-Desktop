# Rust rewrite status

Nexora Desktop is the in-progress single-app Rust rewrite of `MacOS_Dock_TD` and the two Windhawk mods under `mods/`.
The workspace crates are the product implementation; the C++ files and generated analysis directories stay as reference
material and are not runtime dependencies. Keep new behavior in the Rust layers and update the TODO ledger as each slice
lands.

## Reference mapping

| Reference                                      | Rust destination                                                                                         | Current state                                                                                                                                                                                |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MacOS_Dock_TD`                                | `crates/core/src/dock`, `crates/platform/src/wins`, `crates/app/src/dock`, `crates/ui/src/features/dock` | Dock domain, window tracking, rendering, pinning, previews, and several effects exist. Per-edge placement, monitor selection, recovery, and parity QA remain tracked in `docs/todo/TODO.md`. |
| `mods/local@mac-island-dock-fork.wh.cpp`       | Dock, Finder/Menu Bar, Launchpad, window management, hot corners, and shared platform modules            | Ported by capability into Rust modules. Not all Windhawk behaviors have app wiring or Windows QA yet.                                                                                        |
| `mods/local@dynamic-island-for-windows.wh.cpp` | `crates/core/src/dynamic_island`, `crates/platform/src/media.rs`, `crates/app/src/dynamic_island`        | Island state, activities, media, downloads, timer, shelf, and presentation are in progress. Notification handoff remains blocked on a verified unpackaged-app notification source.           |

## Current platform slice

- Registry and Scheduled Task startup methods are available, selected from the saved setting, and the installer removes
  both startup entries on uninstall.
- Windows monitor inventory is available from `nexora_platform::monitors::list()` with device id, friendly name, bounds,
  work area, and primary status. Dock rebuilds on display-change messages; persisted monitor choice still needs app wiring
  and hotplug QA.
- Start Menu and Desktop shortcut enumeration is implemented in `platform/apps.rs`.
- DWM thumbnail registration, sizing, placement, and cleanup are implemented in `platform/wins/thumbnail.rs` and used by
  Dock previews and window overview.
- Audio and brightness controls and CPU/memory counters have platform backends; Finder rendering and settings wiring are
  still incomplete where listed in the TODO ledger.

## Validation state

`cargo check --workspace` passed on 2026-11-27 after the current platform changes. This is compile evidence only. Manual
Windows 11 procedures, installer upgrade/removal, crash recovery, idle wake-up measurements, and footprint measurements
remain open in [`todo/windows_dependency.md`](todo/windows_dependency.md) and [`qa/README.md`](qa/README.md).
