//! Persistent Finder (menu bar / top bar) preferences, projected from the flat on-disk [`Settings`].

use super::{BarStyle, Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinderSettings {
  pub enabled: bool,
  pub style: BarStyle,
  pub glass: bool,
  pub material: bool,
  pub clock_24h: bool,
  pub battery_pct: u8,
}

impl FinderSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.menubar = self.enabled;
    s.menubar_style = self.style;
    s.menubar_glass = self.glass;
    s.menubar_material = self.material;
    s.clock_24h = self.clock_24h;
    s.battery_pct = self.battery_pct;
  }
}

impl From<&Settings> for FinderSettings {
  fn from(s: &Settings) -> Self {
    FinderSettings {
      enabled: s.menubar,
      style: s.menubar_style,
      glass: s.menubar_glass,
      material: s.menubar_material,
      clock_24h: s.clock_24h,
      battery_pct: s.battery_pct,
    }
  }
}

impl Settings {
  pub fn finder_settings(&self) -> FinderSettings {
    FinderSettings::from(self)
  }

  /// Former name of [`Settings::finder_settings`] (ADR-0003).
  pub fn menubar_settings(&self) -> FinderSettings {
    self.finder_settings()
  }
}
