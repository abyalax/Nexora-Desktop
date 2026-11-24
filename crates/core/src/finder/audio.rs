//! Audio menu model (F-13): master volume, mute and the output device list. The platform (`platform/audio.rs`) reads
//! and writes the real values; this module decides how steps, mute and icons behave.

/// Largest volume.
pub const MAX: u8 = 100;

/// Which speaker glyph to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
  Muted,
  Low,
  Medium,
  High,
}

/// Master volume and mute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Audio {
  volume: u8,
  muted: bool,
}

impl Audio {
  pub fn new(volume: u8, muted: bool) -> Self {
    Audio {
      volume: volume.min(MAX),
      muted,
    }
  }

  pub fn volume(&self) -> u8 {
    self.volume
  }

  pub fn is_muted(&self) -> bool {
    self.muted
  }

  /// Sets the volume (clamped to 0–100). Moving a slider while muted unmutes, like the system volume does.
  pub fn set(&mut self, volume: u8) {
    self.volume = volume.min(MAX);
    if self.volume > 0 {
      self.muted = false;
    }
  }

  /// Changes the volume by `delta` percent points, clamped to 0–100. Raising it while muted unmutes.
  pub fn step(&mut self, delta: i16) {
    let v = (i16::from(self.volume) + delta).clamp(0, i16::from(MAX)) as u8;
    if delta > 0 {
      self.set(v);
    } else {
      self.volume = v;
    }
  }

  pub fn toggle_mute(&mut self) {
    self.muted = !self.muted;
  }

  /// The value the slider shows: zero while muted.
  pub fn shown(&self) -> u8 {
    if self.muted { 0 } else { self.volume }
  }

  pub fn level(&self) -> Level {
    match self.shown() {
      0 => Level::Muted,
      1..=33 => Level::Low,
      34..=66 => Level::Medium,
      _ => Level::High,
    }
  }
}

/// One output device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
  pub id: String,
  pub name: String,
}

/// The output devices and which one is the default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outputs {
  list: Vec<Output>,
  current: Option<usize>,
}

impl Outputs {
  /// Builds the list from the platform's devices; `default_id` marks the current one (unknown ids select nothing).
  pub fn new(list: Vec<Output>, default_id: &str) -> Self {
    let current = list.iter().position(|o| o.id == default_id);
    Outputs { list, current }
  }

  pub fn list(&self) -> &[Output] {
    &self.list
  }

  pub fn current(&self) -> Option<&Output> {
    self.current.map(|i| &self.list[i])
  }

  /// Selects a device by id. Returns `false` and keeps the selection when the id is unknown (it was unplugged).
  pub fn select(&mut self, id: &str) -> bool {
    match self.list.iter().position(|o| o.id == id) {
      Some(i) => {
        self.current = Some(i);
        true
      }
      None => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn volume_is_clamped_on_construction_and_set() {
    assert_eq!(Audio::new(250, false).volume(), 100);
    let mut a = Audio::new(10, false);
    a.set(180);
    assert_eq!(a.volume(), 100);
  }

  #[test]
  fn steps_clamp_at_both_ends() {
    let mut a = Audio::new(98, false);
    a.step(5);
    assert_eq!(a.volume(), 100);
    a.step(-250);
    assert_eq!(a.volume(), 0);
    a.step(-5);
    assert_eq!(a.volume(), 0);
  }

  #[test]
  fn raising_the_volume_unmutes_but_lowering_it_does_not() {
    let mut a = Audio::new(50, true);
    a.step(-10);
    assert!(a.is_muted());
    assert_eq!(a.volume(), 40);
    a.step(10);
    assert!(!a.is_muted());
    assert_eq!(a.volume(), 50);
  }

  #[test]
  fn setting_a_nonzero_volume_unmutes_and_zero_does_not() {
    let mut a = Audio::new(50, true);
    a.set(0);
    assert!(a.is_muted());
    a.set(20);
    assert!(!a.is_muted());
  }

  #[test]
  fn mute_toggles_and_keeps_the_volume() {
    let mut a = Audio::new(60, false);
    a.toggle_mute();
    assert_eq!((a.shown(), a.volume()), (0, 60));
    a.toggle_mute();
    assert_eq!(a.shown(), 60);
  }

  #[test]
  fn the_glyph_follows_the_shown_volume() {
    assert_eq!(Audio::new(0, false).level(), Level::Muted);
    assert_eq!(Audio::new(80, true).level(), Level::Muted);
    assert_eq!(Audio::new(1, false).level(), Level::Low);
    assert_eq!(Audio::new(33, false).level(), Level::Low);
    assert_eq!(Audio::new(34, false).level(), Level::Medium);
    assert_eq!(Audio::new(66, false).level(), Level::Medium);
    assert_eq!(Audio::new(67, false).level(), Level::High);
  }

  fn outputs() -> Outputs {
    let dev = |id: &str, name: &str| Output {
      id: id.into(),
      name: name.into(),
    };
    Outputs::new(vec![dev("a", "Speakers"), dev("b", "Headphones")], "b")
  }

  #[test]
  fn the_default_device_is_current() {
    assert_eq!(outputs().current().map(|o| o.name.as_str()), Some("Headphones"));
    assert_eq!(Outputs::new(vec![], "x").current(), None);
    assert_eq!(Outputs::new(outputs().list().to_vec(), "gone").current(), None);
  }

  #[test]
  fn selecting_an_unknown_device_keeps_the_selection() {
    let mut o = outputs();
    assert!(o.select("a"));
    assert_eq!(o.current().unwrap().id, "a");
    assert!(!o.select("unplugged"));
    assert_eq!(o.current().unwrap().id, "a");
  }
}
