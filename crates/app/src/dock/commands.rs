//! Context menu, stack popup and command dispatch.

use super::*;

impl Dock {
  pub(super) fn menu_info(&self, x: f32) -> MenuInfo {
    match self.entry_at(x) {
      Some((_, Entry::App(i))) => MenuInfo {
        item: self.items.get(i).cloned(),
        trash: false,
        special: None,
      },
      Some((_, Entry::Trash)) => MenuInfo {
        item: None,
        trash: true,
        special: None,
      },
      Some((_, Entry::Downloads)) => MenuInfo {
        item: None,
        trash: false,
        special: Some(DOWNLOADS_KEY),
      },
      Some((_, Entry::Start)) => MenuInfo {
        item: None,
        trash: false,
        special: Some(START_KEY),
      },
      Some((_, Entry::Launchpad)) => MenuInfo {
        item: None,
        trash: false,
        special: Some(LAUNCHPAD_KEY),
      },
      _ => MenuInfo {
        item: None,
        trash: false,
        special: None,
      },
    }
  }

  pub(super) fn open_menu(&mut self, x: f32) {
    self.state.menu_open = true;
    self.hide_popups();
    let info = self.menu_info(x);
    let mut items = Vec::new();
    if info.trash {
      items.push(crate::menu::Item::action(
        CMD_TRASH_OPEN,
        crate::prefs::tr("Buka"),
        true,
      ));
      items.push(crate::menu::Item::action(
        CMD_TRASH_EMPTY,
        crate::prefs::tr("Kosongkan tempat sampah"),
        true,
      ));
    }
    if let Some(it) = &info.item {
      items.push(crate::menu::Item::action(CMD_OPEN, crate::prefs::tr("Buka"), true));
      let can_pin = it.pinned || !it.path.contains('?');
      items.push(crate::menu::Item::action(
        CMD_PIN,
        crate::prefs::tr(if it.pinned { "Hapus dari Dock" } else { "Simpan di Dock" }),
        can_pin,
      ));
      if !it.pinned && !it.running() {
        items.push(crate::menu::Item::action(
          CMD_FORGET,
          crate::prefs::tr("Hapus dari Dock"),
          true,
        ));
      }
      items.push(crate::menu::Item::action(
        CMD_ICON,
        crate::prefs::tr("Ubah ikon…"),
        true,
      ));
      if custom_icons::has(&it.path) {
        items.push(crate::menu::Item::action(
          CMD_ICON_RESET,
          crate::prefs::tr("Kembalikan ikon asli"),
          true,
        ));
      }
      if it.running() {
        items.push(crate::menu::Item::separator());
        items.push(crate::menu::Item::action(CMD_CLOSE, crate::prefs::tr("Tutup"), true));
      }
    }
    if info.special == Some(DOWNLOADS_KEY) {
      items.push(crate::menu::Item::action(
        CMD_STACK_FOLDER,
        crate::prefs::tr("Buka folder Unduhan"),
        true,
      ));
      items.push(crate::menu::Item::separator());
    } else if let Some(key) = info.special {
      items.push(crate::menu::Item::action(
        CMD_OPEN,
        crate::prefs::tr(if key == START_KEY {
          "Buka menu Mulai"
        } else {
          "Buka Apps"
        }),
        true,
      ));
      items.push(crate::menu::Item::action(
        CMD_ICON,
        crate::prefs::tr("Ubah ikon…"),
        true,
      ));
      if custom_icons::has(key) {
        items.push(crate::menu::Item::action(
          CMD_ICON_RESET,
          crate::prefs::tr("Kembalikan ikon asli"),
          true,
        ));
      }
      items.push(crate::menu::Item::separator());
    }
    let general = info.item.is_none() && !info.trash && (info.special.is_none() || info.special == Some(START_KEY));
    if general {
      items.push(crate::menu::Item::action(
        CMD_ADD,
        crate::prefs::tr("Tambah app ke Dock…"),
        true,
      ));
      items.push(crate::menu::Item::action(
        CMD_SETTINGS,
        crate::prefs::tr("Pengaturan nexora…"),
        true,
      ));
      items.push(crate::menu::Item::separator());
      items.push(crate::menu::Item::action(
        CMD_QUIT,
        crate::prefs::tr("Keluar dari nexora"),
        true,
      ));
    }
    while items.last().is_some_and(|i| i.separator) {
      items.pop();
    }
    self.pending_menu = Some(info);
    if self.menu.is_none() {
      self.menu = Menu::new(&self.comp, self.hwnd, self.dpi, self.screen, self.tier).ok();
    }
    let mut pt = POINT::default();
    unsafe {
      let _ = GetCursorPos(&mut pt);
    }
    let bottom = self.win_pos.1 + self.card_y as i32 - (6.0 * self.dpi) as i32;
    log::line(
      "dock",
      &format!("menu: kursor=({},{}) bawah={bottom} dari klik x={x}", pt.x, pt.y),
    );
    if let Some(m) = self.menu.as_mut() {
      if let Err(e) = m.show(&self.comp, &self.gfx, items, pt.x, bottom) {
        log::line("menu", &format!("gagal: {e}"));
        self.on_menu_close();
      }
    } else {
      self.on_menu_close();
    }
  }

  pub(super) fn open_stack(&mut self) {
    self.state.menu_open = true;
    self.hide_popups();
    let files = nexora_platform::downloads::latest(10);
    let mut items = Vec::new();
    self.stack_files.clear();
    for (name, path) in files.iter().rev() {
      let mut label: String = name.chars().take(48).collect();
      if name.chars().count() > 48 {
        label.push('…');
      }
      items.push(crate::menu::Item::action(
        CMD_STACK + self.stack_files.len(),
        &label,
        true,
      ));
      self.stack_files.push(path.clone());
    }
    if !items.is_empty() {
      items.push(crate::menu::Item::separator());
    }
    items.push(crate::menu::Item::action(
      CMD_STACK_FOLDER,
      crate::prefs::tr("Buka folder Unduhan"),
      true,
    ));
    self.pending_menu = Some(MenuInfo {
      item: None,
      trash: false,
      special: Some(DOWNLOADS_KEY),
    });
    if self.menu.is_none() {
      self.menu = Menu::new(&self.comp, self.hwnd, self.dpi, self.screen, self.tier).ok();
    }
    let mut pt = POINT::default();
    unsafe {
      let _ = GetCursorPos(&mut pt);
    }
    let bottom = self.win_pos.1 + self.card_y as i32 - (6.0 * self.dpi) as i32;
    if let Some(m) = self.menu.as_mut() {
      if let Err(e) = m.show(&self.comp, &self.gfx, items, pt.x, bottom) {
        log::line("menu", &format!("gagal: {e}"));
        self.on_menu_close();
      }
    } else {
      self.on_menu_close();
    }
  }

  pub(super) fn on_menu_pick(&mut self, id: usize) {
    log::line("dock", &format!("menu: pilih {id}"));
    self.state.menu_open = false;
    self.start_grace();
    if let Some(info) = self.pending_menu.take() {
      self.run_command(id, info);
    }
    self.update_visibility();
  }

  pub(super) fn on_menu_close(&mut self) {
    log::line("dock", "menu: ditutup");
    if let Some(m) = &self.menu {
      m.hide();
    }
    self.state.menu_open = false;
    self.pending_menu = None;
    self.start_grace();
    self.update_visibility();
  }

  pub(super) fn run_command(&mut self, cmd: usize, info: MenuInfo) {
    match cmd {
      CMD_STACK_FOLDER => {
        if let Some(d) = nexora_platform::downloads::folder() {
          launch::open(&d.to_string_lossy());
        }
      }
      c if c >= CMD_STACK => {
        if let Some(p) = self.stack_files.get(c - CMD_STACK).cloned() {
          launch::open(&p);
        }
      }
      CMD_OPEN => {
        if let Some(it) = &info.item {
          launch::open(&it.path);
        } else if let Some(key) = info.special {
          if key == START_KEY {
            launch::toggle_start(false);
          } else {
            crate::launchpad::toggle();
          }
        }
      }
      CMD_CLOSE => {
        if let Some(it) = &info.item {
          for h in &it.hwnds {
            focus::request_close(*h);
          }
        }
      }
      CMD_PIN => {
        if let Some(it) = &info.item {
          if it.pinned {
            self.pins.retain(|p| key_of(&p.path) != it.key);
          } else if !it.path.contains('?') {
            self.pins.push(Pin {
              path: it.path.clone(),
              name: it.name.clone(),
            });
          }
          if let Err(e) = pins::save(&self.pins_path, &self.pins) {
            log::line("pins", &format!("simpan gagal: {e}"));
          }
          self.signature.clear();
          self.refresh_windows();
        }
      }
      CMD_FORGET => {
        if let Some(it) = &info.item {
          self.recents.retain(|r| key_of(&r.path) != it.key);
          if let Err(e) = pins::save(&self.recents_path, &self.recents) {
            log::line("recents", &format!("simpan gagal: {e}"));
          }
          self.signature.clear();
          self.refresh_windows();
        }
      }
      CMD_SETTINGS => crate::settings_win::open(&self.comp, None),
      CMD_TRASH_OPEN => launch::open("shell:RecycleBinFolder"),
      CMD_TRASH_EMPTY => unsafe {
        let _ = windows::Win32::UI::Shell::SHEmptyRecycleBinW(Some(self.hwnd), windows::core::PCWSTR::null(), 0);
        timer(self.hwnd, TIMER_TRASH, 300);
      },
      CMD_ADD => {
        let picked = nexora_platform::pick::open_file(
          crate::prefs::tr("Pilih aplikasi"),
          &[(crate::prefs::tr("Aplikasi"), "*.exe;*.lnk"), ("*.*", "*.*")],
        );
        if let Some(p) = picked {
          self.add_app(&p);
        }
      }
      CMD_ICON if info.special.is_some() => {
        let key = info.special.unwrap_or(START_KEY);
        let picked = nexora_platform::pick::open_file(
          crate::prefs::tr("Pilih gambar ikon"),
          &[(crate::prefs::tr("Gambar"), "*.png;*.jpg;*.jpeg;*.ico;*.bmp;*.gif")],
        );
        if let Some(img) = picked {
          if custom_icons::set(key, &img) {
            self.reload_icon(key);
            self.signature.clear();
            self.refresh_windows();
          } else {
            log::line("dock", "gambar ikon tidak valid");
          }
        }
      }
      CMD_ICON_RESET if info.special.is_some() => {
        let key = info.special.unwrap_or(START_KEY);
        custom_icons::reset(key);
        self.surfaces.remove(key);
        self.signature.clear();
        self.refresh_windows();
      }
      CMD_ICON => {
        if let Some(it) = &info.item {
          let picked = nexora_platform::pick::open_file(
            crate::prefs::tr("Pilih gambar ikon"),
            &[(crate::prefs::tr("Gambar"), "*.png;*.jpg;*.jpeg;*.ico;*.bmp;*.gif")],
          );
          if let Some(img) = picked {
            if custom_icons::set(&it.path, &img) {
              self.reload_icon(&it.path);
            } else {
              log::line("dock", "gambar ikon tidak valid");
            }
          }
        }
      }
      CMD_ICON_RESET => {
        if let Some(it) = &info.item {
          custom_icons::reset(&it.path);
          self.reload_icon(&it.path);
        }
      }
      CMD_QUIT => unsafe {
        let _ = PostMessageW(
          Some(self.main_hwnd),
          WM_COMMAND,
          WPARAM(crate::mainwin::ID_QUIT),
          LPARAM(0),
        );
      },
      _ => {}
    }
  }
}
