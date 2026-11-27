# Dock reorder lock (D-3.2)

## Purpose
Verify that the lock prevents drag reordering while ordinary Dock actions remain available.

## Prerequisites
- Windows 11; Dock enabled; at least three pinned apps.

## Test Steps
1. Enable the Dock reorder lock and attempt to drag one pinned item across another.
2. Unlock and repeat the drag.
3. Launch an item and open its context menu while locked.

## Expected Behavior
The locked drag leaves item order unchanged. Unlocked drag changes order. Launch and menu actions still work.

## Actual Behavior
Not run on Windows 11.

## Recovery
Confirm taskbar visibility and work area are unchanged after exiting Nexora.
