# Dock overflow scrolling

## Purpose

Verify that a Dock with more icons than fit on the display can be browsed horizontally without making the host wider than the display.

## Prerequisites

- Windows 11 with Nexora running.
- Pin enough applications for the Dock row, including magnified width, to exceed the active display width.

## Test Steps

1. Open the Dock and confirm its visible host stays within the display bounds.
2. Move the pointer over the Dock and scroll the mouse wheel down several notches.
3. Confirm later icons appear and remain clickable; continue scrolling at the end.
4. Scroll up to the beginning and continue scrolling there.
5. Remove pins or reduce Dock icon size until the row fits, then scroll over the Dock again.

## Expected Behavior

- Each wheel notch moves the row by one icon-and-gap step.
- The offset stops at the first and last icons; it never wraps or exposes blank overscroll.
- Hover magnification and click hit testing follow the icons after scrolling.
- A fitting Dock remains centered and does not move when the wheel is used.
- The feature does not modify taskbar visibility or reserve work area.

## Actual Behavior

Not run yet.

## Taskbar Recovery Check

- [ ] Taskbar is visible after exit (not changed by this feature)
- [ ] Taskbar auto-hide setting is restored (not changed by this feature)
- [ ] Work area is properly cleared (not changed by this feature)

## Work Area Recovery Check

- [ ] Work area is restored to original state (not changed by this feature)
- [ ] No leftover reserved space (not changed by this feature)
- [ ] Apps can maximize to full screen

## Notes

The workspace test suite covers the pure scrolling layout. This procedure covers actual pointer messages, Composition animation and clipping on Windows.

## Date

Not run yet.
