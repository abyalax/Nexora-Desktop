//! Sinkronisasi pengaturan pengguna ke state Dock.

use super::*;

impl Dock {
    pub(super) fn apply_prefs(&mut self) {
        let s = crate::prefs::get();
        let key = (
            s.icon_style,
            s.icon_tint,
            s.icon_pack,
            crate::prefs::icon_dark(),
        );
        if key != self.icon_key {
            self.icon_key = key;
            nexora_platform::iconpack::configure(&paths::config_dir().join("iconpack"), s.icon_pack);
            self.surfaces.clear();
            self.requested.clear();
            self.signature.clear();
            self.refresh_windows();
        }
        if s.dock_plates != self.plates {
            self.plates = s.dock_plates;
            self.surfaces.clear();
            self.requested.clear();
            self.signature.clear();
            self.refresh_windows();
        }
        if self.off == s.dock_enabled {
            self.off = !s.dock_enabled;
            if self.off {
                nexora_platform::shell::appbar::release_bottom();
            } else if self.opts.appbar {
                self.reserve_strip();
            }
            log::line(
                "dock",
                if self.off {
                    "dinonaktifkan"
                } else {
                    "diaktifkan"
                },
            );
            self.update_visibility();
        }
        let (mode, appbar) = (s.dock.mode(), s.dock.reserves());
        if mode != self.opts.mode || appbar != self.opts.appbar {
            let was_appbar = self.opts.appbar;
            self.opts.mode = mode;
            self.opts.appbar = appbar;
            if mode == Mode::Always {
                self.strip = None;
                self.edge_armed = false;
            } else if self.strip.is_none() {
                self.strip = Strip::new(self.hwnd, STRIP_ENTER, STRIP_LEAVE).ok();
                self.place_strip();
            }
            if appbar {
                self.reserve_strip();
            } else if was_appbar {
                nexora_platform::shell::appbar::release_bottom();
            }
            self.sync_strip();
            self.update_visibility();
            log::line("dock", &format!("perilaku → {mode:?} cadangan={appbar}"));
        }
        self.apply_metrics();
        let (themed, dark) = (
            crate::prefs::look(crate::prefs::Role::Dock),
            crate::prefs::look(crate::prefs::Role::Dark),
        );
        let base = GlassParams::dark((10.0, 10.0), 22.0 * self.dpi);
        let _ = self.panel.set_params(crate::prefs::apply(base, &themed));
        let _ = self
            .panel
            .set_radius(22.0 * self.dpi * crate::prefs::round(crate::prefs::Role::Dock));
        let pd = crate::prefs::apply(base, &dark);
        if let Some(t) = self.tooltip.as_mut() {
            t.apply_look(pd);
        }
        if let Some(p) = self.preview.as_mut() {
            let _ = p.panel_mut().set_params(pd);
        }
        if let Some(m) = self.menu.as_mut() {
            m.apply_look(pd);
        }
    }

    pub(super) fn refresh_tier(&mut self) {
        let t = self
            .opts
            .glass
            .unwrap_or_else(|| auto_tier(&sys::conditions()));
        if t != self.tier {
            self.tier = t;
            let _ = self.panel.set_tier(t);
            if let Some(tt) = self.tooltip.as_mut() {
                tt.set_tier(t);
            }
            if let Some(p) = self.preview.as_mut() {
                let _ = p.panel_mut().set_tier(t);
            }
            if let Some(m) = self.menu.as_mut() {
                m.set_tier(t);
            }
            log::line("dock", &format!("tier={t:?}"));
        }
    }
}
