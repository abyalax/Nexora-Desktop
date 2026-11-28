//! Runtime Dock state, kept apart from the persistent
//! [`DockSettings`](crate::settings::dock::DockSettings).
//!
//! The window host in `app` owns one [`DockState`], feeds it OS facts (overlap, drag) and applies the visibility
//! [`Transition`] it returns. Nothing here touches the OS.

use super::visibility::{Signals, Transition, transition};
use crate::visibility::policy::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DockState {
  /// The Dock window is currently revealed.
  pub shown: bool,
  /// The Dock is still starting up.
  pub starting: bool,
  /// The Dock is being revealed (for example by a hot corner).
  pub revealing: bool,
  /// Short grace period after the pointer left.
  pub grace: bool,
  /// The pointer dwelled on the activation edge.
  pub edge_armed: bool,
  /// A context menu is open.
  pub menu_open: bool,
  /// The pointer is over a preview popup.
  pub preview_hover: bool,
  /// The pointer is inside the Dock.
  pub tracking: bool,
  /// The Dock is switched off by the user.
  pub off: bool,
}

/// Facts about the surrounding desktop that only the host can observe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Surroundings {
  pub overlapped: bool,
  pub fullscreen: bool,
  pub dragging: bool,
  pub desktop: bool,
}

impl DockState {
  /// State at window creation: revealed, starting, and off when the user disabled the Dock.
  pub fn new(enabled: bool) -> Self {
    DockState {
      shown: true,
      starting: true,
      revealing: false,
      grace: false,
      edge_armed: false,
      menu_open: false,
      preview_hover: false,
      tracking: false,
      off: !enabled,
    }
  }

  /// Inputs of the visibility decision: this state plus what the host observed.
  pub fn signals(&self, around: Surroundings) -> Signals {
    Signals {
      overlapped: around.overlapped,
      fullscreen: around.fullscreen,
      tracking: self.tracking,
      preview_hover: self.preview_hover,
      menu_open: self.menu_open,
      dragging: around.dragging,
      grace: self.grace,
      edge_armed: self.edge_armed,
      starting: self.starting,
      revealing: self.revealing,
      off: self.off,
      desktop: around.desktop,
    }
  }

  /// Decide the next visibility step and record it in `shown`; the host performs the side effects.
  pub fn next_transition(&mut self, mode: Mode, around: Surroundings) -> Transition {
    let t = transition(mode, &self.signals(around), self.shown);
    match t {
      Transition::Hide => self.shown = false,
      Transition::Show => self.shown = true,
      Transition::Keep => {}
    }
    t
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn settled() -> DockState {
    DockState {
      starting: false,
      ..DockState::new(true)
    }
  }

  #[test]
  fn new_state_is_shown_and_starting() {
    let s = DockState::new(true);
    assert!(s.shown && s.starting && !s.off);
    assert!(DockState::new(false).off);
  }

  #[test]
  fn startup_keeps_an_overlapped_dock_visible_until_it_settles() {
    let around = Surroundings {
      overlapped: true,
      ..Default::default()
    };
    let mut s = DockState::new(true);
    assert_eq!(s.next_transition(Mode::AutoHide, around), Transition::Keep);
    s.starting = false;
    assert_eq!(s.next_transition(Mode::AutoHide, around), Transition::Hide);
    assert!(!s.shown);
  }

  #[test]
  fn hiding_then_clearing_overlap_shows_again() {
    let mut s = settled();
    let covered = Surroundings {
      overlapped: true,
      ..Default::default()
    };
    assert_eq!(s.next_transition(Mode::AutoHide, covered), Transition::Hide);
    assert_eq!(s.next_transition(Mode::AutoHide, covered), Transition::Keep);
    assert_eq!(
      s.next_transition(Mode::AutoHide, Surroundings::default()),
      Transition::Show
    );
    assert!(s.shown);
  }

  #[test]
  fn interaction_flags_reach_the_signals() {
    let mut s = settled();
    s.menu_open = true;
    let sig = s.signals(Surroundings {
      dragging: true,
      ..Default::default()
    });
    assert!(sig.menu_open && sig.dragging && sig.interacting());
  }

  #[test]
  fn switched_off_hides_in_every_mode() {
    let mut s = DockState::new(false);
    assert_eq!(
      s.next_transition(Mode::Always, Surroundings::default()),
      Transition::Hide
    );
  }
}
