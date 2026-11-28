# QA-04 — Agent runtime control smoke

## Purpose
Verify the debug runtime used for Windows layout QA starts, reports state, becomes idle, opens Settings, captures the desktop,
and exits through `nexora-ctl`.

## Environment
- Windows 11 Home Single Language, build 26300
- 1920 × 1080 primary display
- 2026-11-27

## Test steps
1. Start with `NACRE_AGENT=1`, `NACRE_CONFIG_DIR=target\cfg-test`, `cargo run --bin nexora --features agent-debug -- --keep-taskbar --exit-after 60`.
2. Run `nexora-ctl --pid <PID> state` and confirm Deck, Menubar and Visor report rectangles.
3. Run `nexora-ctl --pid <PID> wait_idle`.
4. Run `nexora-ctl --pid <PID> open settings 0` and capture `screenshot all`.
5. Run `nexora-ctl --pid <PID> quit`.

## Expected behavior
The process starts without changing taskbar ownership, reports feature rectangles, reaches idle, creates the Settings window,
returns a desktop PNG, and exits on the control command.

## Actual behavior
- Pass: the process started as PID 27896. State returned Deck `[592,972,736,107]`, Menubar `[0,0,1920,48]`, and Visor `[845,0,1075,42]`.
- Pass: `wait_idle` returned `{"idle":true,"ok":true}`.
- Pass: `open settings 0` returned success; `nexora.log` recorded `settings: dibuka`.
- Pass: `screenshot all` wrote a 1920 × 1080 PNG.
- Pass: `quit` returned success and the Nexora process exited.
- Pass: startup page smoke IDs 0–5 each started, answered `state` and `wait_idle`, and exited via `quit`.
- Pass: after the Finder settings wiring update, a fresh build repeated pages 0–5; each reported state, became idle, and exited through `quit`.
- Pass: latest smoke PID 1712 reported Deck `[592,972,736,107]`, Menubar `[0,0,1920,48]`, and Visor `[845,0,1075,42]`; `open settings 0` succeeded.
- Limitation: the captured desktop had the terminal in front of Settings, so the screenshot does not verify Settings visually.

## Recovery check
- Pass: launched with `--keep-taskbar`; taskbar ownership was not changed by this run.
- Pass: the process exited via `nexora-ctl quit`.
- Not checked: interactive work-area changes; this smoke test did not enable appbar or auto-hide behavior.
