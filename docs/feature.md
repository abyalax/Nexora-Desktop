# Nexora Desktop — Feature Overview

Product-level specification of the three headline features and the shared Settings window. This file lists **what the
user can configure**; implementation status per capability lives in [`feature_checkpoint.md`](feature_checkpoint.md)
and the crate mapping in [`architecture.md`](architecture.md).

| #   | Feature (code name) | UI name  | Settings sidebar group | Capability IDs |
| --- | ------------------- | -------- | ---------------------- | -------------- |
| 1   | Dock                | Deck     | Dock                   | D-1 … D-17     |
| 2   | Menu Bar            | Menu Bar | Menu Bar               | F-1 … F-18     |
| 3   | Dynamic Island      | Visor    | Dynamic Island         | I-1 … I-14     |

Rules for this file:

- Code, docs and modules use *Dock / Menu Bar / Dynamic Island*; user-visible strings use *Deck / Menu Bar / Visor*.
- Every setting below must map to a field of `DockSettings`, `Menu BarSettings`, `IslandSettings` or a shared section
  in `core::settings`; settings are persistent configuration, never runtime state.
- A setting is added here first, then to `core`, then to the Settings UI.

---

## 1. Sidebar (left section)

### 1.1 Ungrouped pages (top of list)

- General
- Appearance
- System Icon Tray
- Screen
- Stage Manager
- Window Animation

### 1.2 Grouped pages

- Dock
  - Look & Behavior
  - Launchpad
  - Advanced
  - Minimize Effects
  - Window Previews

- Menu Bar
  - Look & Behavior
  - Notifications
  - Time & Date
  - Audio
  - Display
  - Hardware Status
  - Battery
- Dynamic Island
  - Look & Behavior
  - Activities

## 2. Page contents (right section)

### 2.1 Ungrouped pages

- General
  - Toggle
    - Enable Menu Bar
    - Enable Dock
    - Automatically Exit When Playing On Full Screen
  - Slider
    - Global Interface Scale
    - Global Interface Blur Intensity
  - Radio
    - Appearance (light, dark, system)
    - Accent Color
  - Select Option
    - Start On Boot Method
      - Don't Start on Boot
      - Registry Value
      - Scheduled Task
      - Start As a Service
    - (Toggle On/Off) Automatically Hide the taskbar on boot
      - Hide Taskbar on All Display
      - Hide Taskbar on Dock Display
    - Display Language
      - English
      - Indonesian
    - Display Font
      - Segoe UI
      - ...etc
      - (Slider) Modify The global font scaling
- Appearance
  - (Toggle On/Off) Make Dock use custom blur and transparency values
    - (Slider) Blur Intensity
    - (Slider) Transparency
  - (Toggle On/Off) Make pinned icons reflect from the Dock
    - (Slider) Reflection Blur Strength
    - (Slider) Reflection Transparency
    - (Slider) Adjust the icon reflection distance
    - (Slider) Adjust the gradient distance of the icon reflection
  - (Toggle On/Off) Make Dock icon labels use custom blur and transparency values
    - (Slider) Blur intensity
    - (Slider) Transparency
    - (Slider) Border transparency
  - (Toggle On/Off) Make context menus use custom blur and transparency values
    - (Slider) Blur intensity
    - (Slider) Transparency
    - (Slider) Border transparency
  - (Toggle On/Off) Make the folder contents grid use custom blur and transparency values
    - (Slider) Blur intensity
    - (Slider) Transparency
    - (Slider) Border transparency
    - (Slider) Folder grid icon size
  - (Toggle On/Off) Make Nexora menus use custom blur and transparency values
    - (Slider) Blur intensity
    - (Slider) Transparency
    - (Slider) Border transparency
  - (Toggle On/Off) Make Menu Bar use custom blur and transparency values
    - (Slider) Blur intensity
    - (Slider) Transparency
    - (Slider) Wallpaper tint intensity
  - (Toggle On/Off) Launchpad uses custom background blur
  - (Toggle On/Off) Display Launchpad above windows instead of hiding them
  - (Toggle On/Off) Launchpad blurs the background instead of pre-rendering it
    - (Slider) Blur intensity
- System Icon Tray
  - (Toggle On/Off) Always display all tray icons in Menu Bar
  - (Toggle On/Off) Always display all tray icons in Dock
  - (Toggle On/Off) Displays the system's own tray icon in Menu Bar
  - (Slider) The newly run tray icon shows the time
- Stage Manager
  - Check Input
    - Show Stage Manager icon in Menu Bar
    - Show window title on mouseover
    - Blur background window icons
    - Click on the desktop to hide all windows
    - Automatically pause and play background window media
    - Show native window preview on hover
  - Select Option
    - Show Desktop Icons
      - Always Hide Icon
      - Auto Show Hide
      - Always Show
    - Adjust Animation Speed
      - 0.5x
      - 0.75x
      - 1.0x
      - 1.25x
      - 1.5x
  - (Toggle) Open Stage Manager
  - The screen position where the window list is located (Select, left or right)
  - (Select Option) Window List Show Hidden Mode
    - Default
    - Always show
    - Always hide
  - (Toggle On/Off) Keep Window Border
  - Slider
    - Window list icon size
    - Window list icon spacing
    - Number of window list icons
- Window Animation
  - (Toggle) Window opening animation
  - (Radio) Window opening animation effect
  - Adjust animation speed
  - Add excluded programs
  - Manage running program blacklist
  - Remove windows for excluded programs

### 2.2 Dock

- Look and Behavior
  - (Slider) Icon size
  - (Select Option) Icon effect 
    - None
    - Scaling
    - Select
    - Illumination
    - Scaling and light
    - Select and light
  - (Slider) Icon magnification
  - (Slider) Icon light intensity
  - (Slider) Icon background light intensity
  - (Slider, R,G,B) Icon background light color
  - (Slider) Distance between icons
  - (Slider) Dock distance from screen edge
  - (Select Option) Position on screen
    - Left
    - Bottom
    - Right
    - Top
  - Select the screen to display Dock (list detected display)
- Launchpad
  - (Slider) Launchpad icon size
  - (Slider) Icon horizontal spacing
  - (Slider) Icon vertical spacing
  - (Select Option) Choose which icons will be displayed by Launchpad
    - Added icons
    - Added icons&Start Menu
    - Added icons&Desktop
    - Added icons&Start Menu&Desktop
  - Edit the deleted Start Menu program
  - Select the screen to display Launchpad
  - Follow Dock
- Advanced
  - (Toggle On/Off) Scale all icons to macOS standards
  - (Toggle On/Off) Double-click the folder icon to open in Menu Bar
  - (Toggle On/Off) Hide the open folder button
  - (Toggle On/Off) Lock icon dragging
  - (Toggle On/Off) Show icon labels
  - (Toggle On/Off) Animate opening applications
  - (Toggle On/Off) Turn on window borders to prevent windows from hiding Dock and Menu Bar
  - (Toggle On/Off) Show indicators under running programs
  - (Toggle On/Off) Show icons of unpinned running programs
  - (Toggle On/Off) Separate unpinned running programs
  - (Select Option) Bottom left hot-corner function
    - Do nothing
    - Show all windows
    - Open Launchpad
    - Put display to sleep
    - Open Start Menu
    - Show Desktop
    - Open Mission Control
    - Lock Screen
    - Windows widget
    - Advanced
  - (Select Option) Dock display mode
    - Default
    - Overlay
    - Auto-hide
    - Smart auto-hide
    - Always show
    - Only display with…
    - Always force visibility
    - Display only on the desktop
  - Select how the Dock shows and hides the detection window
    - All Program Windows
    - The Current Program Window
  - Select the Dock to show the hidden effect
    - Move
    - Fade Away
  - The Dock shows how it was activated
    - The edge of the screen
    - Activation bar
    - The edge of the screen&Activation bar
  - (Slider) Activation bar width
  - (Slider) Activation bar height
  - (Slider) Activation bar distance from screen edge
  - (Slider) Mouse delay activates dock time
- Minimize Effects
  - (Toggle On/Off) Use Nexora minimize effect
  - (Toggle On/Off) Don't animate minimizing windows
  - (Toggle On/Off) Hold down 'Shift' to slow down an animation
  - (Toggle On/Off) Minimize windows into their Dock icon
  - (Toggle On/Off) Dynamically display window content when minimized
  - Minimize window icon position
    - Next to the icon on the right
    - Independently displayed on the right
    - The right side is displayed independently using a separator
  - Minimize animation
    - Genie
    - Scale
    - Suck
  - Minimize animation speed
  - Minimize animation blacklist
- Window Previews
  - (Toggle On/Off) Enable window previews
  - (Slider) Window preview size
  - (Slider) Preview delay on hover
  - Show preview elements
    - (Toggle On/Off) Application name
    - (Toggle On/Off) Window control buttons
    - (Toggle On/Off) Show native window preview on hover

### 2.3 Menu Bar

- Look and Behavior
  - Menu Bar menu icon
  - Auto-Hide Menu Bar
  - Shut down confirmation box
  - Maximized windows affect Menu Bar appearance
  - Click the upper right corner to close the current maximized window
  - Lock dragging tray icons
  - Show window options in Menu Bar
  - Show colored tray icons
  - Show input method menu
  - Show search icon
  - The account icon is displayed
  - Displays the media playback icon
  - Show music player lyrics menu
  - Show Color Player Lyrics
- Battery
  - Show Battery Status menu in Menu Bar
  - Show Battery percentage in Menu Bar status menu
  - Displays the remaining battery time
  - Automatically switch power plans
  - Power plans when using power
  - Power plans when using batteries

### 2.4 Dynamic Island

- Look & Behavior
  - (Toggle) Enable Dynamic Island
  - (Select Option) Shape
    - Attached to the top edge
    - Floating capsule
  - (Select Option) Display mode
    - Fixed
    - Smart
    - On demand
  - (Toggle On/Off) Open on hover
  - (Select Option) Idle style
  - Greeting name and text
  - Material and rounding
- Activities
  - (Toggle On/Off) Media and lyrics (compact lyrics, online lyrics lookup)
  - (Toggle On/Off) Downloads
  - (Toggle On/Off) File shelf
  - (Toggle On/Off) Send to phone
  - (Toggle On/Off) Timer and calendar strip
