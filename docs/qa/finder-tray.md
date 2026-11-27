# Finder tray icons (F-11)

## Purpose
Verify tray enumeration, change events, click forwarding, overflow, and reorder.

## Prerequisites
- Windows 11; apps with legacy and modern notification area icons.

## Test Steps
1. Start and stop tray apps while Nexora runs; compare visible icons.
2. Exercise left/right click, context menu, tooltip, and overflow.
3. Reorder icons and restart Nexora.

## Expected Behavior
Icons update as apps change; actions reach the owning app; ordering and overflow match settings.

## Actual Behavior
Not run. F-11 API spike and hosting behavior must be validated before this procedure can pass.

## Recovery
Exit Nexora and confirm Windows Explorer owns the normal notification area again.
