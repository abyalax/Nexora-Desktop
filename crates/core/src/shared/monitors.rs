//! Choosing which monitor hosts a surface (Dock, Launchpad, Finder). Works on plain descriptions, not OS handles.

use super::geometry::Rect;

/// One connected display as reported by the platform layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monitor<'a> {
  /// Stable device identifier, e.g. `\\.\DISPLAY2`.
  pub id: &'a str,
  pub bounds: Rect,
  pub primary: bool,
}

/// Returns the index of the monitor to use.
///
/// `wanted` is the stored device id. An empty id, or an id that is no longer connected, selects the primary monitor
/// (or the first one when none is flagged primary). Returns `None` only when no monitor is connected.
pub fn pick(monitors: &[Monitor<'_>], wanted: &str) -> Option<usize> {
  let wanted = wanted.trim();
  if !wanted.is_empty() {
    if let Some(i) = monitors.iter().position(|m| m.id.eq_ignore_ascii_case(wanted)) {
      return Some(i);
    }
  }
  monitors
    .iter()
    .position(|m| m.primary)
    .or(if monitors.is_empty() { None } else { Some(0) })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn mons() -> Vec<Monitor<'static>> {
    vec![
      Monitor {
        id: "\\\\.\\DISPLAY1",
        bounds: Rect::new(0.0, 0.0, 1920.0, 1080.0),
        primary: true,
      },
      Monitor {
        id: "\\\\.\\DISPLAY2",
        bounds: Rect::new(1920.0, 0.0, 2560.0, 1440.0),
        primary: false,
      },
    ]
  }

  #[test]
  fn empty_choice_is_the_primary_monitor() {
    let mut m = mons();
    assert_eq!(pick(&m, ""), Some(0));
    m.swap(0, 1);
    assert_eq!(pick(&m, "  "), Some(1));
  }

  #[test]
  fn named_monitor_is_found_case_insensitively() {
    assert_eq!(pick(&mons(), "\\\\.\\display2"), Some(1));
  }

  #[test]
  fn unplugged_monitor_falls_back_to_the_primary() {
    assert_eq!(pick(&mons(), "\\\\.\\DISPLAY9"), Some(0));
  }

  #[test]
  fn no_primary_flag_uses_the_first_and_no_monitors_gives_none() {
    let mut m = mons();
    m[0].primary = false;
    assert_eq!(pick(&m, ""), Some(0));
    assert_eq!(pick(&[], "x"), None);
  }
}
