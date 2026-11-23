//! Persistent General preferences (takeover, start-up, language, UI scale), projected from the flat [`Settings`].

use super::Settings;
use crate::i18n::Lang;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneralSettings {
  pub takeover: bool,
  pub autostart: bool,
  pub lang: Lang,
  pub ui_scale: u8,
  pub above_fullscreen: bool,
  pub anim: bool,
}

impl GeneralSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.takeover = self.takeover;
    s.autostart = self.autostart;
    s.lang = self.lang;
    s.ui_scale = self.ui_scale;
    s.above_fullscreen = self.above_fullscreen;
    s.anim = self.anim;
  }
}

impl From<&Settings> for GeneralSettings {
  fn from(s: &Settings) -> Self {
    GeneralSettings {
      takeover: s.takeover,
      autostart: s.autostart,
      lang: s.lang,
      ui_scale: s.ui_scale,
      above_fullscreen: s.above_fullscreen,
      anim: s.anim,
    }
  }
}

impl Settings {
  pub fn general_settings(&self) -> GeneralSettings {
    GeneralSettings::from(self)
  }
}
