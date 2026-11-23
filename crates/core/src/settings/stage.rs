//! Persistent Stage Manager preferences, projected from the flat [`Settings`].
//!
//! `StageSettings` is configuration; the runtime state lives in `crate::window_management::stage`.

use super::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageSettings {
  pub enabled: bool,
  pub switcher: bool,
  /// `true` = strip on the right edge, `false` = left edge.
  pub right: bool,
  pub size: u8,
  pub gap: u8,
  pub count: u8,
  pub speed: u8,
  pub desktop_click: bool,
  pub media: bool,
  pub icons: u8,
  pub titles: bool,
  pub hide_desktop_icons: bool,
}

impl StageSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.stage = self.enabled;
    s.switcher = self.switcher;
    s.stage_right = self.right;
    s.stage_size = self.size;
    s.stage_gap = self.gap;
    s.stage_count = self.count;
    s.stage_speed = self.speed;
    s.stage_desktop_click = self.desktop_click;
    s.stage_media = self.media;
    s.stage_icons = self.icons;
    s.stage_titles = self.titles;
    s.hide_desktop_icons = self.hide_desktop_icons;
  }
}

impl From<&Settings> for StageSettings {
  fn from(s: &Settings) -> Self {
    StageSettings {
      enabled: s.stage,
      switcher: s.switcher,
      right: s.stage_right,
      size: s.stage_size,
      gap: s.stage_gap,
      count: s.stage_count,
      speed: s.stage_speed,
      desktop_click: s.stage_desktop_click,
      media: s.stage_media,
      icons: s.stage_icons,
      titles: s.stage_titles,
      hide_desktop_icons: s.hide_desktop_icons,
    }
  }
}

impl Settings {
  pub fn stage_settings(&self) -> StageSettings {
    StageSettings::from(self)
  }
}
