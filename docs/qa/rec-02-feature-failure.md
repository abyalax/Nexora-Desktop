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

## Result

Windows 11 · 2026-11-27 · debug build with `agent-debug`. Each run used a fresh `target/cfg-rec02-<case>-<guid>` config directory,
`NACRE_AGENT=1`, `NEXORA_FAIL_FEATURE=<case>`, and `--keep-taskbar --exit-after 60`. For each case, `nexora-ctl state` and
`wait_idle` succeeded before `nexora-ctl quit`; the isolated `nexora.log` contains the expected forced-failure marker(s) and
normal `exit: exit`, `appbar ok`, `fx ok`, and `taskbar ok` cleanup lines. This confirms startup/control-channel survival and
the restore path. It does not verify every non-forced feature's visible behavior.

| Forced feature | Startup/control OK | Expected log line(s) | `nexora-ctl quit` |
| -------------- | ------------------ | -------------------- | ----------------- |
| dock           | yes                | 1                    | yes               |
| notch          | yes                | 1                    | yes               |
| mission        | yes                | 1                    | yes               |
| launchpad      | yes                | 1                    | yes               |
| macshortcuts   | yes                | 1                    | yes               |
| clipboard      | yes                | 1                    | yes               |
| spotlight      | yes                | 1                    | yes               |
| stage          | yes                | 1                    | yes               |
| controlcenter  | yes                | 1                    | yes               |
| switcher       | yes                | 1                    | yes               |
| menubar        | yes                | 1                    | yes               |
| topmost        | yes                | 1                    | yes               |
| hotcorner      | yes                | 1                    | yes               |
| fx             | yes                | 1                    | yes               |
| widgets        | yes                | 1                    | yes               |
| all            | yes                | 15                   | yes               |

## Notes
Release builds use `panic = "abort"` (workspace `Cargo.toml`), so a panic inside a feature still ends the process; this test covers
`Err` returns only. A panic is the REC-01 "panic" path.

## Date / tester
2026-11-27 / Codex (nexora-ctl agent-control run)
