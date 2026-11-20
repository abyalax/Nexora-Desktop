//! Popup Dock: tooltip label dan pratinjau jendela.

use super::*;

impl Dock {
    pub(super) fn hide_popups(&mut self) {
        kill(self.hwnd, TIMER_TIP);
        kill(self.hwnd, TIMER_PREVIEW);
        if let Some(t) = &self.tooltip {
            t.hide();
        }
        if let Some(p) = self.preview.as_mut() {
            p.hide();
        }
        self.preview_hover = false;
    }

    pub(super) fn icon_anchor(&self, slot: usize) -> Option<(i32, i32)> {
        let c = self.layout.cells.get(slot)?;
        let cx = self.win_pos.0 as f32 + c.x + c.w / 2.0;
        let top = self.win_pos.1 as f32 + self.icon_y + self.m.icon - self.m.icon * c.scale;
        Some((cx as i32, top as i32))
    }

    pub(super) fn show_tooltip(&mut self) {
        let Some(slot) = self.hover else { return };
        if self.dragging.is_some() || self.preview.as_ref().is_some_and(|p| p.is_visible()) {
            return;
        }
        let label = match self.entries.get(slot) {
            Some(Entry::Start) => crate::prefs::tr("Mulai").to_string(),
            Some(Entry::Launchpad) => crate::prefs::tr("Apps").to_string(),
            Some(Entry::Trash) => crate::prefs::tr("Tempat sampah").to_string(),
            Some(Entry::Downloads) => crate::prefs::tr("Unduhan").to_string(),
            Some(Entry::App(i)) => match self.items.get(*i) {
                Some(it) => {
                    let f = nexora_platform::wins::names::friendly(&it.path);
                    if f.is_empty() {
                        it.name.clone()
                    } else {
                        f
                    }
                }
                None => return,
            },
            _ => return,
        };
        let Some((cx, top)) = self.icon_anchor(slot) else {
            return;
        };
        if self.tooltip.is_none() {
            self.tooltip = Tooltip::new(&self.comp, self.dpi, self.screen.0)
                .ok()
                .map(|mut t| {
                    t.set_tier(self.tier);
                    t
                });
        }
        if let Some(t) = self.tooltip.as_mut() {
            let bottom = top - (3.0 * self.dpi) as i32;
            if let Err(e) = t.show(&self.comp, &self.gfx, &label, cx, bottom) {
                log::line("tooltip", &format!("gagal: {e}"));
            }
        }
    }

    pub(super) fn show_preview(&mut self) {
        let Some(slot) = self.hover else { return };
        if self.dragging.is_some() {
            return;
        }
        let Some(Entry::App(i)) = self.entries.get(slot).copied() else {
            return;
        };
        let Some(it) = self.items.get(i) else { return };
        if !it.running() {
            return;
        }
        let wins: Vec<(isize, String, bool)> = it
            .hwnds
            .iter()
            .map(|h| (*h, focus::title(*h), focus::is_minimized(*h)))
            .collect();
        let icon = self.surfaces.get(&it.path).cloned();
        let Some((cx, top)) = self.icon_anchor(slot) else {
            return;
        };
        if self.preview.is_none() {
            self.preview = Preview::new(&self.comp, self.hwnd, self.dpi, self.screen.0)
                .ok()
                .map(|mut p| {
                    let _ = p.panel_mut().set_tier(self.tier);
                    p
                });
        }
        if let Some(t) = &self.tooltip {
            t.hide();
        }
        if let Some(p) = self.preview.as_mut() {
            let bottom = top - (10.0 * self.dpi) as i32;
            if let Err(e) = p.show(&self.comp, &self.gfx, &wins, icon.as_ref(), cx, bottom) {
                log::line("preview", &format!("gagal: {e}"));
            }
        }
    }

    pub(super) fn on_preview_click(&mut self, x: i32) {
        let target = self.preview.as_ref().and_then(|p| p.hit(x));
        if let Some(h) = target {
            let ok = self.activate_target(h);
            log::line("dock", &format!("pratinjau → aktifkan hwnd={h} ok={ok}"));
        }
        self.hide_popups();
        self.start_grace();
    }
}
