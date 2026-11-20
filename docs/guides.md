# nexora User Guide

nexora transforms the Windows 11 taskbar into a modern desktop experience featuring **Deck**, **Visor (Capsule)**, **MenuBar**, **Quick Search**, **Apps**, **Overview**, **Scene**, **Quick Panel**, and **Screen Corners** — all with a lightweight Crystal-inspired design.

> This guide is available in English. For the project overview, build instructions, and development information, see the [README](../README.md).

## Table of Contents

1. [Installation and Getting Started](#1-installation-and-getting-started)
2. [Deck](#2-deck)
3. [Visor](#3-visor)
4. [MenuBar](#4-menubar)
5. [Quick Panel](#5-quick-panel)
6. [Quick Search](#6-quick-search)
7. [Apps](#7-apps)
8. [Overview](#8-overview)
9. [Scene](#9-scene)
10. [Screen Corners](#10-screen-corners)
11. [Settings](#11-settings)
12. [Keyboard Shortcuts](#12-keyboard-shortcuts)
13. [Privacy and Security](#13-privacy-and-security)
14. [Troubleshooting](#14-troubleshooting)
15. [Known Limitations](#15-known-limitations)
16. [Uninstallation](#16-uninstallation)

---

## 1. Installation and Getting Started

**Requirements:** Windows 11 (build 22000 or later), x64.

**Option A — Installer**

If available on the Releases page, download and run `nexora-Setup-<version>.exe`, then follow the installation instructions.

Administrator privileges are not required. nexora is installed for the current Windows user only.

**Option B — Portable ZIP**

Download `nexora-<version>-win-x64.zip`, extract it to any folder, and run `nexora.exe`.

**Keep `nexora.exe` and `nexora-guard.exe` in the same folder.**

> nexora binaries are currently unsigned. Windows SmartScreen may display the "Windows protected your PC" warning.
>
> Select **More info → Run anyway** only if you downloaded the files from the project's official release page.

Once `nexora.exe` starts, the native Windows taskbar is hidden and Deck appears at the bottom of the screen.

### Basic Operations

- **Exit nexora:** Right-click an icon on Deck or the nexora tray icon, then select **Exit nexora**. The Windows taskbar is automatically restored.
- **Unexpected shutdown:** `nexora-guard.exe` automatically restores the Windows taskbar and any windows parked by Scene. If recovery does not occur, launch `nexora.exe` again and exit normally.
- **Open Settings:** Right-click Deck and select **nexora Settings...**. You can also use the Settings gear icon on Deck or the circular nexora logo on MenuBar.
- **Launch at startup:** Go to Settings → General → **Launch at Windows startup**.

Only one nexora instance can run at a time. Launching it again simply brings the existing instance back to the foreground.

nexora does not permanently disable the Windows taskbar. It only hides it while nexora is running. This behavior can be changed under Settings → General → **Hide Windows taskbar**.

## 2. Deck

Deck is the primary application launcher and taskbar replacement.

### Application Management

- **Launch or activate an app:** Click its icon. Clicking the icon of an already active foreground app minimizes its window.
- **Add an app:** Drag an executable or shortcut (`.exe`, `.lnk`) onto Deck. Alternatively, right-click Deck and select **Add App to Deck...**, or right-click a running app and select **Keep in Deck**. Microsoft Store apps, including WhatsApp, are supported.
- **Reorder apps:** Drag pinned icons to their desired positions.
- **Remove an app:** Right-click its icon and select **Remove from Deck**, or drag the icon outside Deck.
- **Change an icon:** Right-click an icon and select **Change Icon...**. Supported formats include PNG, JPG, ICO, BMP, and GIF. Select **Restore Original Icon** to revert the change. This also applies to the Start button at the left end of Deck.

### App Grid Button

The colored dot-grid icon immediately to the right of the Start button opens Apps.

Right-click it and select **Change Icon...** to customize its appearance. You can also use the command-line option:

`--set-icon launchpad <image>`

### Icon Appearance

- **Uniform icons:** Icons are displayed inside consistently sized squircle containers with subtle shadows. Irregularly shaped icons are placed on a white background plate.
- Disable this option under Settings → Deck if you prefer the original Windows icons.
- **Window previews:** Hover over an icon belonging to a running app to display its window previews. Click a preview to activate that window.
- **Icon zoom:** Icons enlarge as the pointer moves over them. Icon size, spacing, zoom level, and screen-edge distance are configurable under the **Size** settings.

### Optional Icon Packs

You can provide your own icon pack by placing a folder containing PNG icons in:

`%APPDATA%\nexora\iconpack\`

nexora automatically matches supported Windows applications with recognizable icon filenames. Examples include:

| Windows Application         | Icon Filename      |
| --------------------------- | ------------------ |
| File Explorer               | `Finder`           |
| Calculator                  | `Calculator`       |
| Notepad                     | `TextEdit`         |
| Task Manager                | `Activity Monitor` |
| Terminal / CMD / PowerShell | `Terminal`         |
| Settings                    | `System Settings`  |
| Microsoft Store             | `App Store`        |
| Apps launcher               | `Apps`             |

Other supported system applications include Mail, Calendar, Maps, Photos, and Weather.

Browsers and third-party applications are never automatically replaced.

Icons supplied through an icon pack are displayed as-is, without additional background plates or visual effects. This applies to Deck, Quick Search, and Apps.

Manually assigned icons always take priority over icon pack icons.

To disable icon packs, go to Settings → Appearance → **Icon Pack**.

**nexora does not include or distribute any icon packs.** Users are responsible for supplying their own icons and ensuring compliance with the applicable licenses.

### Additional Deck Features

- **Downloads Stack:** Located immediately before the Recycle Bin. Click it to view and open the 10 most recently modified files in your Downloads folder, or select **Open Downloads Folder**. Files that are still downloading are excluded. This feature can be disabled under Settings → Deck.
- **Recently Used Apps:** The three most recently closed apps that are not pinned remain available at the end of the app list. Their history is stored in `recents.txt`. This feature can be disabled under Settings → Deck.
- **Tooltips:** Hovering over an icon displays its name in a small tooltip with a pointer.
- **Recycle Bin:** Located at the far right of Deck. Its icon changes depending on whether the Recycle Bin is empty or contains files. The status is refreshed every few seconds. Right-click to open or empty it; Windows still displays its usual confirmation dialog. If your icon pack provides a Recycle Bin icon, that icon is used.

### Deck Behavior and Animations

Deck supports several display modes, including **Fixed**, **Always on Top**, **Auto-hide**, and **Touch Edge**. Configure these under Settings → Deck.

Surface materials (**Blur**, **Crystal**, or **Opaque Black**) and the global light/dark theme are managed under Settings → Appearance.

**Minimize animation styles:**

- **Funnel** (default): Windows appear to flow toward their corresponding Deck icons when minimized and return from those icons when restored.
- **Scale:** Windows shrink directly toward their icons using a shorter animation.

Window minimize, open, and close animations can be disabled under Settings → General. Active window shadows can be configured under Settings → Windows.

### Icon Styles

Go to Settings → Appearance → **Icon Style** to customize the appearance of Deck icons, the Apps button, and the Apps grid.

Available styles:

- **Dark:** Darkens icon backgrounds.
- **Glass:** Uses a translucent gray glass appearance.
- **Colored:** Applies a single color tint to icons.

Available tint colors include Blue, Purple, Pink, Orange, and Green.

Manually customized icons are not affected by these styles.

## 3. Visor

Visor is a compact capsule positioned near the top-center of the screen, with a subtle shadow beneath it.

Its material can be configured under Settings → Appearance → **Visor Material**:

- Blur
- Crystal
- Black

Text is always displayed in white.

### Idle State

When idle, Visor periodically rotates through greeting and system information displays approximately every nine seconds.

Available information includes:

- Greeting text (for example, "Good evening, Rani").
- Current time.
- Current date.
- Battery status, with an icon on the left and the value on the right.

Under Settings → Visor → **When Visor Is Idle** → **Display**, choose one of the following:

- **Greeting & Info**
- **Music & Lyrics**
- **Empty**

You can also configure your **Display Name** and **Greeting Text** in this section.

### Music and Lyrics

When a song is playing or paused, the idle Visor expands slightly and moves downward.

It displays:

- A compact album cover with the playback application's icon in its corner (Spotify, browser, Media Player, and others).
- Previous, Play/Pause, and Next controls beneath the album artwork.
- Synchronized lyrics on the right, using the same visual style as the Play tab.

The active lyric line is highlighted in bold, while surrounding lines are dimmed and blurred. Lyrics scroll upward as playback progresses.

Playback controls are directly clickable. Hovering over these controls does not open the expanded panel. Hovering over or clicking other areas opens the panel normally.

If synchronized lyrics are unavailable, Visor displays the track title and artist instead.

### Activity Priority

Ongoing activities replace the idle greeting according to the following priority order:

1. Files being dragged onto Visor.
2. Timers with less than one minute remaining.
3. Active downloads.
4. Lyrics.
5. Music playback (album artwork and artwork-colored visualizer).
6. Battery charging activity.
7. Running timers.
8. Number of files in the Shelf.

Small privacy indicators also appear at the right edge:

- **Orange dot:** An application is using the microphone.
- **Green dot:** An application is using the camera.

### Opening the Visor Panel

Hover over Visor for approximately 0.3 seconds to open its panel.

You can also click it if **Open Automatically on Hover** is disabled.

The panel automatically closes shortly after the pointer leaves it.

The panel contains three tabs. Click a tab or scroll with the mouse wheel to switch between them.

#### Play

Displays a music card containing:

- Album artwork.
- Track title and artist.
- A draggable playback progress bar.
- Previous, Play/Pause, and Next controls.

The panel also displays large synchronized lyrics. The active line is highlighted, while preceding and following lines appear dimmed and slightly blurred. Lyrics automatically scroll as the track progresses.

#### Shelf

Displays files dropped onto Visor.

- Click a file to open it.
- Right-click a file to access **Show in Folder**, **Copy Path**, or **Remove from Shelf**.
- Select **Clear All** to remove every item.

The Shelf stores **file paths only**. It does not copy file contents, and its contents are cleared when nexora exits.

#### Timer

The Timer tab contains two sections.

**Left — Timer**

- −5 minutes
- Start/Pause
- Reset
- +5 minutes

**Right — Calendar**

Displays a horizontal calendar containing the current month and five dates surrounding the selected date.

Today's date is highlighted using the accent color. Events scheduled for the selected date are retrieved from Windows Calendar.

If no events are available, the calendar displays **No events today**.

Click a date to view its events. The calendar automatically returns to today's date when the panel closes.

The panel also includes a battery indicator and a shortcut to nexora Settings in its upper-right corner.

### Drag-and-Drop Actions

When dragging files from File Explorer onto Visor, four drop zones become available:

- **Add to Shelf:** Stores the file path in the Shelf.
- **Send:** Transfers the file to a mobile device (see below).
- **Copy to...:** Selects a destination folder and copies the file using the native Windows copy dialog.
- **Open With...:** Opens the Windows **Open With** dialog for the first selected file.

### Send to Phone

Drop a file onto the **Send** zone, or right-click a file in the Shelf and select **Send to Phone**.

The Shelf tab displays a QR code.

Scan the QR code using an Android or iPhone camera. No additional mobile application is required. Open the displayed link and tap **Download**.

Transfer progress appears in the file card and in the collapsed Visor, represented by a green progress arc with an upward arrow.

Once the transfer completes, Visor displays **Sent ✓**.

**Requirements and behavior:**

- Your laptop and phone must be connected to the **same Wi-Fi network or local network**.
- Windows Firewall may request permission the first time this feature is used. Allow nexora on **Private networks** so your phone can access the transfer session.
- A transfer session ends when **Done** is clicked, after 15 minutes of inactivity, or when nexora exits.
- This feature can be disabled under Settings → Visor → Activity → **Send to Phone via QR (Same Wi-Fi)**.

Visor display behavior, including **Fixed**, **Always on Top**, **Auto-hide**, and **When Covered**, can be configured under Settings → Visor.

Visor can also be disabled using **Enable Visor**.

### Downloads in Visor

When a download is active, the collapsed Visor expands slightly.

It displays:

- The downloading application's icon on the left (Microsoft Store, Steam, or a browser such as Brave).
- A green progress arc on the right, filling clockwise from approximately the 8 o'clock position to the 4 o'clock position.
- A download arrow inside the arc.
- The download percentage or downloaded size beneath it.

Progress reporting depends on the source:

- **Microsoft Store and Steam:** Provide actual download percentages.
- **Browsers:** Do not expose the total download size. The progress arc therefore rotates continuously, while the number displays the downloaded file size (for example, `26M`).

nexora detects temporary `.crdownload` and `.part` files in the Downloads folder.

When a download completes, a full progress arc with a checkmark appears for approximately three seconds.

To disable this feature, go to Settings → Visor → Activity → **Show Downloads (Store, Steam, Browsers)**.

### Lyrics in Visor

When music is playing and synchronized lyrics are available, the collapsed Visor displays the **currently active lyric line** between the album artwork on the left and the visualizer on the right.

The displayed line updates according to the playback position.

The expanded **Play** tab displays the larger synchronized lyrics described earlier.

Lyrics are retrieved in the following order:

1. **Local `.lrc` files:** Place your own lyric files in `%APPDATA%\nexora\lyrics`. Supported filenames are `Artist - Title.lrc` and `Title.lrc`. Filename matching is case-insensitive.

   Open this folder through Settings → Visor → Lyrics → **Lyrics Folder (.lrc)**.

2. **Local cache:** Previously retrieved lyrics are reused from the local cache.

3. **LRCLIB online:** If enabled (on by default), nexora searches [LRCLIB](https://lrclib.net) for missing lyrics. Tracks that cannot be found are retried after three days.

Lyrics without timestamps are displayed only in the Lyrics card.

To hide lyrics from the collapsed Visor, disable **Show Lyrics in Visor**.

## 4. Menubar

Enable Menubar under Settings → General → **Enable Menubar**.

Menubar provides a menu bar across the top of the screen.

### Left Side

The left side contains the following elements.

**nexora circular logo**

Provides shortcuts to:

- About This PC
- System Settings
- Microsoft Store
- nexora Settings
- Apps
- Overview
- Force Quit (opens Task Manager)
- Sleep
- Restart
- Shut Down
- Lock Screen

Power actions require confirmation before execution.

**Active Application Name**

Displays the currently active application's name in bold.

Click it to access:

- About
- Hide
- Hide Others
- Show All
- Quit

**Application Menus**

When the active application exposes a classic Win32 menu (File, Edit, and others), Menubar displays the original menu and allows direct interaction.

Applications without classic menus, including browsers, Electron applications, and the modern Notepad, use a standard menu containing:

- File
- Edit
- View
- Window
- Help

These menus send the corresponding keyboard shortcuts to the active application.

Once a menu is open, move the pointer over another menu title to switch menus without clicking.

### Right Side

From right to left, Menubar displays:

- Clock — clicking it opens Windows Notification Center.
- Quick Panel.
- Quick Search.
- Wi-Fi.
- Battery.
- Scene indicator (when Scene is enabled).

Clicking Wi-Fi or Battery opens the corresponding Windows settings interface.

### Wi-Fi Menu

Click the Wi-Fi icon to open the network menu.

It includes:

- Wi-Fi toggle.
- **Known Networks:** Displays saved networks, with a blue circle indicating the currently connected network. Signal strength and lock indicators are also shown.
- **Other Networks:** Displays available networks.
- **Wi-Fi Settings...**

Click a known or open network to connect immediately.

For a newly discovered password-protected network, Windows opens its network selection interface so you can enter the password.

> Since Windows 11 24H2, Windows only provides nearby network information to desktop applications when **Location** is enabled.
>
> Go to Windows Settings → Privacy & security → Location and enable **Let desktop apps access your location**.
>
> If Location is disabled, the Wi-Fi menu operates in a limited mode. It still displays the connection status and provides **Enable Location...** and **Other Networks...** shortcuts to the native Windows network interface.
>
> nexora does not read or store your location.

### Menubar Appearance

Configure Menubar's background under Settings → Menubar.

Available styles:

- **Transparent** (default): No background.
- **Light:** Solid white background.
- **Dark:** Solid black background.
- **Material:** Uses the global surface material, such as Blur, with a translucent background. Text colors follow the selected theme.

Text colors automatically adapt to the wallpaper or background color.

The global light/dark theme does not affect Menubar.

### Active Application Tracking

The application name displayed on the left follows the last window clicked with the mouse, even when that window does not become the foreground window.

It also updates when focus changes through keyboard navigation or Scene.

### Always-on-Top Behavior

By default, Deck, Menubar, and Visor remain **above all windows, including fullscreen applications**.

If you prefer them to disappear when a fullscreen application is active (videos, games, or F11 fullscreen mode), disable:

Settings → General → **Always on Top of Fullscreen Windows**.

## 5. Quick Panel

Click the two-toggle icon on Menubar to open Quick Panel.

Quick Panel provides floating glass-style controls for common system functions.

Available modules include:

- Wi-Fi (with the connected network name).
- Bluetooth.
- Nearby Sharing (AirDrop-like functionality).
- Playing.
- Scene.
- Overview.
- Dark Mode.
- Screenshot.
- Focus.
- Volume slider.
- Display brightness slider.

Click a module to toggle its corresponding feature. Drag sliders to adjust their values.

### Wi-Fi and Bluetooth Controls

For the Wi-Fi and Bluetooth modules:

- Click the circular icon to toggle the feature on or off.
- Click the remaining module area, or right-click a compact circular module, to open the network or device selector.

Quick Panel closes before the corresponding Wi-Fi or Bluetooth selector appears in the upper-right corner.

The selector uses the same animation style as Quick Panel and has dimensions comparable to the Volume and Brightness slider cards.

The Bluetooth menu displays previously paired devices. A blue circle indicates a connected device.

Click a **headphone or speaker** to connect or disconnect it directly. Status labels such as **Connecting...** and **Disconnecting...** remain visible until the operation completes. The menu stays open during this process.

Other Bluetooth devices, including keyboards, mice, and phones, open Windows Bluetooth Settings because Windows only provides direct connection support for audio devices.

### Slider Interaction

Slider handles are displayed as white capsules.

While dragging, the handle changes into translucent glass and slightly enlarges, creating a water-droplet effect.

This matches the slider interaction used in Visor.

The circular button on the right side of the Volume module opens the native Windows audio output selector, allowing you to switch between speakers and headphones.

This is equivalent to the `Win+Ctrl+V` shortcut.

### Microphone and Camera Indicators

When an application is using the microphone or camera, a small application-name chip appears above the corresponding Quick Panel modules.

- **Orange dot:** Microphone in use.
- **Green dot:** Camera in use.

Click the chip to open the relevant Windows microphone or camera privacy settings.

nexora only reads Windows usage indicators. It does not access the microphone or camera itself.

### Customizing Quick Panel

Select **Edit Controls...** to enter edit mode and open the **Find Controls** gallery.

Available actions include:

- Click the **−** badge to remove a module.
- Drag a module onto another module to swap their positions.
- Click **Done** to save the layout.
- Browse controls by category, such as Connectivity, Display & Brightness, and Sound.
- Use the search field to find specific controls.
- Click a control to add it to Quick Panel.

Right-click a module while in edit mode to change its size:

- **Small:** Circular.
- **Medium:** Capsule-shaped.
- **Large:** 2×2 layout.

Slider and Playing modules have fixed sizes.

Focus, Hotspot, Nearby Sharing, and Airplane Mode open their corresponding Windows Settings pages.

### Volume and Brightness HUD

When you press a volume or brightness key, a glass-style HUD appears in the upper-right corner, beneath MenuBar.

It displays:

- The current output device name (for example, "Headphones") or "Display" for brightness.
- A thin slider with icons at both ends.
- A scale consisting of 16 markers.

The slider can be dragged directly. The HUD automatically disappears after a short period.

## 6. Quick Search

Open Quick Search using `Ctrl+Space` or the magnifying glass icon on MenuBar.

Quick Search initially appears as a compact, fading capsule labeled "Quick Search".

Hovering over it reveals four circular mode buttons on the right:

- Apps
- Files
- Actions
- Clipboard

Hovering over a button displays its mode name and keyboard shortcut inside the capsule.

Selecting a mode expands the capsule, absorbs the mode buttons, and opens the corresponding results panel with a downward animation.

Typing immediately transforms the capsule into a search results panel.

### Searching

Quick Search supports:

- Application names (including fuzzy matching and initials).
- Mathematical expressions, such as `12*(3+4)`.
- File and folder names.
- Action names.

Results are grouped in the following order:

1. Calculator results.
2. Applications.
3. Actions.
4. Files.
5. Search the Web.

Filter chips below the search field allow you to narrow results by **Apps**, **My Files**, or **Actions**.

**Keyboard navigation:**

- `↑` / `↓`: Select a result.
- `Enter`: Open the selected result.
- `Esc`: Clear the search input, then close Quick Search.

The **Search the Web** result opens Google in your browser only when explicitly selected.

nexora does not independently transmit search queries over the network.

### Search Modes

Switch between modes using the circular buttons or the following shortcuts. These shortcuts work anywhere inside Quick Search.

| Shortcut | Mode          | Description                                                                                                                                                                                                                                                                                                                                                                  |
| -------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Ctrl+1` | **Apps**      | Displays a Suggestions section containing Deck apps, followed by application categories. Each category includes a **View All** row. Start typing to search for applications (see [Apps](#7-apps)).                                                                                                                                                                           |
| `Ctrl+2` | **My Files**  | Without a search query, displays **Recent** files opened in Windows, including thumbnails. Filter by Images, Videos, Documents, or Music. With a query, searches file and folder names in Desktop, Documents, Downloads, Pictures, Videos, Music, and OneDrive. Search depth is limited to five directory levels, with up to 20 top results. This is not a full-disk search. |
| `Ctrl+3` | **Actions**   | Provides quick commands such as Create Note, Start Timer, Send Email, Take Screenshot, Lock Screen, Show Desktop, Toggle Dark Mode, Scene, Overview, Calculator, Task Manager, Settings, and Clear Clipboard History.                                                                                                                                                        |
| `Ctrl+4` | **Clipboard** | Displays text copied within the last eight hours, with entries such as `Text · Copied 15:20`. Press `Enter` or click the circular copy button to copy an entry again. Use **⋯ → Clear History** to remove the history.                                                                                                                                                       |

### Action Quick Keys

Each action can be assigned a short custom key.

In Actions mode, type `nn` and press `Enter` to execute Create Note.

To customize a quick key:

1. Select an action.
2. Press `Tab` to edit its quick key.
3. Enter up to six letters or numbers.
4. Press `Enter` to save or `Esc` to cancel.

Quick keys are stored in `actions.txt` inside the configuration directory.

### Clipboard History

Clipboard History can be disabled under Settings → General → **Clipboard History in Quick Search**.

Clipboard history is stored in memory only. It is cleared when nexora exits and is never written to disk or transmitted elsewhere.

Content marked as password-manager data is ignored.

Only text content is supported.

## 7. Apps

Apps provides a centralized application launcher.

You can open it through:

- The **Apps** button on Deck (colored dot-grid icon).
- `Ctrl+Shift+Space`.
- `Ctrl+1` in Quick Search.
- The Apps option in the nexora logo menu.
- A configured Screen Corner action.

Apps appears as a glass-style panel near the top-center of the screen.

It includes an **Applications** title, category filters, and a five-column application grid with uniform icons.

Available categories:

- All
- Productivity
- Creativity
- Entertainment
- Utilities
- System

### Navigation

- **Search:** Start typing to filter the application grid.
- **Navigate:** Use the arrow keys (Up, Down, Left, Right) or move the mouse over an application. Scroll with the mouse wheel to move between rows.
- **Launch:** Press `Enter` or click an application.
- **Close:** Press `Esc` or click outside the panel.

Pressing `Esc` first clears the search query, then returns to Quick Search if applicable, and finally closes the interface.

## 8. Overview

Overview provides a visual overview of all open windows.

Open it using:

- `Ctrl+↑`.
- The nexora logo menu.
- Quick Panel.
- A configured Screen Corner action.

All open windows are arranged in a grid of live thumbnails.

Window titles appear when hovering over a thumbnail.

A bar at the top displays available Virtual Desktops. Click a desktop to switch to it, or select **+** to create a new desktop.

Click a window thumbnail to activate that window.

Press `Esc` or click an empty area to exit Overview.

Minimized windows are not included.

## 9. Scene

Enable Scene under Settings → **Stage** → **Enable Scene**.

You can also activate it through Quick Panel or a configured Screen Corner action.

Scene organizes your desktop into application workspaces.

The currently active application remains on the **stage**. Other application windows are **parked off-screen rather than minimized** and appear as angled cards in a strip along the side of the display.

### Live Window Cards

Scene cards display continuously updated captures of their original windows, refreshing approximately 20 times per second.

This means content such as YouTube videos in Brave or Chrome, animations, and clocks can continue running inside the cards.

### Interacting with Scene

- **Click a card:** Switches the active application group. The selected window animates from its card back to its original screen position.
- **Shift-click a card:** Adds the application to the current stage group, allowing multiple applications to share one stage.
- **Reveal the card strip:** The strip automatically hides when a stage window overlaps it. Move the pointer to the screen edge on the strip's side to reveal it temporarily.
- **Click the wallpaper:** Parks all windows in the strip and reveals the desktop.
- **Alt+Tab:** Replaces the native Windows switcher with the nexora application switcher, displaying window cards with a blue selection highlight.

Use `Tab`, `Shift+Tab`, or the arrow keys to navigate the switcher. Press `Esc` to cancel or release `Alt` to activate the selected window.

This behavior can be disabled under Settings → Stage → **Replace Alt+Tab with nexora Switcher**.

### Recovery Behavior

When Scene is disabled or nexora exits, all parked windows are restored to their original positions.

If nexora crashes unexpectedly, `nexora-guard` attempts to restore parked windows. They are also restored when nexora is launched again.

### Scene Settings

Scene settings are available under the **Stage** tab.

Configurable options include:

- **Window Strip Position:** Left or right side of the screen.
- **Card Size:** Controls the dimensions of window cards.
- **Card Spacing:** Controls the distance between cards.
- **Maximum Card Count:** Windows exceeding the configured limit are returned to the screen.
- **Animation Speed:** Adjustable from 50% to 200%.
- **Show Window Titles on Hover:** Displays application names above highlighted cards.
- **Click Wallpaper to Park All Windows:** Can be disabled.
- **Pause Background App Media:** Pauses media playback for applications removed from the stage and resumes playback when they return.

  This feature only works with applications that register Windows media sessions.

- **Desktop Icons:** Choose between Hidden, Automatic (shown when the stage is empty), or Visible.
- **Excluded Applications:** Specify executable names or path fragments, one per line, in lowercase, inside `stage_exclude.txt` in the nexora configuration directory.

  Excluded applications are never parked by Scene.

> Applications running with Administrator privileges cannot be managed by a normal nexora process. Scene leaves these windows unchanged.

## 10. Screen Corners

Configure Screen Corners under Settings → Windows → **Screen Corners**.

Click each corner's button to cycle through the available actions:

- None
- Overview
- Apps
- Show Desktop
- Scene
- Notification Center
- Lock Screen

An action is triggered when the pointer remains in a configured corner for approximately 0.25 seconds.

Corner actions are ignored while a mouse button is pressed, preventing accidental activation when dragging windows.

## 11. Settings

nexora Settings uses a modern settings interface with a colored icon sidebar on the left and grouped configuration cards on the right.

Long pages can be navigated using the mouse wheel.

The red button in the upper-left corner closes the Settings window. The window title area can be dragged to reposition it.

### Settings Navigation

The sidebar includes:

- A **Search** field for filtering settings pages.
- A Windows account information card.
- Navigation controls.

Press `Enter` in the search field to open the first matching page. Press `Esc` to clear the search.

The **◀ / ▶** buttons in the title bar navigate backward and forward through previously visited settings pages.

### Settings Pages

| Page               | Available Options                                                                                                                                                                                                                               |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **General**        | Language (Indonesian/English), launch at startup, hide Windows taskbar, hide desktop icons, always on top of fullscreen windows, window animations, active window shadows.                                                                      |
| **Deck**           | Enable Deck, display behavior, uniform icons, icon size, spacing, zoom, screen-edge distance, corner radius.                                                                                                                                    |
| **Visor**          | Enable Visor, display behavior, corner radius, automatically open when hovering over Visor.                                                                                                                                                     |
| **MenuBar**        | Enable MenuBar, background style (Transparent, Light, Dark, Material), 24-hour clock.                                                                                                                                                           |
| **Appearance**     | Automatic, Light, and Dark previews; theme and accent colors; icon and widget styles; light/dark/automatic appearance; tint colors; icon packs; global glass material; Visor material; blur intensity; transparency; and glass-edge refraction. |
| **Scene**          | All Scene configuration options.                                                                                                                                                                                                                |
| **Screen Corners** | Actions assigned to the four screen corners.                                                                                                                                                                                                    |
| **Quick Search**   | Clipboard History configuration.                                                                                                                                                                                                                |

### Appearance Settings

The Appearance page includes preview cards for **Automatic**, **Light**, and **Dark** modes.

Theme customization includes:

- Accent color.
- Multicolor mode, which follows the Windows accent color.
- Eight additional accent color options.
- Icon and widget styles: Default, Dark, Glass, and Colored.
- Light, Dark, and Automatic appearance modes.
- Custom tint colors.
- Icon pack selection.

Glass customization includes:

- Global surface material (Blur, Crystal, or Opaque Black).
- Visor material.
- Blur intensity.
- Transparency.
- Glass-edge refraction.

All settings changes take effect immediately.

### Configuration Directory

nexora stores its configuration files in:

`%APPDATA%\nexora\`

| File                     | Description                                                       |
| ------------------------ | ----------------------------------------------------------------- |
| `settings.cfg`           | Application settings stored as `key=value` entries.               |
| `pins.txt`               | Applications pinned to Deck.                                      |
| `icons.txt` and `icons\` | Custom icon configuration and icon files.                         |
| `cc_layout.txt`          | Quick Panel module layout.                                        |
| `stage_exclude.txt`      | Applications excluded from Scene.                                 |
| `nexora.log`             | Application log file, useful for troubleshooting and bug reports. |

### Command-Line Options

The following command-line options are available:

| Option                            | Description                                      |
| --------------------------------- | ------------------------------------------------ |
| `--keep-taskbar`                  | Prevents nexora from hiding the Windows taskbar. |
| `--open-settings`                 | Opens nexora Settings.                           |
| `--add-app <path>`                | Adds an application to Deck.                     |
| `--set-icon <app\|start> <image>` | Changes an application or Start button icon.     |

The `NACRE_CONFIG_DIR` environment variable specifies an alternative configuration directory.

Using a different configuration directory also creates a separate nexora instance.

## 12. Keyboard Shortcuts

The following keyboard shortcuts are available.

| Shortcut                | Action                                                                                                            |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `Ctrl+Space`            | Open Quick Search.                                                                                                |
| `Ctrl+Shift+Space`      | Open Apps.                                                                                                        |
| `Ctrl+↑`                | Open Overview.                                                                                                    |
| `Alt+Tab`               | Open the nexora application switcher (only when Scene is enabled).                                                |
| `Alt+`` / `Alt+Shift+`` | Switch to the next or previous window of the same application.                                                    |
| `Ctrl+Alt+H`            | Hide (minimize) all windows belonging to the active application.                                                  |
| `Ctrl+Alt+M`            | Minimize the active window.                                                                                       |
| `Ctrl+Alt+Q`            | Gracefully close all windows belonging to the active application. The application may prompt you to save changes. |
| `Esc`                   | Close the active overlay.                                                                                         |

Additional actions, including Screen Corner shortcuts, can be configured under Settings.

If a shortcut does not work, another application may already be using the same key combination. Check `nexora.log` for details.

## 13. Privacy and Security

nexora is designed to operate locally and does not collect usage telemetry.

The following sections describe how its features handle data and system access.

### No Telemetry

nexora does not collect or transmit usage data.

The only network requests initiated directly by nexora are related to **online lyric lookup** in Visor.

When a track does not have a local `.lrc` file, nexora queries [LRCLIB](https://lrclib.net) using only the track title, artist, album, and duration.

Retrieved lyrics are cached locally in:

`%APPDATA%\nexora\lyrics\cache\`

Online lyric lookup is enabled by default and can be disabled under:

Settings → Visor → Lyrics → **Search Lyrics Online (LRCLIB)**.

The **Search the Web** option in Quick Search only opens your browser when explicitly selected.

### Local Data

Application settings, pinned applications, and logs are stored locally in:

`%APPDATA%\nexora\`

nexora does not collect this information.

### Download Monitoring

Visor monitors download activity by reading:

- Temporary filenames and file sizes in the Downloads folder.
- The Windows-owned Microsoft Store installation queue.
- Steam status files (`appmanifest_*.acf`).

File contents are not opened, and no download information is transmitted.

### Wi-Fi and Bluetooth

Network and Bluetooth device lists are retrieved from Windows when their respective menus are opened.

These lists are not stored by nexora.

### Send to Phone

During an active transfer session, nexora starts a temporary lightweight web server on the laptop's local network address.

The server uses a random port and a link containing a random token.

Only files explicitly selected for transfer can be downloaded, and only by devices connected to the local network.

The server shuts down when the transfer session ends.

No data is transmitted over the internet or through an external server.

### Calendar Access

The calendar card in the Timer tab reads event titles and times for a selected date from Windows Calendar.

Access is read-only and occurs only while the calendar card is visible.

Calendar data is not stored or transmitted.

### Keyboard Hook

When Scene and its Alt+Tab switcher are enabled, nexora installs a low-level keyboard hook solely to intercept `Alt+Tab` and display its own application switcher.

Keystrokes are not recorded, stored, or transmitted.

The hook is removed when the feature is disabled.

### Mouse Hook

nexora uses a temporary mouse hook to detect clicks outside an active panel and close it.

### Global Shortcuts

Global shortcuts use the standard Windows `RegisterHotKey` API and standard window messages, such as `WM_CLOSE` and minimize commands.

nexora does not inject code into other applications.

Global shortcuts can be disabled under Settings → General.

### System Commands

System commands, including Lock Screen and Calculator, are executed using absolute paths within `System32` rather than relying on the system `PATH` environment variable.

This prevents executable name spoofing through files with matching names elsewhere in the search path.

### Window Management

Scene moves application windows off-screen and restores them when required.

No window contents are stored on disk.

The only temporary window-related data saved is the position information in `parked.txt`, which allows windows to be restored after an unexpected shutdown.

### Automatic Startup

nexora uses the following Windows registry entry to launch at startup:

`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

The value name is `nexora`.

Administrator privileges are not required.

Automatic startup can be disabled through nexora Settings or Windows Settings → Apps → Startup.

### License and Warranty

This software is provided **as is**, without warranties of any kind.

See the [LICENSE](../LICENSE.md) file for the applicable license terms.

The source code is publicly available for inspection.

## 14. Troubleshooting

The following table covers common issues and their recommended solutions.

| Issue                                                           | Solution                                                                                                                                                                            |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows taskbar remains hidden after nexora exits unexpectedly. | Launch `nexora.exe` and exit normally, or allow `nexora-guard.exe` to restore the taskbar. Ensure both executables are located in the same folder.                                  |
| A window appears to be missing.                                 | It may have been parked by Scene. Restart nexora to restore parked windows, or disable Scene.                                                                                       |
| Keyboard shortcuts do not work.                                 | Another application may already be using the same shortcut. Check `nexora.log`.                                                                                                     |
| Application menus do not appear in MenuBar.                     | The application may not expose a classic Win32 menu. nexora uses its standard menu implementation instead.                                                                          |
| MenuBar or Deck is not visible in fullscreen mode.              | Enable Settings → General → **Always on Top of Fullscreen Windows**. This option is enabled by default. Exclusive fullscreen applications cannot be overlaid.                       |
| Desktop icons are missing.                                      | Go to Settings → General and disable **Hide Desktop Icons While nexora Is Running**. Desktop icons are restored when nexora exits.                                                  |
| Deck icons look different from their original appearance.       | Disable **Uniform Icons** under the Deck settings and set **Icon Style** to Default.                                                                                                |
| The mouse cursor remains circular after nexora exits.           | Restart nexora and exit normally, or allow `nexora-guard.exe` to restore the cursor. You can also change it through Windows Settings → Bluetooth & devices → Mouse → Mouse pointer. |
| Reporting a bug.                                                | Include your Windows version, reproduction steps, and the contents of `nexora.log` when submitting an issue to the project repository.                                              |

## 15. Known Limitations

The following limitations currently apply:

- Dragging Scene cards directly onto the stage is not supported yet. Use Shift-click instead.
- Live Scene cards use Windows.Graphics.Capture and require Windows 10 version 1903 or later. Windows 11 already meets this requirement.
- Applications running with Administrator privileges cannot be managed by Scene.
- Focus, Hotspot, and Airplane Mode controls in Quick Panel currently open their corresponding Windows Settings pages instead of providing direct toggles.
- Dynamically populated classic application menus, such as recent-file lists, are not displayed in MenuBar.
- **Force Quit** opens Windows Task Manager rather than terminating an application directly.
- Testing has been performed on Windows 11 with a single primary display.

## 16. Uninstallation

### Installer Installation

Open:

Windows Settings → Apps → nexora → Uninstall.

The installer automatically stops nexora, restores the Windows taskbar, and removes its automatic startup entry.

### Portable ZIP Installation

1. Disable **Launch at Windows Startup** in nexora Settings.
2. Exit nexora to restore the Windows taskbar.
3. Delete the folder containing the portable installation.

### Optional Configuration Cleanup

To remove all nexora settings and locally stored configuration data, delete:

`%APPDATA%\nexora\`
