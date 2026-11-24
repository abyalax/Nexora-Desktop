//! Preferensi perilaku dan tampilan Dock (model domain murni).
//!
//! Berisi hanya aturan dan nilai bawaan; tidak tahu cara Dock dirender atau
//! bagaimana Windows menempatkan jendelanya. Kunci `settings.cfg` memakai awalan
//! `dock_` sehingga berkas lama tetap terbaca dan nilai yang hilang jatuh ke
//! bawaan.

use crate::dock::overlap::Scope;
use crate::settings::parse_bool;

/// Efek ikon saat kursor melintas di Dock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconEffect {
  None,
  Scaling,
  Select,
  Light,
  ScalingLight,
  SelectLight,
}

impl IconEffect {
  pub const ALL: [IconEffect; 6] = [
    IconEffect::None,
    IconEffect::Scaling,
    IconEffect::Select,
    IconEffect::Light,
    IconEffect::ScalingLight,
    IconEffect::SelectLight,
  ];

  pub fn key(self) -> &'static str {
    match self {
      IconEffect::None => "none",
      IconEffect::Scaling => "scaling",
      IconEffect::Select => "select",
      IconEffect::Light => "light",
      IconEffect::ScalingLight => "scaling-light",
      IconEffect::SelectLight => "select-light",
    }
  }

  pub fn parse(s: &str) -> Option<IconEffect> {
    let s = s.trim().to_ascii_lowercase();
    Self::ALL.into_iter().find(|e| e.key() == s)
  }

  pub fn index(self) -> usize {
    Self::ALL.iter().position(|e| *e == self).unwrap_or(0)
  }

  /// Ikon membesar mengikuti kursor.
  pub fn magnifies(self) -> bool {
    matches!(self, IconEffect::Scaling | IconEffect::ScalingLight)
  }

  /// Ikon di bawah kursor diberi pelat latar.
  pub fn selects(self) -> bool {
    matches!(self, IconEffect::Select | IconEffect::SelectLight)
  }

  /// Ikon di bawah kursor memancarkan cahaya.
  pub fn lights(self) -> bool {
    matches!(
      self,
      IconEffect::Light | IconEffect::ScalingLight | IconEffect::SelectLight
    )
  }
}

/// Cara Dock menghilang saat disembunyikan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HideEffect {
  Move,
  Fade,
}

impl HideEffect {
  pub fn key(self) -> &'static str {
    match self {
      HideEffect::Move => "move",
      HideEffect::Fade => "fade",
    }
  }

  pub fn parse(s: &str) -> Option<HideEffect> {
    match s.trim().to_ascii_lowercase().as_str() {
      "move" | "geser" => Some(HideEffect::Move),
      "fade" | "pudar" => Some(HideEffect::Fade),
      _ => None,
    }
  }
}

/// Seluruh preferensi Dock di luar yang sudah ada di `Settings`
/// (ukuran, jarak, zoom, margin, material, perilaku tampil).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DockConfig {
  pub icon_effect: IconEffect,
  /// Intensitas cahaya ikon, 0..=100.
  pub light_intensity: u8,
  /// Intensitas cahaya latar ikon, 0..=100.
  pub bg_light_intensity: u8,
  pub light_rgb: [u8; 3],
  pub show_labels: bool,
  pub show_indicators: bool,
  pub show_unpinned: bool,
  pub separate_unpinned: bool,
  pub lock_drag: bool,
  pub animate_launch: bool,
  pub hide_effect: HideEffect,
  /// Tunda aktivasi Dock dari tepi layar, dalam kelipatan 10 ms.
  pub activation_delay: u8,
  pub preview_enabled: bool,
  /// Tunda pratinjau jendela saat hover, dalam kelipatan 10 ms.
  pub preview_delay: u8,
  /// Scope for overlap detection (DAD-13).
  pub overlap_scope: Scope,
}

impl Default for DockConfig {
  fn default() -> Self {
    DockConfig {
      icon_effect: IconEffect::Scaling,
      light_intensity: 60,
      bg_light_intensity: 30,
      light_rgb: [255, 255, 255],
      show_labels: true,
      show_indicators: true,
      show_unpinned: true,
      separate_unpinned: true,
      lock_drag: false,
      animate_launch: true,
      hide_effect: HideEffect::Move,
      activation_delay: 20,
      preview_enabled: true,
      preview_delay: 45,
      overlap_scope: Scope::All,
    }
  }
}

fn parse_level(v: &str, max: i32, old: u8) -> u8 {
  v.parse::<i32>().map(|n| n.clamp(0, max) as u8).unwrap_or(old)
}

fn parse_rgb(v: &str) -> Option<[u8; 3]> {
  let mut it = v.split(',').map(|p| p.trim().parse::<i32>());
  let (r, g, b) = (it.next()?.ok()?, it.next()?.ok()?, it.next()?.ok()?);
  if it.next().is_some() {
    return None;
  }
  Some([r.clamp(0, 255) as u8, g.clamp(0, 255) as u8, b.clamp(0, 255) as u8])
}

impl DockConfig {
  pub fn activation_delay_ms(&self) -> u32 {
    self.activation_delay as u32 * 10
  }

  pub fn preview_delay_ms(&self) -> u32 {
    self.preview_delay as u32 * 10
  }

  /// Skala maksimum ikon efektif: `zoom` hanya berlaku bila efeknya membesarkan ikon.
  pub fn effective_zoom(&self, zoom: f32) -> f32 {
    if self.icon_effect.magnifies() {
      zoom.max(1.0)
    } else {
      1.0
    }
  }

  /// Menerapkan satu pasangan kunci/nilai dari `settings.cfg`.
  /// Mengembalikan `false` bila kunci bukan milik Dock.
  pub fn apply(&mut self, key: &str, v: &str) -> bool {
    match key {
      "dock_icon_effect" => self.icon_effect = IconEffect::parse(v).unwrap_or(self.icon_effect),
      "dock_light" => self.light_intensity = parse_level(v, 100, self.light_intensity),
      "dock_bg_light" => self.bg_light_intensity = parse_level(v, 100, self.bg_light_intensity),
      "dock_light_color" => self.light_rgb = parse_rgb(v).unwrap_or(self.light_rgb),
      "dock_labels" => self.show_labels = parse_bool(v).unwrap_or(self.show_labels),
      "dock_indicators" => self.show_indicators = parse_bool(v).unwrap_or(self.show_indicators),
      "dock_unpinned" => self.show_unpinned = parse_bool(v).unwrap_or(self.show_unpinned),
      "dock_separate" => self.separate_unpinned = parse_bool(v).unwrap_or(self.separate_unpinned),
      "dock_lock" => self.lock_drag = parse_bool(v).unwrap_or(self.lock_drag),
      "dock_launch_anim" => self.animate_launch = parse_bool(v).unwrap_or(self.animate_launch),
      "dock_hide_effect" => self.hide_effect = HideEffect::parse(v).unwrap_or(self.hide_effect),
      "dock_wake_delay" => self.activation_delay = parse_level(v, 100, self.activation_delay),
      "dock_preview" => self.preview_enabled = parse_bool(v).unwrap_or(self.preview_enabled),
      "dock_preview_delay" => self.preview_delay = parse_level(v, 100, self.preview_delay),
      "dock_overlap_scope" => self.overlap_scope = Scope::from_index(v.parse().unwrap_or(0)),
      _ => return false,
    }
    true
  }

  /// Menambahkan baris `kunci=nilai` Dock ke keluaran `settings.cfg`.
  pub fn write_to(&self, out: &mut String) {
    use std::fmt::Write;
    let [r, g, b] = self.light_rgb;
    let scope_index = match self.overlap_scope {
      Scope::All => 0,
      Scope::Current => 1,
    };
    let _ = write!(
      out,
      "dock_icon_effect={}\ndock_light={}\ndock_bg_light={}\ndock_light_color={r},{g},{b}\n\
             dock_labels={}\ndock_indicators={}\ndock_unpinned={}\ndock_separate={}\n\
             dock_lock={}\ndock_launch_anim={}\ndock_hide_effect={}\ndock_wake_delay={}\n\
             dock_preview={}\ndock_preview_delay={}\ndock_overlap_scope={}\n",
      self.icon_effect.key(),
      self.light_intensity,
      self.bg_light_intensity,
      self.show_labels,
      self.show_indicators,
      self.show_unpinned,
      self.separate_unpinned,
      self.lock_drag,
      self.animate_launch,
      self.hide_effect.key(),
      self.activation_delay,
      self.preview_enabled,
      self.preview_delay,
      scope_index,
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn roundtrip(c: &DockConfig) -> DockConfig {
    let mut text = String::new();
    c.write_to(&mut text);
    let mut out = DockConfig::default();
    for line in text.lines() {
      let (k, v) = line.split_once('=').unwrap();
      assert!(out.apply(k, v), "kunci tidak dikenal: {k}");
    }
    out
  }

  #[test]
  fn defaults_keep_current_behaviour() {
    let d = DockConfig::default();
    assert_eq!(d.icon_effect, IconEffect::Scaling);
    assert_eq!(d.activation_delay_ms(), 200);
    assert_eq!(d.preview_delay_ms(), 450);
    assert!(d.show_labels && d.show_indicators && d.show_unpinned && d.separate_unpinned);
    assert!(!d.lock_drag && d.preview_enabled && d.animate_launch);
  }

  #[test]
  fn serialize_roundtrips_every_field() {
    let c = DockConfig {
      icon_effect: IconEffect::SelectLight,
      light_intensity: 12,
      bg_light_intensity: 88,
      light_rgb: [1, 2, 3],
      show_labels: false,
      show_indicators: false,
      show_unpinned: false,
      separate_unpinned: false,
      lock_drag: true,
      animate_launch: false,
      hide_effect: HideEffect::Fade,
      activation_delay: 7,
      preview_enabled: false,
      preview_delay: 99,
      overlap_scope: Scope::Current,
    };
    assert_eq!(roundtrip(&c), c);
  }

  #[test]
  fn invalid_values_keep_previous_and_levels_clamp() {
    let mut c = DockConfig::default();
    assert!(c.apply("dock_icon_effect", "banana"));
    assert!(c.apply("dock_light", "999"));
    assert!(c.apply("dock_bg_light", "-4"));
    assert!(c.apply("dock_light_color", "1,2"));
    assert!(c.apply("dock_light_color", "300,-5,7"));
    assert!(c.apply("dock_labels", "maybe"));
    let d = DockConfig::default();
    assert_eq!(c.icon_effect, d.icon_effect);
    assert_eq!(c.light_intensity, 100);
    assert_eq!(c.bg_light_intensity, 0);
    assert_eq!(c.light_rgb, [255, 0, 7]);
    assert_eq!(c.show_labels, d.show_labels);
  }

  #[test]
  fn foreign_keys_are_not_claimed() {
    let mut c = DockConfig::default();
    assert!(!c.apply("dock_size", "50"));
    assert!(!c.apply("notch", "smart"));
  }

  #[test]
  fn effect_flags_are_consistent() {
    for e in IconEffect::ALL {
      assert_eq!(IconEffect::parse(e.key()), Some(e));
      assert_eq!(IconEffect::ALL[e.index()], e);
    }
    assert!(!IconEffect::None.magnifies() && !IconEffect::None.lights());
    assert!(IconEffect::ScalingLight.magnifies() && IconEffect::ScalingLight.lights());
    assert!(IconEffect::SelectLight.selects() && IconEffect::SelectLight.lights());
    assert!(!IconEffect::Select.magnifies() && !IconEffect::Select.lights());
  }

  #[test]
  fn zoom_only_applies_when_effect_magnifies() {
    let mut c = DockConfig::default();
    assert_eq!(c.effective_zoom(2.0), 2.0);
    c.icon_effect = IconEffect::Select;
    assert_eq!(c.effective_zoom(2.0), 1.0);
    c.icon_effect = IconEffect::ScalingLight;
    assert_eq!(c.effective_zoom(0.5), 1.0);
  }
}
