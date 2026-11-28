# QA-02 — Upgrade from a real 0.4.0 `settings.cfg`

Milestone: M7 · Automated part: `crates/core/tests/upgrade_0_4_0.rs` (Linux) · Manual part below needs Windows

## Purpose
A user who upgrades keeps every value, gets no new keys until they change a setting (ADR-0004), and could still read the
file with 0.4.0.

## Prerequisites
- A `settings.cfg` copied from a machine that ran the 0.4.0 installer (not a fixture). Keep a backup.
- The new build installed over 0.4.0 with `installer/NEXORA.iss`.

## Test steps
1. Copy the real 0.4.0 file to `%APPDATA%\nexora\settings.cfg`.
2. Start the new build, open Settings, change nothing, quit. Compare the file byte for byte with the backup.
3. Change one setting that is new in this build (for example Dock position). Quit.
4. Diff again: only that key may be added.
5. Put the file back under 0.4.0 (or `git checkout v0.4.0` build): it starts and ignores the new key.
6. Confirm `lists.cfg` is not created until a list setting is edited.

## Expected behavior
Steps 2 and 4 match the rules above; step 5 shows the old UI unchanged.

## Result / date / tester

### Fixture-backed runtime check — 2026-11-27

- Windows 11 Home Single Language, build 26300; `settings_0_4_0_full.cfg` copied to a unique directory under `target/`.
- Started with `NACRE_AGENT=1`, that directory as `NACRE_CONFIG_DIR`, and `--keep-taskbar`.
- `nexora-ctl state`, `wait_idle`, and `quit` passed.
- `settings.cfg` SHA-256 before and after was `03A35C58B02D4AE2246A3CB01C64D5A2C7CBBD7ADBF1E9BEB56A68CEF1A6EACD`; the file remained byte-identical.
- This exercises the checked-in 0.4.0 fixture, not a user's installed `%APPDATA%` file. Changing one setting, checking compatibility with a 0.4.0 binary, and the `lists.cfg` condition remain untested.
