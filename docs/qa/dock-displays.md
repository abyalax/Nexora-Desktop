# Dock display selection and hotplug (D-12)

## Purpose
Verify selected display, stable fallback, and monitor hotplug handling.

## Prerequisites
- Windows 11; two displays recommended.

## Test Steps
1. Select each detected display and restart Nexora.
2. Disconnect the selected display while Nexora is running, then reconnect it.
3. Repeat with the selected display configured as primary and secondary.

## Expected Behavior
Dock moves to the selected display; on removal it falls back to primary; reconnecting restores the stored choice.

## Actual Behavior
Not run on Windows 11. Platform enumeration exists; app hotplug/selection wiring must be confirmed.

## Recovery
Restore the original display arrangement and verify no off-screen windows or reserved area remain.
