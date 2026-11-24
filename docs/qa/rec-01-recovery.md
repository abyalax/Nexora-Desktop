# REC-01 — Every system change has a restore path

Milestone: M7 · Needs: Windows 11 build · Owner of the restore code: `crates/guard`, `platform/startup`, `app/startup/recovery.rs`

## Purpose
Each change Nexora makes to a system surface must be undone on every way the process can end. This page is the
checklist; one row per surface, one column per exit path.

## Prerequisites
- Release build of `nexora.exe` and `nexora-guard.exe` side by side.
- Two displays if available (taskbar per display, work area per display).
- Settings backed up (`%APPDATA%\nexora\settings.cfg`).

## Surfaces and who restores them
| Surface                         | Set by                                  | Restored by today                              | Status in code |
| ------------------------------- | --------------------------------------- | ---------------------------------------------- | -------------- |
| Windows taskbar (auto-hide)     | `shell/taskbar.rs` + marker file        | `guard` via `taskbar::recover_leftover`        | exists         |
| Parked windows (Scene)          | `wins/park.rs` → `parked.txt`           | `guard` via `park::restore_file`               | exists         |
| System cursors                  | `cursors.rs` → `cursors_replaced.flag`  | `guard` via `cursors::restore`                 | exists         |
| Taskbar **per display** (GEN)   | `shell/taskbar.rs` (not written yet)    | must be added to marker + `guard`              | **open**       |
| Work-area reserve (Dock/Finder) | app bar / `SystemParametersInfo`        | must be added to marker + `guard`              | **open**       |
| Gamma ramp (SCR, if used)       | not written yet                         | must be added to `guard` before it ships       | **open**       |

Core decision helpers already exist: `window_management/taskbar.rs` (`to_hide`, `to_restore`).

## Test steps (repeat for every surface that is on)
For each exit path run the same sequence: turn the surface on, exit that way, check the restore boxes.
1. **Normal exit:** tray menu → Quit.
2. **Panic:** no CLI flag exists for this yet; use a debug build with a temporary `panic!()` after start-up (a flag is a follow-up if this is run often).
3. **Ctrl-C:** run from a terminal, press Ctrl-C.
4. **Kill:** `taskkill /F /IM nexora.exe`; wait up to 30 s for `nexora-guard` (it gives up after 30 s, see `guard/main.rs`).
5. **Guard killed too:** kill both, then start `nexora.exe` again and quit normally (documented fallback in `guides.md`).

## Expected behavior
After every path the Windows desktop looks as it did before the first launch. `nexora-guard.log` shows what it restored.

## Recovery check (copy per run)
- [ ] Taskbar visible on every display, auto-hide setting as before
- [ ] Work area equals the full screen minus the taskbar on every display
- [ ] No window left off-screen or parked
- [ ] Cursors are the Windows defaults
- [ ] Gamma / night-light overlay gone (only when SCR is on)
- [ ] `nexora-guard.exe` is no longer running

## Result matrix (fill in)
| Surface \ exit    | Normal | Panic | Ctrl-C | Kill | Kill + guard |
| ----------------- | ------ | ----- | ------ | ---- | ------------ |
| Taskbar           |        |       |        |      |              |
| Parked windows    |        |       |        |      |              |
| Cursors           |        |       |        |      |              |
| Taskbar / display |        |       |        |      |              |
| Work-area reserve |        |       |        |      |              |
| Gamma (if used)   |        |       |        |      |              |

## Date / tester
