# Finder battery and power plan (F-16)

## Purpose
Verify battery level/time, charging state, automatic plan changes, and hysteresis.

## Prerequisites
- Windows 11 laptop; AC adapter; note the current power mode.

## Test Steps
1. Check battery percentage and remaining-time text while charging and discharging.
2. Enable automatic power plan and connect/disconnect AC around its threshold.
3. Observe plan changes and confirm small fluctuations do not cause rapid toggling.

## Expected Behavior
Battery data follows Windows; unknown estimates are omitted; plan changes follow the selected threshold and hysteresis.

## Actual Behavior
Not run on Windows 11.

## Recovery
Restore the original Windows power mode and confirm charging state is unchanged.
