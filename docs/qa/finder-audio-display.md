# Finder audio and display controls (F-13/F-14)

## Purpose
Verify audio device/volume controls and brightness controls where supported.

## Prerequisites
- Windows 11; output device; laptop or monitor with controllable brightness.

## Test Steps
1. Change volume, mute, and output device in Windows and in Finder.
2. Change brightness in Windows and in Finder; repeat on an unsupported display.
3. Verify status refresh after device changes.

## Expected Behavior
Values stay synchronized; unavailable controls are omitted or disabled without errors.

## Actual Behavior
Not run on Windows 11.

## Recovery
Restore preferred audio and brightness values; verify taskbar/work area remain unchanged.
