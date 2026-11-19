//! Window placement, work-area reservation and show/hide policy.

use super::*;

impl Notch {
    pub(super) fn pill_screen_rect(&self) -> (i32, i32, i32, i32) {
        let base = state::size(state::look(Mode::Rest, None), self.dpi);
        let l = ((sys::screen_size().0 as f32 - base.w) / 2.0) as i32;
        (l, 0, l + base.w as i32, (base.h + 4.0 * self.dpi) as i32)
    }

    pub(super) fn place_strip(&self) {
        if let Some(s) = &self.strip {
            let (l, _, r, _) = self.pill_screen_rect();
            let pad = (60.0 * self.dpi) as i32;
            s.place(l - pad, 0, r - l + 2 * pad, 2);
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

    pub(super) fn interacting(&self) -> bool {
        self.tracking || matches!(self.mode, Mode::Open(_) | Mode::Drop) || self.grace
    }

    pub(super) fn update_visibility(&mut self) {
        let inputs = Inputs {
            overlapped: self.overlap.overlapped,
            interacting: self.interacting(),
            edge_armed: self.edge_armed,
            starting: self.starting,
            fullscreen: self.overlap.fullscreen,
            revealing: false,
        };
        let hide = self.off || should_hide(self.vis, &inputs);
        if let Some(s) = &self.strip {
            s.set_visible(!self.overlap.fullscreen && !self.off);
        }
        if hide && self.shown {
            self.shown = false;
            let y = -(self.size.h + self.body.top_over + 8.0 * self.dpi);
            self.body.stage_y(y, 0.95, 90);
            if let Some(sh) = &self.shadow {
                sh.stage_y(y, 0.95, 90);
            }
            timer(self.hwnd, T_HIDEWIN, 550);
            log::line("notch", "sembunyi");
        } else if !hide && !self.shown {
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
        let (l, _, r, h) = self.pill_screen_rect();
        crate::services::set_notch(l, r, h - (4.0 * self.dpi) as i32);
    }

    pub(super) fn set_reserved(&mut self, on: bool) {
        self.reserved = on;
        if on {
            let strip = (34.0 * self.dpi).ceil() as i32;
            let r = nexora_platform::shell::appbar::reserve_top(self.hwnd, self.screen_w, strip);
            log::line("appbar", &format!("atas strip={strip} hasil={r:?}"));
        } else {
            nexora_platform::shell::appbar::release_top();
        }
    }
}
