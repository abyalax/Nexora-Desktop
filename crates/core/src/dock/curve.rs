//! Icon scale curve per effect (D-14.1, D-14.2).

use super::behavior::IconEffect;
use super::layout::falloff;

/// Scale of an icon whose centre is `distance` pixels from the pointer.
///
/// Only effects that magnify scale at all; `None` and `Select` always return `1.0`. The curve is symmetric
/// (`distance` is absolute), falls off monotonically and reaches `1.0` at `reach`. `zoom` is the peak scale.
pub fn scale_at(effect: IconEffect, zoom: f32, distance: f32, reach: f32) -> f32 {
  if !effect.magnifies() || !zoom.is_finite() || reach <= 0.0 {
    return 1.0;
  }
  1.0 + (zoom.max(1.0) - 1.0) * falloff(distance.abs(), reach)
}

/// Strength `0..=1` of the light glow under the pointer: full at the icon centre, none at `reach`.
pub fn glow_at(effect: IconEffect, distance: f32, reach: f32) -> f32 {
  if !effect.lights() || reach <= 0.0 {
    return 0.0;
  }
  falloff(distance.abs(), reach)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn effects_that_do_not_magnify_stay_at_one() {
    for e in [
      IconEffect::None,
      IconEffect::Select,
      IconEffect::Light,
      IconEffect::SelectLight,
    ] {
      assert_eq!(scale_at(e, 2.0, 0.0, 100.0), 1.0, "{e:?}");
    }
  }

  #[test]
  fn magnifying_effects_peak_at_zero_distance_and_end_at_reach() {
    for e in [IconEffect::Scaling, IconEffect::ScalingLight] {
      assert!((scale_at(e, 1.8, 0.0, 100.0) - 1.8).abs() < 1e-5);
      assert!((scale_at(e, 1.8, 100.0, 100.0) - 1.0).abs() < 1e-5);
      assert_eq!(scale_at(e, 1.8, 500.0, 100.0), 1.0);
    }
  }

  #[test]
  fn curve_is_monotone_and_symmetric() {
    let e = IconEffect::Scaling;
    let mut prev = f32::MAX;
    for d in (0..=100).step_by(5) {
      let v = scale_at(e, 2.0, d as f32, 100.0);
      assert!(v <= prev + 1e-6);
      assert!((v - scale_at(e, 2.0, -(d as f32), 100.0)).abs() < 1e-6);
      prev = v;
    }
  }

  #[test]
  fn bad_input_never_yields_nan_or_shrinks() {
    let e = IconEffect::Scaling;
    assert_eq!(scale_at(e, f32::NAN, 0.0, 100.0), 1.0);
    assert_eq!(scale_at(e, 2.0, 10.0, 0.0), 1.0);
    assert_eq!(scale_at(e, 0.2, 0.0, 100.0), 1.0);
  }

  #[test]
  fn glow_only_for_light_effects() {
    assert_eq!(glow_at(IconEffect::Scaling, 0.0, 100.0), 0.0);
    assert!((glow_at(IconEffect::Light, 0.0, 100.0) - 1.0).abs() < 1e-6);
    assert!(glow_at(IconEffect::ScalingLight, 100.0, 100.0) < 1e-6);
  }
}
