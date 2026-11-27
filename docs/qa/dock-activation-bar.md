# Dock activation bar (D-13)

## Purpose
Verify the invisible hit window reveals the Dock without taking focus.

## Prerequisites
- Windows 11; auto-hide Dock enabled.

## Test Steps
1. Move the pointer to each configured activation zone and wait through the reveal delay.
2. Move away before the delay expires, then repeat while a full-screen app is active.
3. Click through the revealed edge where no Dock item is drawn.

## Expected Behavior
The Dock reveals only after the configured trigger; cancelled dwell does not reveal; the hit target does not activate or block unrelated windows.

## Actual Behavior
As Expected Behavior

## Recovery
Exit Nexora and confirm no invisible topmost hit target remains.
