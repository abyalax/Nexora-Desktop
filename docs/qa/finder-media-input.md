# Finder media, input method, and account items (F-17/F-18)

## Purpose
Verify media rendering, keyboard layout switching, and account identity presentation.

## Prerequisites
- Windows 11; media session with long title; two keyboard layouts; signed-in user.

## Test Steps
1. Play, pause, and switch tracks; inspect title truncation and artwork tint.
2. Switch keyboard layout from Finder and verify the active layout indicator.
3. Inspect account name/avatar and open its expected Windows destination.

## Expected Behavior
Media controls target the active session; text truncates cleanly; layout and account data match Windows.

## Actual Behavior
Not run on Windows 11.

## Recovery
Restore the original keyboard layout and media state; verify the bar closes cleanly.
