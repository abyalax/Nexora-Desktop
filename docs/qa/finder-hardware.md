# Finder hardware sampling (F-15)

## Purpose
Verify CPU/memory readings and ensure the idle sampler does not poll faster than once per second.

## Prerequisites
- Windows 11; `docs/qa/qa-03-footprint.md`; ETW or equivalent timer/process sampling tool.

## Test Steps
1. Open Finder and observe CPU/memory values under idle and load.
2. Close menus and sample Nexora wakeups for at least five minutes.
3. Compare the wakeup rate and private memory with `docs/bench.md`.

## Expected Behavior
Readings update accurately; idle sampling is no faster than 1 Hz and remains within the measured footprint budget.

## Actual Behavior
Not measured on Windows 11.

## Recovery
Close any monitoring tools and verify Nexora returns to its baseline idle state.
