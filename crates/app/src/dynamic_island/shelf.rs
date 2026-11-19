//! File shelf: drop zones, shelf icons and the shelf context menu.

use super::*;

impl Notch {
    pub(super) fn load_shelf_icons(&mut self) {
        let px = (40.0 * self.dpi).round() as i32;
        for it in self.shelf.items().iter().take(8) {
            if self.shelf_icons.contains_key(&it.path) {
                continue;
            }
            let bmp = nexora_platform::icons::thumbnail(&it.path, px)
                .or_else(|| nexora_platform::icons::load(&it.path, px));
            if let Some(b) = bmp {
                if let Ok(s) = self.paint.gfx.surface_from_bgra(b.w, b.h, &b.bgra) {
                    self.shelf_icons.insert(it.path.clone(), s);
                }
            }
        }
    }

    pub(super) fn zone_at(&self, x: f32, y: f32) -> Option<usize> {
        self.regions
            .iter()
            .find(|r| matches!(r.action, Action::Zone(_)) && r.hit(x, y))
            .and_then(|r| match r.action {
                Action::Zone(i) => Some(i),
                _ => None,
            })
    }

    pub(super) fn on_dnd(&mut self, kind: usize, lp: LPARAM) {
        let (sx, sy) = (
            (lp.0 as u32 & 0xFFFF) as u16 as i16 as f32,
            ((lp.0 as u32 >> 16) & 0xFFFF) as u16 as i16 as f32,
        );
        let (cx, cy) = (sx - ((self.screen_w - self.win.0) / 2) as f32, sy);
        match kind {
            drop::ENTER => {
                self.drop_zone = None;
                self.dispatch(Event::DragEnter);
                self.drop_zone = self.zone_at(cx, cy);
                self.render();
            }
            drop::OVER => {
                let z = self.zone_at(cx, cy);
                if z != self.drop_zone {
                    self.drop_zone = z;
                    self.render();
                }
            }
            drop::LEAVE => {
                self.drop_zone = None;
                self.dispatch(Event::DragLeave);
            }
            drop::DROP => {
                let zone = self.zone_at(cx, cy).unwrap_or(0);
                let paths = self.dnd.borrow().clone();
                self.drop_zone = None;
                log::line(
                    "notch",
                    &format!("ditaruh {} berkas di zona {zone}", paths.len()),
                );
                if zone == 0 {
                    for p in &paths {
                        self.shelf.add(p);
                    }
                    self.dispatch(Event::Dropped);
                } else if zone == 1 {
                    self.start_send(paths);
                } else {
                    self.pending = paths;
                    self.dispatch(Event::DragLeave);
                    post_w(self.hwnd.0 as isize, WM_AFTERDROP, zone);
                }
            }
            _ => {}
        }
    }

    pub(super) fn shelf_menu(&mut self, i: usize, x: f32, y: f32) {
        if self.menu.is_none() {
            match Menu::new(
                &self.paint.comp,
                self.hwnd,
                self.dpi,
                sys::screen_size(),
                Tier::Full,
            ) {
                Ok(m) => self.menu = Some(m),
                Err(_) => return,
            }
        }
        let tr = crate::prefs::tr;
        let items = vec![
            MItem::action(1, tr("Buka"), true),
            MItem::action(2, tr("Tampilkan di folder"), true),
            MItem::action(3, tr("Salin jalur"), true),
            MItem::action(5, tr("Kirim ke HP"), crate::prefs::get().notch_send),
            MItem::separator(),
            MItem::action(4, tr("Hapus dari Rak"), true),
        ];
        self.menu_item = Some(i);
        let wx = (self.screen_w - self.win.0) / 2;
        if let Some(m) = self.menu.as_mut() {
            let _ = m.show(
                &self.paint.comp,
                &self.paint.gfx,
                items,
                wx + x as i32,
                y as i32,
            );
        }
    }

    pub(super) fn menu_pick(&mut self, id: usize) {
        if let Some(m) = &self.menu {
            m.hide();
        }
        let Some(i) = self.menu_item.take() else {
            return;
        };
        let Some(path) = self.shelf.items().get(i).map(|it| it.path.clone()) else {
            return;
        };
        match id {
            1 => nexora_platform::launch::open(&path),
            2 => nexora_platform::launch::run("explorer.exe", &format!("/select,\"{path}\"")),
            3 => {
                let _ = nexora_platform::clipboard::copy_text(&path);
            }
            4 => {
                self.shelf.remove(&path);
                self.shelf_icons.remove(&path);
                self.refresh();
            }
            5 => self.start_send(vec![path]),
            _ => {}
        }
    }
}

pub(super) fn after_drop(paths: &[String], zone: usize) {
    if paths.is_empty() {
        return;
    }
    if zone == 2 {
        let Some(dir) = nexora_platform::pick::folder(crate::prefs::tr("Salin ke folder")) else {
            return;
        };
        let from: Vec<u16> = paths
            .iter()
            .flat_map(|p| p.encode_utf16().chain(std::iter::once(0)))
            .chain(std::iter::once(0))
            .collect();
        let to: Vec<u16> = dir.encode_utf16().chain([0, 0]).collect();
        let mut op = windows::Win32::UI::Shell::SHFILEOPSTRUCTW {
            wFunc: windows::Win32::UI::Shell::FO_COPY,
            pFrom: windows::core::PCWSTR(from.as_ptr()),
            pTo: windows::core::PCWSTR(to.as_ptr()),
            fFlags: windows::Win32::UI::Shell::FOF_ALLOWUNDO.0 as u16,
            ..Default::default()
        };
        let r = unsafe { windows::Win32::UI::Shell::SHFileOperationW(&mut op) };
        log::line(
            "notch",
            &format!("salin {} berkas ke folder: hasil={r}", paths.len()),
        );
    } else {
        let file: Vec<u16> = paths[0].encode_utf16().chain(std::iter::once(0)).collect();
        let info = windows::Win32::UI::Shell::OPENASINFO {
            pcszFile: windows::core::PCWSTR(file.as_ptr()),
            pcszClass: windows::core::PCWSTR::null(),
            oaifInFlags: windows::Win32::UI::Shell::OAIF_ALLOW_REGISTRATION
                | windows::Win32::UI::Shell::OAIF_EXEC,
        };
        let _ = unsafe { windows::Win32::UI::Shell::SHOpenWithDialog(None, &info) };
    }
}
