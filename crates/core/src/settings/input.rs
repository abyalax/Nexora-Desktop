//! Persistent Input preferences (hot corners, macOS-style shortcuts, clipboard history), projected from the flat
//! [`Settings`].

use super::{HotAction, Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputSettings {
  /// Hot-corner actions in the order top-left, top-right, bottom-left, bottom-right.
  pub hot: [HotAction; 4],
  pub mac_shortcuts: bool,
  pub clipboard_history: bool,
}

impl InputSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.hot = self.hot;
    s.mac_shortcuts = self.mac_shortcuts;
    s.clipboard_history = self.clipboard_history;
  }
}

impl From<&Settings> for InputSettings {
  fn from(s: &Settings) -> Self {
    InputSettings {
      hot: s.hot,
      mac_shortcuts: s.mac_shortcuts,
      clipboard_history: s.clipboard_history,
    }
  }
}

impl Settings {
  pub fn input_settings(&self) -> InputSettings {
    InputSettings::from(self)
  }
}
