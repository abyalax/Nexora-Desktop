use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Action {
  EditText(u8),
  NotchIdle(u8),
  LyricsCompact,
  NotchDownloads,
  NotchSend,
  NotchMedia,
  NotchShelf,
  NotchTimer,
  LyricsOnline,
  LyricsDir,
  Donate,
  Close,
  Takeover,
  Dock(Behavior),
  Notch(Behavior),
  Theme(Theme),
  Lang(Lang),
  Tab(u8),
  Autostart,
  Anim,
  Refraction,
  DockOn,
  MenuBarOn,
  Clock24,
  BarStyle(u8),
  HotCycle(usize),
  StageOn,
  SwitcherOn,
  StageFlag(u8),
  StageRight(bool),
  StageIcons(u8),
  HideIcons,
  DockPlates,
  IconPack,
  DownloadsStack,
  RecentApps,
  AboveFull,
  ClipHistory,
  MacKeys,
  Accent(u8),
  IconVariant(u8),
  MinEffect(u8),
  Back,
  Forward,
  Search,
  IconStyle(u8),
  IconTint(u8),
  NotchOn,
  HoverOpen,
  WinShadow,
  Material(bool, Material),
  Slider(u8, f32, f32, Option<nexora_core::settings::NumericSlider>),
  MenuLogoPick,
  MenuLogoReset,
  NotchShape(bool),
  WidgetStyle(u8),
  WidgetGallery,
  BatteryPct(u8),
  // M2 DLB settings
  DockIconEffect(u8),
  DockPosition(u8),
  DockDisplay(u8),
  // M2 DLP settings
  LaunchpadSources(u8),
  LaunchpadDisplay(u8),
  LaunchpadFollowDock,
  // M2 DAD settings
  DockDblClickFolder,
  DockHideOpenFolder,
  DockLockDrag,
  DockShowLabels,
  DockAnimateLaunch,
  DockPreventCover,
  DockShowIndicators,
  DockShowUnpinned,
  DockSeparateUnpinned,
  DockDetection(u8),
  DockHiddenEffect(u8),
  DockActivationTrigger(u8),
  // M2 DMN settings
  UseNexoraMinimize,
  DisableMinimizeAnim,
  ShiftSlowAnim,
  MinimizeIntoIcon,
  DynamicMinimizeContent,
  MinimizedIconPos(u8),
  MinimizeAnimSpeed(u8),
  // M2 DPV settings
  WindowPreviews,
  PreviewAppName,
  PreviewControls,
  PreviewNative,
  // M2 STG settings
  StageIcon,
  StageBlurIcons,
  StageNativePreview,
  StageKeepBorder,
  StageHiddenMode(u8),
  // M2 FLB settings
  FinderMenuIcon,
  FinderAutoHide,
  FinderShutdownConfirm,
  FinderMaximizedAffects,
  FinderCloseMaximizedCorner,
  FinderLockTrayDrag,
  FinderWindowOptions,
  FinderColoredTray,
  FinderInputMethod,
  FinderSearchIcon,
  FinderAccountIcon,
  FinderMediaPlayback,
  FinderLyricsMenu,
  FinderColorLyrics,
  // M2 FNO settings
  FinderShowNotif,
  FinderNotifSounds,
  // M2 FTD settings
  FinderShowTime,
  FinderShowDate,
  FinderDateFormat(u8),
  // M2 FAU settings
  FinderShowVolume,
  FinderShowAudioOutput,
  FinderShowMediaControls,
  // M2 FDI settings
  FinderShowBrightness,
  // M2 FHW settings
  FinderShowWifi,
  FinderShowBluetooth,
  FinderShowHardware,
  // M2 FBA settings
  FinderShowBattery,
  FinderRemainingBattery,
  FinderAutoPowerModes,
  FinderPowerPlugged(u8),
  FinderPowerBattery(u8),
  // M2 APP settings
  CustomDockBlur,
  PinnedReflections,
  CustomIconLabels,
  CustomContextMenus,
  CustomFolderGrid,
  CustomNexoraMenus,
  CustomFinderBlur,
  LaunchpadCustomBlur,
  LaunchpadAboveWindows,
  LaunchpadBlurBackground,
  TrayAllFinder,
  TrayAllDock,
  TraySystemFinder,
  NightLight,
  // M2 SCR settings
  ScreenTarget(u8),
  // M2 GEN settings
  StartMethod(u8),
  TaskbarScope(u8),
  DisplayFont(u8),
  // M2 WAN settings
  WindowAnimEffect(u8),
  WindowAnimSpeed(u8),
  MinimizeExcludedWindows,
}

pub(super) struct Region {
  pub(super) x: f32,
  pub(super) y: f32,
  pub(super) w: f32,
  pub(super) h: f32,
  pub(super) a: Action,
  pub(super) scroll: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Page {
  General,
  Dock,
  Island,
  Stage,
  Appearance,
  Advanced,
}

impl Page {
  pub(super) fn index(&self) -> u8 {
    match self {
      Page::General => 0,
      Page::Dock => 1,
      Page::Island => 2,
      Page::Stage => 3,
      Page::Appearance => 4,
      Page::Advanced => 5,
    }
  }

  pub(super) fn name(&self) -> &'static str {
    match self {
      Page::General => "Umum",
      Page::Dock => "Dock",
      Page::Island => "Dynamic Island",
      Page::Stage => "Stage Manager",
      Page::Appearance => "Tampilan",
      Page::Advanced => "Lanjutan",
    }
  }

  pub(super) fn icon(&self) -> &'static str {
    match self {
      Page::General => "\u{E713}",
      Page::Dock => "\u{E7C4}",
      Page::Island => "\u{E7F4}",
      Page::Stage => "\u{E8A0}",
      Page::Appearance => "\u{E790}",
      Page::Advanced => "\u{E73F}",
    }
  }
}

pub(super) enum RowDef {
  Toggle(&'static str, bool, Action),
  Seg(&'static str, Vec<(&'static str, Action)>, usize),
  Slider(&'static str, u8, u8),
  GatedSlider(&'static str, u8, u8, bool),
  WideSlider(
    &'static str,
    f32,
    u8,
    nexora_core::settings::NumericSlider,
    &'static str,
  ),
  Pick(&'static str, String, Action),
  Swatches(&'static str, usize),
  Thumbs(&'static str, Vec<(&'static str, Thumb, Action)>, usize),
  Radios(Vec<(&'static str, Action)>, usize),
  Text(&'static str, String, String, u8),
}

#[derive(Clone, Copy)]
pub(super) enum Thumb {
  Appear(u8),
  Icon(nexora_core::settings::IconStyle),
}

pub(super) type Rows = Vec<(Option<&'static str>, Vec<RowDef>)>;

pub(super) const ACCENT_NAMES: [&str; 9] = [
  "Multiwarna",
  "Biru",
  "Ungu",
  "Merah Muda",
  "Merah",
  "Oranye",
  "Kuning",
  "Hijau",
  "Grafit",
];

pub(super) fn row_label(r: &RowDef) -> &'static str {
  match r {
    RowDef::Toggle(l, ..)
    | RowDef::Seg(l, ..)
    | RowDef::Slider(l, ..)
    | RowDef::GatedSlider(l, ..)
    | RowDef::WideSlider(l, ..)
    | RowDef::Pick(l, ..)
    | RowDef::Swatches(l, ..)
    | RowDef::Thumbs(l, ..)
    | RowDef::Text(l, ..) => l,
    RowDef::Radios(..) => "",
  }
}

pub(super) fn hot_label(a: HotAction) -> &'static str {
  use HotAction::*;
  match a {
    None => "Tidak ada",
    Mission => "Overview",
    Launchpad => "Apps",
    Desktop => "Tampilkan Desktop",
    Stage => "Scene",
    Notifications => "Pusat Notifikasi",
    Lock => "Kunci Layar",
    DisplaySleep => "Matikan Layar",
    StartMenu => "Start Menu",
    WindowsWidgets => "Windows Widgets",
  }
}
