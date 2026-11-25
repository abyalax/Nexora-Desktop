use super::*;

impl Mgr {
  pub(super) fn pitch(&self) -> i32 {
    ((widgets::CELL + widgets::GAP) * self.dpi).round() as i32
  }

  pub(super) fn grid(&self) -> ((i32, i32), i32, (i32, i32)) {
    let ar = area(self.dpi);
    let step = self.pitch();
    let gap = (widgets::GAP * self.dpi).round() as i32;
    (
      (ar.0, ar.1),
      step,
      (((ar.2 + gap) / step).max(1), ((ar.3 + gap) / step).max(1)),
    )
  }

  pub(super) fn cell_of(&self, pos: (i32, i32)) -> (i32, i32) {
    let (o, step, _) = self.grid();
    (
      ((pos.0 - o.0) as f32 / step as f32).round() as i32,
      ((pos.1 - o.1) as f32 / step as f32).round() as i32,
    )
  }

  pub(super) fn cell_px(&self, cell: (i32, i32)) -> (i32, i32) {
    let (o, step, _) = self.grid();
    (o.0 + cell.0 * step, o.1 + cell.1 * step)
  }

  pub(super) fn cells_taken(&self, skip: Option<usize>) -> Vec<widgets::Rect> {
    self
      .list
      .iter()
      .enumerate()
      .filter(|(i, _)| Some(*i) != skip)
      .map(|(_, w)| {
        let c = self.cell_of(w.pos);
        let (cw, ch) = w.kind.cells();
        (c.0, c.1, cw as i32, ch as i32)
      })
      .collect()
  }

  pub(super) fn slot_kind(&self, skip: Option<usize>, kind: Kind, pos: (i32, i32)) -> (i32, i32) {
    let (_, _, grid) = self.grid();
    let (cw, ch) = kind.cells();
    let cell = widgets::nearest_cell(&self.cells_taken(skip), (cw as i32, ch as i32), self.cell_of(pos), grid)
      .unwrap_or_else(|| self.cell_of(pos));
    self.cell_px(cell)
  }

  pub(super) fn slot_for(&self, skip: Option<usize>, _size: (i32, i32), pos: (i32, i32)) -> (i32, i32) {
    let kind = skip
      .and_then(|i| self.list.get(i))
      .map(|w| w.kind)
      .unwrap_or(Kind::Clock);
    self.slot_kind(skip, kind, pos)
  }

  pub(super) fn load(&mut self) {
    let placed = widgets::from_text(&std::fs::read_to_string(list_file()).unwrap_or_default());
    let mut moved = false;
    for p in placed {
      let at = self.slot_kind(None, p.kind, (p.x, p.y));
      moved |= at != (p.x, p.y);
      self.spawn(p.kind, at, false);
    }
    if moved {
      self.save();
    }
    log::line("widgets", &format!("{} widget dimuat", self.list.len()));
  }

  pub(super) fn save(&self) {
    let list: Vec<Placed> = self
      .list
      .iter()
      .map(|w| Placed {
        kind: w.kind,
        x: w.pos.0,
        y: w.pos.1,
      })
      .collect();
    let _ = std::fs::create_dir_all(paths::config_dir());
    let _ = std::fs::write(list_file(), widgets::to_text(&list));
  }

  pub(super) fn rebuild_all(&mut self) {
    let placed: Vec<(Kind, (i32, i32))> = self.list.iter().map(|w| (w.kind, w.pos)).collect();
    for w in self.list.drain(..) {
      unsafe {
        let _ = DestroyWindow(w.popup.hwnd);
      }
    }
    self.cache.clear();
    for (k, p) in placed {
      self.spawn(k, p, false);
    }
  }
}
