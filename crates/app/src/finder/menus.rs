//! Finder menu model and popup handling.

use super::*;
use nexora_core::finder::menus::{self, Entry, MenuKind};

impl Bar {
  pub(super) fn id_at(&self, x: f32) -> Option<u32> {
    lay::hit(&self.left, x).or_else(|| lay::hit(&self.right, x))
  }

  pub(super) fn has_menu(id: u32) -> bool {
    matches!(id, LOGO | APP | M_FILE | M_EDIT | M_VIEW | M_WIN | M_HELP) || (NATIVE0..NATIVE0 + 20).contains(&id)
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
        let s = nexora_core::symbols::for_command(it.id);
        it.with_sym(s)
      })
      .collect()
  }

  pub(super) fn menu_items_raw(&self, id: u32) -> Vec<Item> {
    let a = |id: usize, label: &str| Item::action(id, label, true);
    if (NATIVE0..NATIVE0 + 20).contains(&id) {
      let tops = nexora_platform::menus::read(self.native_hwnd).unwrap_or_else(|| self.native.clone());
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
          let label = format!("{}{}{}", if e.checked { "\u{2713} " } else { "" }, pad, e.label);
          let it = Item::action(
            if e.header { 0 } else { NATIVE_CMD + e.id as usize },
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
        let vol = nexora_platform::audio::latest();
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
        let btxt = match nexora_platform::brightness::latest() {
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
            nexora_platform::brightness::latest().is_some(),
          ),
          Item::action(
            806,
            &prefs::tr("Lebih Redup"),
            nexora_platform::brightness::latest().is_some(),
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
          a(810, &prefs::tr("Pengaturan nexora…")),
          a(811, &prefs::tr("Pengaturan Sistem…")),
        ]
      }
      LOGO => self.static_menu(MenuKind::Logo),
      APP => self.static_menu(MenuKind::App),
      M_FILE => self.static_menu(MenuKind::File),
      M_EDIT => self.static_menu(MenuKind::Edit),
      M_VIEW => self.static_menu(MenuKind::View),
      M_WIN => self.static_menu(MenuKind::Window),
      M_HELP => self.static_menu(MenuKind::Help),
      _ => vec![],
    }
  }

  /// Build a static menu from the core model, translating labels for the current language.
  fn static_menu(&self, kind: MenuKind) -> Vec<Item> {
    let name = if kind == MenuKind::App {
      self.app_label()
    } else {
      String::new()
    };
    menus::entries(kind)
      .iter()
      .map(|entry| match *entry {
        Entry::Separator => Item::separator(),
        Entry::Action { id, label, accel } => Item::action(id, prefs::tr(label), true).with_accel(accel),
        Entry::ForApp { id, label, accel } => {
          Item::action(id, &format!("{} {}", prefs::tr(label), name), true).with_accel(accel)
        }
      })
      .collect()
  }
}
