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

/// Animation speed presets of STG-08 (0.5x … 1.5x), as percentages of `speed`. The file keeps the percentage
/// (`stage_speed`, 50–200), so old files and hand-edited values stay valid; the settings row shows the nearest preset.
pub const SPEED_PRESETS: [u8; 5] = [50, 75, 100, 125, 150];

impl StageSettings {
  /// Index in [`SPEED_PRESETS`] closest to the stored percentage (ties go to the slower preset).
  pub fn speed_preset(&self) -> usize {
    let mut best = 0;
    for (i, p) in SPEED_PRESETS.iter().enumerate() {
      if self.speed.abs_diff(*p) < self.speed.abs_diff(SPEED_PRESETS[best]) {
        best = i;
      }
    }
    best
  }

  /// Store the preset at `index`; an out-of-range index selects the last preset.
  pub fn set_speed_preset(&mut self, index: usize) {
    self.speed = SPEED_PRESETS[index.min(SPEED_PRESETS.len() - 1)];
  }

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

#[cfg(test)]
mod tests {
  use super::*;

  fn with_speed(speed: u8) -> StageSettings {
    StageSettings::from(&Settings {
      stage_speed: speed,
      ..Settings::default()
    })
  }

  #[test]
  fn default_speed_is_the_1x_preset() {
    assert_eq!(StageSettings::from(&Settings::default()).speed_preset(), 2);
  }

  #[test]
  fn every_preset_round_trips_through_the_stored_percentage() {
    for (i, preset) in SPEED_PRESETS.iter().enumerate() {
      let mut st = with_speed(100);
      st.set_speed_preset(i);
      assert_eq!(st.speed_preset(), i);
      let mut flat = Settings::default();
      st.apply(&mut flat);
      assert_eq!(Settings::parse(&flat.serialize()).stage_speed, *preset);
    }
  }

  #[test]
  fn off_preset_values_pick_the_nearest_and_extremes_clamp() {
    assert_eq!(with_speed(50).speed_preset(), 0);
    assert_eq!(with_speed(60).speed_preset(), 0);
    assert_eq!(with_speed(63).speed_preset(), 1);
    assert_eq!(with_speed(200).speed_preset(), 4);
    assert_eq!(with_speed(87).speed_preset(), 1, "tie goes to the slower preset");
    let mut st = with_speed(100);
    st.set_speed_preset(99);
    assert_eq!(st.speed, 150);
  }
}
