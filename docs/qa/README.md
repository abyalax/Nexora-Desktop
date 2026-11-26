# QA index (QA-01)

One file per Windows-only behavior, created from `TEMPLATE.md` (preconditions, steps, expected result, taskbar and work-area
recovery check). Create the file in the same PR that wires the behavior; fill the result only after running it on Windows 11.

## Exists

| File                         | Covers             | Milestone |
| ---------------------------- | ------------------ | --------- |
| `finder-auto-hide.md`        | F-8                | M5        |
| `finder-corner-close.md`     | F-10               | M5        |
| `finder-shutdown-confirm.md` | F-9                | M5        |
| `island-activities.md`       | I-14.3             | M3        |
| `m6-shared-pages.md`         | GEN, STG, WAN, SCR | M6        |
| `rec-01-recovery.md`         | REC-01             | M7        |
| `rec-02-feature-failure.md`  | REC-02             | M7        |
| `qa-02-upgrade.md`           | QA-02              | M7        |
| `qa-03-footprint.md`         | QA-03              | M7        |

## Still to write (create with the behavior)

| Suggested file            | Covers                                             | Milestone |
| ------------------------- | -------------------------------------------------- | --------- |
| `dock-lock-drag.md`       | D-3.2                                              | M4        |
| `dock-previews.md`        | D-6.2, D-6.4                                       | M4        |
| `dock-minimize.md`        | D-7.5, D-7.7, D-7.8                                | M4        |
| `dock-launchpad.md`       | D-8.2, D-8.5, D-8.6                                | M4        |
| `dock-display-modes.md`   | D-10.5…D-10.8 (includes work-area reserve release) | M4        |
| `dock-position.md`        | D-11.5…D-11.8                                      | M4        |
| `dock-displays.md`        | D-12.3 (hotplug)                                   | M4        |
| `dock-activation-bar.md`  | D-13.4                                             | M4        |
| `dock-icon-effects.md`    | D-14.3/14.4, D-15.3 (footprint)                    | M4        |
| `dock-hot-corners.md`     | D-16.2                                             | M4        |
| `dock-folders.md`         | D-17.2/17.3                                        | M4        |
| `finder-tray.md`          | F-11 (after spike)                                 | M5        |
| `finder-notifications.md` | F-12 (after spike)                                 | M5        |
| `finder-audio-display.md` | F-13.2, F-14                                       | M5        |
| `finder-hardware.md`      | F-15.2 (idle polling)                              | M5        |
| `finder-battery.md`       | F-16.4                                             | M5        |
| `finder-media-input.md`   | F-17.2, F-18.2                                     | M5        |
