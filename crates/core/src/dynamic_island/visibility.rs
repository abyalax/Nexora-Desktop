//! Island visibility decisions on top of the shared policy in `crate::visibility::policy`.
//!
//! The window host (`app`) gathers [`Signals`], asks for a [`Transition`] and performs the side effects (slide
//! animation, `ShowWindow`, timers). No OS access happens here.

use super::state::Mode as Panel;
use crate::visibility::policy::{Inputs, Mode, should_hide, transition_to};

pub use crate::visibility::policy::Transition;

/// Everything the Island needs to know to decide whether it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signals {
  /// A window covers the Island's strip.
  pub overlapped: bool,
  /// A full-screen app is in the foreground.
  pub fullscreen: bool,
  /// The pointer is inside the Island.
  pub tracking: bool,
  /// Short grace period after the pointer left.
  pub grace: bool,
  /// The pointer dwelled on the activation edge.
  pub edge_armed: bool,
  /// The Island is still starting up.
  pub starting: bool,
  /// The Island is switched off by the user.
  pub off: bool,
  /// Current panel: an open or dropping Island never hides.
  pub panel: Panel,
}

impl Signals {
  /// True while the user is interacting with the Island and it must not hide.
  pub fn interacting(&self) -> bool {
    self.tracking || matches!(self.panel, Panel::Open(_) | Panel::Drop) || self.grace
  }

  fn inputs(&self) -> Inputs {
    Inputs {
      overlapped: self.overlapped,
      interacting: self.interacting(),
      edge_armed: self.edge_armed,
      starting: self.starting,
      fullscreen: self.fullscreen,
      revealing: false,
    }
  }
}

/// Whether the Island should currently be hidden.
pub fn wants_hidden(mode: Mode, signals: &Signals) -> bool {
  signals.off || should_hide(mode, &signals.inputs())
}

/// The change needed given whether the Island is `shown` right now.
pub fn transition(mode: Mode, signals: &Signals, shown: bool) -> Transition {
  transition_to(wants_hidden(mode, signals), shown)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::dynamic_island::state::Tab;

  fn s() -> Signals {
    Signals {
      overlapped: false,
      fullscreen: false,
      tracking: false,
      grace: false,
      edge_armed: false,
      starting: false,
      off: false,
      panel: Panel::Rest,
    }
  }

  #[test]
  fn open_or_dropping_panel_counts_as_interacting() {
    assert!(!s().interacting());
    assert!(
      Signals {
        panel: Panel::Open(Tab::Play),
        ..s()
      }
      .interacting()
    );
    assert!(
      Signals {
        panel: Panel::Drop,
        ..s()
      }
      .interacting()
    );
    assert!(
      !Signals {
        panel: Panel::Peek,
        ..s()
      }
      .interacting()
    );
  }

  #[test]
  fn auto_hide_hides_when_overlapped_and_idle() {
    let covered = Signals {
      overlapped: true,
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &covered, true), Transition::Hide);
    assert_eq!(transition(Mode::AutoHide, &covered, false), Transition::Keep);
    assert_eq!(transition(Mode::AutoHide, &s(), false), Transition::Show);
  }

  #[test]
  fn open_panel_or_armed_edge_keeps_an_overlapped_island_visible() {
    let open = Signals {
      overlapped: true,
      panel: Panel::Open(Tab::Play),
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &open, true), Transition::Keep);
    let armed = Signals {
      overlapped: true,
      edge_armed: true,
      ..s()
    };
    assert_eq!(transition(Mode::AutoHide, &armed, true), Transition::Keep);
  }

  #[test]
  fn switched_off_always_hides_and_startup_never_does() {
    assert_eq!(
      transition(Mode::Always, &Signals { off: true, ..s() }, true),
      Transition::Hide
    );
    let booting = Signals {
      starting: true,
      overlapped: true,
      ..s()
    };
    assert_eq!(transition(Mode::OnDemand, &booting, true), Transition::Keep);
  }

  #[test]
  fn fullscreen_hides_unless_interacting() {
    let fs = Signals {
      fullscreen: true,
      ..s()
    };
    assert_eq!(transition(Mode::Always, &fs, true), Transition::Hide);
    assert_eq!(
      transition(Mode::Always, &Signals { tracking: true, ..fs }, true),
      Transition::Keep
    );
  }
}
