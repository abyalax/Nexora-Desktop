//! Minimize effect model (D-7): effect names, the Suck path, animation duration, the blacklist and where minimized
//! windows sit in the strip. Pure logic; the window host (`app::fx`) draws what these functions describe.

use crate::settings::{ListKind, ProgramLists};
use crate::shared::geometry::Rect;

/// DMN-07. The on-disk value is the legacy `min_effect` key: `0` and `1` keep the meaning they had in 0.4.0
/// (0 = the funnel/genie slicing in `window_management::genie`, 1 = plain scale); `2` is new.
///
/// Naming (D-7.0): the Settings window calls effect 0 *Funnel*, the spec calls it *Genie*. They are the same effect,
/// `window_management::genie` is its implementation, and *Genie* is the name in new labels (ADR-0010).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
  Genie,
  Scale,
  Suck,
}

impl Effect {
  pub const ALL: [Effect; 3] = [Effect::Genie, Effect::Scale, Effect::Suck];

  /// From the stored `min_effect` value. Anything unknown is the default effect.
  pub fn from_stored(v: u8) -> Effect {
    match v {
      1 => Effect::Scale,
      2 => Effect::Suck,
      _ => Effect::Genie,
    }
  }

  pub fn stored(self) -> u8 {
    match self {
      Effect::Genie => 0,
      Effect::Scale => 1,
      Effect::Suck => 2,
    }
  }

  pub fn key(self) -> &'static str {
    match self {
      Effect::Genie => "genie",
      Effect::Scale => "scale",
      Effect::Suck => "suck",
    }
  }

  /// Accepts the old UI names too (`funnel`, `skala`).
  pub fn parse(s: &str) -> Option<Effect> {
    match s.trim().to_ascii_lowercase().as_str() {
      "genie" | "funnel" => Some(Effect::Genie),
      "scale" | "skala" => Some(Effect::Scale),
      "suck" => Some(Effect::Suck),
      _ => None,
    }
  }
}

/// Window pose during the Suck effect: centre, scale relative to the window's own size, and opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
  pub cx: f32,
  pub cy: f32,
  pub sx: f32,
  pub sy: f32,
  pub opacity: f32,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

/// Pose at animation time `t` in 0..=1 when the window `win` is sucked into `icon` (`restore` plays it backwards).
/// `t = 0` is the window as it is, `t = 1` is a faded-out window at the icon. Every output is finite for any input
/// rectangle, including zero sizes, and distance to the icon, scale and opacity never increase as `t` grows.
pub fn suck_pose(t: f32, restore: bool, win: Rect, icon: Rect) -> Pose {
  let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
  let t = if restore { 1.0 - t } else { t };
  // Ease-in: starts slowly, then accelerates into the icon.
  let p = t * t * t * (3.0 - 2.0 * t);
  let (ww, wh) = (win.w.max(1.0), win.h.max(1.0));
  let end_sx = (icon.w / ww).clamp(0.0, 1.0);
  let end_sy = (icon.h / wh).clamp(0.0, 1.0);
  Pose {
    cx: lerp(win.x + win.w / 2.0, icon.x + icon.w / 2.0, p),
    cy: lerp(win.y + win.h / 2.0, icon.y + icon.h / 2.0, p),
    sx: lerp(1.0, end_sx, p),
    sy: lerp(1.0, end_sy, p),
    opacity: 1.0 - p,
  }
}

/// DMN-08 option list: `0.5x, 0.75x, 1.0x, 1.25x, 1.5x`. Out-of-range index means normal speed.
pub fn speed_from_index(i: u8) -> f32 {
  [0.5, 0.75, 1.0, 1.25, 1.5].get(i as usize).copied().unwrap_or(1.0)
}

/// How much slower an animation runs while Shift is held and DMN-03 is on.
pub const SHIFT_SLOW: f32 = 4.0;

/// Animation length. A higher `speed` shortens it; holding Shift (when allowed) stretches it by [`SHIFT_SLOW`].
/// Never below 1 ms, so a timer never gets a zero interval.
pub fn duration_ms(base_ms: u32, speed: f32, shift_held: bool, shift_slow_enabled: bool) -> u32 {
  let speed = if speed.is_finite() && speed > 0.0 { speed } else { 1.0 };
  let slow = if shift_held && shift_slow_enabled {
    SHIFT_SLOW
  } else {
    1.0
  };
  ((base_ms as f32 / speed) * slow).round().clamp(1.0, 600_000.0) as u32
}

/// DMN-09: is this program on the minimize-animation blacklist (matched by executable name)?
pub fn is_blacklisted(lists: &ProgramLists, exe: &str) -> bool {
  lists.contains(ListKind::MinimizeBlacklist, exe)
}

/// D-7.5: whether a minimize or restore plays the Nexora animation. When `false` the window minimizes the plain
/// Windows way. `global_anim` is the master animation switch, `use_nexora` is DMN-01, `disabled` is DMN-02.
pub fn should_animate(global_anim: bool, use_nexora: bool, disabled: bool, blacklisted: bool) -> bool {
  global_anim && use_nexora && !disabled && !blacklisted
}

/// DMN-06, from the stored option index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconPosition {
  /// Right after the icon of the program the window belongs to.
  NextToIcon,
  /// All minimized windows together at the right end of the programs.
  Independent,
  /// Like `Independent`, with a separator in front of the group.
  Separator,
}

impl IconPosition {
  pub fn from_index(i: u8) -> IconPosition {
    match i {
      1 => IconPosition::Independent,
      2 => IconPosition::Separator,
      _ => IconPosition::NextToIcon,
    }
  }
}

/// One cell of the program area, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
  /// Index of a program, as in `Entry::App`.
  App(usize),
  /// The `win`-th minimized window of program `app`.
  Minimized {
    app: usize,
    win: usize,
  },
  Separator,
}

/// Lays out the programs and their minimized windows (D-7.6). `minimized[i]` is how many minimized windows program `i`
/// has; the programs keep their order in every mode.
pub fn place(minimized: &[usize], pos: IconPosition) -> Vec<Placement> {
  let mut out = Vec::new();
  let all = |out: &mut Vec<Placement>| {
    for (app, &n) in minimized.iter().enumerate() {
      for win in 0..n {
        out.push(Placement::Minimized { app, win });
      }
    }
  };
  match pos {
    IconPosition::NextToIcon => {
      for (app, &n) in minimized.iter().enumerate() {
        out.push(Placement::App(app));
        for win in 0..n {
          out.push(Placement::Minimized { app, win });
        }
      }
    }
    IconPosition::Independent | IconPosition::Separator => {
      out.extend((0..minimized.len()).map(Placement::App));
      if pos == IconPosition::Separator && minimized.iter().any(|&n| n > 0) {
        out.push(Placement::Separator);
      }
      all(&mut out);
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  const WIN: Rect = Rect {
    x: 100.0,
    y: 100.0,
    w: 800.0,
    h: 600.0,
  };
  const ICON: Rect = Rect {
    x: 500.0,
    y: 1000.0,
    w: 48.0,
    h: 48.0,
  };

  #[test]
  fn legacy_values_keep_their_meaning() {
    assert_eq!(Effect::from_stored(0), Effect::Genie);
    assert_eq!(Effect::from_stored(1), Effect::Scale);
    assert_eq!(Effect::from_stored(2), Effect::Suck);
    assert_eq!(Effect::from_stored(200), Effect::Genie);
    for e in Effect::ALL {
      assert_eq!(Effect::from_stored(e.stored()), e);
      assert_eq!(Effect::parse(e.key()), Some(e));
    }
    assert_eq!(Effect::parse(" Funnel "), Some(Effect::Genie));
    assert_eq!(Effect::parse("Skala"), Some(Effect::Scale));
    assert_eq!(Effect::parse("???"), None);
  }

  #[test]
  fn suck_starts_at_the_window_and_ends_faded_at_the_icon() {
    let a = suck_pose(0.0, false, WIN, ICON);
    assert_eq!((a.cx, a.cy, a.sx, a.sy, a.opacity), (500.0, 400.0, 1.0, 1.0, 1.0));
    let b = suck_pose(1.0, false, WIN, ICON);
    assert_eq!((b.cx, b.cy), (524.0, 1024.0));
    assert!((b.sx - 0.06).abs() < 1e-6 && (b.sy - 0.08).abs() < 1e-6);
    assert_eq!(b.opacity, 0.0);
  }

  #[test]
  fn suck_is_monotone() {
    let dist = |p: &Pose| ((p.cx - 524.0).powi(2) + (p.cy - 1024.0).powi(2)).sqrt();
    let mut prev = suck_pose(0.0, false, WIN, ICON);
    for k in 1..=100 {
      let cur = suck_pose(k as f32 / 100.0, false, WIN, ICON);
      assert!(dist(&cur) <= dist(&prev) + 1e-3, "distance grew at step {k}");
      assert!(cur.sx <= prev.sx && cur.sy <= prev.sy && cur.opacity <= prev.opacity);
      prev = cur;
    }
  }

  #[test]
  fn restore_is_the_same_path_backwards() {
    for k in 0..=10 {
      let t = k as f32 / 10.0;
      assert_eq!(suck_pose(t, true, WIN, ICON), suck_pose(1.0 - t, false, WIN, ICON));
    }
  }

  #[test]
  fn suck_is_finite_for_degenerate_input() {
    let zero = Rect {
      x: 0.0,
      y: 0.0,
      w: 0.0,
      h: 0.0,
    };
    for t in [-5.0, 0.0, 0.3, 1.0, 7.0, f32::NAN, f32::INFINITY] {
      for (w, i) in [(zero, zero), (WIN, zero), (zero, ICON)] {
        let p = suck_pose(t, false, w, i);
        assert!(
          [p.cx, p.cy, p.sx, p.sy, p.opacity].iter().all(|v| v.is_finite()),
          "t={t}"
        );
        assert!((0.0..=1.0).contains(&p.opacity) && p.sx >= 0.0 && p.sy >= 0.0);
      }
    }
  }

  #[test]
  fn speed_options_map_to_multipliers() {
    let v: Vec<f32> = (0..5).map(speed_from_index).collect();
    assert_eq!(v, [0.5, 0.75, 1.0, 1.25, 1.5]);
    assert_eq!(speed_from_index(9), 1.0);
  }

  #[test]
  fn duration_scales_with_speed_and_shift() {
    assert_eq!(duration_ms(400, 1.0, false, true), 400);
    assert_eq!(duration_ms(400, 0.5, false, true), 800);
    assert_eq!(duration_ms(400, 2.0, false, true), 200);
    assert_eq!(duration_ms(400, 1.0, true, true), 1600);
    assert_eq!(
      duration_ms(400, 1.0, true, false),
      400,
      "shift ignored when DMN-03 is off"
    );
    assert_eq!(duration_ms(400, 1.0, false, true), duration_ms(400, 1.0, true, false));
    assert!(duration_ms(400, 1.5, false, true) < duration_ms(400, 1.0, false, true));
    assert_eq!(duration_ms(0, 1.0, false, false), 1);
    assert_eq!(duration_ms(400, 0.0, false, false), 400, "bad speed means normal speed");
    assert_eq!(duration_ms(400, f32::NAN, false, false), 400);
  }

  #[test]
  fn blacklist_matches_by_executable_name_ignoring_case() {
    let mut l = ProgramLists::default();
    l.set_from_text(ListKind::MinimizeBlacklist, "game.exe;Foo.EXE");
    assert!(is_blacklisted(&l, "GAME.exe"));
    assert!(is_blacklisted(&l, "foo.exe"));
    assert!(!is_blacklisted(&l, "bar.exe"));
    assert!(!is_blacklisted(&l, ""));
    assert!(
      !l.contains(ListKind::ExcludedPrograms, "game.exe"),
      "other lists are separate"
    );
  }

  #[test]
  fn plain_minimize_unless_every_gate_allows_the_animation() {
    assert!(should_animate(true, true, false, false));
    assert!(!should_animate(false, true, false, false), "master switch off");
    assert!(!should_animate(true, false, false, false), "DMN-01 off");
    assert!(!should_animate(true, true, true, false), "DMN-02 on");
    assert!(!should_animate(true, true, false, true), "blacklisted");
  }

  #[test]
  fn position_option_index() {
    assert_eq!(IconPosition::from_index(0), IconPosition::NextToIcon);
    assert_eq!(IconPosition::from_index(1), IconPosition::Independent);
    assert_eq!(IconPosition::from_index(2), IconPosition::Separator);
    assert_eq!(IconPosition::from_index(9), IconPosition::NextToIcon);
  }

  #[test]
  fn next_to_icon_puts_each_window_after_its_program() {
    use Placement::*;
    assert_eq!(
      place(&[1, 0, 2], IconPosition::NextToIcon),
      [
        App(0),
        Minimized { app: 0, win: 0 },
        App(1),
        App(2),
        Minimized { app: 2, win: 0 },
        Minimized { app: 2, win: 1 }
      ]
    );
  }

  #[test]
  fn independent_groups_windows_at_the_right() {
    use Placement::*;
    assert_eq!(
      place(&[1, 0, 2], IconPosition::Independent),
      [
        App(0),
        App(1),
        App(2),
        Minimized { app: 0, win: 0 },
        Minimized { app: 2, win: 0 },
        Minimized { app: 2, win: 1 }
      ]
    );
  }

  #[test]
  fn separator_mode_adds_a_separator_only_when_something_is_minimized() {
    use Placement::*;
    assert_eq!(
      place(&[1, 0], IconPosition::Separator),
      [App(0), App(1), Separator, Minimized { app: 0, win: 0 }]
    );
    assert_eq!(place(&[0, 0], IconPosition::Separator), [App(0), App(1)]);
    assert!(place(&[], IconPosition::Separator).is_empty());
  }
}
