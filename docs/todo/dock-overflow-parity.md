# Dock overflow scrolling parity

## Evidence and scope

- `C:\MacOS_Dock_TD\config.ini` is the portable Dock configuration. Its `[normal]` section exposes icon sizing (`defaultsize`, `maxsize`), icon effects (`dockicoeffect`), running-app markers (`runicon`, `runicontag`), and auto-hide (`showhidetype`). `[minimize]` selects the `genie` effect; `[DWMThumbnail]` enables native window thumbnails. This establishes configured capabilities, not observed runtime behavior.
- `mods/local@mac-island-dock-fork.wh.cpp:581-583` describes `dockMaxWidth` as a percentage cap whose overflow is scrolled with the wheel. Lines `4059-4073` compute the visible width, scroll maximum and icon step; lines `4901-4910` clamp wheel input to that range.
- `crates/app/src/dock/window.rs:191-195` sizes the Nexora host to the screen width when content needs more room. `crates/app/src/dock/wndproc.rs` handles pointer, button, timer and drop messages but has no wheel case. `crates/core/src/dock/layout.rs` lays out and hit-tests items without a scroll offset.
- `Dock_64.exe` is a 64-bit PE (`SHA-256 944470e250b30b60798b1fdf4d02ba512cc4a056e74188df9b791aaa58fae60d`). REA identified it but native decompilation could not run because Ghidra is not configured on this machine. No behavior claim below relies on decompiling the executable.

## TODO

- [x] Add a pure core layout operation that positions items in a wider virtual row, applies a clamped horizontal offset, and keeps hit testing in viewport coordinates.
- [x] Keep the Dock window viewport bounded to the display while preserving its existing centered layout when the whole row fits.
- [x] Handle vertical mouse-wheel input in the Dock window; move by one icon-and-gap step and clamp at both ends.
- [x] Recompute the scroll range after pins, running apps, DPI, screen size, or Dock metrics change; reset or clamp stale offsets.
- [x] Add core tests for fitting rows, overflow bounds, hit testing while scrolled, and magnified layout at both ends.
- [x] Run the workspace format, clippy, test and check commands from `AGENTS.md` on Windows.
- [x] Add the capability and Windows QA procedure to the feature docs.
- [ ] Run the manual Windows QA procedure with a Dock row wider than the display.

## Implementation choices

The first slice uses the display width as its viewport limit and does not add a persistent setting. Windhawk's configurable percentage cap is useful evidence for the behavior, but a new setting would alter Nexora's settings schema and 0.4.0 compatibility path. The row remains centered when it fits; wheel scrolling only becomes active when its magnified width exceeds the viewport.
