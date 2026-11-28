//! Dock visibility decisions on top of the shared policy in `crate::visibility::policy`.
//!
//! The window host (`app`) gathers [`Signals`] from the OS and its own state, asks for a [`Transition`], and performs the
//! side effects (animation, `ShowWindow`, timers). No OS access happens here.

use crate::visibility::policy::{Inputs, Mode, should_hide, transition_to};

/// Everything the Dock needs to know to decide whether it is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Signals {
  /// A window covers the Dock's strip.
  pub overlapped: bool,
  /// A full-screen app is in the foreground.
  pub fullscreen: bool,
  /// The pointer is inside the Dock.
  pub tracking: bool,
  /// The pointer is over a preview popup.
  pub preview_hover: bool,
  /// A context menu is open.
  pub menu_open: bool,
  /// An icon is being dragged.
  pub dragging: bool,
  /// Short grace period after the pointer left.
  pub grace: bool,
  /// The pointer dwelled on the activation edge.
  pub edge_armed: bool,
  /// The Dock is still starting up.
  pub starting: bool,
  /// The Dock is being revealed (for example by a hot corner).
  pub revealing: bool,
  /// The Dock is switched off by the user.
  pub off: bool,
  /// The desktop is in front (read by the desktop-only mode).
  pub desktop: bool,
}

impl Signals {
  /// True while the user is interacting with the Dock and it must not hide.
  pub fn interacting(&self) -> bool {
    self.tracking || self.preview_hover || self.menu_open || self.dragging || self.grace
  }

  fn inputs(&self) -> Inputs {
    Inputs {
      overlapped: self.overlapped,
      interacting: self.interacting(),
      edge_armed: self.edge_armed,
      starting: self.starting,
      fullscreen: self.fullscreen,
      revealing: self.revealing,
      desktop: self.desktop,
    }
  }
}

pub use crate::visibility::policy::Transition;

/// Whether the Dock should currently be hidden.
pub fn wants_hidden(mode: Mode, signals: &Signals) -> bool {
  signals.off || should_hide(mode, &signals.inputs())
}

/// The change needed given whether the Dock is `shown` right now.
pub fn transition(mode: Mode, signals: &Signals, shown: bool) -> Transition {
  transition_to(wants_hidden(mode, signals), shown)
}

/// Entering the strip starts the reveal dwell timer unless the Dock is always visible.
pub fn arms_dwell_on_enter(mode: Mode) -> bool {
  mode.can_hide()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn s() -> Signals {
    Signals::default()
  }

  #[test]
  fn each_interaction_signal_counts_as_interacting() {
    assert!(!s().interacting());
    for sig in [
      Signals { tracking: true, ..s() },
      Signals {
        preview_hover: true,
        ..s()
      },
      Signals { menu_open: true, ..s() },
      Signals { dragging: true, ..s() },
      Signals { grace: true, ..s() },
    ] {
      assert!(sig.interacting());
    }
    assert!(
      !Signals {
        overlapped: true,
        edge_armed: true,
        ..s()
      }
      .interacting()
    );
  }

  #[test]
  fn auto_hide_hides_when_overlapped_and_idle_then_shows_again() {
    let covered = Signals {
      overlapped: true,
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &covered, true), Transition::Hide);
    assert_eq!(transition(Mode::AutoHide, &covered, false), Transition::Keep);
    assert_eq!(transition(Mode::AutoHide, &s(), false), Transition::Show);
    assert_eq!(transition(Mode::AutoHide, &s(), true), Transition::Keep);
  }

  #[test]
  fn interaction_or_armed_edge_keeps_an_overlapped_dock_visible() {
    for sig in [
      Signals {
        overlapped: true,
        tracking: true,
        ..s()
      },
      Signals {
        overlapped: true,
        menu_open: true,
        ..s()
      },
      Signals {
        overlapped: true,
        grace: true,
        ..s()
      },
      Signals {
        overlapped: true,
        edge_armed: true,
        ..s()
      },
    ] {
      assert_eq!(transition(Mode::AutoHide, &sig, true), Transition::Keep);
    }
  }

  #[test]
  fn switched_off_always_hides_even_in_always_mode() {
    let off = Signals { off: true, ..s() };
    assert_eq!(transition(Mode::Always, &off, true), Transition::Hide);
    assert_eq!(transition(Mode::Always, &off, false), Transition::Keep);
  }

  #[test]
  fn always_mode_ignores_overlap_but_not_fullscreen() {
    let covered = Signals {
      overlapped: true,
      ..s()
    };
    assert_eq!(transition(Mode::Always, &covered, true), Transition::Keep);
    let full = Signals {
      fullscreen: true,
      ..s()
    };
    assert_eq!(transition(Mode::Always, &full, true), Transition::Hide);
    let full_hover = Signals {
      fullscreen: true,
      tracking: true,
      ..s()
    };
    assert_eq!(transition(Mode::Always, &full_hover, true), Transition::Keep);
  }

  #[test]
  fn revealing_and_startup_never_hide() {
    let reveal = Signals {
      overlapped: true,
      revealing: true,
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &reveal, true), Transition::Keep);
    let start = Signals {
      overlapped: true,
      starting: true,
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &start, true), Transition::Keep);
  }

  #[test]
  fn dwell_timer_is_skipped_only_for_always_visible_docks() {
    assert!(!arms_dwell_on_enter(Mode::Always));
    assert!(arms_dwell_on_enter(Mode::AutoHide));
    assert!(arms_dwell_on_enter(Mode::OnDemand));
  }
}
