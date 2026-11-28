# Appearance: per-surface glass (D-15.1)

## Purpose
Check that each "custom blur and transparency" group on the Appearance page changes only its own surface, and that
turning it off returns the surface to the global Blur and Transparency sliders.

## Prerequisites
- Windows 11; build from this branch; `settings.cfg` at defaults (every `custom_*` switch off).
- Appearance page open. Dock material = Crystal so the glass is easy to read.

## Test Steps
1. Move the global **Tingkat blur** and **Transparansi** sliders. Confirm the Dock, tooltips, context menus and menu bar
   menus all follow them.
2. For each surface below, turn its custom switch on, set a clearly different blur and transparency, and look only at
   that surface. Then turn the switch off again.
   - Dock (`custom_dock_blur`): the Dock panel.
   - Icon labels (`custom_icon_labels`): the Dock icon tooltip.
   - Context menus (`custom_context_menus`): the right-click menu of a Dock icon.
   - Nexora menus (`custom_nexora_menus`): the net/card menus opened from the menu bar.
   - Finder (`custom_finder_blur`): the menu bar background.
3. Leave the menu open while changing a slider; reopen it and confirm the new values apply on the next show.
4. Create a new tooltip after changing the label values (hover an icon that has not shown one yet).

## Expected Behavior
- A surface with its switch off follows the global sliders; moving its own sliders does nothing.
- A surface with its switch on uses its own blur (0-50) and transparency (0-100, 100 is fully see-through).
- No other surface changes when one surface's values change.
- Folder grid, Launchpad, reflections and wallpaper tint have no effect yet (no renderer): record this, do not file it.

## Actual Behavior
Not run on Windows 11. The core rule (`Extra::custom_glass`) is covered by unit tests; the app wiring was type-checked
for `x86_64-pc-windows-msvc` only.

## Recovery
Turn every custom switch off and confirm all surfaces match the global glass again after restart.
