//! Pointer input: hover, press, drag-to-reorder, click and file drop.

use super::*;

impl Dock {
  pub(super) fn entry_at(&self, x: f32) -> Option<(usize, Entry)> {
    let i = hit(&self.slots, &self.layout, x, &self.m)?;
    Some((i, *self.entries.get(i)?))
  }

  pub(super) fn on_move(&mut self, x: f32, y: f32) {
    if self.press.is_some() {
      self.drag_update(x, y);
      return;
    }
    if !self.region_full {
      kill(self.hwnd, TIMER_REGION);
      self.set_region(true);
    }
    if self.grace {
      self.grace = false;
      kill(self.hwnd, TIMER_GRACE);
    }
    let slot = self.entry_at(x).map(|(i, _)| i);
    if slot != self.hover {
      self.hover = slot;
      if let Some(t) = &self.tooltip {
        t.hide();
      }
      kill(self.hwnd, TIMER_TIP);
      kill(self.hwnd, TIMER_PREVIEW);
      if slot.is_some() {
        let preview_up = self.preview.as_ref().is_some_and(|p| p.is_visible());
        if preview_up {
          self.show_preview();
          if !self.preview.as_ref().is_some_and(|p| p.is_visible()) {
            timer(self.hwnd, TIMER_TIP, 250);
          }
        } else {
          timer(self.hwnd, TIMER_TIP, 250);
          timer(self.hwnd, TIMER_PREVIEW, 450);
        }
      }
    }
    if self.cursor.is_some_and(|c| (c - x).abs() < 0.5)
      || (!self.exprs_ok && self.last_apply.elapsed() < Duration::from_millis(10))
    {
      return;
    }
    self.cursor = Some(x);
    if self.exprs_ok {
      self.layout = layout(&self.slots, Some(x), self.win.0 as f32, &self.m);
      let _ = self.props.InsertScalar(&HSTRING::from("cx"), x);
      self.set_influence(true);
      self.last_apply = Instant::now();
    } else {
      self.apply(Some(x), true);
    }
    self.update_visibility();
  }

  pub(super) fn on_leave(&mut self) {
    self.tracking = false;
    if self.dragging.is_some() {
      return;
    }
    self.cursor = None;
    self.hover = None;
    kill(self.hwnd, TIMER_TIP);
    kill(self.hwnd, TIMER_PREVIEW);
    if let Some(t) = &self.tooltip {
      t.hide();
    }
    if self.preview.as_ref().is_some_and(|p| p.is_visible()) {
      timer(self.hwnd, TIMER_PCLOSE, 200);
    }
    if self.exprs_ok {
      self.set_influence(false);
      self.layout = layout(&self.slots, None, self.win.0 as f32, &self.m);
    } else {
      self.apply(None, true);
    }
    timer(self.hwnd, TIMER_REGION, 450);
    self.start_grace();
    self.update_visibility();
  }

  pub(super) fn on_down(&mut self, x: f32, y: f32) {
    self.start_open_at_press = launch::is_start_open();
    if let Some((slot, entry)) = self.entry_at(x) {
      let pinned_item = match entry {
        Entry::App(i) if self.items.get(i).is_some_and(|it| it.pinned) => Some(i),
        _ => None,
      };
      self.press = Some(Press {
        slot,
        x,
        y,
        pinned_item,
      });
      unsafe {
        SetCapture(self.hwnd);
      }
    }
  }

  pub(super) fn drag_update(&mut self, x: f32, y: f32) {
    let Some(p) = &self.press else { return };
    if self.dragging.is_none() {
      let far = (x - p.x).abs().max((y - p.y).abs()) > 6.0 * self.dpi;
      if !far || p.pinned_item.is_none() {
        return;
      }
      self.dragging = Some(p.slot);
      self.hide_popups();
      self.cursor = None;
      self.apply(None, false);
    }
    if let Some(v) = self.dragging.and_then(|s| self.views.get(s)) {
      let _ = v.drag_to(x - self.m.icon / 2.0, self.icon_y - 14.0 * self.dpi);
    }
    self.shift_neighbors(x);
  }

  pub(super) fn shift_neighbors(&mut self, x: f32) {
    let Some(dragged) = self.dragging else { return };
    let rest = layout(&self.slots, None, self.win.0 as f32, &self.m);
    let pinned_slots: Vec<usize> = self
      .entries
      .iter()
      .enumerate()
      .filter(|(_, e)| matches!(e, Entry::App(i) if self.items.get(*i).is_some_and(|it| it.pinned)))
      .map(|(s, _)| s)
      .collect();
    let Some(from) = pinned_slots.iter().position(|s| *s == dragged) else {
      return;
    };
    let centers: Vec<f32> = pinned_slots
      .iter()
      .map(|s| rest.cells[*s].x + rest.cells[*s].w / 2.0)
      .collect();
    let mut order = pinned_slots.clone();
    reorder::move_to(&mut order, from, reorder::target_index(&centers, x));
    let mut display: Vec<usize> = (0..self.views.len()).collect();
    for (k, pos) in pinned_slots.iter().enumerate() {
      display[*pos] = order[k];
    }
    for (pos, view_idx) in display.iter().enumerate() {
      if *view_idx == dragged {
        continue;
      }
      if let (Some(v), Some(c)) = (self.views.get(*view_idx), rest.cells.get(pos)) {
        let _ = v.place(c.x, c.w, 1.0, self.icon_y, self.dot_y, true);
      }
    }
  }

  pub(super) fn on_up(&mut self, x: f32, y: f32) {
    unsafe {
      let _ = ReleaseCapture();
    }
    let press = self.press.take();
    match (self.dragging.take(), press) {
      (Some(_), Some(p)) => self.finish_drag(x, y, p),
      (None, Some(_)) => self.on_click(x),
      _ => {}
    }
  }

  pub(super) fn finish_drag(&mut self, x: f32, y: f32, p: Press) {
    let Some(from) = p.pinned_item else { return };
    let rest = layout(&self.slots, None, self.win.0 as f32, &self.m);
    let centers: Vec<f32> = self
      .entries
      .iter()
      .zip(rest.cells.iter())
      .filter(|(e, _)| matches!(e, Entry::App(i) if self.items.get(*i).is_some_and(|it| it.pinned)))
      .map(|(_, c)| c.x + c.w / 2.0)
      .collect();
    let dropped_above = y < self.card_y - self.m.icon * 0.9;
    let changed = if dropped_above {
      let key = self.items.get(from).map(|i| i.key.clone());
      let before = self.pins.len();
      if let Some(k) = key {
        self.pins.retain(|p| key_of(&p.path) != k);
      }
      self.pins.len() != before
    } else {
      let to = reorder::target_index(&centers, x);
      reorder::move_to(&mut self.pins, from, to)
    };
    if changed {
      if let Err(e) = pins::save(&self.pins_path, &self.pins) {
        log::line("pins", &format!("simpan gagal: {e}"));
      }
    }
    log::line(
      "dock",
      &format!("drag selesai dari={from} atas={dropped_above} berubah={changed}"),
    );
    self.signature.clear();
    self.refresh_windows();
    self.apply(self.cursor, false);
  }

  pub(super) fn on_click(&mut self, x: f32) {
    let Some((idx, entry)) = self.entry_at(x) else {
      return;
    };
    self.hide_popups();
    match entry {
      Entry::Start => launch::toggle_start(self.start_open_at_press),
      Entry::Launchpad => crate::launchpad::toggle(),
      Entry::App(i) => {
        let Some(it) = self.items.get(i).cloned() else {
          return;
        };
        if it.running() {
          let target = if it.hwnds.len() > 1 {
            next_in_ring(&it.hwnds, focus::foreground()).unwrap_or(it.hwnds[0])
          } else {
            it.hwnds[0]
          };
          let ok = self.activate_target(target);
          log::line("dock", &format!("aktifkan {} hwnd={target} ok={ok}", it.name));
        } else {
          launch::open(&it.path);
          if let Some(v) = self.views.get(idx) {
            let _ = v.bounce(self.icon_y);
          }
          log::line("dock", &format!("luncurkan {}", it.path));
        }
      }
      Entry::Sep => {}
      Entry::Trash => launch::open("shell:RecycleBinFolder"),
      Entry::Downloads => self.open_stack(),
    }
  }

  pub(super) fn activate_target(&self, target: isize) -> bool {
    if focus::is_minimized(target) {
      if let Some(r) = self.icon_rect(target) {
        if crate::fx::restore(target, r, self.hwnd.0 as isize) {
          return true;
        }
      }
    }
    focus::activate(target, self.hwnd.0 as isize)
  }

  pub(super) fn icon_rect(&self, hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    let item = self.items.iter().position(|it| it.hwnds.contains(&hwnd))?;
    let j = self
      .entries
      .iter()
      .position(|e| matches!(e, Entry::App(i) if *i == item))?;
    let rest = layout(&self.slots, None, self.win.0 as f32, &self.m);
    let c = rest.cells.get(j)?;
    Some((
      self.win_pos.0 + c.x as i32,
      self.win_pos.1 + self.icon_y as i32,
      c.w as i32,
      self.m.icon as i32,
    ))
  }

  pub(super) fn on_middle_click(&mut self, x: f32) {
    if let Some((_, Entry::App(i))) = self.entry_at(x) {
      if let Some(it) = self.items.get(i) {
        launch::open(&it.path);
      }
    }
  }

  pub(super) fn on_drop(&mut self, hdrop: windows::Win32::UI::Shell::HDROP) {
    use windows::Win32::UI::Shell::{DragFinish, DragQueryFileW};
    unsafe {
      let n = DragQueryFileW(hdrop, u32::MAX, None);
      for i in 0..n {
        let len = DragQueryFileW(hdrop, i, None) as usize;
        let mut buf = vec![0u16; len + 1];
        DragQueryFileW(hdrop, i, Some(&mut buf));
        self.add_app(&String::from_utf16_lossy(&buf[..len]));
      }
      DragFinish(hdrop);
    }
  }
}
