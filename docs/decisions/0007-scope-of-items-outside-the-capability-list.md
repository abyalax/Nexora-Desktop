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

## Addendum — spikes F-11.0 (tray) and F-12.0 (notifications): desk research, **not verified**

Status: **open**. This was written without a Windows machine and without access to Microsoft's documentation, so it only
records what the repository's Windhawk analyses show and what the author recalls. Every statement tagged *(recalled)*
must be checked on Windows 11 before F-11.5…7 or F-12.2…4 starts. `TODO.md` keeps both spikes unchecked for that reason.

### F-11.0 Tray icons

What the repository shows:

* `mods/local@mac-island-dock-fork.wh.cpp` finds the taskbar by the class names `Shell_TrayWnd` and `Shell_SecondaryTrayWnd`
  and walks the XAML tree for `SystemTray.SystemTrayFrame` / `SystemTrayFrameGrid` (around lines 5135 and 5217). So on the
  Windows 11 build that mod targets, the tray is XAML, not a classic toolbar window.
* The same mod hooks the shell's answer to `Shell_NotifyIconGetRect` (around line 7148): that is the app-to-shell
  direction, so it does not give a way to list icons either.
* Both mods run **injected into `explorer.exe`**. Nexora does not inject (AGENTS.md §6: Windhawk mods are references, not
  dependencies), so none of this is directly reusable.

What is recalled (verify):

* There is no public API to enumerate notification-area icons. The old technique, reading the `ToolbarWindow32` buttons
  inside `TrayNotifyWnd` with cross-process `TB_GETBUTTON`, *(recalled)* stops working on Windows 11 builds where the tray is XAML.
* *(recalled)* Windows 11 22H2 and later keep one registry key per tray icon under `HKCU\Control Panel\NotifyIconSettings`,
  with the executable path and a promoted flag, and possibly an icon snapshot. If so, it can supply a **list** (name, order
  hint, promoted/overflow) without injection, but no live events and no way to forward a click.
* A click can only reach an app through the callback window message the app registered with `Shell_NotifyIcon`; only the
  process that owns the tray knows it. Forwarding clicks therefore means owning the tray or hooking explorer.

Options to decide, none chosen:

| Option | Gives | Costs |
| ------ | ----- | ----- |
| A. Registry list, click launches or focuses the owning app | Enumeration, promoted/overflow, icons (if the snapshot exists) | No live updates, no real tray menu or tooltip |
| B. Own the tray (register `Shell_TrayWnd`, receive `Shell_NotifyIcon` data) | Full behavior | Replaces the shell's tray; conflicts with "no shell replacement" and the guard's restore path |
| C. Inject into explorer | Full behavior | Out of scope for this project |
| D. Drop F-11.5…7, keep the core model | Nothing new | TRY-01…04, FLB-06, FLB-08 stay without an effect and leave the settings window |

Recommendation (unverified): spike **A** first. Success criterion: on a Windows 11 test machine, a plain unpackaged
program lists the registry entries for at least five running apps and shows which are promoted. Record the Windows build
numbers tested. If the icon snapshot is missing, A has no icons and D is the honest outcome. The core model (`finder/tray.rs`)
is useful for A, B and C.

### F-12.0 Notifications

What the repository shows (`mods/local@dynamic-island-for-windows.wh.cpp`, lines 5775–5850):

* The only API used is WinRT `UserNotificationListener`: `RequestAccessAsync()` must return `Allowed`, then
  `GetNotificationsAsync(NotificationKinds::Toast)` is polled and the notification ids are diffed.
* The mod waits **30 s after process start** before first use, with a comment that creating the listener too early on
  boot can bind to an uninitialised COM proxy and break notifications for the whole process. It then retries every 3 s.
* A `Denied` result is logged with the user-facing switch: Settings > Privacy & security > Notifications > "Let apps
  access your notifications".

Not shown by the repository: that mod runs inside `explorer.exe`, which has an identity and trust a standalone
executable does not have. **Nothing in it proves that an unpackaged Rust program can call this API.** This is the
question the spike must answer.

What is recalled (verify):

* *(recalled)* The API needs the user's consent in all cases, and for an app without package identity the call may fail
  outright instead of prompting.
* *(recalled)* A sparse package (MSIX with an external location) can give an unpackaged app an identity without moving
  its files, but it needs a trusted signature and a registration step. Check this against README's "installer needs no
  administrator rights" before choosing it.

Spike steps: (1) a 30-line unpackaged console program using the `windows` crate that calls `RequestAccessAsync` and then
`GetNotificationsAsync`; record the result, the error code, and the Windows build. (2) If it fails, repeat with a sparse
package and record what registration needs. (3) Record the polling interval that does not violate the one-wake-up-per-second rule
(see `finder/hardware.rs`). Outcomes: *works as is*, *needs a sparse package* (decision for the maintainer), or *not
possible* (drop F-12.2…4 and I-13; keep `finder/notifications.rs` only if something else feeds it).
