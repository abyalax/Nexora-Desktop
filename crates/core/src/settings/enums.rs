use crate::visibility::policy::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
  Fixed,
  Overlay,
  Smart,
  /// Island only: hidden until summoned (spec "On demand", ILB-03). Maps to `Mode::OnDemand`.
  OnDemand,
}

impl Behavior {
  pub fn parse(s: &str) -> Option<Behavior> {
    match s.trim().to_ascii_lowercase().as_str() {
      "fixed" | "tetap" | "permanent" => Some(Behavior::Fixed),
      "overlay" | "menimpa" => Some(Behavior::Overlay),
      "smart" | "autohide" | "auto-hide" | "auto" | "peek" | "hidden" => Some(Behavior::Smart),
      "ondemand" | "on_demand" | "on-demand" | "on demand" | "demand" => Some(Behavior::OnDemand),
      _ => None,
    }
  }

  pub fn key(self) -> &'static str {
    match self {
      Behavior::Fixed => "fixed",
      Behavior::Overlay => "overlay",
      Behavior::Smart => "smart",
      Behavior::OnDemand => "ondemand",
    }
  }

  pub fn mode(self) -> Mode {
    match self {
      Behavior::Fixed | Behavior::Overlay => Mode::Always,
      Behavior::Smart => Mode::AutoHide,
      Behavior::OnDemand => Mode::OnDemand,
    }
  }

  pub fn reserves(self) -> bool {
    self == Behavior::Fixed
  }

  pub const ALL: [Behavior; 4] = [Behavior::Fixed, Behavior::Overlay, Behavior::Smart, Behavior::OnDemand];

  /// Modes the Dock offers today (D-10 adds more later). The Island offers all of [`Behavior::ALL`].
  pub const DOCK: [Behavior; 3] = [Behavior::Fixed, Behavior::Overlay, Behavior::Smart];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
  Auto,
  Light,
  Dark,
}

impl Theme {
  pub fn parse(s: &str) -> Option<Theme> {
    match s.trim().to_ascii_lowercase().as_str() {
      "auto" | "system" => Some(Theme::Auto),
      "light" | "terang" => Some(Theme::Light),
      "dark" | "gelap" => Some(Theme::Dark),
      _ => None,
    }
  }
  pub fn key(self) -> &'static str {
    match self {
      Theme::Auto => "auto",
      Theme::Light => "light",
      Theme::Dark => "dark",
    }
  }
  pub const ALL: [Theme; 3] = [Theme::Auto, Theme::Light, Theme::Dark];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlurStyle {
  Frosted,
  Clear,
  Vivid,
}

impl BlurStyle {
  pub fn parse(s: &str) -> Option<BlurStyle> {
    match s.trim().to_ascii_lowercase().as_str() {
      "frosted" | "buram" => Some(BlurStyle::Frosted),
      "clear" | "bening" => Some(BlurStyle::Clear),
      "vivid" | "hidup" => Some(BlurStyle::Vivid),
      _ => None,
    }
  }
  pub fn key(self) -> &'static str {
    match self {
      BlurStyle::Frosted => "frosted",
      BlurStyle::Clear => "clear",
      BlurStyle::Vivid => "vivid",
    }
  }
  pub const ALL: [BlurStyle; 3] = [BlurStyle::Frosted, BlurStyle::Clear, BlurStyle::Vivid];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
  Blur,
  Glass,
  Black,
}

impl Material {
  pub fn parse(s: &str) -> Option<Material> {
    match s.trim().to_ascii_lowercase().as_str() {
      "blur" | "buram" | "frosted" | "translucent" => Some(Material::Blur),
      "glass" | "liquid" | "liquidglass" => Some(Material::Glass),
      "black" | "hitam" | "opaque" => Some(Material::Black),
      "clear" | "transparan" | "transparent" => Some(Material::Glass),
      _ => None,
    }
  }
  pub fn key(self) -> &'static str {
    match self {
      Material::Blur => "blur",
      Material::Glass => "glass",
      Material::Black => "black",
    }
  }
  pub const ALL: [Material; 3] = [Material::Blur, Material::Glass, Material::Black];

  /// Name used by the design prototype (SPEC-03 row 16): Glass / Opaque / Translucent.
  pub fn design_name(self) -> &'static str {
    match self {
      Material::Blur => "Translucent",
      Material::Glass => "Glass",
      Material::Black => "Opaque",
    }
  }
}

/// What the Island shows while idle (`notch_idle`, ILB-05). Stored numbers keep their 0.4.0 meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleStyle {
  /// 0: greeting, time/date/battery and rotating glance items.
  Greeting,
  /// 1: empty pill.
  Blank,
  /// 2: wide music player when media is present.
  Music,
}

impl IdleStyle {
  pub const ALL: [IdleStyle; 3] = [IdleStyle::Greeting, IdleStyle::Blank, IdleStyle::Music];

  /// Stored value; anything out of range falls back to `Greeting`, the default.
  pub fn from_stored(n: u8) -> IdleStyle {
    match n {
      1 => IdleStyle::Blank,
      2 => IdleStyle::Music,
      _ => IdleStyle::Greeting,
    }
  }

  pub fn stored(self) -> u8 {
    match self {
      IdleStyle::Greeting => 0,
      IdleStyle::Blank => 1,
      IdleStyle::Music => 2,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarStyle {
  Light,
  Dark,
}

impl BarStyle {
  pub fn parse(s: &str) -> Option<BarStyle> {
    match s.trim().to_ascii_lowercase().as_str() {
      "light" | "terang" => Some(BarStyle::Light),
      "dark" | "gelap" => Some(BarStyle::Dark),
      _ => None,
    }
  }
  pub fn key(self) -> &'static str {
    match self {
      BarStyle::Light => "light",
      BarStyle::Dark => "dark",
    }
  }
}

pub const ACCENT_MULTI: u8 = 255;

pub const ACCENTS: [[u8; 3]; 8] = [
  [10, 132, 255],
  [175, 82, 222],
  [255, 55, 95],
  [255, 69, 58],
  [255, 149, 10],
  [255, 204, 0],
  [48, 209, 88],
  [142, 142, 147],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconStyle {
  Default,
  Dark,
  Clear,
  Tinted,
}

impl IconStyle {
  pub fn parse(s: &str) -> Option<IconStyle> {
    match s.trim().to_ascii_lowercase().as_str() {
      "default" | "bawaan" => Some(IconStyle::Default),
      "dark" | "gelap" => Some(IconStyle::Dark),
      "clear" | "bening" => Some(IconStyle::Clear),
      "tinted" | "berwarna" => Some(IconStyle::Tinted),
      _ => None,
    }
  }
  pub fn key(self) -> &'static str {
    match self {
      IconStyle::Default => "default",
      IconStyle::Dark => "dark",
      IconStyle::Clear => "clear",
      IconStyle::Tinted => "tinted",
    }
  }
}

pub fn round_factor(round: u8) -> f32 {
  0.2 + 1.6 * (round.min(100) as f32 / 100.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotAction {
  None,
  Mission,
  Launchpad,
  Desktop,
  Stage,
  Notifications,
  Lock,
  DisplaySleep,
  StartMenu,
  WindowsWidgets,
}

impl HotAction {
  pub const ALL: [HotAction; 10] = [
    HotAction::None,
    HotAction::Mission,
    HotAction::Launchpad,
    HotAction::Desktop,
    HotAction::Stage,
    HotAction::Notifications,
    HotAction::Lock,
    HotAction::DisplaySleep,
    HotAction::StartMenu,
    HotAction::WindowsWidgets,
  ];

  pub fn parse(s: &str) -> Option<HotAction> {
    Self::ALL.into_iter().find(|a| a.key() == s.trim().to_ascii_lowercase())
  }
  pub fn key(self) -> &'static str {
    match self {
      HotAction::None => "none",
      HotAction::Mission => "mission",
      HotAction::Launchpad => "launchpad",
      HotAction::Desktop => "desktop",
      HotAction::Stage => "stage",
      HotAction::Notifications => "notifications",
      HotAction::Lock => "lock",
      HotAction::DisplaySleep => "display_sleep",
      HotAction::StartMenu => "start_menu",
      HotAction::WindowsWidgets => "windows_widgets",
    }
  }
  pub fn next(self) -> HotAction {
    let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
    Self::ALL[(i + 1) % Self::ALL.len()]
  }
  /// Check if this is a known variant (for D-16.2 future variants)
  pub fn is_known(self) -> bool {
    matches!(
      self,
      HotAction::None
        | HotAction::Mission
        | HotAction::Launchpad
        | HotAction::Desktop
        | HotAction::Stage
        | HotAction::Notifications
        | HotAction::Lock
        | HotAction::DisplaySleep
        | HotAction::StartMenu
        | HotAction::WindowsWidgets
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn behavior_maps_to_visibility_modes_exhaustively() {
    // I-12.1: one mapping, every variant listed so a new variant fails to compile here first.
    for b in Behavior::ALL {
      let (mode, reserves) = match b {
        Behavior::Fixed => (Mode::Always, true),
        Behavior::Overlay => (Mode::Always, false),
        Behavior::Smart => (Mode::AutoHide, false),
        Behavior::OnDemand => (Mode::OnDemand, false),
      };
      assert_eq!(b.mode(), mode, "{b:?}");
      assert_eq!(b.reserves(), reserves, "{b:?}");
      assert_eq!(Behavior::parse(b.key()), Some(b), "{b:?} round-trips through its key");
    }
  }

  #[test]
  fn on_demand_has_aliases_and_old_names_keep_their_meaning() {
    for s in ["ondemand", "On Demand", "on-demand", "on_demand", "demand"] {
      assert_eq!(Behavior::parse(s), Some(Behavior::OnDemand), "{s}");
    }
    for s in ["peek", "autohide", "hidden"] {
      assert_eq!(Behavior::parse(s), Some(Behavior::Smart), "legacy {s}");
    }
    assert_eq!(Behavior::DOCK.len(), 3);
    assert!(!Behavior::DOCK.contains(&Behavior::OnDemand));
  }

  #[test]
  fn material_design_names_map_both_ways() {
    // I-12.2: Opaque = Black, Translucent = Blur, Glass = Glass.
    for m in Material::ALL {
      assert_eq!(Material::parse(m.design_name()), Some(m), "{m:?}");
      assert_eq!(Material::parse(m.key()), Some(m), "{m:?}");
    }
    assert_eq!(Material::parse("Opaque"), Some(Material::Black));
    assert_eq!(Material::parse("Translucent"), Some(Material::Blur));
    assert_eq!(Material::parse("sparkly"), None);
  }

  #[test]
  fn hot_actions_keep_legacy_keys_and_round_trip_new_windows_actions() {
    assert_eq!(HotAction::parse("mission"), Some(HotAction::Mission));
    for action in HotAction::ALL {
      assert_eq!(HotAction::parse(action.key()), Some(action));
      assert!(action.is_known());
    }
    assert_eq!(HotAction::ALL.len(), 10);
  }

  #[test]
  fn idle_style_keeps_legacy_numbers() {
    // I-3.1: stored 0/1/2 keep their 0.4.0 meaning; out-of-range falls back to the default.
    assert_eq!(IdleStyle::from_stored(0), IdleStyle::Greeting);
    assert_eq!(IdleStyle::from_stored(1), IdleStyle::Blank);
    assert_eq!(IdleStyle::from_stored(2), IdleStyle::Music);
    assert_eq!(IdleStyle::from_stored(9), IdleStyle::Greeting);
    for s in IdleStyle::ALL {
      assert_eq!(IdleStyle::from_stored(s.stored()), s);
    }
  }
}
