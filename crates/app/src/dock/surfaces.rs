//! Permukaan ikon: pemuatan async, tempat sampah, dan penggantian ikon.

use super::*;

impl Dock {
    pub(super) fn make_trash_surface(&mut self) {
        let px = ((self.m.icon * self.m.max_scale * 1.15) as i32).clamp(64, 256) as u32;
        let p = crate::prefs::get();
        let raw = nexora_core::iconplate::free(&nexora_core::trash::icon(px, self.trash_full), px);
        let data = nexora_core::iconplate::style_icon_v(
            &raw,
            p.icon_style,
            p.icon_tint,
            crate::prefs::icon_dark(),
        );
        if let Ok(s) = self.gfx.surface_from_bgra(px, px, &data) {
            self.surfaces.insert(TRASH_PATH.to_string(), s);
        }
    }

    pub(super) fn update_trash_icon(&mut self) {
        if icons::is_themed(TRASH_PATH) {
            return;
        }
        self.surfaces.remove(TRASH_PATH);
        self.make_trash_surface();
        if let Some(s) = self.surfaces.get(TRASH_PATH).cloned() {
            for v in self.views.iter_mut() {
                if matches!(&v.kind, Kind::App { path } if path == TRASH_PATH) {
                    let _ = v.set_surface(&self.comp, &s);
                }
            }
        }
    }

    pub fn on_icons(&mut self) {
        while let Ok((path, bmp)) = self.worker.results.try_recv() {
            let Some(mut b) = bmp else { continue };
            if crate::prefs::get().dock_plates && b.w == b.h && !icons::is_themed(&path) {
                b.bgra = if path == TRASH_PATH || path == downloads_key() {
                    nexora_core::iconplate::free(&b.bgra, b.w)
                } else {
                    nexora_core::iconplate::plate(&b.bgra, b.w)
                };
            }
            if !icons::is_themed(&path) {
                let p = crate::prefs::get();
                b.bgra = nexora_core::iconplate::style_icon_v(
                    &b.bgra,
                    p.icon_style,
                    p.icon_tint,
                    crate::prefs::icon_dark(),
                );
            }
            match self.gfx.surface_from_bgra(b.w, b.h, &b.bgra) {
                Ok(surface) => {
                    for v in self.views.iter_mut() {
                        if matches!(&v.kind, Kind::App { path: p } if *p == path) {
                            let _ = v.set_surface(&self.comp, &surface);
                        }
                    }
                    self.surfaces.insert(path, surface);
                }
                Err(e) => log::line("icons", &format!("unggah gagal: {e}")),
            }
        }
    }

    pub(super) fn reload_icon(&mut self, path: &str) {
        self.surfaces.remove(path);
        self.requested.insert(path.to_string());
        let _ = self.worker.requests.send(path.to_string());
    }
}
