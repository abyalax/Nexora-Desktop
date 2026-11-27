# Dock folder popup (D-17)

## Purpose
Verify folder contents, entry cap, lazy icons, popup grid, and Explorer launch.

## Prerequisites
- Windows 11; test folder containing files, subfolders, and more entries than the configured cap.

## Test Steps
1. Pin the folder and open its popup.
2. Scroll or page through entries and inspect icon loading.
3. Double-click a file and a subfolder; compare with the open-folder button setting.

## Expected Behavior
Entries remain within the cap; icons load without blocking; double-click opens the selected item in Explorer.

## Actual Behavior
Not run on Windows 11.

## Recovery
Close Explorer and popup; verify taskbar and work area are unchanged.
