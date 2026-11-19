# Nexora Desktop — Feature Specification

This document specifies the three headline features of Nexora Desktop and how each one maps onto the crate boundaries
defined in [`architecture.md`](architecture.md) and [`../AGENTS.md`](../AGENTS.md).

| #   | Feature            | Product name in the UI | One-line description                                                                  |
| --- | ------------------ | ---------------------- | ------------------------------------------------------------------------------------- |
| 1   | **Dock**           | Deck                   | Magnifying app strip: pinned and running apps, previews, minimize effects, Trash.     |
| 2   | **Finder**         | Rail                   | Top bar: logo menu, active-app menus, tray status (Wi-Fi, battery, clock), search.    |
| 3   | **Dynamic Island** | Visor                  | Glass pill at the top: media and lyrics, downloads, timer, file shelf, send-to-phone. |

> Naming: code, docs and crate modules use *Dock / Finder / Dynamic Island*. User-visible strings use *Deck / Rail /
> Visor* (see the disclaimer in the README). Do not rename modules to match the UI names.

Status legend used below: ✅ implemented · 🟡 partial · ⬜ planned (not in the settings model or UI yet).
Statuses were derived from the README and the `Settings` struct at commit `1e95f83`; re-verify when you start an item.

---

## 1. Cross-cutting rules

These apply to all three features and are the acceptance baseline for every item in sections 2–4.

1. **Layering.** Each feature is split into domain logic (`core`), OS integration (`platform`) and presentation (`ui`),
   wired together in `app`. No Win32/COM/WinRT in `core`. No business rules in `ui`. (`AGENTS.md` §2, §5.)
2. **Settings vs. state.** Persistent preferences (`DockSettings`, `FinderSettings`, `IslandSettings`) are separate types
   from runtime state (`DockState`, `FinderState`, `IslandState`). (`architecture.md` §6.5.)
3. **Independence.** Each feature can be turned off without affecting the others (Dock only, Island only, Finder only,
   any mix). A feature that fails to initialize logs the error and the rest of the app keeps running.
4. **Shared visibility policy.** Dock and Island use the same pure policy (`core::visibility::policy`): `Always`,
   `AutoHide`, `OnDemand`, with inputs `overlapped / interacting / edge_armed / starting / fullscreen / revealing`.
5. **Glass and theme.** Blur, tint, rounding and accent come from design tokens (`ui::theme`) and effects
   (`ui::effects`), not from per-feature constants.
6. **Localization.** Every user-visible string goes through `core::i18n` (English and Indonesian).
7. **Footprint.** Idle memory stays near the 15 MB private-memory figure advertised in the README. New always-on
   background work needs a justification in `docs/decisions/`.
8. **Recoverability.** Anything that hides or replaces a system surface (taskbar, cursors, parked windows) must be
   restored on normal exit, panic, Ctrl-C and crash (`nexora-guard`).

---

## 2. Feature: Dock

### 2.1 Goal

A macOS-style Dock for Windows 11: launch, switch and manage applications from one magnifying glass strip.

### 2.2 Capabilities

| ID   | Capability                                                                             | Status                                                        |
| ---- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| D-1  | Show pinned apps and running apps; running apps get an indicator                       | ✅                                                             |
| D-2  | Click to launch or focus; group a running app's windows, sorted by recent focus        | ✅                                                             |
| D-3  | Pin / unpin, drag to reorder, lock reorder                                             | ✅ / ⬜ lock                                                    |
| D-4  | Recently used apps stay after they quit (toggle)                                       | ✅                                                             |
| D-5  | Magnification on hover, icon size, gap, edge margin                                    | ✅                                                             |
| D-6  | Live window previews on hover for running apps                                         | ✅                                                             |
| D-7  | Minimize-into-Dock animation (Funnel, Scale)                                           | 🟡 Genie / Suck / per-app blacklist ⬜                          |
| D-8  | App Grid (Launchpad) button, Downloads stack, Trash with *Empty Trash*                 | ✅                                                             |
| D-9  | Custom icons per app and icon packs                                                    | ✅                                                             |
| D-10 | Display behavior: Fixed, Overlay, Smart auto-hide; above full-screen toggle            | 🟡 *Always visible*, *desktop only*, hide effect (move/fade) ⬜ |
| D-11 | Position on any screen edge (bottom, left, right, top)                                 | ⬜                                                             |
| D-12 | Choose the display that hosts the Dock (multi-monitor)                                 | ⬜                                                             |
| D-13 | Activation bar / edge reveal with configurable width, height, offset and delay         | ⬜                                                             |
| D-14 | Icon effects: scaling, selection, illumination (light intensity and color)             | 🟡 scaling only                                                |
| D-15 | Icon reflection and per-surface blur/transparency (labels, context menus, folder grid) | ⬜                                                             |
| D-16 | Hot corners (including bottom-left) mapped to actions                                  | ✅ (4 corners)                                                 |
| D-17 | Folder icons open a grid; *double-click opens in Explorer*; hide *open folder* button  | ⬜                                                             |

### 2.3 Settings (sidebar group **Dock**)

* **Look & Behavior**: icon size · icon effect (None, Scaling, Select, Illumination, Scaling+Light, Select+Light) ·
  magnification · light intensity · background light intensity · background light color (R, G, B) · icon spacing ·
  distance from screen edge · position (Left, Bottom, Right, Top) · display.
* **Launchpad**: icon size · horizontal / vertical spacing · source (Added icons; + Start Menu; + Desktop; + both) ·
  edit hidden Start Menu programs · display · follow Dock.
* **Advanced**: scale all icons to macOS standards · double-click folder opens in Explorer · hide open-folder button ·
  lock icon dragging · show labels · animate app launch · window borders keep windows from covering Dock and Finder ·
  running indicators · show unpinned running apps · separate unpinned running apps · hot-corner action ·
  display mode (Default, Overlay, Auto-hide, Smart auto-hide, Always show, Only with…, Always force visible, Desktop
  only) · detection window (All program windows / Current program window) · hide effect (Move / Fade) · activation
  method (Screen edge / Activation bar / both) · activation bar width, height, offset · mouse delay.
* **Minimize Effects**: use Nexora effect · don't animate · hold Shift to slow · minimize into Dock icon · live window
  content · minimized-icon position (next to the icon, separate on the right, separate with divider) · animation
  (Genie, Scale, Suck) · speed · blacklist.
* **Window Previews**: enable · size · hover delay · show app name / window buttons / native preview.
* **Appearance** (shared page): custom blur and transparency for the Dock, icon labels, context menus, folder grid and
  Nexora menus; icon reflection (blur, transparency, distance, gradient).

### 2.4 Architecture mapping

| Concern                      | Target (per `architecture.md`)                            | Today                                                                                                         |
| ---------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Items, pins, order, recents  | `core/dock/{items,pins,reorder,recents}.rs`               | ✅ in place                                                                                                    |
| Layout and geometry          | `core/dock/{layout,geometry,entries,ring}.rs`             | ✅ in place                                                                                                    |
| Behavior and visibility      | `core/dock/{behavior,visibility}.rs`                      | ⬜ policy lives in shared `core/visibility/policy.rs`; Dock-specific rules are inside `app/dock/visibility.rs` |
| Settings vs. state           | `core/settings/dock.rs`, `core/dock/state.rs`             | ⬜ one flat `Settings` struct (`dock_*` fields)                                                                |
| Running apps, focus, launch  | `platform/windows/*`, `platform/filesystem/icons.rs`      | ✅ implemented, flat layout until the platform regrouping lands                                                |
| Rendering                    | `ui/features/dock/{view,item,preview}.rs`                 | 🟡 `ui/features/dock/view.rs`; render orchestration in `app/dock/render.rs`                                    |
| Preview and tooltip popups   | `ui/features/dock/preview.rs`, `ui/components/tooltip.rs` | ⬜ `app/dock/{preview,tooltip}.rs`                                                                             |
| Window host, wndproc, timers | `app/dock/{window,wndproc,timers,input,hover}.rs`         | ✅ stays in `app` (composition and message pump)                                                               |

### 2.5 Acceptance criteria

* `cargo test -p nexora-core` covers pin parsing, ordering, grouping, layout and visibility decisions without any
  OS access.
* Unpinning or reordering never changes the order of other items; a pin whose target no longer exists is shown but
  does not crash enumeration.
* Auto-hide hides only when the policy says so and always reveals while the pointer is on the activation edge.
* Switching display mode or position at runtime applies without restart and without leaving a reserved work area
  behind.
* Quitting Nexora (menu, Ctrl-C, panic) restores the Windows taskbar.

---

## 3. Feature: Finder

### 3.1 Goal

A top bar that gives the active application its menus and puts system status (network, battery, time, search, controls)
in one place, replacing the need for the Windows taskbar's notification area.

### 3.2 Capabilities

| ID   | Capability                                                                                                                 | Status                                         |
| ---- | -------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| F-1  | Logo menu: About, System Settings, Store, Nexora Settings, App Grid, Overview, Force Quit, Sleep, Restart, Shut Down, Lock | ✅                                              |
| F-2  | Custom logo image                                                                                                          | ✅                                              |
| F-3  | Active-app name and its classic menus                                                                                      | ✅                                              |
| F-4  | Status items: battery (inside / beside / hidden percentage), Wi-Fi with live signal, clock (12/24 h)                       | ✅                                              |
| F-5  | Wi-Fi menu: switch, saved networks, one-click connect                                                                      | ✅                                              |
| F-6  | Quick Search and Quick Panel entry points                                                                                  | ✅                                              |
| F-7  | Background: transparent with shade, light, dark, glass material                                                            | ✅                                              |
| F-8  | Auto-hide Finder; maximized windows change Finder appearance                                                               | ⬜                                              |
| F-9  | Shut-down confirmation dialog                                                                                              | ⬜                                              |
| F-10 | Click the top-right corner to close the maximized window                                                                   | ⬜                                              |
| F-11 | System tray icons (show all, colored, lock dragging)                                                                       | ⬜                                              |
| F-12 | Notifications menu                                                                                                         | ⬜                                              |
| F-13 | Audio menu (output device, volume)                                                                                         | 🟡 sliders live in Quick Panel                  |
| F-14 | Display menu (brightness)                                                                                                  | 🟡 slider lives in Quick Panel                  |
| F-15 | Hardware status (CPU, memory) menu                                                                                         | 🟡 System Monitor widget exists, no Finder item |
| F-16 | Battery menu: remaining time, power plans on AC / battery, auto-switch                                                     | ⬜                                              |
| F-17 | Media playback icon and lyrics in the bar (plain or colored)                                                               | ⬜                                              |
| F-18 | Input-method menu, search icon and account icon toggles                                                                    | ⬜                                              |

### 3.3 Settings (sidebar group **Finder**)

* **Look & Behavior**: Finder menu icon · auto-hide · shut-down confirmation · maximized windows affect appearance ·
  top-right corner closes maximized window · lock tray-icon dragging · show window options · colored tray icons ·
  input-method menu · search icon · account icon · media playback icon · lyrics menu · colored lyrics.
* **Notifications**, **Time & Date**, **Audio**, **Display**, **Hardware Status**: per-item visibility and format.
* **Battery**: show battery menu · show percentage · remaining time · auto-switch power plan · plan on power · plan on
  battery.
* **System Icon Tray** (ungrouped page): always show all tray icons in Finder / Dock · show system tray in Finder ·
  newly started tray icon shows time.

### 3.4 Architecture mapping

| Concern                        | Target                                               | Today                                                                               |
| ------------------------------ | ---------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Item layout, clock formatting  | `core/finder/{layout,clock}.rs`                      | ✅ in place                                                                          |
| Menu model (logo, app menus)   | `core/finder/menus.rs`                               | ⬜ menu items are built in `app/finder/menus.rs`                                     |
| Settings vs. state             | `core/settings/finder.rs`, `core/finder/state.rs`    | ⬜ `menubar_*`, `clock_24h`, `battery_pct` in the flat `Settings`                    |
| Battery, Wi-Fi, audio, display | `platform/system/{battery,wifi,audio,brightness}.rs` | ✅ implemented, flat layout until the platform regrouping lands                      |
| Drawing                        | `ui/features/finder/{view,item}.rs`                  | ⬜ `ui/features/finder` does not exist; drawing is `app/finder/draw.rs` (≈730 lines) |
| Window host, wndproc, actions  | `app/finder/{wndproc,actions,appearance}.rs`         | ✅ stays in `app`                                                                    |

### 3.5 Acceptance criteria

* Clock text, item layout and truncation are unit-tested in `core` for English and Indonesian locales.
* The Finder never draws over a full-screen app unless *above full-screen* is enabled.
* Turning Finder off leaves Dock and Dynamic Island unaffected and releases the work area it reserved.
* Wi-Fi and Bluetooth state changes are reflected without polling faster than once per second while idle.

---

## 4. Feature: Dynamic Island

### 4.1 Goal

A glass pill at the top-center of the screen that surfaces ongoing activity (music, downloads, timers, transfers) and
accepts dropped files. It rests as a small pill, peeks on hover, and opens into a panel on dwell or click.

### 4.2 Capabilities

| ID   | Capability                                                                                                                         | Status                            |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| I-1  | State machine: Rest → Peek → Open(tab) ↔ Drop; events Enter, Leave, Dwell, Click, SelectTab, DragEnter, DragLeave, Dropped, Escape | ✅ (`core::dynamic_island::state`) |
| I-2  | Two shapes: attached to the top edge, or floating capsule                                                                          | ✅                                 |
| I-3  | Idle content: greeting (time, date, battery) and rotating glance items                                                             | ✅                                 |
| I-4  | Play tab: artwork, title, seekable progress, controls                                                                              | ✅                                 |
| I-5  | Lyrics synced to playback: local `.lrc` files, optional LRCLIB lookup                                                              | ✅                                 |
| I-6  | Audio visualizer (loopback bands)                                                                                                  | ✅                                 |
| I-7  | Downloads: Microsoft Store, Steam, browser; progress ring                                                                          | ✅                                 |
| I-8  | Shelf tab: drop zones *Keep on Shelf · Send · Copy to… · Open with…*                                                               | ✅                                 |
| I-9  | Send to phone: local HTTP server + QR code, same-Wi-Fi, idle timeout                                                               | ✅                                 |
| I-10 | Timer tab with calendar strip from Windows calendar                                                                                | ✅                                 |
| I-11 | Privacy indicators (microphone/camera in use)                                                                                      | ✅                                 |
| I-12 | Hover-to-open toggle, idle style, greeting text, material, rounding                                                                | ✅                                 |
| I-13 | Notification surfacing (app toasts as transient island content)                                                                    | ⬜                                 |
| I-14 | Per-activity enable switches beyond downloads and send                                                                             | ⬜                                 |

### 4.3 Settings (sidebar group **Dynamic Island**)

Enable · shape (attached / capsule) · display mode (Fixed / Smart / On demand) · open on hover · idle style · greeting
name and text · material and rounding · lyrics compact · online lyrics · downloads · send to phone.

### 4.4 Architecture mapping

| Concern                                                | Target                                                           | Today                                                                                       |
| ------------------------------------------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| State machine, events                                  | `core/dynamic_island/{state,events,interactions}.rs`             | ✅ `state.rs` (pure `step()`); events are inside it                                          |
| Timer, shelf, glance, download, send, lyrics, bands    | `core/dynamic_island/*`                                          | ✅ in place                                                                                  |
| Layout                                                 | `core/dynamic_island/layout.rs`                                  | ⬜ pill geometry in `app/dynamic_island/{ears,body}.rs`                                      |
| Media, loopback, calendar, download watch, send server | `platform/media/*`, `platform/system/*`, `platform/filesystem/*` | ✅ implemented, flat layout until the platform regrouping lands                              |
| Rendering and animation                                | `ui/features/dynamic_island/{view,animations}.rs`                | ⬜ `app/dynamic_island/{views,draw,body,shadow}.rs` (≈2 900 lines)                           |
| Window host, wndproc, timers                           | `app/dynamic_island/*`                                           | 🟡 one 2 300-line `Notch` type mixes host, timers, platform events, domain steps and drawing |

### 4.5 Acceptance criteria

* `core::dynamic_island::state::step` is exhaustively unit-tested for every (mode, event, open-on) combination.
* Dropping files never starts a transfer until the user chooses a zone; *Send* shuts its server down after the idle
  timeout.
* With no media session and no activity the Island makes no periodic wake-ups beyond its minute tick.
* Lyrics lookup is off unless *online lyrics* is enabled.

---

## 5. Shared Settings window

One window with a sidebar on the left and the selected page on the right.

**Ungrouped (top):** General · Appearance · System Icon Tray · Screen · Stage Manager · Window Animation
**Dock:** Look & Behavior · Launchpad · Advanced · Minimize Effects · Window Previews
**Finder:** Look & Behavior · Notifications · Time & Date · Audio · Display · Hardware Status · Battery
**Dynamic Island:** Look & Behavior · Activities (media, downloads, shelf, send, timer)

### 5.1 General

* Toggles: enable Finder · enable Dock · enable Dynamic Island · exit automatically when a full-screen app runs.
* Sliders: global interface scale · global blur intensity.
* Radio: appearance (Light, Dark, System) · accent color.
* Select: start-on-boot method (Don't start · Registry value · Scheduled task · Service) · hide taskbar on boot (all
  displays / Dock display) · display language (English, Indonesian) · display font (+ global font scale).

### 5.2 Appearance

Per-surface *custom blur and transparency* switches with sliders (blur, transparency, border transparency): Dock,
icon labels, context menus, folder grid (+ folder icon size), Nexora menus, Finder (+ wallpaper tint), Launchpad
(blur, pre-render vs. live blur, show above windows). Icon reflection: blur, transparency, distance, gradient.

### 5.3 Screen / Stage Manager / Window Animation

* **Stage Manager**: show icon in Finder · title on hover · blur background icons · click desktop to hide windows ·
  pause/play background media · native preview on hover · desktop icons (Hide, Auto, Show) · animation speed
  (0.5×–1.5×) · list position (left/right) · list visibility (Default, Always show, Always hide) · keep window
  border · icon size, spacing and count.
* **Window Animation**: enable open animation · effect · speed · excluded programs.

### 5.4 Settings schema target

The flat `Settings` struct (≈65 fields) is split along the sections above so each feature owns its preferences:

| Target type          | Examples of fields it owns                                                                                                                 |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `GeneralSettings`    | `takeover`, `autostart`, `lang`, `ui_scale`, `above_fullscreen`, `anim`                                                                    |
| `AppearanceSettings` | `theme`, `accent`, `blur`, `style`, `transparency`, `refraction`, `win_shadow`, `icon_*`                                                   |
| `DockSettings`       | `dock_enabled`, `dock` (behavior), `dock_size/gap/zoom/margin/material/round/plates`, `min_effect`, `recent_apps`, `downloads_stack`       |
| `FinderSettings`     | `menubar`, `menubar_style/glass/material`, `clock_24h`, `battery_pct`                                                                      |
| `IslandSettings`     | `notch_enabled`, `notch` (behavior), `notch_island/material/round/idle/hover_open`, `greet_*`, `lyrics_*`, `notch_downloads`, `notch_send` |
| `StageSettings`      | `stage`, `stage_*`, `switcher`, `hide_desktop_icons`                                                                                       |
| `InputSettings`      | `hot`, `mac_shortcuts`, `clipboard_history`                                                                                                |

The on-disk `settings.cfg` format must keep reading existing files: keys are not renamed, only regrouped in code.

---

## 6. Implementation order

Follows `architecture.md` §9. Items marked ✅ are done on this branch; the rest are queued in dependency order.

1. ✅ Toolchain hygiene: crate names `nexora-*`, deny-level clippy errors fixed, icon path case fixed.
2. ⬜ App composition root: `bootstrap.rs`, `application.rs`, `lifecycle.rs`, `startup/`; keep `main.rs` minimal.
3. ⬜ Dynamic Island host: split the `Notch` implementation by responsibility inside `app/dynamic_island/`.
4. ⬜ `core` regrouping into feature folders (`settings/`, `launchpad/`, `window_management/`, `panels/`, `shared/`).
5. ⬜ `platform` regrouping into `windows/`, `system/`, `media/`, `filesystem/`, `input/`.
6. ⬜ Settings schema split (section 5.4) with a compatibility test over a real `settings.cfg`.
7. ⬜ Move presentation into `ui/features/{dock,finder,dynamic_island}` by injecting a theme/settings snapshot instead of
   reading the global `app::prefs` from drawing code.
8. ⬜ New capabilities marked ⬜ above, one feature at a time, each with `core` unit tests first.

Definition of done for every step (from `AGENTS.md` §7):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo check --workspace
```
