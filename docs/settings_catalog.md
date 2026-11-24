# Nexora Desktop — Settings Catalog

Single list of every setting that is **in scope** (SPEC-02). It is generated from `TODO.md` §5 and already applies
ADR-0004…0008 and `spec_reconciliation.md` (SPEC-03): for an existing setting the range/default of `core` wins, for a
new setting the design wins. Product-level description of each page stays in [`feature.md`](feature.md); implementation
status stays in [`feature_checkpoint.md`](feature_checkpoint.md) and `TODO.md`.

Columns: **Design key** is the key in `design/settings-schema.js` (`section.name`). **Owner section** is the Rust
settings struct from `feature_checkpoint.md` §5.4 (`FinderSettings` per ADR-0003; UI name *Menu Bar*). **Cap** is
the capability slice. **Rust field today** is the field name in `core::settings` (✔ exists, ◐ only in an uncompiled
file, ~ exists with a different model, ✗ does not exist, ⊘ not in the spec before ADR-0007). Persistence follows
ADR-0004: new keys are written only when ≠ default; lists live in `lists.cfg` (ADR-0005).

**175 settings in scope** on 20 pages.


## System ▸ General (12)

| ID     | Setting (UI label, English)            | Design key               | Type · range / options                                           | Default             | Owner section   | Cap | Rust field today     | Note                                                 |
| ------ | -------------------------------------- | ------------------------ | ---------------------------------------------------------------- | ------------------- | --------------- | --- | -------------------- | ---------------------------------------------------- |
| GEN-01 | Enable Menu Bar                        | shared.enableFinder      | toggle                                                           | on                  | GeneralSettings | F-* | ✔ `menubar`          |                                                      |
| GEN-02 | Enable Dock                            | shared.enableDock        | toggle                                                           | on                  | GeneralSettings | D-* | ✔ `dock_enabled`     |                                                      |
| GEN-03 | Stay above full-screen windows         | shared.aboveFullscreen   | toggle                                                           | on                  | GeneralSettings | GEN | ~ `above_fullscreen` | inverse of the design label; core `above_fullscreen` |
| GEN-04 | Global Interface Scale                 | shared.globalScale       | range 75–200 (%)                                                 | 100                 | GeneralSettings | GEN | ~ `ui_scale`         | core `ui_scale`                                      |
| GEN-05 | Global Interface Blur Intensity        | shared.globalBlur        | range 0–100                                                      | 60                  | GeneralSettings | GEN | ~ `blur`             | core `blur`                                          |
| GEN-06 | Appearance                             | shared.appearance        | select [light \| dark \| system]                                 | system              | GeneralSettings | GEN | ✔ `theme`            | core `theme`                                         |
| GEN-07 | Accent Color                           | shared.accentColor       | palette swatch (index)                                           | 0                   | GeneralSettings | GEN | ~ `accent`           | core `accent`; custom hex deferred                   |
| GEN-08 | Start On Boot Method                   | shared.startOnBoot       | select [Don't Start On Boot \| Registry Value \| Scheduled Task] | Don't Start On Boot | GeneralSettings | GEN | ~ `autostart`        | ADR-0007 drops service; legacy `autostart` derived   |
| GEN-09 | Automatically hide the taskbar on boot | shared.hideTaskbarOnBoot | select [none \| allDisplays \| dockDisplay]                      | allDisplays         | GeneralSettings | GEN | ~ `takeover`         | legacy `takeover` derived                            |
| GEN-10 | Display Language                       | shared.displayLanguage   | select [English \| Indonesian]                                   | Indonesian          | GeneralSettings | GEN | ✔ `lang`             | core `lang` default                                  |
| GEN-11 | Display Font                           | shared.displayFont       | select (5 options)                                               | Segoe UI            | GeneralSettings | GEN | ✗                    |                                                      |
| GEN-12 | Global Font Scaling                    | shared.globalFontScale   | range 0.75–1.5                                                   | 1                   | GeneralSettings | GEN | ✗                    |                                                      |

## System ▸ Appearance (33)

| ID     | Setting (UI label, English)                                       | Design key                        | Type · range / options | Default | Owner section      | Cap  | Rust field today | Note |
| ------ | ----------------------------------------------------------------- | --------------------------------- | ---------------------- | ------- | ------------------ | ---- | ---------------- | ---- |
| APP-01 | Make Dock use custom blur and transparency values                 | shared.customDockBlur             | toggle                 | off     | AppearanceSettings | D-15 | ✗                |      |
| APP-02 | Dock blur intensity                                               | shared.dockBlur                   | range 0–50             | 22      | AppearanceSettings | D-15 | ✗                |      |
| APP-03 | Dock transparency                                                 | shared.dockTransparency           | range 0–100            | 62      | AppearanceSettings | D-15 | ✗                |      |
| APP-04 | Make pinned icons reflect from the Dock                           | shared.pinnedReflections          | toggle                 | off     | AppearanceSettings | D-15 | ✗                |      |
| APP-05 | Reflection blur strength                                          | shared.reflectionBlur             | range 0–30             | 5       | AppearanceSettings | D-15 | ✗                |      |
| APP-06 | Reflection transparency                                           | shared.reflectionTransparency     | range 0–100            | 35      | AppearanceSettings | D-15 | ✗                |      |
| APP-07 | Icon reflection distance                                          | shared.reflectionDistance         | range 0–20             | 3       | AppearanceSettings | D-15 | ✗                |      |
| APP-08 | Gradient distance of icon reflection                              | shared.reflectionGradientDistance | range 0–30             | 8       | AppearanceSettings | D-15 | ✗                |      |
| APP-09 | Make Dock icon labels use custom blur and transparency values     | shared.customIconLabels           | toggle                 | off     | AppearanceSettings | D-15 | ✗                |      |
| APP-10 | Label blur intensity                                              | shared.labelBlur                  | range 0–40             | 18      | AppearanceSettings | D-15 | ✗                |      |
| APP-11 | Label transparency                                                | shared.labelTransparency          | range 0–100            | 70      | AppearanceSettings | D-15 | ✗                |      |
| APP-12 | Label border transparency                                         | shared.labelBorderTransparency    | range 0–100            | 12      | AppearanceSettings | D-15 | ✗                |      |
| APP-13 | Make context menus use custom blur and transparency values        | shared.customContextMenus         | toggle                 | off     | AppearanceSettings | D-15 | ✗                |      |
| APP-14 | Context menu blur intensity                                       | shared.contextBlur                | range 0–50             | 22      | AppearanceSettings | D-15 | ✗                |      |
| APP-15 | Context menu transparency                                         | shared.contextTransparency        | range 0–100            | 85      | AppearanceSettings | D-15 | ✗                |      |
| APP-16 | Context menu border transparency                                  | shared.contextBorder              | range 0–100            | 12      | AppearanceSettings | D-15 | ✗                |      |
| APP-17 | Make folder contents grid use custom blur and transparency values | shared.customFolderGrid           | toggle                 | off     | AppearanceSettings | D-17 | ✗                |      |
| APP-18 | Folder grid blur intensity                                        | shared.folderBlur                 | range 0–50             | 22      | AppearanceSettings | D-17 | ✗                |      |
| APP-19 | Folder grid transparency                                          | shared.folderTransparency         | range 0–100            | 85      | AppearanceSettings | D-17 | ✗                |      |
| APP-20 | Folder grid border transparency                                   | shared.folderBorder               | range 0–100            | 12      | AppearanceSettings | D-17 | ✗                |      |
| APP-21 | Folder grid icon size                                             | shared.folderIconSize             | range 24–96            | 48      | AppearanceSettings | D-17 | ✗                |      |
| APP-22 | Make Nexora menus use custom blur and transparency values         | shared.customNexoraMenus          | toggle                 | off     | AppearanceSettings | D-15 | ✗                |      |
| APP-23 | Nexora menu blur intensity                                        | shared.menuBlur                   | range 0–50             | 26      | AppearanceSettings | D-15 | ✗                |      |
| APP-24 | Nexora menu transparency                                          | shared.menuTransparency           | range 0–100            | 90      | AppearanceSettings | D-15 | ✗                |      |
| APP-25 | Nexora menu border transparency                                   | shared.menuBorder                 | range 0–100            | 10      | AppearanceSettings | D-15 | ✗                |      |
| APP-26 | Make Menu Bar use custom blur and transparency values             | shared.customFinderBlur           | toggle                 | off     | AppearanceSettings | F-7  | ✗                |      |
| APP-27 | Menu Bar blur intensity                                           | shared.finderBlur                 | range 0–50             | 26      | AppearanceSettings | F-7  | ✗                |      |
| APP-28 | Menu Bar transparency                                             | shared.finderTransparency         | range 0–100            | 88      | AppearanceSettings | F-7  | ✗                |      |
| APP-29 | Wallpaper tint intensity                                          | shared.wallpaperTint              | range 0–100            | 0       | AppearanceSettings | F-7  | ✗                |      |
| APP-30 | Launchpad uses custom background blur                             | shared.launchpadCustomBlur        | toggle                 | off     | AppearanceSettings | D-8  | ✗                |      |
| APP-31 | Display Launchpad above windows instead of hiding them            | shared.launchpadAboveWindows      | toggle                 | off     | AppearanceSettings | D-8  | ✗                |      |
| APP-32 | Launchpad blurs the background instead of pre-rendering it        | shared.launchpadBlurBackground    | toggle                 | on      | AppearanceSettings | D-8  | ✗                |      |
| APP-33 | Launchpad background blur intensity                               | shared.launchpadBlur              | range 0–50             | 24      | AppearanceSettings | D-8  | ✗                |      |

## System ▸ System Icon Tray (4)

| ID     | Setting (UI label, English)                    | Design key                | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | ---------------------------------------------- | ------------------------- | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| TRY-01 | Always display all tray icons in Menu Bar      | shared.trayAllFinder      | toggle                 | off     | FinderSettings | F-11 | ✗                |      |
| TRY-02 | Always display all tray icons in Dock          | shared.trayAllDock        | toggle                 | off     | FinderSettings | F-11 | ✗                |      |
| TRY-03 | Display the system's own tray icon in Menu Bar | shared.traySystemFinder   | toggle                 | on      | FinderSettings | F-11 | ✗                |      |
| TRY-04 | Newly run tray icon display time (seconds)     | shared.newTrayIconSeconds | range 0–30             | 5       | FinderSettings | F-11 | ✗                |      |

## System ▸ Screen (4)

| ID     | Setting (UI label, English) | Design key                 | Type · range / options                                     | Default         | Owner section   | Cap | Rust field today | Note |
| ------ | --------------------------- | -------------------------- | ---------------------------------------------------------- | --------------- | --------------- | --- | ---------------- | ---- |
| SCR-01 | Display brightness overlay  | shared.screenBrightness    | range 0–100                                                | 100             | GeneralSettings | SCR | ✗                |      |
| SCR-02 | Enable night light          | shared.nightLight          | toggle                                                     | off             | GeneralSettings | SCR | ✗                |      |
| SCR-03 | Night light intensity       | shared.nightLightIntensity | range 0–100                                                | 25              | GeneralSettings | SCR | ✗                |      |
| SCR-04 | Display target              | shared.screenTarget        | select [All displays \| Primary display \| Active display] | Primary display | GeneralSettings | SCR | ✗                |      |

## System ▸ Stage Manager (15)

| ID     | Setting (UI label, English)                          | Design key                   | Type · range / options                                     | Default          | Owner section | Cap | Rust field today        | Note                                |
| ------ | ---------------------------------------------------- | ---------------------------- | ---------------------------------------------------------- | ---------------- | ------------- | --- | ----------------------- | ----------------------------------- |
| STG-01 | Show Stage Manager icon in Menu Bar                  | shared.stageIcon             | toggle                                                     | on               | StageSettings | STG | ✗                       |                                     |
| STG-02 | Show window title on mouseover                       | shared.stageWindowTitle      | toggle                                                     | on               | StageSettings | STG | ✔ `stage_titles`        |                                     |
| STG-03 | Blur background window icons                         | shared.stageBlurIcons        | toggle                                                     | on               | StageSettings | STG | ✗                       |                                     |
| STG-04 | Click on the desktop to hide all windows             | shared.stageClickDesktopHide | toggle                                                     | on               | StageSettings | STG | ✔ `stage_desktop_click` |                                     |
| STG-05 | Automatically pause and play background window media | shared.stageAutoPauseMedia   | toggle                                                     | off              | StageSettings | STG | ✔ `stage_media`         |                                     |
| STG-06 | Show native window preview on hover                  | shared.stageNativePreview    | toggle                                                     | on               | StageSettings | STG | ✗                       |                                     |
| STG-07 | Show Desktop Icons                                   | shared.stageDesktopIcons     | select [Always Hide Icon \| Auto Show Hide \| Always Show] | Always Hide Icon | StageSettings | STG | ~ `hide_desktop_icons`  | legacy `hide_desktop_icons` derived |
| STG-08 | Adjust Animation Speed                               | shared.stageAnimationSpeed   | select [0.5x \| 0.75x \| 1.0x \| 1.25x \| 1.5x]            | 1.0x             | StageSettings | STG | ~ `stage_speed`         | presets of `stage_speed` 50–200 %   |
| STG-09 | Open Stage Manager                                   | shared.stageEnabled          | toggle                                                     | off              | StageSettings | STG | ✔ `stage`               |                                     |
| STG-10 | Window list screen position                          | shared.stagePosition         | select [left \| right]                                     | left             | StageSettings | STG | ✔ `stage_right`         |                                     |
| STG-11 | Window List Show Hidden Mode                         | shared.stageHiddenMode       | select [default \| alwaysShow \| alwaysHide]               | default          | StageSettings | STG | ✗                       |                                     |
| STG-12 | Keep Window Border                                   | shared.stageKeepBorder       | toggle                                                     | off              | StageSettings | STG | ✗                       |                                     |
| STG-13 | Window list icon size                                | shared.stageIconSize         | range 70–260                                               | 170              | StageSettings | STG | ~ `stage_size`          | core `stage_size` (card size)       |
| STG-14 | Window list icon spacing                             | shared.stageIconSpacing      | range 0–60                                                 | 20               | StageSettings | STG | ✔ `stage_gap`           | core `stage_gap`                    |
| STG-15 | Number of window list icons                          | shared.stageIconCount        | range 2–10                                                 | 6                | StageSettings | STG | ✔ `stage_count`         | core `stage_count`                  |

## System ▸ Window Animation (6)

| ID     | Setting (UI label, English)                 | Design key                     | Type · range / options                  | Default | Owner section   | Cap | Rust field today          | Note |
| ------ | ------------------------------------------- | ------------------------------ | --------------------------------------- | ------- | --------------- | --- | ------------------------- | ---- |
| WAN-01 | Window opening animation                    | shared.windowAnimation         | toggle                                  | on      | GeneralSettings | WAN | ✔ `anim`                  |      |
| WAN-02 | Window opening animation effect             | shared.windowAnimationEffect   | select [scale \| fade \| slide \| zoom] | scale   | GeneralSettings | WAN | ✗                         |      |
| WAN-03 | Adjust animation speed                      | shared.windowAnimationSpeed    | select (5 options)                      | 1.0x    | GeneralSettings | WAN | ✗                         |      |
| WAN-04 | Add excluded programs (semicolon separated) | shared.excludedPrograms        | text                                    | empty   | GeneralSettings | WAN | ✗ list value → ADR-0005   |      |
| WAN-05 | Manage running program blacklist            | shared.programBlacklist        | text                                    | empty   | GeneralSettings | WAN | ✗ list value → ADR-0005   |      |
| WAN-06 | Remove windows for excluded programs        | shared.minimizeExcludedWindows | toggle                                  | on      | GeneralSettings | WAN | ✗ meaning unclear in spec |      |

## Dock ▸ Look & Behavior (10)

| ID     | Setting (UI label, English)     | Design key               | Type · range / options                            | Default         | Owner section | Cap  | Rust field today                                 | Note                                  |
| ------ | ------------------------------- | ------------------------ | ------------------------------------------------- | --------------- | ------------- | ---- | ------------------------------------------------ | ------------------------------------- |
| DLB-01 | Icon size                       | dock.iconSize            | range 32–72                                       | 48              | DockSettings  | D-5  | ✔ `dock_size`                                    | core `dock_size`                      |
| DLB-02 | Icon effect                     | dock.iconEffect          | select (6 options)                                | Scaling         | DockSettings  | D-14 | ◐ `IconEffect` in orphan `core/dock/behavior.rs` |                                       |
| DLB-03 | Icon magnification              | dock.iconMagnification   | range 1–2.5                                       | 2               | DockSettings  | D-5  | ✔ `dock_zoom`                                    | core `dock_zoom` 100–250, default 200 |
| DLB-04 | Icon light intensity            | dock.iconLightIntensity  | range 0–100                                       | 35              | DockSettings  | D-14 | ◐ `DockConfig.light_intensity`                   |                                       |
| DLB-05 | Icon background light intensity | dock.iconBackgroundLight | range 0–100                                       | 25              | DockSettings  | D-14 | ◐ `DockConfig.bg_light_intensity`                |                                       |
| DLB-06 | Icon background light color     | dock.iconBackgroundColor | color                                             | #63c7dc         | DockSettings  | D-14 | ◐ `DockConfig.light_rgb` (design is hex)         |                                       |
| DLB-07 | Distance between icons          | dock.iconSpacing         | range 0–24                                        | 14              | DockSettings  | D-5  | ✔ `dock_gap`                                     | core `dock_gap`                       |
| DLB-08 | Dock distance from screen edge  | dock.edgeDistance        | range 0–60                                        | 8               | DockSettings  | D-5  | ✔ `dock_margin`                                  | core `dock_margin`                    |
| DLB-09 | Position on screen              | dock.position            | select [left \| bottom \| right \| top]           | bottom          | DockSettings  | D-11 | ✗                                                |                                       |
| DLB-10 | Display on screen               | dock.display             | select [Primary display \| each detected display] | Primary display | DockSettings  | D-12 | ✗                                                | ADR-0007 drops "All displays"         |

## Dock ▸ Launchpad (7)

| ID     | Setting (UI label, English)                       | Design key                      | Type · range / options                                             | Default         | Owner section | Cap | Rust field today        | Note |
| ------ | ------------------------------------------------- | ------------------------------- | ------------------------------------------------------------------ | --------------- | ------------- | --- | ----------------------- | ---- |
| DLP-01 | Launchpad icon size                               | dock.launchpadIconSize          | range 24–100                                                       | 52              | DockSettings  | D-8 | ✗                       |      |
| DLP-02 | Icon horizontal spacing                           | dock.launchpadHorizontalSpacing | range 0–60                                                         | 22              | DockSettings  | D-8 | ✗                       |      |
| DLP-03 | Icon vertical spacing                             | dock.launchpadVerticalSpacing   | range 0–60                                                         | 24              | DockSettings  | D-8 | ✗                       |      |
| DLP-04 | Choose which icons will be displayed by Launchpad | dock.launchpadSources           | select [added \| addedStartMenu \| addedDesktop \| all]            | added           | DockSettings  | D-8 | ✗                       |      |
| DLP-05 | Edit deleted Start Menu programs                  | dock.deletedStartMenuPrograms   | text                                                               | empty           | DockSettings  | D-8 | ✗ list value → ADR-0005 |      |
| DLP-06 | Select the screen to display Launchpad            | dock.launchpadDisplay           | select [Primary display \| All displays \| Display 1 \| Display 2] | Primary display | DockSettings  | D-8 | ✗                       |      |
| DLP-07 | Follow Dock                                       | dock.launchpadFollowDock        | toggle                                                             | on              | DockSettings  | D-8 | ✗                       |      |

## Dock ▸ Advanced (19)

| ID     | Setting (UI label, English)                                             | Design key                   | Type · range / options                                                                                                                                                                                    | Default                | Owner section | Cap  | Rust field today                                        | Note                                                                                 |
| ------ | ----------------------------------------------------------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ------------- | ---- | ------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| DAD-01 | Scale all icons to macOS standards                                      | dock.scaleMacIcons           | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-9  | ~ `dock_plates` (squircle plates: confirm same meaning) |                                                                                      |
| DAD-02 | Double-click the folder icon to open in Menu Bar                        | dock.doubleClickFolderFinder | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-17 | ✗                                                       |                                                                                      |
| DAD-03 | Hide the open folder button                                             | dock.hideOpenFolderButton    | toggle                                                                                                                                                                                                    | off                    | DockSettings  | D-17 | ✗                                                       |                                                                                      |
| DAD-04 | Lock icon dragging                                                      | dock.lockIconDragging        | toggle                                                                                                                                                                                                    | off                    | DockSettings  | D-3  | ◐ `DockConfig.lock_drag`                                |                                                                                      |
| DAD-05 | Show icon labels                                                        | dock.showIconLabels          | toggle                                                                                                                                                                                                    | off                    | DockSettings  | D-1  | ◐ `DockConfig.show_labels`                              |                                                                                      |
| DAD-06 | Animate opening applications                                            | dock.animateAppLaunch        | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-1  | ◐ `DockConfig.animate_launch`                           |                                                                                      |
| DAD-07 | Turn on window borders to prevent windows from hiding Dock and Menu Bar | dock.preventWindowsCoverDock | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-10 | ✗                                                       |                                                                                      |
| DAD-08 | Show indicators under running programs                                  | dock.showRunningIndicators   | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-1  | ◐ `DockConfig.show_indicators`                          |                                                                                      |
| DAD-09 | Show icons of unpinned running programs                                 | dock.showUnpinnedRunning     | toggle                                                                                                                                                                                                    | on                     | DockSettings  | D-1  | ◐ `DockConfig.show_unpinned`                            |                                                                                      |
| DAD-10 | Separate unpinned running programs                                      | dock.separateUnpinned        | toggle                                                                                                                                                                                                    | off                    | DockSettings  | D-1  | ◐ `DockConfig.separate_unpinned`                        |                                                                                      |
| DAD-11 | Bottom left hot-corner function                                         | dock.bottomLeftHotCorner     | select [Do nothing \| Mission Control (Show all windows) \| Open Launchpad \| Show Desktop \| Stage Manager \| Notifications \| Lock Screen \| Put display to sleep \| Open Start Menu \| Windows widget] | Do nothing             | DockSettings  | D-16 | ~ `hot[2]`                                              | 10 actions; ADR-0007 drops "Advanced"                                                |
| DAD-12 | Dock display mode                                                       | dock.displayMode             | select [Default \| Overlay \| Auto-hide \| Smart auto-hide \| Always show \| Always force visibility \| Display only on the desktop]                                                                      | Default                | DockSettings  | D-10 | ~ `dock: Behavior`                                      | ADR-0007 drops "Only display with…"; D-10 maps Default/Overlay/Smart onto `Behavior` |
| DAD-13 | Detection window                                                        | dock.detectionWindow         | select [All Program Windows \| The Current Program Window]                                                                                                                                                | All Program Windows    | DockSettings  | D-10 | ✗                                                       |                                                                                      |
| DAD-14 | Dock hidden effect                                                      | dock.hiddenEffect            | select [Move \| Fade Away]                                                                                                                                                                                | Move                   | DockSettings  | D-10 | ◐ `DockConfig.hide_effect`                              |                                                                                      |
| DAD-15 | Dock activation trigger                                                 | dock.activationTrigger       | select [The edge of the screen \| Activation bar \| The edge of the screen & Activation bar]                                                                                                              | The edge of the screen | DockSettings  | D-13 | ✗                                                       |                                                                                      |
| DAD-16 | Activation bar width                                                    | dock.activationBarWidth      | range 1–100                                                                                                                                                                                               | 4                      | DockSettings  | D-13 | ✗                                                       |                                                                                      |
| DAD-17 | Activation bar height                                                   | dock.activationBarHeight     | range 1–30                                                                                                                                                                                                | 2                      | DockSettings  | D-13 | ✗                                                       |                                                                                      |
| DAD-18 | Activation bar distance from screen edge                                | dock.activationBarDistance   | range 0–40                                                                                                                                                                                                | 0                      | DockSettings  | D-13 | ✗                                                       |                                                                                      |
| DAD-19 | Mouse delay activates dock time (ms)                                    | dock.mouseActivationDelay    | range 0–1500 (ms)                                                                                                                                                                                         | 150                    | DockSettings  | D-13 | ◐ `DockConfig.activation_delay`                         | design wins: draft is not compiled                                                   |

## Dock ▸ Minimize Effects (9)

| ID     | Setting (UI label, English)                       | Design key                    | Type · range / options                                                                                                                 | Default                       | Owner section | Cap | Rust field today                            | Note |
| ------ | ------------------------------------------------- | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------- | --- | ------------------------------------------- | ---- |
| DMN-01 | Use Nexora minimize effect                        | dock.useNexoraMinimize        | toggle                                                                                                                                 | on                            | DockSettings  | D-7 | ✗                                           |      |
| DMN-02 | Don't animate minimizing windows                  | dock.disableMinimizeAnimation | toggle                                                                                                                                 | off                           | DockSettings  | D-7 | ✗                                           |      |
| DMN-03 | Hold down Shift to slow down an animation         | dock.shiftSlowAnimation       | toggle                                                                                                                                 | on                            | DockSettings  | D-7 | ✗                                           |      |
| DMN-04 | Minimize windows into their Dock icon             | dock.minimizeIntoIcon         | toggle                                                                                                                                 | on                            | DockSettings  | D-7 | ✗                                           |      |
| DMN-05 | Dynamically display window content when minimized | dock.dynamicMinimizeContent   | toggle                                                                                                                                 | on                            | DockSettings  | D-7 | ✗                                           |      |
| DMN-06 | Minimize window icon position                     | dock.minimizedIconPosition    | select [Next to the icon on the right \| Independently displayed on the right \| Right side displayed independently using a separator] | Next to the icon on the right | DockSettings  | D-7 | ✗                                           |      |
| DMN-07 | Minimize animation                                | dock.minimizeAnimation        | select [Genie \| Scale \| Suck]                                                                                                        | Genie                         | DockSettings  | D-7 | ~ `min_effect` (0 Funnel, 1 Scale; no Suck) |      |
| DMN-08 | Minimize animation speed                          | dock.minimizeAnimationSpeed   | select (5 options)                                                                                                                     | 1.0x                          | DockSettings  | D-7 | ✗                                           |      |
| DMN-09 | Minimize animation blacklist                      | dock.minimizeBlacklist        | text                                                                                                                                   | empty                         | DockSettings  | D-7 | ✗ list value → ADR-0005                     |      |

## Dock ▸ Window Previews (6)

| ID     | Setting (UI label, English)         | Design key           | Type · range / options | Default | Owner section | Cap | Rust field today                                        | Note |
| ------ | ----------------------------------- | -------------------- | ---------------------- | ------- | ------------- | --- | ------------------------------------------------------- | ---- |
| DPV-01 | Enable window previews              | dock.windowPreviews  | toggle                 | on      | DockSettings  | D-6 | ◐ `DockConfig.preview_enabled`                          |      |
| DPV-02 | Window preview size                 | dock.previewSize     | range 120–520          | 260     | DockSettings  | D-6 | ✗                                                       |      |
| DPV-03 | Preview delay on hover (ms)         | dock.previewDelay    | range 0–1500           | 450     | DockSettings  | D-6 | ◐ `DockConfig.preview_delay` (max 1000 ms; design 1500) |      |
| DPV-04 | Application name                    | dock.previewAppName  | toggle                 | on      | DockSettings  | D-6 | ✗                                                       |      |
| DPV-05 | Window control buttons              | dock.previewControls | toggle                 | on      | DockSettings  | D-6 | ✗                                                       |      |
| DPV-06 | Show native window preview on hover | dock.previewNative   | toggle                 | on      | DockSettings  | D-6 | ✗                                                       |      |

## Menu Bar ▸ Look & Behavior (14)

| ID     | Setting (UI label, English)                                        | Design key                        | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | ------------------------------------------------------------------ | --------------------------------- | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| FLB-01 | Menu Bar menu icon                                                 | finder.menuIcon                   | toggle                 | on      | FinderSettings | F-1  | ✗                |      |
| FLB-02 | Auto-Hide Menu Bar                                                 | finder.autoHide                   | toggle                 | off     | FinderSettings | F-8  | ✗                |      |
| FLB-03 | Shut down confirmation box                                         | finder.shutdownConfirmation       | toggle                 | on      | FinderSettings | F-9  | ✗                |      |
| FLB-04 | Maximized windows affect Menu Bar appearance                       | finder.maximizedAffectsAppearance | toggle                 | on      | FinderSettings | F-8  | ✗                |      |
| FLB-05 | Click the upper right corner to close the current maximized window | finder.closeMaximizedOnCorner     | toggle                 | off     | FinderSettings | F-10 | ✗                |      |
| FLB-06 | Lock dragging tray icons                                           | finder.lockTrayDragging           | toggle                 | off     | FinderSettings | F-11 | ✗                |      |
| FLB-07 | Show window options in Menu Bar                                    | finder.windowOptions              | toggle                 | on      | FinderSettings | F-3  | ✗                |      |
| FLB-08 | Show colored tray icons                                            | finder.coloredTrayIcons           | toggle                 | on      | FinderSettings | F-11 | ✗                |      |
| FLB-09 | Show input method menu                                             | finder.inputMethodMenu            | toggle                 | on      | FinderSettings | F-18 | ✗                |      |
| FLB-10 | Show search icon                                                   | finder.searchIcon                 | toggle                 | on      | FinderSettings | F-18 | ✗                |      |
| FLB-11 | The account icon is displayed                                      | finder.accountIcon                | toggle                 | on      | FinderSettings | F-18 | ✗                |      |
| FLB-12 | Displays the media playback icon                                   | finder.mediaPlaybackIcon          | toggle                 | on      | FinderSettings | F-17 | ✗                |      |
| FLB-13 | Show music player lyrics menu                                      | finder.lyricsMenu                 | toggle                 | on      | FinderSettings | F-17 | ✗                |      |
| FLB-14 | Show Color Player Lyrics                                           | finder.colorPlayerLyrics          | toggle                 | on      | FinderSettings | F-17 | ✗                |      |

## Menu Bar ▸ Notifications (3)

| ID     | Setting (UI label, English)             | Design key                  | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | --------------------------------------- | --------------------------- | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| FNO-01 | Show desktop notifications              | finder.showNotifications    | toggle                 | on      | FinderSettings | F-12 | ✗                |      |
| FNO-02 | Notification sounds                     | finder.notificationSounds   | toggle                 | on      | FinderSettings | F-12 | ✗                |      |
| FNO-03 | Notification display duration (seconds) | finder.notificationDuration | range 1–15             | 5       | FinderSettings | F-12 | ✗                |      |

## Menu Bar ▸ Time & Date (4)

| ID     | Setting (UI label, English) | Design key        | Type · range / options        | Default | Owner section  | Cap | Rust field today     | Note |
| ------ | --------------------------- | ----------------- | ----------------------------- | ------- | -------------- | --- | -------------------- | ---- |
| FTD-01 | Show time in Menu Bar       | finder.showTime   | toggle                        | on      | FinderSettings | F-4 | ✗                    |      |
| FTD-02 | Show date in Menu Bar       | finder.showDate   | toggle                        | on      | FinderSettings | F-4 | ✗                    |      |
| FTD-03 | Time format                 | finder.timeFormat | select [12-hour \| 24-hour]   | 24-hour | FinderSettings | F-4 | ~ `clock_24h` (bool) |      |
| FTD-04 | Date format                 | finder.dateFormat | select [Short \| Long \| ISO] | Short   | FinderSettings | F-4 | ✗                    |      |

## Menu Bar ▸ Audio (3)

| ID     | Setting (UI label, English)  | Design key               | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | ---------------------------- | ------------------------ | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| FAU-01 | Show volume control          | finder.showVolume        | toggle                 | on      | FinderSettings | F-13 | ✗                |      |
| FAU-02 | Show audio output selector   | finder.showAudioOutput   | toggle                 | on      | FinderSettings | F-13 | ✗                |      |
| FAU-03 | Show media playback controls | finder.showMediaControls | toggle                 | on      | FinderSettings | F-13 | ✗                |      |

## Menu Bar ▸ Display (1)

| ID     | Setting (UI label, English)     | Design key            | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | ------------------------------- | --------------------- | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| FDI-01 | Show display brightness control | finder.showBrightness | toggle                 | on      | FinderSettings | F-14 | ✗                |      |

## Menu Bar ▸ Hardware Status (3)

| ID     | Setting (UI label, English) | Design key               | Type · range / options | Default | Owner section  | Cap  | Rust field today | Note |
| ------ | --------------------------- | ------------------------ | ---------------------- | ------- | -------------- | ---- | ---------------- | ---- |
| FHW-01 | Show Wi-Fi status           | finder.showWifi          | toggle                 | on      | FinderSettings | F-4  | ✗                |      |
| FHW-02 | Show Bluetooth status       | finder.showBluetooth     | toggle                 | on      | FinderSettings | F-15 | ✗                |      |
| FHW-03 | Show device status icons    | finder.showHardwareIcons | toggle                 | on      | FinderSettings | F-15 | ✗                |      |

## Menu Bar ▸ Battery (6)

| ID     | Setting (UI label, English)                     | Design key                  | Type · range / options                                      | Default  | Owner section  | Cap  | Rust field today                              | Note |
| ------ | ----------------------------------------------- | --------------------------- | ----------------------------------------------------------- | -------- | -------------- | ---- | --------------------------------------------- | ---- |
| FBA-01 | Show Battery Status menu in Menu Bar            | finder.showBatteryMenu      | toggle                                                      | on       | FinderSettings | F-16 | ✗                                             |      |
| FBA-02 | Show Battery percentage in Menu Bar status menu | finder.batteryPercentage    | toggle                                                      | on       | FinderSettings | F-16 | ~ `battery_pct` (3 modes; design is a toggle) |      |
| FBA-03 | Display the remaining battery time              | finder.remainingBatteryTime | toggle                                                      | on       | FinderSettings | F-16 | ✗                                             |      |
| FBA-04 | Automatically switch power modes                | finder.autoPowerPlans       | toggle                                                      | off      | FinderSettings | F-16 | ✗                                             |      |
| FBA-05 | Power mode when plugged in                      | finder.powerPlanPlugged     | select [balanced \| bestPerformance \| bestPowerEfficiency] | balanced | FinderSettings | F-16 | ✗                                             |      |
| FBA-06 | Power mode when using batteries                 | finder.powerPlanBattery     | select [balanced \| bestPerformance \| bestPowerEfficiency] | balanced | FinderSettings | F-16 | ✗                                             |      |

## Dynamic Island ▸ Look & Behavior (9)

| ID     | Setting (UI label, English) | Design key          | Type · range / options                          | Default  | Owner section  | Cap  | Rust field today     | Note                                                             |
| ------ | --------------------------- | ------------------- | ----------------------------------------------- | -------- | -------------- | ---- | -------------------- | ---------------------------------------------------------------- |
| ILB-01 | Enable Dynamic Island       | island.enabled      | toggle                                          | on       | IslandSettings | I-12 | ✔ `notch_enabled`    |                                                                  |
| ILB-02 | Shape                       | island.shape        | select [attached \| floating capsule]           | attached | IslandSettings | I-2  | ✔ `notch_island`     |                                                                  |
| ILB-03 | Display mode                | island.displayMode  | select [Fixed \| Overlay \| Smart \| On demand] | Fixed    | IslandSettings | I-12 | ~ `notch: Behavior`  | core `notch: Behavior` + new On demand                           |
| ILB-04 | Open on hover               | island.openOnHover  | toggle                                          | on       | IslandSettings | I-12 | ✔ `notch_hover_open` |                                                                  |
| ILB-05 | Idle style                  | island.idleStyle    | select (3 styles, `notch_idle` 0–2)             | style 0  | IslandSettings | I-3  | ~ `notch_idle`       | Toothless eyes out of scope (ADR-0007); names set in ILB-05 task |
| ILB-06 | Greeting name               | island.greetingName | text                                            | empty    | IslandSettings | I-3  | ✔ `greet_name`       |                                                                  |
| ILB-07 | Greeting text               | island.greetingText | text                                            | "Hello!" | IslandSettings | I-3  | ✔ `greet_text`       |                                                                  |
| ILB-08 | Material                    | island.material     | select [Blur \| Glass \| Black]                 | Black    | IslandSettings | I-12 | ~ `notch_material`   | core `Material` names; design Opaque→Black, Translucent→Blur     |
| ILB-09 | Material rounding           | island.rounding     | range 0–100                                     | 50       | IslandSettings | I-12 | ✔ `notch_round`      | core `notch_round`                                               |

## Dynamic Island ▸ Activities (7)

| ID     | Setting (UI label, English) | Design key           | Type · range / options | Default | Owner section  | Cap  | Rust field today    | Note                         |
| ------ | --------------------------- | -------------------- | ---------------------- | ------- | -------------- | ---- | ------------------- | ---------------------------- |
| IAC-01 | Media and lyrics            | island.mediaLyrics   | toggle                 | on      | IslandSettings | I-14 | ✗                   |                              |
| IAC-02 | Compact lyrics              | island.compactLyrics | toggle                 | on      | IslandSettings | I-5  | ✔ `lyrics_compact`  |                              |
| IAC-03 | Online lyrics lookup        | island.onlineLyrics  | toggle                 | on      | IslandSettings | I-5  | ✔ `lyrics_online`   | core `lyrics_online` default |
| IAC-04 | Downloads                   | island.downloads     | toggle                 | on      | IslandSettings | I-7  | ✔ `notch_downloads` |                              |
| IAC-05 | File shelf                  | island.fileShelf     | toggle                 | on      | IslandSettings | I-14 | ✗                   |                              |
| IAC-06 | Send to phone               | island.sendToPhone   | toggle                 | on      | IslandSettings | I-9  | ✔ `notch_send`      | core `notch_send` default    |
| IAC-07 | Timer and calendar strip    | island.timerCalendar | toggle                 | on      | IslandSettings | I-14 | ✗                   |                              |

## Deferred, design-only (ADR-0007)

Not built in Rust; the HTML prototype may keep them. No capability ID.

| ID     | Setting (UI label, English)  | Design key             | Type · range / options | Default   | Owner section  | Cap | Rust field today | Note |
| ------ | ---------------------------- | ---------------------- | ---------------------- | --------- | -------------- | --- | ---------------- | ---- |
| ILB-10 | Show Toothless eye companion | island.eyeEnabled      | toggle                 | on        | IslandSettings | —   | ⊘                |      |
| ILB-11 | Eye color theme              | island.eyeTheme        | select (6 options)     | toothless | IslandSettings | —   | ⊘                |      |
| ILB-12 | Eye shape                    | island.eyeShape        | select (5 options)     | toothless | IslandSettings | —   | ⊘                |      |
| ILB-13 | Eye perception radius (px)   | island.eyeRadius       | range 120–500          | 280       | IslandSettings | —   | ⊘                |      |
| ILB-14 | Autonomous eye glances       | island.eyeAutonomous   | toggle                 | on        | IslandSettings | —   | ⊘                |      |
| ILB-15 | Autonomous glance chance (%) | island.eyeGlanceChance | range 0–100            | 78        | IslandSettings | —   | ⊘                |      |
| ILB-16 | Minimum glance interval (ms) | island.eyeMinInterval  | range 1000–20000       | 5500      | IslandSettings | —   | ⊘                |      |
| ILB-17 | Maximum glance interval (ms) | island.eyeMaxInterval  | range 2000–30000       | 11000     | IslandSettings | —   | ⊘                |      |

## Dropped (ADR-0007)

* FDI-02 — Show screen mirroring control (screen mirroring)
* Boot method option *Start as a Service* (GEN-08), Dock display option *All displays* (DLB-10), Dock display mode
  *Only display with…* (DAD-12), hot-corner action *Advanced* (DAD-11) are removed from their option lists.
