//! Daftar item Dock: sinkronisasi jendela, pin, dan app terakhir dipakai.

use super::*;

impl Dock {
    pub fn refresh_windows(&mut self) {
        let wins = enumerate::list(std::process::id());
        tracker::prune(focus::is_alive);
        let mut items = merge(&self.pins, &wins, &tracker::recent, &mut self.seen);
        self.heal_pins(&mut items);
        self.track_recents(&items);
        self.append_recents(&mut items);
        let sig: Vec<_> = items
            .iter()
            .map(|i| (i.key.clone(), i.pinned, i.hwnds.len(), i.path.clone()))
            .collect();
        self.items = items;
        if sig != self.signature {
            self.signature = sig;
            self.rebuild();
        }
        self.overlap = overlap::evaluate(
            tracker::external_foreground(),
            std::process::id(),
            self.card_screen_rect(),
            self.screen,
        );
        if self.overlap.fullscreen && crate::prefs::get().above_fullscreen {
            self.overlap = overlap::Overlap {
                overlapped: false,
                fullscreen: false,
            };
        }
        self.update_visibility();
    }

    pub(super) fn rebuild(&mut self) {
        log::line(
            "dock",
            &format!(
                "item: {}",
                self.items
                    .iter()
                    .map(|i| format!(
                        "{}[{}]{}",
                        i.name,
                        i.hwnds.len(),
                        if i.pinned { "*" } else { "" }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        self.entries.clear();
        self.slots.clear();
        self.entries.push(Entry::Start);
        self.entries.push(Entry::Launchpad);
        let pinned = self.items.iter().filter(|i| i.pinned).count();
        for i in 0..self.items.len() {
            if i == pinned && pinned > 0 {
                self.entries.push(Entry::Sep);
            }
            self.entries.push(Entry::App(i));
        }
        self.entries.push(Entry::Sep);
        if crate::prefs::get().downloads_stack && nexora_platform::downloads::folder().is_some() {
            self.entries.push(Entry::Downloads);
        }
        self.entries.push(Entry::Trash);
        for e in &self.entries {
            self.slots.push(if matches!(e, Entry::Sep) {
                Slot::Separator
            } else {
                Slot::Item
            });
        }

        if let Ok(kids) = self.content.Children() {
            let _ = kids.RemoveAll();
        }
        self.views.clear();
        let icon = self.m.icon;
        if !icons::is_themed(LAUNCHPAD_KEY) && !self.surfaces.contains_key(LAUNCHPAD_KEY) {
            let px = ((self.m.icon * self.m.max_scale * 1.15) as i32).clamp(64, 256) as u32;
            let p = crate::prefs::get();
            let data = nexora_core::iconplate::style_icon_v(
                &nexora_core::iconplate::launchpad_icon(px),
                p.icon_style,
                p.icon_tint,
                crate::prefs::icon_dark(),
            );
            if let Ok(s) = self.gfx.surface_from_bgra(px, px, &data) {
                self.surfaces.insert(LAUNCHPAD_KEY.to_string(), s);
            }
        }
        if !icons::is_themed(TRASH_PATH) && !self.surfaces.contains_key(TRASH_PATH) {
            self.make_trash_surface();
        }
        for e in self.entries.clone() {
            let v = match e {
                Entry::Start if custom_icons::has(START_KEY) => View::app(
                    &self.comp,
                    &self.content,
                    START_KEY.to_string(),
                    icon,
                    false,
                    self.dpi,
                ),
                Entry::Start => View::start(&self.comp, &self.content, icon),
                Entry::Launchpad => View::app(
                    &self.comp,
                    &self.content,
                    LAUNCHPAD_KEY.to_string(),
                    icon,
                    false,
                    self.dpi,
                ),
                Entry::Sep => View::separator(&self.comp, &self.content, self.m.separator, icon),
                Entry::Trash => View::app(
                    &self.comp,
                    &self.content,
                    TRASH_PATH.to_string(),
                    icon,
                    false,
                    self.dpi,
                ),
                Entry::Downloads => View::app(
                    &self.comp,
                    &self.content,
                    downloads_key(),
                    icon,
                    false,
                    self.dpi,
                ),
                Entry::App(i) => {
                    let it = &self.items[i];
                    View::app(
                        &self.comp,
                        &self.content,
                        it.path.clone(),
                        icon,
                        it.running(),
                        self.dpi,
                    )
                }
            };
            match v {
                Ok(mut v) => {
                    if let Kind::App { path } = &v.kind {
                        if let Some(s) = self.surfaces.get(path) {
                            let _ = v.set_surface(&self.comp, s);
                        } else if self.requested.insert(path.clone()) {
                            let _ = self.worker.requests.send(path.clone());
                        }
                    }
                    self.views.push(v);
                }
                Err(e) => log::line("dock", &format!("gagal membuat view: {e}")),
            }
        }
        if self.views.len() != self.entries.len() {
            self.entries.truncate(self.views.len());
            self.slots.truncate(self.views.len());
        }
        self.hover = None;
        self.resize_window();
        self.apply(None, false);
        self.bind_expressions();
        self.set_region(self.tracking);
    }

    pub(super) fn heal_pins(&mut self, items: &mut [Item]) {
        let mut changed = false;
        for it in items.iter_mut().filter(|i| i.pinned) {
            if it.running() {
                for p in self
                    .pins
                    .iter_mut()
                    .filter(|p| key_of(&p.path) == it.key && p.path != it.path)
                {
                    p.path = it.path.clone();
                    changed = true;
                }
            } else if !std::path::Path::new(&it.path).exists() {
                if let Some(family) = package_family(&it.path) {
                    let prefix = format!("shell:appsfolder\\{}!", family.to_lowercase());
                    if let Some(app) = nexora_platform::apps::snapshot()
                        .into_iter()
                        .find(|a| a.path.to_lowercase().starts_with(&prefix))
                    {
                        it.path = app.path;
                    }
                }
            }
        }
        if changed {
            if let Err(e) = pins::save(&self.pins_path, &self.pins) {
                log::line("pins", &format!("simpan gagal: {e}"));
            }
        }
    }

    pub(super) fn track_recents(&mut self, items: &[Item]) {
        if !crate::prefs::get().recent_apps {
            return;
        }
        let mut changed = false;
        for it in items
            .iter()
            .filter(|i| i.running() && !i.pinned && !i.path.is_empty() && !i.path.contains('?'))
            .rev()
        {
            if !self.recents.iter().any(|r| key_of(&r.path) == it.key) {
                self.recents.insert(
                    0,
                    Pin {
                        path: it.path.clone(),
                        name: it.name.clone(),
                    },
                );
                changed = true;
            }
        }
        self.recents.truncate(RECENTS_KEEP);
        if changed {
            if let Err(e) = pins::save(&self.recents_path, &self.recents) {
                log::line("recents", &format!("simpan gagal: {e}"));
            }
        }
    }

    pub(super) fn append_recents(&self, items: &mut Vec<Item>) {
        if !crate::prefs::get().recent_apps {
            return;
        }
        let mut added = 0;
        for r in &self.recents {
            if added >= RECENTS_SHOW {
                break;
            }
            let k = key_of(&r.path);
            if items.iter().any(|i| i.key == k) {
                continue;
            }
            items.push(Item {
                key: k,
                name: r.name.clone(),
                path: r.path.clone(),
                pinned: false,
                hwnds: vec![],
            });
            added += 1;
        }
    }
}
