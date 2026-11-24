//! Window opening animation (WAN-01…WAN-06): effect shapes and the decision to animate.
//!
//! `frame` returns plain numbers; `app/fx.rs` and `app/motion.rs` apply them to a window. Speed presets and the
//! duration function are shared with the minimize animation (`dock::minimize`).

use crate::dock::minimize::{duration_ms, speed_from_index};
use crate::settings::{ListKind, ProgramLists};

/// WAN-02 (stored: `scale`, `fade`, `slide`, `zoom`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
  Scale,
  Fade,
  Slide,
  Zoom,
}

impl Effect {
  /// From the stored index. Unknown values mean `Scale`.
  pub fn from_index(i: u8) -> Effect {
    match i {
      1 => Effect::Fade,
      2 => Effect::Slide,
      3 => Effect::Zoom,
      _ => Effect::Scale,
    }
  }
}

/// Visual state of the window at one moment of the animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
  /// Size factor, 1.0 = final size.
  pub scale: f32,
  /// 0.0 transparent … 1.0 opaque.
  pub opacity: f32,
  /// Vertical offset in pixels, 0 = final position (positive = lower).
  pub dy: f32,
}

/// Base length of the opening animation at normal speed.
pub const BASE_MS: u32 = 220;

fn ease_out(t: f32) -> f32 {
  let t = 1.0 - t;
  1.0 - t * t * t
}

/// State at progress `t` (0 = start, 1 = end; out-of-range and NaN are clamped). At `t = 1` every effect is
/// the identity frame, so the window always ends exactly where the system placed it.
pub fn frame(effect: Effect, t: f32) -> Frame {
  let t = if t.is_nan() { 1.0 } else { t.clamp(0.0, 1.0) };
  let e = ease_out(t);
  let lerp = |from: f32, to: f32| from + (to - from) * e;
  match effect {
    Effect::Scale => Frame {
      scale: lerp(0.92, 1.0),
      opacity: lerp(0.0, 1.0),
      dy: 0.0,
    },
    Effect::Fade => Frame {
      scale: 1.0,
      opacity: lerp(0.0, 1.0),
      dy: 0.0,
    },
    Effect::Slide => Frame {
      scale: 1.0,
      opacity: lerp(0.0, 1.0),
      dy: lerp(24.0, 0.0),
    },
    Effect::Zoom => Frame {
      scale: lerp(0.6, 1.0),
      opacity: lerp(0.0, 1.0),
      dy: 0.0,
    },
  }
}

/// Length in ms for the stored speed preset (WAN-03).
pub fn duration(speed_index: u8) -> u32 {
  duration_ms(BASE_MS, speed_from_index(speed_index), false, false)
}

/// Whether a window opening is animated: the global switch is on and the program is not excluded (WAN-04/06).
/// The running-program blacklist (WAN-05) also blocks the animation.
pub fn should_animate(global: bool, lists: &ProgramLists, exe: &str) -> bool {
  global && !lists.contains(ListKind::ExcludedPrograms, exe) && !lists.contains(ListKind::ProgramBlacklist, exe)
}

#[cfg(test)]
mod tests {
  use super::*;

  const ALL: [Effect; 4] = [Effect::Scale, Effect::Fade, Effect::Slide, Effect::Zoom];

  #[test]
  fn index_maps_and_unknown_is_scale() {
    let e: Vec<_> = (0..4).map(Effect::from_index).collect();
    assert_eq!(e, ALL);
    assert_eq!(Effect::from_index(50), Effect::Scale);
  }

  #[test]
  fn every_effect_ends_on_the_identity_frame() {
    for e in ALL {
      assert_eq!(
        frame(e, 1.0),
        Frame {
          scale: 1.0,
          opacity: 1.0,
          dy: 0.0
        },
        "{e:?}"
      );
    }
  }

  #[test]
  fn every_effect_starts_invisible() {
    for e in ALL {
      assert_eq!(frame(e, 0.0).opacity, 0.0, "{e:?}");
    }
  }

  #[test]
  fn opacity_and_scale_never_decrease_and_stay_finite() {
    for e in ALL {
      let mut prev = frame(e, 0.0);
      for i in 1..=100 {
        let f = frame(e, i as f32 / 100.0);
        assert!(f.scale.is_finite() && f.opacity.is_finite() && f.dy.is_finite());
        assert!(
          f.opacity >= prev.opacity && f.scale >= prev.scale && f.dy <= prev.dy,
          "{e:?} step {i}"
        );
        assert!((0.0..=1.0).contains(&f.opacity));
        prev = f;
      }
    }
  }

  #[test]
  fn progress_outside_the_range_is_clamped() {
    for e in ALL {
      assert_eq!(frame(e, -3.0), frame(e, 0.0));
      assert_eq!(frame(e, 9.0), frame(e, 1.0));
      assert_eq!(
        frame(e, f32::NAN),
        frame(e, 1.0),
        "NaN jumps to the end instead of poisoning the window"
      );
    }
  }

  #[test]
  fn duration_follows_the_speed_presets() {
    assert_eq!(duration(2), BASE_MS);
    assert!(duration(4) < duration(2) && duration(0) > duration(2));
    assert_eq!(duration(99), BASE_MS, "unknown preset means normal speed");
  }

  #[test]
  fn excluded_and_blacklisted_programs_do_not_animate() {
    let mut l = ProgramLists::default();
    l.add(ListKind::ExcludedPrograms, "game.exe");
    l.add(ListKind::ProgramBlacklist, "tool.exe");
    assert!(should_animate(true, &l, "notepad.exe"));
    assert!(!should_animate(true, &l, "GAME"));
    assert!(!should_animate(true, &l, "tool.exe"));
    assert!(!should_animate(false, &l, "notepad.exe"), "global off wins");
  }
}
