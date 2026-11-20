# 0002 — Regroup `core` by feature

Status: accepted

## Context

`crates/core/src` mixed ~25 flat files with the three feature folders. `architecture.md` §3.2 wants domain code grouped
by feature, and `Rect` was defined three times (`genie`, `mission`, `marker`).

## Decision

* Move flat files into `launchpad/`, `window_management/`, `panels/`, `shared/`, `dock/` (trash) and
  `dynamic_island/` (qr). No logic changed.
* Keep every former top-level path alive with `pub use` in `lib.rs`. Callers migrate later and the re-exports are
  deleted in the same change.
* `genie::Rect` and `mission::Rect` become aliases of `shared::geometry::Rect`. `marker::Rect` stays: it holds integer
  window-pixel coordinates, not layout floats.

## Verification

* `cargo test -p nexora-core`, including golden `settings.cfg` round-trips.
* A scratch crate imports all 179 `nexora_core::...` paths used by `app`, `ui`, `platform` and `guard`. Removing one
  re-export makes it fail.
* `platform`, `ui` and `app` were not compiled (the `windows` crate was unavailable in the build sandbox).
