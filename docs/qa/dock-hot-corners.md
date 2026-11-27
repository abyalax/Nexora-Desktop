# Dock hot corners (D-16)

## Purpose
Verify each configured Windows action and corner dwell behavior.

## Prerequisites
- Windows 11; configure one action at a time; save work before power actions.

## Test Steps
1. Move into each configured corner and verify the action after the dwell period.
2. Move away before the delay expires and repeat while holding a mouse button.
3. Check display sleep, Start Menu, and Widgets actions when configured.

## Expected Behavior
Only the selected action runs; cancelled dwell and held-button entry do not trigger it.

## Actual Behavior
Not run on Windows 11. Record unsupported actions separately.

## Recovery
Confirm all overlays close and the taskbar remains available.
