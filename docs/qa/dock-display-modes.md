# Dock visibility and reserved work area (D-10)

## Purpose
Verify visibility modes, overlap scope, hide animation, reservation, and live setting changes.

## Prerequisites
- Windows 11; Dock enabled; a maximized and a normal app window.

## Test Steps
1. Exercise Always Show, Force Visible, Desktop Only, and auto-hide with both window types.
2. Switch overlap scope and Move/Fade while the Dock is running.
3. Toggle work-area reservation, then change visibility mode without restarting.
4. Exit normally and repeat after terminating Nexora to check recovery.

## Expected Behavior
Visibility follows the selected policy; setting changes apply live; reservation is removed on hide, mode change, and exit.

## Actual Behavior
Not run on Windows 11.

## Recovery
Verify the full original work area and taskbar state after each exit path.
