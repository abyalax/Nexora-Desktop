//! WM_TIMER dispatch.

use super::*;

impl Dock {
  pub(super) fn on_timer(&mut self, id: usize) {
    kill(self.hwnd, id);
    match id {
      TIMER_WINDOWS => self.refresh_windows(),
      TIMER_APPBAR => {
        if self.opts.appbar {
          self.reserve_strip();
        }
      }
      TIMER_TIP => self.show_tooltip(),
      TIMER_PREVIEW => self.show_preview(),
      TIMER_GRACE => {
        self.grace = false;
        self.update_visibility();
      }
      TIMER_DWELL => {
        self.edge_armed = true;
        self.update_visibility();
      }
      TIMER_PCLOSE => {
        if !self.preview_hover && !self.tracking {
          if let Some(p) = self.preview.as_mut() {
            p.hide();
          }
        }
        self.update_visibility();
      }
      TIMER_HIDEWIN => {
        if !self.shown {
          unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
          }
        }
      }
      TIMER_START => {
        self.starting = false;
        self.update_visibility();
      }
      TIMER_REVEAL => {
        self.revealing = false;
        self.update_visibility();
      }
      TIMER_REGION => {
        if !self.tracking {
          self.set_region(false);
        }
      }
      TIMER_RECONF => self.on_reconfigure(),
      TIMER_TRASH => {
        let full = nexora_platform::recycle::is_full();
        if full != self.trash_full {
          self.trash_full = full;
          self.update_trash_icon();
        }
        timer(self.hwnd, TIMER_TRASH, TRASH_POLL_MS);
      }
      _ => {}
    }
  }
}
