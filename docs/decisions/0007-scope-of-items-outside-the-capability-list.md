# 0007 — Scope of items outside the capability list

Status: accepted · Scope: `docs/feature.md`, `docs/settings_catalog.md`, capability slices in `TODO.md`

## Context

The design prototype and `feature.md` contain items that have no capability ID, are cut off, conflict with product
claims, or cannot be shown to be feasible. Each one is decided here so no row is built on a guess.

## Decision

| Item                                                  | Decision | Notes                                                                                                                                            |
| ----------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Screen page (brightness overlay, night light, target) | **In**   | New capability `SCR`. Page has 4 settings: brightness overlay, night light, night light intensity, display target (SCR-01…04).                                                            |
| Toothless eye companion (8 settings, ILB-10…17)       | **Out for now** | Design-only. No I-15, no core timing model. Revisit after M3. Rows stay `⊘` and are not built in Rust.                                      |
| Screen mirroring toggle (FDI-02)                      | **Drop** | Remove from spec and design schema.                                                                                                              |
| "Start as a Service" boot method                      | **Drop** | Conflicts with README ("no background services", installer needs no administrator rights). Keep *Don't start*, *Registry Value*, *Scheduled Task*. |
| Dock display option "All displays"                    | **Drop** | The display list shows *Primary display* and each detected display (D-12, `shared::monitors::pick`). One Dock per display is not planned.        |
| Dock display mode "Only display with…"                | **Drop** | The spec text is cut off and no behavior is defined. Dock modes become 7.                                                                         |
| Hot-corner action "Advanced"                          | **Drop** | No behavior defined. `HotAction` gains the three missing real actions only (see SPEC-03).                                                        |
| WAN-06 "Remove windows for excluded programs"         | **Define** | Toggle. When on, windows of programs in the *excluded programs* list are not animated and are left out of the Stage Manager window list and the switcher. Maintainer to confirm before WAN-06 starts. |
| Battery "power plans" (FBA-04…06)                     | **Decide: power modes** | The design options (Best power efficiency / Balanced / Best performance) are Windows 11 power *modes* (overlay schemes), not classic plans. UI label becomes *Power mode*. Exact API to be confirmed in the FBA spike. |
| Tray icons (F-11) and notifications (F-12)            | **Spike first** | One time-boxed spike each, result recorded as an addendum to this ADR, before any of their rows start. Rows stay blocked until then.        |

## Consequences

* SPEC-01 writes the pages marked **In** into `feature.md`; dropped items are removed from spec and design schema.
* Settings count in `TODO.md` §5 drops by the dropped rows (FDI-02, the "All displays" option, etc.) when SPEC-02
  regenerates the catalog.

## Addendum — spikes F-11.0 (tray) and F-12.0 (notifications)

Status: **spikes complete; feature work remains gated**. The tray registry check below was run on Windows 11 Home Single
Language build 26300 on 2026-11-27. Microsoft documentation was checked the same day. This resolves feasibility, not the
remaining platform/app/UI work.

### F-11.0 Tray icons

What the repository shows:

* `mods/local@mac-island-dock-fork.wh.cpp` finds the taskbar by the class names `Shell_TrayWnd` and `Shell_SecondaryTrayWnd`
  and walks the XAML tree for `SystemTray.SystemTrayFrame` / `SystemTrayFrameGrid` (around lines 5135 and 5217). So on the
  Windows 11 build that mod targets, the tray is XAML, not a classic toolbar window.
* The same mod hooks the shell's answer to `Shell_NotifyIconGetRect` (around line 7148): that is the app-to-shell
  direction, so it does not give a way to list icons either.
* Both mods run **injected into `explorer.exe`**. Nexora does not inject (AGENTS.md §6: Windhawk mods are references, not
  dependencies), so none of this is directly reusable.

Windows check:

* `HKCU\Control Panel\NotifyIconSettings` exists and had 56 entries. All 56 exposed `ExecutablePath`; 6 had
  `IsPromoted=1`. Twenty entries matched executable names of currently running processes, including five promoted entries.
  The check intentionally recorded counts only, not personal app names or paths.
* This proves the registry is a usable local metadata source on build 26300. It does not prove live change notifications,
  stable ordering, icon decoding, or click forwarding. Microsoft documents `Shell_NotifyIcon` as an add/modify/delete API
  and callback messages to the registered owner window; `Shell_NotifyIconGetRect` retrieves the rectangle only for an
  already identified icon. It does not document a general enumeration API.

Options to decide, none chosen:

| Option | Gives | Costs |
| ------ | ----- | ----- |
| A. Registry list, click launches or focuses the owning app | Enumeration, promoted/overflow, icons (if the snapshot exists) | No live updates, no real tray menu or tooltip |
| B. Own the tray (register `Shell_TrayWnd`, receive `Shell_NotifyIcon` data) | Full behavior | Replaces the shell's tray; conflicts with "no shell replacement" and the guard's restore path |
| C. Inject into explorer | Full behavior | Out of scope for this project |
| D. Drop F-11.5…7, keep the core model | Nothing new | TRY-01…04, FLB-06, FLB-08 stay without an effect and leave the settings window |

The local registry check meets the spike's inventory criterion (20 running-process matches, five promoted). It does not
meet functional parity, so option D remains the current product decision. The core model (`finder/tray.rs`) remains useful
if a reduced read-only inventory is approved later.

### F-12.0 Notifications

What the repository shows (`mods/local@dynamic-island-for-windows.wh.cpp`, lines 5775–5850):

* The only API used is WinRT `UserNotificationListener`: `RequestAccessAsync()` must return `Allowed`, then
  `GetNotificationsAsync(NotificationKinds::Toast)` is polled and the notification ids are diffed.
* The mod waits **30 s after process start** before first use, with a comment that creating the listener too early on
  boot can bind to an uninitialised COM proxy and break notifications for the whole process. It then retries every 3 s.
* A `Denied` result is logged with the user-facing switch: Settings > Privacy & security > Notifications > "Let apps
  access your notifications".

Microsoft documentation check:

* `UserNotificationListener` requires explicit user permission requested on a UI thread. The `userNotificationListener`
  capability is a package capability. Microsoft documents sparse packages with external location as a way to grant an
  existing desktop executable package identity, which unlocks privacy consent prompts and APIs otherwise unavailable to
  unpackaged apps. Therefore the current unpackaged Nexora installer does not satisfy the documented capability/identity
  setup for this API. Do not add a listener until packaging and consent UX are an explicit product decision.
* Sources: [UserNotificationListener](https://learn.microsoft.com/en-us/uwp/api/windows.ui.notifications.management.usernotificationlistener),
  [app capability declarations](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations),
  [grant package identity to non-packaged apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps),
  [notification area APIs](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area),
  [Shell_NotifyIconGetRect](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyicongetrect).

Decision: F-12.0 spike is complete. F-12.2…4 remain gated on a packaging and user-consent decision; the current project
stays unpackaged.
