use super::*;

impl Mgr {
  pub(super) fn add(&mut self, kind: Kind, at: Option<(i32, i32)>) -> Option<usize> {
    let d = self.dpi;
    let (w, h) = widgets::size_of(kind, d);
    let _ = (w, h, d);
    let pos = match at {
      Some(p) => p,
      None => {
        let (_, _, grid) = self.grid();
        let (cw, ch) = kind.cells();
        let cell = widgets::first_cell(&self.cells_taken(None), (cw as i32, ch as i32), grid).unwrap_or((0, 0));
        self.cell_px(cell)
      }
    };
    let i = self.spawn(kind, pos, at.is_none())?;
    if at.is_none() {
      self.save();
    }
    Some(i)
  }

  pub(super) fn remove(&mut self, i: usize) {
    if i >= self.list.len() {
      return;
    }
    if self.pending_remove.is_some() {
      self.finish_remove();
    }
    let c = self.center(i);
    let _ = motion::pop_out(&self.comp, self.list[i].popup.root(), c, 0.4, 180);
    self.pending_remove = Some(self.list[i].popup.hwnd);
    unsafe {
      SetTimer(Some(self.ctl), T_REMOVE, 190, None);
    }
  }

  pub(super) fn finish_remove(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_REMOVE);
    }
    let Some(h) = self.pending_remove.take() else {
      return;
    };
    if let Some(i) = self.list.iter().position(|w| w.popup.hwnd == h) {
      let w = self.list.remove(i);
      unsafe {
        let _ = DestroyWindow(w.popup.hwnd);
      }
      self.save();
    }
  }

  pub(super) fn index_of(&self, h: HWND) -> Option<usize> {
    self.list.iter().position(|w| w.popup.hwnd == h)
  }

  pub(super) fn open_app(kind: Kind) {
    match kind {
      Kind::Clock | Kind::WorldClock | Kind::Digital => launch::open("ms-clock:"),
      Kind::Calendar | Kind::Month => launch::open("ms-actioncenter:"),
      Kind::Battery => launch::open("ms-settings:batterysaver"),
      Kind::System => launch::run("taskmgr.exe", ""),
    }
  }

  pub(super) fn show_menu(&mut self, i: usize) {
    let (x, y) = cursor();
    let d = self.dpi;
    if self.menu.is_none() {
      self.menu = Menu::new(&self.comp, self.ctl, d, sys::screen_size(), Tier::Full).ok();
    }
    let items = vec![
      MItem::action(CMD_EDIT, prefs::tr("Edit Widget\u{2026}"), true),
      MItem::action(CMD_REMOVE, prefs::tr("Hapus Widget"), true),
    ];
    self.menu_for = Some(i);
    let comp = self.comp.clone();
    if let (Some(m), Some(g)) = (self.menu.as_mut(), self.gfx.as_ref()) {
      let _ = m.show(&comp, g, items, x, y);
    }
  }

  pub(super) fn set_edit(&mut self, on: bool) {
    if self.edit == on {
      return;
    }
    self.edit = on;
    for i in 0..self.list.len() {
      self.paint(i);
      let c = self.center(i);
      let _ = motion::pop_in(&self.comp, self.list[i].popup.root(), c, 0.9, 1.04, i as u32 * 18, 320);
    }
  }
}
