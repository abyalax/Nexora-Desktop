# Dock minimize behavior (D-7)

## Purpose
Verify plain minimize, configured animation, live thumbnail behavior, and minimize-into-icon semantics.

## Prerequisites
- Windows 11; Dock enabled; a resizable app window.

## Test Steps
1. Minimize with Nexora effects disabled, then enabled.
2. Repeat with each minimize effect and with an app on the minimize blacklist.
3. Restore the window from the Dock and inspect its running indicator and preview.

## Expected Behavior
Disabled or excluded effects use ordinary Windows minimize. Enabled effects end at the corresponding Dock item; restore returns the same window.

## Actual Behavior
Not run on Windows 11.

## Recovery
Close test windows and confirm the taskbar and work area are restored after exit.
