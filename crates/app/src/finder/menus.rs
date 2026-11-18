//! Finder menu model and popup handling.

use super::*;

impl Bar {
    pub(super) fn id_at(&self, x: f32) -> Option<u32> {
        lay::hit(&self.left, x).or_else(|| lay::hit(&self.right, x))
    }

    pub(super) fn has_menu(id: u32) -> bool {
        matches!(id, LOGO | APP | M_FILE | M_EDIT | M_VIEW | M_WIN | M_HELP)
            || (NATIVE0..NATIVE0 + 20).contains(&id)
    }

    pub(super) fn open_menu(&mut self, id: u32) {
        let items = self.menu_items(id);
        log::line("menubar", &format!("buka menu {id}: {} item", items.len()));
        let Some((x, _)) = self.rect_of(id) else {
            return;
        };
        let (comp, gfx) = (self.comp.clone(), &self.gfx);
        let top = self.h;
        let r = self.menu.show_below(&comp, gfx, items, x as i32, top);
        if let Err(e) = &r {
            log::line("menubar", &format!("menu gagal: {e}"));
        }
        if r.is_ok() {
            self.open = Some(id);
            self.update_capsule();
        }
    }

    pub(super) fn close_menu(&mut self) {
        self.menu.hide();
        self.open = None;
        self.update_capsule();
    }

    pub(super) fn menu_items(&self, id: u32) -> Vec<Item> {
        let items = self.menu_items_raw(id);
        if (NATIVE0..NATIVE0 + 20).contains(&id) {
            return items;
        }
        items
            .into_iter()
            .map(|it| {
                let s = nacre_core::symbols::for_command(it.id);
                it.with_sym(s)
            })
            .collect()
    }

    pub(super) fn menu_items_raw(&self, id: u32) -> Vec<Item> {
        let a = |id: usize, label: &str| Item::action(id, label, true);
        if (NATIVE0..NATIVE0 + 20).contains(&id) {
            let tops = nacre_platform::menus::read(self.native_hwnd)
                .unwrap_or_else(|| self.native.clone());
            let Some(top) = tops.get((id - NATIVE0) as usize) else {
                return vec![];
            };
            return top
                .entries
                .iter()
                .map(|e| {
                    if e.separator {
                        return Item::separator();
                    }
                    let pad = "    ".repeat(e.depth as usize);
                    let label = format!(
                        "{}{}{}",
                        if e.checked { "\u{2713} " } else { "" },
                        pad,
                        e.label
                    );
                    let it = Item::action(
                        if e.header {
                            0
                        } else {
                            NATIVE_CMD + e.id as usize
                        },
                        &label,
                        e.enabled && !e.header,
                    );
                    if e.accel.is_empty() {
                        it
                    } else {
                        it.with_accel(&e.accel)
                    }
                })
                .collect();
        }
        match id {
            CTRL => {
                let on = |b: Option<bool>| match b {
                    Some(true) => format!("\u{2713} {}", prefs::tr("Aktif")),
                    Some(false) => prefs::tr("Mati").to_string(),
                    None => "-".to_string(),
                };
                let vol = nacre_platform::audio::latest();
                let vtxt = if vol.available {
                    format!(
                        "{} {}%{}",
                        prefs::tr("Volume"),
                        (vol.level * 100.0).round() as i32,
                        if vol.muted {
                            format!(" {}", prefs::tr("(senyap)"))
                        } else {
                            String::new()
                        }
                    )
                } else {
                    prefs::tr("Volume").to_string()
                };
                let btxt = match nacre_platform::brightness::latest() {
                    Some(v) => format!("{} {}%", prefs::tr("Kecerahan"), v),
                    None => prefs::tr("Kecerahan").to_string(),
                };
                vec![
                    a(800, &format!("Wi-Fi  \u{2013}  {}", on(self.wifi))).with_accel(""),
                    a(801, &format!("Bluetooth  \u{2013}  {}", on(self.bt))),
                    Item::separator(),
                    Item::action(0, &vtxt, false),
                    Item::action(802, &prefs::tr("Naikkan Volume"), vol.available),
                    Item::action(803, &prefs::tr("Turunkan Volume"), vol.available),
                    Item::action(804, &prefs::tr("Bisukan / Bunyikan"), vol.available),
                    Item::separator(),
                    Item::action(0, &btxt, false),
                    Item::action(
                        805,
                        &prefs::tr("Lebih Terang"),
                        nacre_platform::brightness::latest().is_some(),
                    ),
                    Item::action(
                        806,
                        &prefs::tr("Lebih Redup"),
                        nacre_platform::brightness::latest().is_some(),
                    ),
                    Item::separator(),
                    a(
                        807,
                        &format!(
                            "{}{}",
                            if prefs::get().stage { "\u{2713} " } else { "" },
                            prefs::tr("Scene")
                        ),
                    ),
                    a(808, &prefs::tr("Overview")).with_accel("Ctrl+↑"),
                    a(809, &prefs::tr("Apps")).with_accel("Ctrl+Shift+Space"),
                    Item::separator(),
                    a(810, &prefs::tr("Pengaturan NEXORA…")),
                    a(811, &prefs::tr("Pengaturan Sistem…")),
                ]
            }
            LOGO => vec![
                a(100, &prefs::tr("Tentang PC Ini")),
                Item::separator(),
                a(101, &prefs::tr("Pengaturan Sistem…")),
                a(102, &prefs::tr("Toko Microsoft…")),
                Item::separator(),
                a(103, &prefs::tr("Pengaturan NEXORA…")),
                a(105, &prefs::tr("Apps")).with_accel("Ctrl+Shift+Space"),
                a(106, &prefs::tr("Overview")).with_accel("Ctrl+↑"),
                a(104, &prefs::tr("Paksa Keluar…")),
                Item::separator(),
                a(110, &prefs::tr("Tidur")),
                a(111, &prefs::tr("Mulai Ulang…")),
                a(112, &prefs::tr("Matikan…")),
                Item::separator(),
                a(113, &prefs::tr("Kunci Layar")),
                a(114, &prefs::tr("Keluar…")),
            ],
            APP => {
                let name = self.app_label();
                vec![
                    a(200, &format!("{} {}", prefs::tr("Tentang"), name)),
                    Item::separator(),
                    a(201, &format!("{} {}", prefs::tr("Sembunyikan"), name))
                        .with_accel("Ctrl+Alt+H"),
                    a(202, &prefs::tr("Sembunyikan Lainnya")),
                    a(203, &prefs::tr("Tampilkan Semua")),
                    Item::separator(),
                    a(204, &format!("{} {}", prefs::tr("Keluar dari"), name))
                        .with_accel("Ctrl+Alt+Q"),
                ]
            }
            M_FILE => vec![
                a(300, &prefs::tr("Baru")).with_accel("Ctrl+N"),
                a(301, &prefs::tr("Buka…")).with_accel("Ctrl+O"),
                Item::separator(),
                a(302, &prefs::tr("Simpan")).with_accel("Ctrl+S"),
                a(303, &prefs::tr("Tutup Jendela")).with_accel("Ctrl+W"),
                Item::separator(),
                a(304, &prefs::tr("Cetak…")).with_accel("Ctrl+P"),
            ],
            M_EDIT => vec![
                a(400, &prefs::tr("Urungkan")).with_accel("Ctrl+Z"),
                a(401, &prefs::tr("Ulangi")).with_accel("Ctrl+Y"),
                Item::separator(),
                a(402, &prefs::tr("Potong")).with_accel("Ctrl+X"),
                a(403, &prefs::tr("Salin")).with_accel("Ctrl+C"),
                a(404, &prefs::tr("Tempel")).with_accel("Ctrl+V"),
                a(405, &prefs::tr("Pilih Semua")).with_accel("Ctrl+A"),
                Item::separator(),
                a(406, &prefs::tr("Cari…")).with_accel("Ctrl+F"),
            ],
            M_VIEW => vec![
                a(500, &prefs::tr("Layar Penuh")).with_accel("F11"),
                Item::separator(),
                a(501, &prefs::tr("Perbesar")).with_accel("Ctrl++"),
                a(502, &prefs::tr("Perkecil")).with_accel("Ctrl+-"),
                a(503, &prefs::tr("Ukuran Asli")).with_accel("Ctrl+0"),
            ],
            M_WIN => vec![
                a(600, &prefs::tr("Minimalkan")).with_accel("Win+↓"),
                a(601, &prefs::tr("Zoom")).with_accel("Win+↑"),
            ],
            M_HELP => vec![a(700, &prefs::tr("Cari Bantuan")).with_accel("F1")],
            _ => vec![],
        }
    }
}
