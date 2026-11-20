//! Persistent Dock preferences, projected from the flat on-disk [`Settings`].
//!
//! `DockSettings` is configuration; the runtime Dock state lives in `crate::dock`.

use super::{Behavior, Material, Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DockSettings {
  pub enabled: bool,
  pub behavior: Behavior,
  pub size: u8,
  pub gap: u8,
  pub zoom: u8,
  pub margin: u8,
  pub material: Material,
  pub round: u8,
  pub plates: bool,
  pub min_effect: u8,
  pub recent_apps: bool,
  pub downloads_stack: bool,
}

impl DockSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.dock_enabled = self.enabled;
    s.dock = self.behavior;
    s.dock_size = self.size;
    s.dock_gap = self.gap;
    s.dock_zoom = self.zoom;
    s.dock_margin = self.margin;
    s.dock_material = self.material;
    s.dock_round = self.round;
    s.dock_plates = self.plates;
    s.min_effect = self.min_effect;
    s.recent_apps = self.recent_apps;
    s.downloads_stack = self.downloads_stack;
  }
}

impl From<&Settings> for DockSettings {
  fn from(s: &Settings) -> Self {
    DockSettings {
      enabled: s.dock_enabled,
      behavior: s.dock,
      size: s.dock_size,
      gap: s.dock_gap,
      zoom: s.dock_zoom,
      margin: s.dock_margin,
      material: s.dock_material,
      round: s.dock_round,
      plates: s.dock_plates,
      min_effect: s.min_effect,
      recent_apps: s.recent_apps,
      downloads_stack: s.downloads_stack,
    }
  }
}

impl Settings {
  pub fn dock_settings(&self) -> DockSettings {
    DockSettings::from(self)
  }
}
