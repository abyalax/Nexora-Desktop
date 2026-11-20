//! Geometri kartu, ukuran jendela, region klik, dan cadangan appbar.

use super::*;

impl Dock {
    pub(super) fn compute_geometry(&mut self) {
        let d = self.dpi;
        let icon = self.m.icon;
        let (pad_top, pad_bottom) = (12.0 * d, 18.0 * d);
        let margin = crate::prefs::get().dock_margin as f32 * d;
        self.margin = margin;
        self.card_h = pad_top + icon + pad_bottom;
        let headroom = icon * (self.m.max_scale - 1.0) + 6.0 * d;
        let win_h = (margin + self.card_h + headroom).ceil();
        self.card_y = win_h - margin - self.card_h;
        self.icon_y = self.card_y + pad_top;
        self.dot_y = self.icon_y + icon + (pad_bottom - 4.0 * d) / 2.0 + 1.0 * d;
        self.win.1 = win_h as i32;
    }

    pub(super) fn max_content_width(&self) -> f32 {
        let area = 4000.0;
        let base = layout(&self.slots, None, area, &self.m);
        let (Some(first), Some(last)) = (base.cells.first(), base.cells.last()) else {
            return 0.0;
        };
        let (from, to) = (first.x, last.x + last.w);
        let mut best = base.total;
        let mut x = from;
        while x <= to {
            best = best.max(layout(&self.slots, Some(x), area, &self.m).total);
            x += self.m.icon / 4.0;
        }
        best
    }

    pub(super) fn resize_window(&mut self) {
        let need = self.max_content_width() + 2.0 * self.pad_side + 40.0 * self.dpi;
        let w = (need.ceil() as i32).clamp(200, self.screen.0);
        self.win.0 = w;
        self.win_pos = ((self.screen.0 - w) / 2, self.screen.1 - self.win.1);
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                self.win_pos.0,
                self.win_pos.1,
                w,
                self.win.1,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
        let _ = self._surface.root.SetSize(windows_numerics::Vector2 {
            X: w as f32,
            Y: self.win.1 as f32,
        });
        self.place_strip();
        log::line(
            "dock",
            &format!(
                "jendela=({},{})-({},{}) layar={}x{} dpi={}",
                self.win_pos.0,
                self.win_pos.1,
                self.win_pos.0 + w,
                self.win_pos.1 + self.win.1,
                self.screen.0,
                self.screen.1,
                self.dpi
            ),
        );
    }

    pub(super) fn on_reconfigure(&mut self) {
        let screen = sys::screen_size();
        let dpi =
            unsafe { GetDpiForWindow(self.hwnd) }.max(96) as f32 / 96.0 * crate::prefs::ui_scale();
        if screen == self.screen && (dpi - self.dpi).abs() < 0.01 {
            return;
        }
        log::line(
            "dock",
            &format!(
                "reconfigurasi {}x{}@{} → {}x{}@{}",
                self.screen.0, self.screen.1, self.dpi, screen.0, screen.1, dpi
            ),
        );
        self.hide_popups();
        self.tooltip = None;
        self.preview = None;
        self.screen = screen;
        self.dpi = dpi;
        self.m = metrics_from_prefs(dpi);
        self.pad_side = 10.0 * dpi;
        self.compute_geometry();
        let _ = self
            .panel
            .set_radius(22.0 * dpi * crate::prefs::round(crate::prefs::Role::Dock));
        self.last_card = (0.0, 0.0, 0.0, 0.0);
        self.signature.clear();
        self.refresh_windows();
        if self.opts.appbar {
            self.reserve_strip();
        }
    }

    pub(super) fn apply_metrics(&mut self) {
        let s = crate::prefs::get();
        let m = metrics_from_prefs(self.dpi);
        let same = (m.icon - self.m.icon).abs() < 0.01
            && (m.gap - self.m.gap).abs() < 0.01
            && (m.max_scale - self.m.max_scale).abs() < 0.001
            && (s.dock_margin as f32 * self.dpi - self.margin).abs() < 0.01;
        if same {
            return;
        }
        self.hide_popups();
        self.m = m;
        let px = ((m.icon * m.max_scale * 1.15) as i32).clamp(64, 256);
        if px != self.icon_px {
            self.icon_px = px;
            let hw = self.hwnd.0 as isize;
            self.worker = icons::spawn_worker(px, move || unsafe {
                let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_ICONS, WPARAM(0), LPARAM(0));
            });
            self.surfaces.clear();
            self.requested.clear();
        }
        self.compute_geometry();
        self.last_card = (0.0, 0.0, 0.0, 0.0);
        self.signature.clear();
        self.refresh_windows();
        if self.opts.appbar {
            self.reserve_strip();
        }
        log::line(
            "dock",
            &format!(
                "metrik: ikon={} jarak={} zoom={} margin={}",
                s.dock_size, s.dock_gap, s.dock_zoom, s.dock_margin
            ),
        );
    }

    pub(super) fn reserve_strip(&mut self) {
        if self.off {
            return;
        }
        let d = self.dpi;
        let strip = (self.margin + self.card_h + 4.0 * d).ceil() as i32;
        let r = nexora_platform::shell::appbar::reserve_bottom(
            self.hwnd,
            self.screen.0,
            self.screen.1,
            strip,
        );
        log::line("appbar", &format!("strip={strip} hasil={r:?}"));
    }

    pub(super) fn set_region(&mut self, full: bool) {
        self.region_full = full;
        let (l, t, r, b) = if full || self.layout.cells.is_empty() {
            (0, 0, self.win.0, self.win.1)
        } else {
            let rest = layout(&self.slots, None, self.win.0 as f32, &self.m);
            let cw = rest.total + 2.0 * self.pad_side;
            let cx = (self.win.0 as f32 - cw) / 2.0;
            (
                cx as i32,
                self.card_y as i32,
                (cx + cw).ceil() as i32,
                (self.card_y + self.card_h).ceil() as i32,
            )
        };
        unsafe {
            let rgn = CreateRectRgn(l, t, r, b);
            SetWindowRgn(self.hwnd, Some(rgn), true);
        }
    }

    pub(super) fn card_screen_rect(&self) -> (i32, i32, i32, i32) {
        let rest = layout(&self.slots, None, self.win.0 as f32, &self.m);
        let cw = rest.total + 2.0 * self.pad_side;
        let cx = (self.win.0 as f32 - cw) / 2.0;
        let l = self.win_pos.0 + cx as i32;
        let t = self.win_pos.1 + self.card_y as i32;
        (
            l,
            t,
            l + cw.ceil() as i32,
            t + (self.card_h + self.margin) as i32,
        )
    }

    pub(super) fn place_strip(&self) {
        if let Some(s) = &self.strip {
            s.place(self.win_pos.0, self.screen.1 - 2, self.win.0, 2);
        }
    }
}
