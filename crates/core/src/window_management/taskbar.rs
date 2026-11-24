//! Which monitors lose their Windows taskbar when Nexora takes over (GEN-09).
//!
//! Pure selection only. Hiding and restoring the taskbar is OS work in `platform/shell/taskbar.rs`, and
//! `nexora-guard` restores it after a crash (REC-01).

use crate::shared::monitors::{Monitor, pick};

/// Stored choice of `taskbar_scope`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
  None,
  AllDisplays,
  DockDisplay,
}

impl Scope {
  /// From the stored index (`none`, `all_displays`, `dock_display`). Unknown values mean `None` (never hide by mistake).
  pub fn from_index(i: u8) -> Scope {
    match i {
      1 => Scope::AllDisplays,
      2 => Scope::DockDisplay,
      _ => Scope::None,
    }
  }
}

/// Indices into `monitors` whose taskbar must be hidden. `dock_monitor` is the stored id of the Dock's display.
pub fn to_hide(scope: Scope, monitors: &[Monitor<'_>], dock_monitor: &str) -> Vec<usize> {
  match scope {
    Scope::None => Vec::new(),
    Scope::AllDisplays => (0..monitors.len()).collect(),
    Scope::DockDisplay => pick(monitors, dock_monitor).into_iter().collect(),
  }
}

/// Monitors to give their taskbar back when the selection changes: hidden before, not hidden now.
pub fn to_restore(before: &[usize], now: &[usize]) -> Vec<usize> {
  before.iter().copied().filter(|i| !now.contains(i)).collect()
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
        primary: true,
      },
      Monitor {
        id: "D2",
        bounds: Rect::new(1920.0, 0.0, 1920.0, 1080.0),
        primary: false,
      },
    ]
  }

  #[test]
  fn index_maps_and_unknown_never_hides() {
    assert_eq!(Scope::from_index(0), Scope::None);
    assert_eq!(Scope::from_index(1), Scope::AllDisplays);
    assert_eq!(Scope::from_index(2), Scope::DockDisplay);
    assert_eq!(Scope::from_index(9), Scope::None);
  }

  #[test]
  fn none_hides_nothing_and_all_hides_every_display() {
    assert!(to_hide(Scope::None, &mons(), "D1").is_empty());
    assert_eq!(to_hide(Scope::AllDisplays, &mons(), "D1"), [0, 1]);
  }

  #[test]
  fn dock_display_hides_only_the_dock_monitor() {
    assert_eq!(to_hide(Scope::DockDisplay, &mons(), "D2"), [1]);
  }

  #[test]
  fn dock_display_falls_back_to_primary_when_gone() {
    assert_eq!(to_hide(Scope::DockDisplay, &mons(), "D9"), [0]);
    assert_eq!(to_hide(Scope::DockDisplay, &[], "D1"), Vec::<usize>::new());
  }

  #[test]
  fn restore_lists_only_monitors_no_longer_hidden() {
    assert_eq!(to_restore(&[0, 1], &[1]), [0]);
    assert_eq!(to_restore(&[0], &[0, 1]), Vec::<usize>::new());
    assert_eq!(to_restore(&[0, 1], &[]), [0, 1]);
  }
}
