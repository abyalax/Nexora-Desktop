//! Preference, display, power and tier changes.

use super::*;

impl Notch {
    pub(super) fn poll_privacy(&mut self) {
        if self.shown && !self.off {
            let a = nexora_platform::privacy::active();
            let v = (a.iter().any(|u| !u.camera), a.iter().any(|u| u.camera));
            if v != self.privacy {
                self.privacy = v;
                self.render();
            }
        }
        timer(self.hwnd, T_PRIVACY, 3000);
    }

    pub(super) fn on_power(&mut self) {
        let b = sys::battery();
        let was = self.battery.is_some_and(|x| x.1);
        let now = b.is_some_and(|x| x.1);
        self.battery = b;
        if now && !was {
            self.charging_fresh = true;
            timer(self.hwnd, T_CHARGE, 4000);
        }
        self.refresh();
    }

    pub(super) fn apply_prefs(&mut self) {
        let s = crate::prefs::get();
        self.off = !s.notch_enabled;
        let (vis, reserve) = (
            s.notch.mode(),
            s.notch.reserves() && s.notch_enabled && !s.menubar,
        );
        if vis != self.vis {
            self.vis = vis;
            if vis == Vis::Always {
                self.strip = None;
                self.edge_armed = false;
            } else if self.strip.is_none() {
                self.strip = Strip::new(self.hwnd, STRIP_ENTER, STRIP_LEAVE).ok();
                self.place_strip();
            }
        }
        if reserve != self.reserved {
            self.set_reserved(reserve);
        }
        self.publish_notch();
        let _ = self
            .body
            .panel
            .set_params(body::glass_params(self.size.radius));
        self.body.paint_ears(self.tier);
        if let Some(sh) = &self.shadow {
            sh.set_visible(self.shown && !self.off);
        }
        self.update_visibility();
        self.sync_download_watch();
        self.first = true;
        self.refresh();
        log::line(
            "notch",
            &format!(
                "pengaturan: perilaku={vis:?} cadangan={reserve} mati={}",
                self.off
            ),
        );
    }

    pub(super) fn refresh_tier(&mut self) {
        let t = auto_tier(self.glass_override);
        if t != self.tier {
            self.tier = t;
            let _ = self.body.panel.set_tier(t);
            self.body.paint_ears(t);
        }
    }

    pub(super) fn on_reconfigure(&mut self) {
        let screen_w = sys::screen_size().0;
        let dpi =
            unsafe { GetDpiForWindow(self.hwnd) }.max(96) as f32 / 96.0 * crate::prefs::ui_scale();
        let win = win_size(dpi, screen_w);
        if win == self.win && screen_w == self.screen_w && (dpi - self.dpi).abs() < 0.01 {
            return;
        }
        log::line(
            "notch",
            &format!("konfigurasi ulang layar_w={screen_w} dpi={dpi}"),
        );
        let dpi_changed = (dpi - self.dpi).abs() >= 0.01;
        self.screen_w = screen_w;
        self.dpi = dpi;
        self.win = win;
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                (screen_w - win.0) / 2,
                0,
                win.0,
                win.1,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
        let _ = self._surface.root.SetSize(Vector2 {
            X: win.0 as f32,
            Y: win.1 as f32,
        });
        self.body.resize(win);
        self.paint.reset(dpi);
        if dpi_changed {
            self.body.remake_ears(&self.paint.gfx, dpi, self.tier);
            if let Some(sh) = self.shadow.as_mut() {
                sh.pad = (SHADOW_PAD * dpi).round();
            }
        }
        self.place_shadow_window();
        self.place_strip();
        self.publish_notch();
        self.first = true;
        let look = self.look;
        self.transition(look);
    }
}
