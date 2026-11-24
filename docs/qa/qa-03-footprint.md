# QA-03 — Footprint at the end of each milestone

Milestone: M7 (repeat at every milestone) · Script: `tools/bench/m1-verify.ps1` · Results file: `docs/bench.md`

## Purpose
Keep the README claim (about 15 MB idle private memory) honest, and catch always-on work added by a milestone.

## Test steps
1. `powershell -File tools/bench/m1-verify.ps1` on Windows 11 with a clean profile; wait the script's 30 s.
2. Append the private-memory figure and commit hash to `docs/bench.md` (one row per milestone).
3. Idle wake-ups (`I-12.3`, `F-15.2`): with the Island idle, watch the process in Process Monitor or ETW for 60 s.
   Allowed: the minute tick. Not allowed: a timer faster than once per second (Finder hardware sampler).
4. Any new always-on work gets a note in `docs/decisions/`.

## Expected behavior
Private memory within a few MB of the previous row; no timer faster than 1 s while idle.

## Results table (fill in; no value has been measured yet)
| Milestone | Commit | Idle private MB | Idle wake-ups > 1/s | Date |
| --------- | ------ | --------------- | ------------------- | ---- |
| M1        |        |                 |                     |      |
| M7        |        |                 |                     |      |
