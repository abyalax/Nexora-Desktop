//! Window placement, work-area reservation and show/hide policy.

use super::*;

impl Notch {
  pub(super) fn pill_screen_rect(&self) -> (i32, i32, i32, i32) {
    let base = state::size(state::look(Mode::Rest, None), self.dpi);
    layout::pill_screen_rect(sys::screen_size().0, base, self.dpi)
  }

  pub(super) fn place_strip(&self) {
    if let Some(s) = &self.strip {
      let (x, y, w, h) = layout::activation_strip(self.pill_screen_rect(), self.dpi);
      s.place(x, y, w, h);
      s.set_visible(!self.overlap.fullscreen && !self.off);
    }
  }

  pub(super) fn place_shadow_window(&self) {
    if let Some(sh) = &self.shadow {
      let x = (self.screen_w - self.win.0) / 2 - sh.pad as i32;
      sh.place_window(
        self.hwnd,
        x,
        0,
        self.win.0 + 2 * sh.pad as i32,
        self.win.1 + sh.pad as i32,
      );
      sh.set_visible(self.shown && !self.off);
    }
  }

  /// Snapshot of the host state that drives the visibility decision in `nexora_core::dynamic_island::visibility`.
  fn visibility_signals(&self) -> visibility::Signals {
    visibility::Signals {
      overlapped: self.overlap.overlapped,
      fullscreen: self.overlap.fullscreen,
      tracking: self.tracking,
      grace: self.grace,
      edge_armed: self.edge_armed,
      starting: self.starting,
      off: self.off,
      panel: self.mode,
    }
  }

  pub(super) fn update_visibility(&mut self) {
    let transition = visibility::transition(self.vis, &self.visibility_signals(), self.shown);
    if let Some(s) = &self.strip {
      s.set_visible(!self.overlap.fullscreen && !self.off);
    }
    if transition == Transition::Hide {
      self.shown = false;
      let y = -(self.size.h + self.body.top_over + 8.0 * self.dpi);
      self.body.stage_y(y, 0.95, 90);
      if let Some(sh) = &self.shadow {
        sh.stage_y(y, 0.95, 90);
      }
      timer(self.hwnd, T_HIDEWIN, 550);
      log::line("notch", "sembunyi");
    } else if transition == Transition::Show {
      self.shown = true;
      kill(self.hwnd, T_HIDEWIN);
      unsafe {
        let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
      }
      crate::topmost::request_raise();
      if let Some(sh) = &self.shadow {
        sh.set_visible(true);
        sh.stage_y(0.0, 0.8, 90);
      }
      self.body.stage_y(0.0, 0.8, 90);
      log::line("notch", "tampil");
    }
  }

  pub(super) fn publish_notch(&self) {
    if self.off {
      crate::services::set_notch(0, 0, 0);
      return;
    }
    let (l, r, h) = layout::published_extent(self.pill_screen_rect(), self.dpi);
    crate::services::set_notch(l, r, h);
  }

  pub(super) fn set_reserved(&mut self, on: bool) {
    self.reserved = on;
    if on {
      let strip = layout::reserve_height(self.dpi);
      let r = nexora_platform::shell::appbar::reserve_top(self.hwnd, self.screen_w, strip);
      log::line("appbar", &format!("atas strip={strip} hasil={r:?}"));
    } else {
      nexora_platform::shell::appbar::release_top();
    }
  }
}
