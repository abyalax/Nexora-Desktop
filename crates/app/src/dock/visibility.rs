//! Auto-hide policy: when the Dock is shown, hidden or kept alive.

use super::*;

impl Dock {
    pub(super) fn interacting(&self) -> bool {
        self.tracking
            || self.preview_hover
            || self.menu_open
            || self.dragging.is_some()
            || self.grace
    }

    pub(super) fn update_visibility(&mut self) {
        let inputs = Inputs {
            overlapped: self.overlap.overlapped,
            interacting: self.interacting(),
            edge_armed: self.edge_armed,
            starting: self.starting,
            fullscreen: self.overlap.fullscreen,
            revealing: self.revealing,
        };
        let hide = self.off || should_hide(self.opts.mode, &inputs);
        self.sync_strip();
        if hide && self.shown {
            self.shown = false;
            self.hide_popups();
            self.animate_stage(self.win.1 as f32 + 8.0);
            timer(self.hwnd, TIMER_HIDEWIN, 500);
            log::line("dock", "sembunyi");
        } else if !hide && !self.shown {
            self.shown = true;
            kill(self.hwnd, TIMER_HIDEWIN);
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
            crate::topmost::request_raise();
            self.animate_stage(0.0);
            log::line("dock", "tampil");
        }
    }

    pub(super) fn start_grace(&mut self) {
        self.grace = true;
        timer(self.hwnd, TIMER_GRACE, 800);
    }

    pub(super) fn on_strip_enter(&mut self) {
        if self.opts.mode != Mode::Always {
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
