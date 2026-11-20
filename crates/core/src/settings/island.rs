//! Persistent Dynamic Island preferences, projected from the flat on-disk [`Settings`].
//!
//! `IslandSettings` is configuration; the runtime state machine lives in `crate::dynamic_island`.

use super::{Behavior, Material, Settings, ShortText};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandSettings {
  pub enabled: bool,
  pub behavior: Behavior,
  /// `true` = floating capsule, `false` = attached to the top edge.
  pub capsule: bool,
  pub material: Material,
  pub round: u8,
  pub idle: u8,
  pub hover_open: bool,
  pub greet_name: ShortText,
  pub greet_text: ShortText,
  pub lyrics_compact: bool,
  pub lyrics_online: bool,
  pub downloads: bool,
  pub send: bool,
}

impl IslandSettings {
  /// Write this section back into the flat settings; fields owned by other sections are untouched.
  pub fn apply(&self, s: &mut Settings) {
    s.notch_enabled = self.enabled;
    s.notch = self.behavior;
    s.notch_island = self.capsule;
    s.notch_material = self.material;
    s.notch_round = self.round;
    s.notch_idle = self.idle;
    s.notch_hover_open = self.hover_open;
    s.greet_name = self.greet_name;
    s.greet_text = self.greet_text;
    s.lyrics_compact = self.lyrics_compact;
    s.lyrics_online = self.lyrics_online;
    s.notch_downloads = self.downloads;
    s.notch_send = self.send;
  }
}

impl From<&Settings> for IslandSettings {
  fn from(s: &Settings) -> Self {
    IslandSettings {
      enabled: s.notch_enabled,
      behavior: s.notch,
      capsule: s.notch_island,
      material: s.notch_material,
      round: s.notch_round,
      idle: s.notch_idle,
      hover_open: s.notch_hover_open,
      greet_name: s.greet_name,
      greet_text: s.greet_text,
      lyrics_compact: s.lyrics_compact,
      lyrics_online: s.lyrics_online,
      downloads: s.notch_downloads,
      send: s.notch_send,
    }
  }
}

impl Settings {
  pub fn island_settings(&self) -> IslandSettings {
    IslandSettings::from(self)
  }
}
