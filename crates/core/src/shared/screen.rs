//! Screen page curves (SCR-01…SCR-04): brightness overlay and night light.
//!
//! Pure mapping from the stored sliders to overlay colour and alpha, or to per-channel gamma gain. The overlay
//! window (layered, click-through) or the gamma ramp is OS work and needs a restore path (REC-01).

use super::monitors::{Monitor, pick};

/// The brightness overlay never goes fully black, so the screen cannot be made unreadable by accident.
pub const MAX_DIM: f32 = 0.85;

/// Overlay alpha for the brightness slider (0–100). 100 = no overlay, 0 = [`MAX_DIM`].
pub fn dim_alpha(brightness: u16) -> f32 {
  (1.0 - brightness.min(100) as f32 / 100.0) * MAX_DIM
}

/// Warm tint colour of the night light overlay (RGB).
pub const WARM_RGB: [u8; 3] = [255, 147, 41];

/// Strongest tint alpha at intensity 100.
pub const MAX_TINT: f32 = 0.45;

/// Night light overlay as `(rgb, alpha)`, or `None` when it is off or the intensity is 0.
pub fn night_light(enabled: bool, intensity: u16) -> Option<([u8; 3], f32)> {
  let i = intensity.min(100);
  (enabled && i > 0).then(|| (WARM_RGB, i as f32 / 100.0 * MAX_TINT))
}

/// Gamma-ramp gain per channel `[r, g, b]` for the night light. 1.0 = unchanged. Red never changes, so the
/// ramp only removes blue and some green.
pub fn gamma_gain(enabled: bool, intensity: u16) -> [f32; 3] {
  if !enabled {
    return [1.0; 3];
  }
  let k = intensity.min(100) as f32 / 100.0;
  [1.0, 1.0 - 0.22 * k, 1.0 - 0.55 * k]
}

/// SCR-04 (stored: `all`, `primary`, `active`): monitor indices that get the effect.
/// `active` is the monitor that holds the pointer or the foreground window.
pub fn targets(choice: u8, monitors: &[Monitor<'_>], active: Option<usize>) -> Vec<usize> {
  match choice {
    0 => (0..monitors.len()).collect(),
    2 => active
      .filter(|i| *i < monitors.len())
      .or_else(|| pick(monitors, ""))
      .into_iter()
      .collect(),
    _ => pick(monitors, "").into_iter().collect(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::shared::geometry::Rect;

  fn mons() -> Vec<Monitor<'static>> {
    vec![
      Monitor {
        id: "D1",
        bounds: Rect::new(0.0, 0.0, 1920.0, 1080.0),
        primary: false,
      },
      Monitor {
        id: "D2",
        bounds: Rect::new(1920.0, 0.0, 1920.0, 1080.0),
        primary: true,
      },
    ]
  }

  #[test]
  fn full_brightness_draws_nothing_and_zero_keeps_the_screen_readable() {
    assert_eq!(dim_alpha(100), 0.0);
    assert_eq!(dim_alpha(0), MAX_DIM);
    assert_eq!(dim_alpha(500), 0.0, "out of range clamps");
  }

  #[test]
  fn dim_alpha_is_monotone_and_finite() {
    let mut prev = f32::MAX;
    for b in 0..=100 {
      let a = dim_alpha(b);
      assert!(a.is_finite() && a <= prev && (0.0..=MAX_DIM).contains(&a));
      prev = a;
    }
  }

  #[test]
  fn night_light_is_off_when_disabled_or_zero() {
    assert_eq!(night_light(false, 80), None);
    assert_eq!(night_light(true, 0), None);
    let (rgb, a) = night_light(true, 100).unwrap();
    assert_eq!((rgb, a), (WARM_RGB, MAX_TINT));
    assert!(night_light(true, 50).unwrap().1 < a);
    assert_eq!(night_light(true, 999).unwrap().1, MAX_TINT);
  }

  #[test]
  fn gamma_gain_is_identity_when_off_and_never_touches_red() {
    assert_eq!(gamma_gain(false, 100), [1.0; 3]);
    assert_eq!(gamma_gain(true, 0), [1.0; 3]);
    let g = gamma_gain(true, 100);
    assert_eq!(g[0], 1.0);
    assert!(g[1] < 1.0 && g[2] < g[1] && g[2] > 0.0);
  }

  #[test]
  fn targets_follow_the_choice() {
    assert_eq!(targets(0, &mons(), None), [0, 1]);
    assert_eq!(targets(1, &mons(), Some(0)), [1], "primary, not active");
    assert_eq!(targets(2, &mons(), Some(0)), [0]);
    assert_eq!(
      targets(2, &mons(), None),
      [1],
      "no active monitor falls back to primary"
    );
    assert_eq!(targets(2, &mons(), Some(9)), [1], "stale index falls back to primary");
    assert!(targets(1, &[], None).is_empty());
  }
}
