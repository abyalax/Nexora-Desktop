//! Auto-hide policy: when the Dock is shown, hidden or kept alive.

use super::*;
use nexora_core::dock::visibility::{self, Signals, Transition};

impl Dock {
  /// Snapshot of the host state that drives the visibility decision in `nexora_core::dock::visibility`.
  fn signals(&self) -> Signals {
    Signals {
      overlapped: self.overlap.overlapped,
      fullscreen: self.overlap.fullscreen,
      tracking: self.tracking,
      preview_hover: self.preview_hover,
      menu_open: self.menu_open,
      dragging: self.dragging.is_some(),
      grace: self.grace,
      edge_armed: self.edge_armed,
      starting: self.starting,
      revealing: self.revealing,
      off: self.off,
    }
  }

  pub(super) fn update_visibility(&mut self) {
    let transition = visibility::transition(self.opts.mode, &self.signals(), self.shown);
    self.sync_strip();
    match transition {
      Transition::Hide => {
        self.shown = false;
        self.hide_popups();
        self.animate_stage(self.win.1 as f32 + 8.0);
        timer(self.hwnd, TIMER_HIDEWIN, 500);
        log::line("dock", "sembunyi");
      }
      Transition::Show => {
        self.shown = true;
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
    self.grace = true;
    timer(self.hwnd, TIMER_GRACE, 800);
  }

  pub(super) fn on_strip_enter(&mut self) {
    if visibility::arms_dwell_on_enter(self.opts.mode) {
      timer(self.hwnd, TIMER_DWELL, 200);
    }
  }

  pub(super) fn on_strip_leave(&mut self) {
    kill(self.hwnd, TIMER_DWELL);
    if self.edge_armed {
      self.edge_armed = false;
      self.start_grace();
      self.update_visibility();
    }
  }
}
