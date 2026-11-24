//! Window-overlap detection scope for the auto-hide Dock (D-10.4).

use crate::shared::geometry::Rect;

/// Which windows count when deciding that the Dock is covered (DAD-13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
  /// Every visible program window.
  All,
  /// Only the foreground program's windows.
  Current,
}

impl Scope {
  /// From the stored choice index (`all`, `current`). Unknown values fall back to `All`.
  pub fn from_index(i: u8) -> Scope {
    if i == 1 { Scope::Current } else { Scope::All }
  }
}

/// One visible top-level window as reported by the platform layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Win {
  pub rect: Rect,
  /// Belongs to the foreground program.
  pub foreground: bool,
  /// Minimized or cloaked windows never cover anything.
  pub visible: bool,
}

/// True when a counted window intersects the Dock strip.
pub fn overlapped(scope: Scope, strip: &Rect, windows: &[Win]) -> bool {
  windows
    .iter()
    .filter(|w| w.visible)
    .filter(|w| scope == Scope::All || w.foreground)
    .any(|w| w.rect.intersects(strip))
}

#[cfg(test)]
mod tests {
  use super::*;

  const STRIP: Rect = Rect::new(0.0, 500.0, 1000.0, 100.0);

  fn win(y: f32, foreground: bool, visible: bool) -> Win {
    Win {
      rect: Rect::new(0.0, y, 800.0, 300.0),
      foreground,
      visible,
    }
  }

  #[test]
  fn all_scope_counts_background_windows() {
    assert!(overlapped(Scope::All, &STRIP, &[win(400.0, false, true)]));
  }

  #[test]
  fn current_scope_ignores_background_windows() {
    let ws = [win(400.0, false, true)];
    assert!(!overlapped(Scope::Current, &STRIP, &ws));
    assert!(overlapped(Scope::Current, &STRIP, &[win(400.0, true, true)]));
  }

  #[test]
  fn hidden_windows_never_cover() {
    assert!(!overlapped(Scope::All, &STRIP, &[win(400.0, true, false)]));
  }

  #[test]
  fn windows_clear_of_the_strip_and_empty_lists_do_not_overlap() {
    assert!(!overlapped(Scope::All, &STRIP, &[win(0.0, true, true)]));
    assert!(!overlapped(Scope::All, &STRIP, &[]));
  }

  #[test]
  fn touching_edges_do_not_count() {
    assert!(!overlapped(Scope::All, &STRIP, &[win(200.0, true, true)]));
  }

  #[test]
  fn unknown_index_is_all() {
    assert_eq!(Scope::from_index(7), Scope::All);
    assert_eq!(Scope::from_index(1), Scope::Current);
  }
}
