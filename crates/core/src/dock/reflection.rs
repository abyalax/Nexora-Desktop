//! Pinned-icon reflection parameters (D-15.2). Pure derivation; the renderer lives in `ui`.

/// Raw user values (APP-05 to APP-08).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Raw {
  pub blur: u16,
  pub transparency: u16,
  pub distance: u16,
  pub gradient: u16,
}

/// Clamped values ready for drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
  /// Blur radius in logical pixels, `0..=30`.
  pub blur_px: f32,
  /// Reflection opacity at its top, `0.0..=1.0` (transparency 100 means invisible).
  pub opacity: f32,
  /// Gap between the icon and its reflection, `0..=20` px.
  pub offset_px: f32,
  /// Length over which the reflection fades out, `0..=30` px.
  pub fade_px: f32,
}

pub fn derive(raw: Raw) -> Params {
  Params {
    blur_px: raw.blur.min(30) as f32,
    opacity: 1.0 - raw.transparency.min(100) as f32 / 100.0,
    offset_px: raw.distance.min(20) as f32,
    fade_px: raw.gradient.min(30) as f32,
  }
}

/// Opacity of the reflection `y` pixels below its top edge: linear fade to `0` over `fade_px`.
pub fn opacity_at(p: &Params, y: f32) -> f32 {
  if p.fade_px <= 0.0 {
    return if y <= 0.0 { p.opacity } else { 0.0 };
  }
  (p.opacity * (1.0 - y / p.fade_px)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_derive_expected_values() {
    let p = derive(Raw {
      blur: 5,
      transparency: 35,
      distance: 3,
      gradient: 8,
    });
    assert_eq!((p.blur_px, p.offset_px, p.fade_px), (5.0, 3.0, 8.0));
    assert!((p.opacity - 0.65).abs() < 1e-6);
  }

  #[test]
  fn out_of_range_values_are_clamped() {
    let p = derive(Raw {
      blur: 999,
      transparency: 999,
      distance: 999,
      gradient: 999,
    });
    assert_eq!((p.blur_px, p.opacity, p.offset_px, p.fade_px), (30.0, 0.0, 20.0, 30.0));
  }

  #[test]
  fn fade_is_linear_monotone_and_ends_at_zero() {
    let p = derive(Raw {
      blur: 0,
      transparency: 0,
      distance: 0,
      gradient: 10,
    });
    assert_eq!(opacity_at(&p, 0.0), 1.0);
    assert!((opacity_at(&p, 5.0) - 0.5).abs() < 1e-6);
    assert_eq!(opacity_at(&p, 10.0), 0.0);
    assert_eq!(opacity_at(&p, 99.0), 0.0);
  }

  #[test]
  fn zero_gradient_has_no_division_by_zero() {
    let p = derive(Raw {
      blur: 0,
      transparency: 50,
      distance: 0,
      gradient: 0,
    });
    assert_eq!(opacity_at(&p, 0.0), 0.5);
    assert_eq!(opacity_at(&p, 1.0), 0.0);
  }
}
