use crate::visibility::policy::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
  Fixed,
  Overlay,
  Smart,
}

impl Behavior {
  pub fn parse(s: &str) -> Option<Behavior> {
    match s.trim().to_ascii_lowercase().as_str() {
      "fixed" | "tetap" | "permanent" => Some(Behavior::Fixed),
      "overlay" | "menimpa" => Some(Behavior::Overlay),
      "smart" | "autohide" | "auto-hide" | "auto" | "peek" | "hidden" => Some(Behavior::Smart),
      _ => None,
    }
  }

  pub fn key(self) -> &'static str {
    match self {
      Behavior::Fixed => "fixed",
      Behavior::Overlay => "overlay",
      Behavior::Smart => "smart",
    }
  }

  pub fn mode(self) -> Mode {
    match self {
      Behavior::Fixed | Behavior::Overlay => Mode::Always,
      Behavior::Smart => Mode::AutoHide,
    }
  }

  pub fn reserves(self) -> bool {
    self == Behavior::Fixed
  }

  pub const ALL: [Behavior; 3] = [Behavior::Fixed, Behavior::Overlay, Behavior::Smart];
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
      "blur" | "buram" | "frosted" => Some(Material::Blur),
      "glass" | "liquid" | "liquidglass" => Some(Material::Glass),
      "black" | "hitam" => Some(Material::Black),
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
}

impl HotAction {
  pub const ALL: [HotAction; 7] = [
    HotAction::None,
    HotAction::Mission,
    HotAction::Launchpad,
    HotAction::Desktop,
    HotAction::Stage,
    HotAction::Notifications,
    HotAction::Lock,
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
    }
  }
  pub fn next(self) -> HotAction {
    let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
    Self::ALL[(i + 1) % Self::ALL.len()]
  }
}
