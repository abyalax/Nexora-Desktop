# 0001 — Split the Dynamic Island host by responsibility

Status: accepted · Scope: `crates/app/src/dynamic_island/` · Implements step 3 of `docs/feature_checkpoint.md` §6.

## Context

`app/dynamic_island/mod.rs` held a single 2 347-line file: the `Notch` struct, about 75 methods in one `impl Notch`,
the window procedure and several free functions. Input handling, animation, media and lyrics sync, downloads, the
send-to-phone session, visibility policy and timers were interleaved, which made every change a scroll through unrelated
code. `AGENTS.md` §4 asks for small cohesive modules and `architecture.md` §6.2 asks to avoid monolithic files.

## Decision

Keep one `Notch` type and one `app` composition root, and spread its `impl` over child modules, one per responsibility:

| Module        | Responsibility                                                      |
| ------------- | ------------------------------------------------------------------- |
| `mod.rs`      | `Notch` struct, `Options`, `create`, `dispatch`, `refresh`, helpers |
| `wndproc.rs`  | Win32 message → `Notch` call translation                            |
| `pointer.rs`  | press, drag, hover, click, wheel, context menu                      |
| `timers.rs`   | periodic ticks and the `WM_TIMER` dispatcher                        |
| `present.rs`  | look transitions, sizing, region, render pass                       |
| `placement.rs`| window placement, work-area reservation, show/hide policy           |
| `reconfigure.rs` | preferences, display, power, glass tier changes                  |
| `playback.rs` | media session state, progress, visualizer                           |
| `lyric_sync.rs` | lyrics lookup and per-line sync                                   |
| `activity.rs` | glance rotation, download watch, calendar agenda                    |
| `shelf.rs`    | file shelf, drop zones, shelf menu, after-drop actions              |
| `send.rs`     | send-to-phone session                                               |

Rules used for the move:

* Methods are moved verbatim. Only `fn` → `pub(super) fn` changes, so siblings and `mod.rs` can call them.
* Child modules start with `use super::*;`; private fields stay private to `dynamic_island` and its descendants.
* No behavior, public interface or dependency changes. `Notch::create` is still the only entry used by `bootstrap.rs`.
* A module is not named after an imported name (`media` is `nexora_platform::media` in `mod.rs`, hence `playback.rs`).

## Consequences

* Positive: each file is 80–340 lines and maps to one capability row in `feature_checkpoint.md` §4.2.
* Not solved: `Notch` still owns host, platform and domain state in one type. Separating `IslandState` (core) from the
  window host and moving drawing into `ui/features/dynamic_island` is step 7 and needs a theme/settings snapshot first.
* Verification: the move was checked by a line-multiset comparison against the original file (no lines added or lost
  beyond module headers). It has **not** been compiled in the authoring environment (no Windows target / crates.io);
  run the four commands from `AGENTS.md` §7 on Windows before merging.
