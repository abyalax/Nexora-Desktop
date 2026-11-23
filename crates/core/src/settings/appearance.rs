//! Persistent Appearance preferences (theme, accent, glass, icon style), projected from the flat [`Settings`].

use super::{BlurStyle, IconStyle, Settings, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppearanceSettings {
  pub theme: Theme,
  pub accent: u8,
  pub blur: u8,
  pub style: BlurStyle,
  pub transparency: u8,
  pub refraction: bool,
  pub win_shadow: bool,
  pub icon_style: IconStyle,
  pub icon_tint: u8,
  pub icon_variant: u8,
  pub icon_pack: bool,
}

impl AppearanceSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.theme = self.theme;
    s.accent = self.accent;
    s.blur = self.blur;
    s.style = self.style;
    s.transparency = self.transparency;
    s.refraction = self.refraction;
    s.win_shadow = self.win_shadow;
    s.icon_style = self.icon_style;
    s.icon_tint = self.icon_tint;
    s.icon_variant = self.icon_variant;
    s.icon_pack = self.icon_pack;
  }
}

impl From<&Settings> for AppearanceSettings {
  fn from(s: &Settings) -> Self {
    AppearanceSettings {
      theme: s.theme,
      accent: s.accent,
      blur: s.blur,
      style: s.style,
      transparency: s.transparency,
      refraction: s.refraction,
      win_shadow: s.win_shadow,
      icon_style: s.icon_style,
      icon_tint: s.icon_tint,
      icon_variant: s.icon_variant,
      icon_pack: s.icon_pack,
    }
  }
}

impl Settings {
  pub fn appearance_settings(&self) -> AppearanceSettings {
    AppearanceSettings::from(self)
  }
}
