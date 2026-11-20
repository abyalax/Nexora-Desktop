//! Static Finder menu model: the logo menu, the active-app menu and the classic File/Edit/View/Window/Help menus.
//!
//! Labels are translation keys (the Indonesian source strings of `crate::i18n`). The host translates and renders them,
//! so this module stays free of UI and OS types. Dynamic menus (Quick Panel, native app menus) are built by the host.

/// Which static menu to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
  Logo,
  App,
  File,
  Edit,
  View,
  Window,
  Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
  Separator,
  /// A plain command. `accel` is display-only text and may be empty.
  Action {
    id: usize,
    label: &'static str,
    accel: &'static str,
  },
  /// A command whose label is `"<label> <app name>"`.
  ForApp {
    id: usize,
    label: &'static str,
    accel: &'static str,
  },
}

impl Entry {
  /// Command id, or `None` for a separator.
  pub fn id(&self) -> Option<usize> {
    match self {
      Entry::Separator => None,
      Entry::Action { id, .. } | Entry::ForApp { id, .. } => Some(*id),
    }
  }

  /// Translation key, or `None` for a separator.
  pub fn label(&self) -> Option<&'static str> {
    match self {
      Entry::Separator => None,
      Entry::Action { label, .. } | Entry::ForApp { label, .. } => Some(label),
    }
  }
}

const fn act(id: usize, label: &'static str) -> Entry {
  Entry::Action { id, label, accel: "" }
}

const fn key(id: usize, label: &'static str, accel: &'static str) -> Entry {
  Entry::Action { id, label, accel }
}

const fn app(id: usize, label: &'static str, accel: &'static str) -> Entry {
  Entry::ForApp { id, label, accel }
}

const SEP: Entry = Entry::Separator;

const LOGO: [Entry; 16] = [
  act(100, "Tentang PC Ini"),
  SEP,
  act(101, "Pengaturan Sistem\u{2026}"),
  act(102, "Toko Microsoft\u{2026}"),
  SEP,
  act(103, "Pengaturan nexora\u{2026}"),
  key(105, "Apps", "Ctrl+Shift+Space"),
  key(106, "Overview", "Ctrl+\u{2191}"),
  act(104, "Paksa Keluar\u{2026}"),
  SEP,
  act(110, "Tidur"),
  act(111, "Mulai Ulang\u{2026}"),
  act(112, "Matikan\u{2026}"),
  SEP,
  act(113, "Kunci Layar"),
  act(114, "Keluar\u{2026}"),
];

const APP: [Entry; 7] = [
  app(200, "Tentang", ""),
  SEP,
  app(201, "Sembunyikan", "Ctrl+Alt+H"),
  act(202, "Sembunyikan Lainnya"),
  act(203, "Tampilkan Semua"),
  SEP,
  app(204, "Keluar dari", "Ctrl+Alt+Q"),
];

const FILE: [Entry; 7] = [
  key(300, "Baru", "Ctrl+N"),
  key(301, "Buka\u{2026}", "Ctrl+O"),
  SEP,
  key(302, "Simpan", "Ctrl+S"),
  key(303, "Tutup Jendela", "Ctrl+W"),
  SEP,
  key(304, "Cetak\u{2026}", "Ctrl+P"),
];

const EDIT: [Entry; 9] = [
  key(400, "Urungkan", "Ctrl+Z"),
  key(401, "Ulangi", "Ctrl+Y"),
  SEP,
  key(402, "Potong", "Ctrl+X"),
  key(403, "Salin", "Ctrl+C"),
  key(404, "Tempel", "Ctrl+V"),
  key(405, "Pilih Semua", "Ctrl+A"),
  SEP,
  key(406, "Cari\u{2026}", "Ctrl+F"),
];

const VIEW: [Entry; 5] = [
  key(500, "Layar Penuh", "F11"),
  SEP,
  key(501, "Perbesar", "Ctrl++"),
  key(502, "Perkecil", "Ctrl+-"),
  key(503, "Ukuran Asli", "Ctrl+0"),
];

const WINDOW: [Entry; 2] = [key(600, "Minimalkan", "Win+\u{2193}"), key(601, "Zoom", "Win+\u{2191}")];

const HELP: [Entry; 1] = [key(700, "Cari Bantuan", "F1")];

/// The entries of a static menu, in display order.
pub fn entries(kind: MenuKind) -> &'static [Entry] {
  match kind {
    MenuKind::Logo => &LOGO,
    MenuKind::App => &APP,
    MenuKind::File => &FILE,
    MenuKind::Edit => &EDIT,
    MenuKind::View => &VIEW,
    MenuKind::Window => &WINDOW,
    MenuKind::Help => &HELP,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::i18n::{self, Lang};

  const ALL: [MenuKind; 7] = [
    MenuKind::Logo,
    MenuKind::App,
    MenuKind::File,
    MenuKind::Edit,
    MenuKind::View,
    MenuKind::Window,
    MenuKind::Help,
  ];

  #[test]
  fn command_ids_are_unique_across_all_menus() {
    let mut seen = std::collections::HashSet::new();
    for kind in ALL {
      for id in entries(kind).iter().filter_map(Entry::id) {
        assert!(seen.insert(id), "duplicate command id {id} in {kind:?}");
      }
    }
  }

  #[test]
  fn separators_only_sit_between_commands() {
    for kind in ALL {
      let e = entries(kind);
      assert_ne!(e.first(), Some(&Entry::Separator), "{kind:?} starts with a separator");
      assert_ne!(e.last(), Some(&Entry::Separator), "{kind:?} ends with a separator");
      assert!(
        e.windows(2)
          .all(|w| !(w[0] == Entry::Separator && w[1] == Entry::Separator)),
        "{kind:?} has adjacent separators"
      );
    }
  }

  #[test]
  fn every_label_is_translated_to_english() {
    for kind in ALL {
      for label in entries(kind).iter().filter_map(Entry::label) {
        assert!(!label.is_empty());
        assert!(i18n::is_known(label), "missing English text for {label:?} in {kind:?}");
        assert!(!i18n::tr(Lang::En, label).is_empty());
      }
    }
  }

  #[test]
  fn logo_menu_keeps_the_documented_system_commands() {
    let ids: Vec<usize> = entries(MenuKind::Logo).iter().filter_map(Entry::id).collect();
    // About, System Settings, Store, Nexora Settings, App Grid, Overview, Force Quit, Sleep, Restart, Shut Down, Lock.
    for id in [100, 101, 102, 103, 105, 106, 104, 110, 111, 112, 113] {
      assert!(ids.contains(&id), "logo menu lost command {id}");
    }
  }

  #[test]
  fn only_the_app_menu_uses_the_app_name() {
    for kind in ALL {
      let has_app = entries(kind).iter().any(|e| matches!(e, Entry::ForApp { .. }));
      assert_eq!(has_app, kind == MenuKind::App, "{kind:?}");
    }
  }
}
