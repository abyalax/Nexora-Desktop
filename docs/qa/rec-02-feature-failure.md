# REC-02 — A feature that fails to start does not take the rest down

Milestone: M7 · Rule: `docs/feature.md` cross-cutting rule 3 · Logic and tests: `core/shared/features.rs` (run on Linux) · Hook: `app/bootstrap.rs`

## Purpose
Prove on Windows that each feature can fail at start-up while the others keep working.

## How a failure is forced
Set `NEXORA_FAIL_FEATURE` before launching. It takes feature names separated by `,` or `;` (case-insensitive) or `all`.
The feature's init is not run; `nexora.log` gets a line `gagal: dipaksa gagal (NEXORA_FAIL_FEATURE)` under its name.

Names, in start order: `dock`, `notch`, `mission`, `launchpad`, `macshortcuts`, `clipboard`, `spotlight`, `stage`,
`controlcenter`, `switcher`, `menubar`, `topmost`, `hotcorner`, `fx`, `widgets`.

## Test steps
1. PowerShell: `$env:NEXORA_FAIL_FEATURE = "stage"; .\nexora.exe`
2. Confirm the log line for `stage` and that nothing else in the log says `gagal`.
3. Use the features that were **not** forced (Dock, Island, Settings window, Menu Bar, tray menu → Quit).
4. Quit normally. Repeat for each name above, then once with `all` (the app should still open the tray icon and quit).

## Expected behavior
- Only the forced feature is missing; every other one works.
- The process does not exit or hang, and Quit restores the taskbar (see REC-01).
- With `all`, no feature is active but the tray icon and Quit still work.

## Result (fill in)
| Forced feature | Others OK | Log line seen | Quit OK |
| -------------- | --------- | ------------- | ------- |
| dock           |           |               |         |
| notch          |           |               |         |
| mission        |           |               |         |
| launchpad      |           |               |         |
| macshortcuts   |           |               |         |
| clipboard      |           |               |         |
| spotlight      |           |               |         |
| stage          |           |               |         |
| controlcenter  |           |               |         |
| switcher       |           |               |         |
| menubar        |           |               |         |
| topmost        |           |               |         |
| hotcorner      |           |               |         |
| fx             |           |               |         |
| widgets        |           |               |         |
| all            |           |               |         |

## Notes
Release builds use `panic = "abort"` (workspace `Cargo.toml`), so a panic inside a feature still ends the process; this test covers
`Err` returns only. A panic is the REC-01 "panic" path.

## Date / tester
