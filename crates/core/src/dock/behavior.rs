//! Enum perilaku dan tampilan Dock (model domain murni).
//!
//! Berisi hanya aturan; tidak tahu cara Dock dirender atau bagaimana Windows menempatkan jendelanya. Nilai
//! tersimpannya ada di `settings::extra` (ADR-0004, tabel `dock_*`); struct `DockConfig` lama sudah dihapus (BASE-02).

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

  /// Skala maksimum ikon efektif: `zoom` hanya berlaku bila efeknya membesarkan ikon.
  pub fn effective_zoom(self, zoom: f32) -> f32 {
    if self.magnifies() { zoom.max(1.0) } else { 1.0 }
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

#[cfg(test)]
mod tests {
  use super::*;

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
    assert_eq!(IconEffect::Scaling.effective_zoom(2.0), 2.0);
    assert_eq!(IconEffect::Select.effective_zoom(2.0), 1.0);
    assert_eq!(IconEffect::ScalingLight.effective_zoom(0.5), 1.0);
  }

  #[test]
  fn hide_effect_parses_english_and_indonesian_names() {
    assert_eq!(HideEffect::parse("Geser"), Some(HideEffect::Move));
    assert_eq!(HideEffect::parse("fade"), Some(HideEffect::Fade));
    assert_eq!(HideEffect::parse(HideEffect::Fade.key()), Some(HideEffect::Fade));
    assert_eq!(HideEffect::parse("banana"), None);
  }
}
