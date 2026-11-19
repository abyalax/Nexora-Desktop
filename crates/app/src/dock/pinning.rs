//! Pin and recent-item bookkeeping.

use super::*;

impl Dock {
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
        if recents::track(&mut self.recents, items) {
            if let Err(e) = pins::save(&self.recents_path, &self.recents) {
                log::line("recents", &format!("simpan gagal: {e}"));
            }
        }
    }

    pub(super) fn append_recents(&self, items: &mut Vec<Item>) {
        if crate::prefs::get().recent_apps {
            recents::append(items, &self.recents);
        }
    }

    pub fn add_app(&mut self, picked: &str) {
        let lower = picked.to_ascii_lowercase();
        let Some(target) = nexora_platform::pick::resolve_shortcut(picked) else {
            log::line("dock", &format!("pintasan tak terbaca: {picked}"));
            return;
        };
        let is_store = target.to_ascii_lowercase().starts_with("shell:appsfolder");
        if !(lower.ends_with(".exe") || lower.ends_with(".lnk"))
            || !(is_store || std::path::Path::new(&target).exists())
        {
            log::line("dock", &format!("bukan app yang valid: {picked}"));
            return;
        }
        let k = key_of(&target);
        if self.pins.iter().any(|p| key_of(&p.path) == k) {
            return;
        }
        let name = if lower.ends_with(".lnk") {
            std::path::Path::new(picked)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| pins::name_for(&target))
        } else {
            pins::name_for(&target)
        };
        log::line("dock", &format!("tambah app: {name} -> {target}"));
        self.pins.push(Pin { path: target, name });
        if let Err(e) = pins::save(&self.pins_path, &self.pins) {
            log::line("pins", &format!("simpan gagal: {e}"));
        }
        self.signature.clear();
        self.refresh_windows();
    }
}
