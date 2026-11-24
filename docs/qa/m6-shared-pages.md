# M6 Shared Pages QA

Windows-only behavior for General, Stage Manager, Window Animation and Screen. Core logic is covered by unit tests;
this checklist covers what only a real desktop can show. Format follows `TEMPLATE.md`.

## Prerequisites
- Windows 11, two monitors for the multi-display cases, a clean `settings.cfg`.
- Run with `--open-settings` to change values live.

## Test Steps
1. **Start on boot**: set *Registry Value*, sign out and in; Nexora starts. Repeat for *Scheduled Task*. Set *Don't start*; no entry remains.
2. **Uninstall**: run the uninstaller; no Registry value and no Scheduled Task remain.
3. **Hide taskbar**: set *All displays*, then *Dock display*; only the chosen taskbars hide. Switch back; the others return.
4. **Display font**: pick each font; text changes live. Uninstall the chosen font (or rename it); text falls back, never blank.
5. **Stage Manager**: for *Hidden mode* (3 values) and *Show desktop icons* (3 values), check the strip and icons follow the table in `core/window_management/stage_options.rs`.
6. **Window animation**: each effect and each speed; windows of excluded or blacklisted programs open without animation.
7. **Screen**: brightness overlay at 0, 50, 100; night light on/off; each display target.

## Expected Behavior
Every setting takes effect without a restart, and turning it off removes its effect completely.

## Actual Behavior
_To be filled during testing._

## Taskbar Recovery Check
- [ ] Taskbar is visible on every monitor after normal exit
- [ ] Taskbar is visible after `nexora-guard` runs (kill Nexora from Task Manager)
- [ ] Taskbar auto-hide setting is restored

## Work Area Recovery Check
- [ ] Work area is restored after switching *Hide taskbar* scope
- [ ] No leftover reserved space
- [ ] Gamma ramp (if used) is restored after exit and after kill

## Notes
`HiddenMode` meanings (Default / Always show / Always hide) are an assumption; confirm with the maintainer.

## Date
_Date of test execution._
