const defaultSettings = {
  shared: {
    enableFinder: false, enableDock: true, aboveFullscreen: true, globalScale: 1, globalBlur: 60,
    appearance: "dark", accentColor: "#63c7dc", startOnBoot: "Don't Start On Boot", hideTaskbarOnBoot: "none",
    displayLanguage: "Indonesian", displayFont: "Segoe UI", globalFontScale: 1,
    customDockBlur: false, dockBlur: 22, dockTransparency: 62, pinnedReflections: false,
    reflectionBlur: 5, reflectionTransparency: 35, reflectionDistance: 3, reflectionGradientDistance: 8,
    customIconLabels: false, labelBlur: 18, labelTransparency: 70, labelBorderTransparency: 12,
    customContextMenus: false, contextBlur: 22, contextTransparency: 85, contextBorder: 12,
    customFolderGrid: false, folderBlur: 22, folderTransparency: 85, folderBorder: 12, folderIconSize: 48,
    customNexoraMenus: false, menuBlur: 26, menuTransparency: 90, menuBorder: 10,
    customFinderBlur: false, finderBlur: 26, finderTransparency: 88, wallpaperTint: 0,
    screenBrightness: 100, nightLight: false, nightLightIntensity: 25, screenTarget: "Primary display",
    launchpadCustomBlur: false, launchpadAboveWindows: false, launchpadBlurBackground: true, launchpadBlur: 24,
    trayAllFinder: false, trayAllDock: false, traySystemFinder: true, newTrayIconSeconds: 5,
    stageIcon: true, stageWindowTitle: true, stageBlurIcons: true, stageClickDesktopHide: true,
    stageAutoPauseMedia: false, stageNativePreview: true, stageDesktopIcons: "Auto Show Hide", stageAnimationSpeed: "1.0x",
    stageEnabled: false, stagePosition: "left", stageHiddenMode: "default", stageKeepBorder: false,
    stageIconSize: 36, stageIconSpacing: 20, stageIconCount: 6,
    windowAnimation: true, windowAnimationEffect: "scale", windowAnimationSpeed: "1.0x",
    excludedPrograms: "", programBlacklist: "", minimizeExcludedWindows: true
  },
  dock: {
    iconSize: 48, iconEffect: "Scaling", iconMagnification: 2, iconLightIntensity: 35,
    iconBackgroundLight: 25, iconBackgroundColor: "#63c7dc", iconSpacing: 14, edgeDistance: 8,
    position: "bottom", display: "Primary display", launchpadIconSize: 52, launchpadHorizontalSpacing: 22,
    launchpadVerticalSpacing: 24, launchpadSources: "added", deletedStartMenuPrograms: "",
    launchpadDisplay: "Primary display", launchpadFollowDock: true, scaleMacIcons: true, doubleClickFolderFinder: true,
    hideOpenFolderButton: false, lockIconDragging: false, showIconLabels: false, animateAppLaunch: true,
    preventWindowsCoverDock: true, showRunningIndicators: true, showUnpinnedRunning: true, separateUnpinned: false,
    bottomLeftHotCorner: "Do nothing", displayMode: "default", detectionWindow: "All Program Windows", hiddenEffect: "Move",
    activationTrigger: "The edge of the screen", activationBarWidth: 4, activationBarHeight: 2, activationBarDistance: 0,
    mouseActivationDelay: 150, useNexoraMinimize: true, disableMinimizeAnimation: false, shiftSlowAnimation: true,
    minimizeIntoIcon: true, dynamicMinimizeContent: true, minimizedIconPosition: "Next to the icon on the right",
    minimizeAnimation: "Genie", minimizeAnimationSpeed: "1.0x", minimizeBlacklist: "",
    windowPreviews: true, previewSize: 260, previewDelay: 450, previewAppName: true, previewControls: true,
    previewNative: true
  },
  finder: {
    menuIcon: true, autoHide: false, shutdownConfirmation: true, maximizedAffectsAppearance: true,
    closeMaximizedOnCorner: false, lockTrayDragging: false, windowOptions: true, coloredTrayIcons: true,
    inputMethodMenu: true, searchIcon: true, accountIcon: true, mediaPlaybackIcon: true, lyricsMenu: true,
    colorPlayerLyrics: true, showBatteryMenu: true, batteryPercentage: true, remainingBatteryTime: true,
    showNotifications: true, notificationSounds: true, notificationDuration: 5,
    showTime: true, showDate: true, timeFormat: "24-hour", dateFormat: "Short",
    showVolume: true, showAudioOutput: true, showMediaControls: true, showBrightness: true,
    showWifi: true, showBluetooth: true, showHardwareIcons: true,
    autoPowerPlans: false, powerPlanPlugged: "balanced", powerPlanBattery: "balanced"
  },
  island: {
    enabled: true, shape: "attached", displayMode: "Fixed", openOnHover: true, idleStyle: "Toothless eyes",
    greetingName: "", greetingText: "Hello!", material: "Black", rounding: 50,
    mediaLyrics: true, compactLyrics: true, onlineLyrics: true, downloads: true, fileShelf: true,
    sendToPhone: true, timerCalendar: true, eyeEnabled: true, eyeTheme: "toothless", eyeShape: "toothless",
    eyeRadius: 280, eyeAutonomous: true, eyeGlanceChance: 78, eyeMinInterval: 5500, eyeMaxInterval: 11000
  }
};

const prefPages = [
  {
    group: "System", name: "General", section: "shared", fields: [
      ["toggle", "Enable Finder", "enableFinder"], ["toggle", "Enable Dock", "enableDock"], ["toggle", "Stay above full-screen windows", "aboveFullscreen"],
      ["range", "Global Interface Scale", "globalScale", .75, 2, .01], ["range", "Global Interface Blur Intensity", "globalBlur", 0, 100, 1],
      ["select", "Appearance", "appearance", ["light", "dark", "system"]], ["color", "Accent Color", "accentColor"],
      ["select", "Start On Boot Method", "startOnBoot", ["Don't Start On Boot", "Registry Value", "Scheduled Task", "Start as a Service"]],
      ["select", "Automatically hide the taskbar on boot", "hideTaskbarOnBoot", ["none", "allDisplays", "dockDisplay"]],
      ["select", "Display Language", "displayLanguage", ["English", "Indonesian"]],
      ["select", "Display Font", "displayFont", ["Segoe UI", "Arial", "Inter", "Roboto", "system-ui"]],
      ["range", "Global Font Scaling", "globalFontScale", .75, 1.5, .01]
    ]
  },
  {
    group: "System", name: "Appearance", section: "shared", fields: [
      ["toggle", "Make Dock use custom blur and transparency values", "customDockBlur"], ["range", "Dock blur intensity", "dockBlur", 0, 50, 1], ["range", "Dock transparency", "dockTransparency", 0, 100, 1],
      ["toggle", "Make pinned icons reflect from the Dock", "pinnedReflections"], ["range", "Reflection blur strength", "reflectionBlur", 0, 30, 1], ["range", "Reflection transparency", "reflectionTransparency", 0, 100, 1], ["range", "Icon reflection distance", "reflectionDistance", 0, 20, 1], ["range", "Gradient distance of icon reflection", "reflectionGradientDistance", 0, 30, 1],
      ["toggle", "Make Dock icon labels use custom blur and transparency values", "customIconLabels"], ["range", "Label blur intensity", "labelBlur", 0, 40, 1], ["range", "Label transparency", "labelTransparency", 0, 100, 1], ["range", "Label border transparency", "labelBorderTransparency", 0, 100, 1],
      ["toggle", "Make context menus use custom blur and transparency values", "customContextMenus"], ["range", "Context menu blur intensity", "contextBlur", 0, 50, 1], ["range", "Context menu transparency", "contextTransparency", 0, 100, 1], ["range", "Context menu border transparency", "contextBorder", 0, 100, 1],
      ["toggle", "Make folder contents grid use custom blur and transparency values", "customFolderGrid"], ["range", "Folder grid blur intensity", "folderBlur", 0, 50, 1], ["range", "Folder grid transparency", "folderTransparency", 0, 100, 1], ["range", "Folder grid border transparency", "folderBorder", 0, 100, 1], ["range", "Folder grid icon size", "folderIconSize", 24, 96, 1],
      ["toggle", "Make Nexora menus use custom blur and transparency values", "customNexoraMenus"], ["range", "Nexora menu blur intensity", "menuBlur", 0, 50, 1], ["range", "Nexora menu transparency", "menuTransparency", 0, 100, 1], ["range", "Nexora menu border transparency", "menuBorder", 0, 100, 1],
      ["toggle", "Make Finder use custom blur and transparency values", "customFinderBlur"], ["range", "Finder blur intensity", "finderBlur", 0, 50, 1], ["range", "Finder transparency", "finderTransparency", 0, 100, 1], ["range", "Wallpaper tint intensity", "wallpaperTint", 0, 100, 1],
      ["toggle", "Launchpad uses custom background blur", "launchpadCustomBlur"], ["toggle", "Display Launchpad above windows instead of hiding them", "launchpadAboveWindows"], ["toggle", "Launchpad blurs the background instead of pre-rendering it", "launchpadBlurBackground"], ["range", "Launchpad background blur intensity", "launchpadBlur", 0, 50, 1]
    ]
  },
  {
    group: "System", name: "System Icon Tray", section: "shared", fields: [
      ["toggle", "Always display all tray icons in Finder", "trayAllFinder"], ["toggle", "Always display all tray icons in Dock", "trayAllDock"], ["toggle", "Display the system's own tray icon in Finder", "traySystemFinder"], ["range", "Newly run tray icon display time (seconds)", "newTrayIconSeconds", 0, 30, 1]
    ]
  },
  {
    group: "System", name: "Screen", section: "shared", fields: [
      ["range", "Display brightness overlay", "screenBrightness", 0, 100, 1], ["toggle", "Enable night light", "nightLight"], ["range", "Night light intensity", "nightLightIntensity", 0, 100, 1],
      ["select", "Display target", "screenTarget", ["All displays", "Primary display", "Active display"]]
    ]
  },
  {
    group: "System", name: "Stage Manager", section: "shared", fields: [
      ["toggle", "Show Stage Manager icon in Finder", "stageIcon"], ["toggle", "Show window title on mouseover", "stageWindowTitle"], ["toggle", "Blur background window icons", "stageBlurIcons"], ["toggle", "Click on the desktop to hide all windows", "stageClickDesktopHide"], ["toggle", "Automatically pause and play background window media", "stageAutoPauseMedia"], ["toggle", "Show native window preview on hover", "stageNativePreview"],
      ["select", "Show Desktop Icons", "stageDesktopIcons", ["Always Hide Icon", "Auto Show Hide", "Always Show"]], ["select", "Adjust Animation Speed", "stageAnimationSpeed", ["0.5x", "0.75x", "1.0x", "1.25x", "1.5x"]],
      ["toggle", "Open Stage Manager", "stageEnabled"], ["select", "Window list screen position", "stagePosition", ["left", "right"]], ["select", "Window List Show Hidden Mode", "stageHiddenMode", ["default", "alwaysShow", "alwaysHide"]], ["toggle", "Keep Window Border", "stageKeepBorder"],
      ["range", "Window list icon size", "stageIconSize", 16, 80, 1], ["range", "Window list icon spacing", "stageIconSpacing", 0, 60, 1], ["range", "Number of window list icons", "stageIconCount", 2, 10, 1]
    ]
  },
  {
    group: "System", name: "Window Animation", section: "shared", fields: [
      ["toggle", "Window opening animation", "windowAnimation"], ["select", "Window opening animation effect", "windowAnimationEffect", ["scale", "fade", "slide", "zoom"]],
      ["select", "Adjust animation speed", "windowAnimationSpeed", ["0.5x", "0.75x", "1.0x", "1.25x", "1.5x"]],
      ["text", "Add excluded programs (semicolon separated)", "excludedPrograms"], ["text", "Manage running program blacklist", "programBlacklist"], ["toggle", "Remove windows for excluded programs", "minimizeExcludedWindows"]
    ]
  },
  {
    group: "Dock", name: "Look & Behavior", section: "dock", fields: [
      ["range", "Icon size", "iconSize", 32, 72, 1], ["select", "Icon effect", "iconEffect", ["None", "Scaling", "Select", "Illumination", "Scaling and light", "Select and light"]],
      ["range", "Icon magnification", "iconMagnification", 1, 2.5, .05], ["range", "Icon light intensity", "iconLightIntensity", 0, 100, 1], ["range", "Icon background light intensity", "iconBackgroundLight", 0, 100, 1], ["color", "Icon background light color", "iconBackgroundColor"],
      ["range", "Distance between icons", "iconSpacing", 0, 24, 1], ["range", "Dock distance from screen edge", "edgeDistance", 0, 60, 1], ["select", "Position on screen", "position", ["left", "bottom", "right", "top"]], ["select", "Display on screen", "display", ["Primary display", "All displays", "Display 1", "Display 2"]]
    ]
  },
  {
    group: "Dock", name: "Launchpad", section: "dock", fields: [
      ["range", "Launchpad icon size", "launchpadIconSize", 24, 100, 1], ["range", "Icon horizontal spacing", "launchpadHorizontalSpacing", 0, 60, 1], ["range", "Icon vertical spacing", "launchpadVerticalSpacing", 0, 60, 1],
      ["select", "Choose which icons will be displayed by Launchpad", "launchpadSources", ["added", "addedStartMenu", "addedDesktop", "all"]],
      ["text", "Edit deleted Start Menu programs", "deletedStartMenuPrograms"], ["select", "Select the screen to display Launchpad", "launchpadDisplay", ["Primary display", "All displays", "Display 1", "Display 2"]], ["toggle", "Follow Dock", "launchpadFollowDock"]
    ]
  },
  {
    group: "Dock", name: "Advanced", section: "dock", fields: [
      ["toggle", "Scale all icons to macOS standards", "scaleMacIcons"], ["toggle", "Double-click the folder icon to open in Finder", "doubleClickFolderFinder"], ["toggle", "Hide the open folder button", "hideOpenFolderButton"], ["toggle", "Lock icon dragging", "lockIconDragging"], ["toggle", "Show icon labels", "showIconLabels"], ["toggle", "Animate opening applications", "animateAppLaunch"], ["toggle", "Turn on window borders to prevent windows from hiding Dock and Finder", "preventWindowsCoverDock"], ["toggle", "Show indicators under running programs", "showRunningIndicators"], ["toggle", "Show icons of unpinned running programs", "showUnpinnedRunning"], ["toggle", "Separate unpinned running programs", "separateUnpinned"],
      ["select", "Bottom left hot-corner function", "bottomLeftHotCorner", ["Do nothing", "Show all windows", "Open Launchpad", "Put display to sleep", "Open Start Menu", "Show Desktop", "Open Mission Control", "Lock Screen", "Windows widget", "Advanced"]],
      ["select", "Dock display mode", "displayMode", ["default", "overlay", "autoHide", "smartAutoHide", "alwaysShow", "forceVisible", "desktopOnly"]],
      ["select", "Detection window", "detectionWindow", ["All Program Windows", "The Current Program Window"]], ["select", "Dock hidden effect", "hiddenEffect", ["Move", "Fade Away"]], ["select", "Dock activation trigger", "activationTrigger", ["The edge of the screen", "Activation bar", "The edge of the screen & Activation bar"]],
      ["range", "Activation bar width", "activationBarWidth", 1, 100, 1], ["range", "Activation bar height", "activationBarHeight", 1, 30, 1], ["range", "Activation bar distance from screen edge", "activationBarDistance", 0, 40, 1], ["range", "Mouse delay activates dock time (ms)", "mouseActivationDelay", 0, 1500, 10]
    ]
  },
  {
    group: "Dock", name: "Minimize Effects", section: "dock", fields: [
      ["toggle", "Use Nexora minimize effect", "useNexoraMinimize"], ["toggle", "Don't animate minimizing windows", "disableMinimizeAnimation"], ["toggle", "Hold down Shift to slow down an animation", "shiftSlowAnimation"], ["toggle", "Minimize windows into their Dock icon", "minimizeIntoIcon"], ["toggle", "Dynamically display window content when minimized", "dynamicMinimizeContent"],
      ["select", "Minimize window icon position", "minimizedIconPosition", ["Next to the icon on the right", "Independently displayed on the right", "Right side displayed independently using a separator"]],
      ["select", "Minimize animation", "minimizeAnimation", ["Genie", "Scale", "Suck"]], ["select", "Minimize animation speed", "minimizeAnimationSpeed", ["0.5x", "0.75x", "1.0x", "1.25x", "1.5x"]], ["text", "Minimize animation blacklist", "minimizeBlacklist"]
    ]
  },
  {
    group: "Dock", name: "Window Previews", section: "dock", fields: [
      ["toggle", "Enable window previews", "windowPreviews"], ["range", "Window preview size", "previewSize", 120, 520, 10], ["range", "Preview delay on hover (ms)", "previewDelay", 0, 1500, 25],
      ["toggle", "Application name", "previewAppName"], ["toggle", "Window control buttons", "previewControls"], ["toggle", "Show native window preview on hover", "previewNative"]
    ]
  },
  {
    group: "Finder", name: "Look & Behavior", section: "finder", fields: [
      ["toggle", "Finder menu icon", "menuIcon"], ["toggle", "Auto-Hide Finder", "autoHide"], ["toggle", "Shut down confirmation box", "shutdownConfirmation"], ["toggle", "Maximized windows affect Finder appearance", "maximizedAffectsAppearance"], ["toggle", "Click the upper right corner to close the current maximized window", "closeMaximizedOnCorner"], ["toggle", "Lock dragging tray icons", "lockTrayDragging"], ["toggle", "Show window options in Finder", "windowOptions"], ["toggle", "Show colored tray icons", "coloredTrayIcons"], ["toggle", "Show input method menu", "inputMethodMenu"], ["toggle", "Show search icon", "searchIcon"], ["toggle", "The account icon is displayed", "accountIcon"], ["toggle", "Displays the media playback icon", "mediaPlaybackIcon"], ["toggle", "Show music player lyrics menu", "lyricsMenu"], ["toggle", "Show Color Player Lyrics", "colorPlayerLyrics"]
    ]
  },
  {
    group: "Finder", name: "Notifications", section: "finder", fields: [
      ["toggle", "Show desktop notifications", "showNotifications"], ["toggle", "Notification sounds", "notificationSounds"], ["range", "Notification display duration (seconds)", "notificationDuration", 1, 15, 1]
    ]
  },
  {
    group: "Finder", name: "Time & Date", section: "finder", fields: [
      ["toggle", "Show time in Menu Bar", "showTime"], ["toggle", "Show date in Menu Bar", "showDate"], ["select", "Time format", "timeFormat", ["12-hour", "24-hour"]], ["select", "Date format", "dateFormat", ["Short", "Long", "ISO"]]
    ]
  },
  {
    group: "Finder", name: "Audio", section: "finder", fields: [
      ["toggle", "Show volume control", "showVolume"], ["toggle", "Show audio output selector", "showAudioOutput"], ["toggle", "Show media playback controls", "showMediaControls"]
    ]
  },
  {
    group: "Finder", name: "Display", section: "finder", fields: [
      ["toggle", "Show display brightness control", "showBrightness"]
    ]
  },
  {
    group: "Finder", name: "Hardware Status", section: "finder", fields: [
      ["toggle", "Show Wi-Fi status", "showWifi"], ["toggle", "Show Bluetooth status", "showBluetooth"], ["toggle", "Show device status icons", "showHardwareIcons"]
    ]
  },
  {
    group: "Finder", name: "Battery", section: "finder", fields: [
      ["toggle", "Show Battery Status menu in Finder", "showBatteryMenu"], ["toggle", "Show Battery percentage in Finder status menu", "batteryPercentage"], ["toggle", "Display the remaining battery time", "remainingBatteryTime"], ["toggle", "Automatically switch power plans", "autoPowerPlans"],
      ["select", "Power plan when plugged in", "powerPlanPlugged", ["balanced", "bestPerformance", "bestPowerEfficiency"]], ["select", "Power plan when using batteries", "powerPlanBattery", ["balanced", "bestPerformance", "bestPowerEfficiency"]]
    ]
  },
  {
    group: "Dynamic Island", name: "Look & Behavior", section: "island", fields: [
      ["toggle", "Enable Dynamic Island", "enabled"], ["select", "Shape", "shape", ["attached", "floating capsule"]], ["select", "Display mode", "displayMode", ["Fixed", "Overlay", "Smart", "On demand"]], ["toggle", "Open on hover", "openOnHover"],
      ["select", "Idle style", "idleStyle", ["Minimal", "Toothless eyes", "Media", "Clock"]], ["text", "Greeting name", "greetingName"], ["text", "Greeting text", "greetingText"], ["select", "Material", "material", ["Blur", "Glass", "Black"]], ["range", "Material rounding", "rounding", 0, 100, 1],
      ["toggle", "Show Toothless eye companion", "eyeEnabled"], ["select", "Eye color theme", "eyeTheme", ["toothless", "ocean", "sunset", "violet", "crimson", "ice"]], ["select", "Eye shape", "eyeShape", ["toothless", "round", "wide", "almond", "softTriangle"]],
      ["range", "Eye perception radius (px)", "eyeRadius", 120, 500, 10], ["toggle", "Autonomous eye glances", "eyeAutonomous"], ["range", "Autonomous glance chance (%)", "eyeGlanceChance", 0, 100, 1], ["range", "Minimum glance interval (ms)", "eyeMinInterval", 1000, 20000, 250], ["range", "Maximum glance interval (ms)", "eyeMaxInterval", 2000, 30000, 250]
    ]
  },
  {
    group: "Dynamic Island", name: "Activities", section: "island", fields: [
      ["toggle", "Media and lyrics", "mediaLyrics"], ["toggle", "Compact lyrics", "compactLyrics"], ["toggle", "Online lyrics lookup", "onlineLyrics"], ["toggle", "Downloads", "downloads"], ["toggle", "File shelf", "fileShelf"], ["toggle", "Send to phone", "sendToPhone"], ["toggle", "Timer and calendar strip", "timerCalendar"]
    ]
  }
];

// HTM-06: where each spec section starts on a page (docs/feature.md section 2). A page with no entry is one card.
// The first field always opens a card, titled by the page name unless it has an entry here.
const pageSections = {
  "System/General": { enableFinder: "Menu Bar and Dock", globalScale: "Interface", appearance: "Appearance", startOnBoot: "Startup, language and font" },
  "System/Appearance": {
    customDockBlur: "Dock glass", pinnedReflections: "Reflections", customIconLabels: "Icon labels", customContextMenus: "Context menus",
    customFolderGrid: "Folder grid", customNexoraMenus: "Nexora menus", customFinderBlur: "Menu Bar", launchpadCustomBlur: "Launchpad"
  },
  "System/Stage Manager": { stageIcon: "Options", stageDesktopIcons: "Desktop icons and speed", stageEnabled: "Window list", stageIconSize: "Window list icons" },
  "Dock/Look & Behavior": { iconSize: "Icons", iconSpacing: "Placement" },
  "Dock/Advanced": { scaleMacIcons: "Icons and indicators", bottomLeftHotCorner: "Hot corner", displayMode: "Display behavior", activationBarWidth: "Activation bar" },
  "Finder/Look & Behavior": { menuIcon: "Menu Bar", lockTrayDragging: "Tray", inputMethodMenu: "Menu items", lyricsMenu: "Music player" },
  "Dynamic Island/Look & Behavior": { enabled: "Island", idleStyle: "Idle and appearance", eyeEnabled: "Eye companion" }
};
for (const page of prefPages) page.starts = pageSections[`${page.group}/${page.name}`] || {};

// Cards of a page: [{ title, fields }]. A new card opens at every field named in `starts`.
function pageCards(page) {
  const cards = [];
  page.fields.forEach((field, i) => {
    const title = page.starts[field[2]];
    if (i === 0 || title) cards.push({ title: title || page.name, fields: [] });
    cards[cards.length - 1].fields.push(field);
  });
  return cards;
}

const schema = { defaultSettings, prefPages, pageCards };
if (typeof window !== 'undefined') window.NEXORA_SETTINGS_SCHEMA = schema;
if (typeof module === 'object' && module.exports) module.exports = schema;
