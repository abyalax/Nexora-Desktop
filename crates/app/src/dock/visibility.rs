//! Auto-hide policy: when the Dock is shown, hidden or kept alive.

use super::*;
use nexora_core::dock::state::Surroundings;
use nexora_core::dock::visibility::{self, Transition};

impl Dock {
  /// What the host observes from the desktop; everything else lives in `DockState`.
  fn surroundings(&self) -> Surroundings {
    Surroundings {
      overlapped: self.overlap.overlapped,
      fullscreen: self.overlap.fullscreen,
      dragging: self.dragging.is_some(),
    }
  }

  pub(super) fn update_visibility(&mut self) {
    let around = self.surroundings();
    let transition = self.state.next_transition(self.opts.mode, around);
    self.sync_strip();
    match transition {
      Transition::Hide => {
        self.hide_popups();
        self.animate_stage(self.win.1 as f32 + 8.0);
        timer(self.hwnd, TIMER_HIDEWIN, 500);
        log::line("dock", "sembunyi");
      }
      Transition::Show => {
        kill(self.hwnd, TIMER_HIDEWIN);
        unsafe {
          let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
        crate::topmost::request_raise();
        self.animate_stage(0.0);
        log::line("dock", "tampil");
      }
      Transition::Keep => {}
    }
  }

  pub(super) fn start_grace(&mut self) {
    self.state.grace = true;
    timer(self.hwnd, TIMER_GRACE, 800);
  }

  pub(super) fn on_strip_enter(&mut self) {
    if visibility::arms_dwell_on_enter(self.opts.mode) {
      timer(self.hwnd, TIMER_DWELL, 200);
    }
  }

  pub(super) fn on_strip_leave(&mut self) {
    kill(self.hwnd, TIMER_DWELL);
    if self.state.edge_armed {
      self.state.edge_armed = false;
      self.start_grace();
      self.update_visibility();
    }
  }
}
