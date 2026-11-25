use super::*;

impl Mgr {
  pub(super) fn tick(&mut self) {
    self.ticks = self.ticks.wrapping_add(1);
    if self.ticks % 2 == 0 {
      if let (Some(prev), Some(now)) = (self.cpu_prev, sys::cpu_times()) {
        self.cpu = widgets::cpu_percent(prev, now);
        self.cpu_prev = Some(now);
      }
      self.ram = sys::memory_load().unwrap_or(self.ram);
    }
    for i in 0..self.list.len() {
      let kind = self.list[i].kind;
      if self.sig_for(kind) != self.list[i].sig && self.drag.as_ref().map(|d| d.idx) != Some(i) {
        self.paint(i);
      }
      self.turn_hands(i, true);
    }
  }

  pub(super) fn set_hover(&mut self, i: usize, on: bool) {
    if i >= self.list.len() || self.list[i].hover == on {
      return;
    }
    self.list[i].hover = on;
    let c = self.center(i);
    let _ = motion::hover_pop(&self.comp, self.list[i].popup.root(), c, if on { 1.03 } else { 1.0 });
  }

  pub(super) fn move_to(&self, i: usize, pos: (i32, i32), topmost: bool) {
    let m = (MARGIN * self.dpi) as i32;
    let w = &self.list[i];
    unsafe {
      let after = if topmost { HWND_TOPMOST } else { HWND_BOTTOM };
      let _ = SetWindowPos(
        w.popup.hwnd,
        Some(after),
        pos.0 - m,
        pos.1 - m,
        0,
        0,
        SWP_NOSIZE | SWP_NOACTIVATE,
      );
    }
  }

  pub(super) fn lift(&mut self, i: usize) {
    FLOAT.store(true, Ordering::Relaxed);
    let c = self.center(i);
    let _ = motion::hover_pop(&self.comp, self.list[i].popup.root(), c, 1.06);
    let pos = self.list[i].pos;
    self.move_to(i, pos, true);
  }

  pub(super) fn drag_to(&mut self, i: usize, pos: (i32, i32)) {
    if i >= self.list.len() {
      return;
    }
    self.list[i].pos = pos;
    self.move_to(i, pos, true);
    let size = (self.list[i].size.0 as i32, self.list[i].size.1 as i32);
    let spot = self.slot_for(Some(i), size, pos);
    self.show_slot((spot.0, spot.1, size.0, size.1));
    unsafe {
      let _ = SetWindowPos(
        self.list[i].popup.hwnd,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
    }
  }

  pub(super) fn show_slot(&mut self, r: (i32, i32, i32, i32)) {
    if self.slot_at == Some(r) {
      return;
    }
    let d = self.dpi;
    let m = (MARGIN * d) as i32;
    if self.slot.is_none() {
      let base = GlassParams::dark((10.0, 10.0), 0.0);
      self.slot = Popup::new(
        &self.comp,
        w!("NexoraWidgetSlot"),
        Some(slot_proc),
        WS_EX_TRANSPARENT,
        base,
      )
      .ok();
    }
    let resized = self.slot_at.map(|s| (s.2, s.3)) != Some((r.2, r.3));
    let comp = self.comp.clone();
    let Some(slot) = self.slot.as_mut() else {
      return;
    };
    let _ = slot.set_rect(r.0 - m, r.1 - m, r.2 + 2 * m, r.3 + 2 * m);
    let _ = slot.panel.set_rect(0.0, 0.0, 0.0, 0.0);
    let _ = slot.content.SetOffset(Vector3 {
      X: m as f32,
      Y: m as f32,
      Z: 0.0,
    });
    let content = slot.content.clone();
    let root = slot.root().clone();
    let hwnd = slot.hwnd;
    if resized {
      if let Ok(k) = content.Children() {
        let _ = k.RemoveAll();
      }
      let (w, h) = (r.2 as f32, r.3 as f32);
      let _ = self.rr(
        &content,
        (0.0, 0.0, w, h),
        RADIUS * d,
        Color {
          A: 70,
          R: 255,
          G: 255,
          B: 255,
        },
      );
      let t = 2.0 * d;
      let _ = self.rr(
        &content,
        (t, t, w - 2.0 * t, h - 2.0 * t),
        RADIUS * d - t,
        Color {
          A: 40,
          R: 120,
          G: 120,
          B: 128,
        },
      );
    }
    let center = Vector3 {
      X: m as f32 + r.2 as f32 / 2.0,
      Y: m as f32 + r.3 as f32 / 2.0,
      Z: 0.0,
    };
    if self.slot_at.is_none() {
      let _ = motion::pop_in(&comp, &root, center, 0.85, 1.03, 0, 260);
    } else {
      let _ = root.SetCenterPoint(center);
      let _ = root.SetScale(Vector3 {
        X: 0.94,
        Y: 0.94,
        Z: 1.0,
      });
      let _ = motion::hover_pop(&comp, &root, center, 1.0);
    }
    unsafe {
      let _ = SetWindowPos(
        hwnd,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      );
    }
    self.slot_at = Some(r);
  }

  pub(super) fn hide_slot(&mut self) {
    self.slot_at = None;
    if let Some(s) = &self.slot {
      s.hide();
    }
  }

  pub(super) fn drop_at(&mut self, i: usize) {
    let w = &self.list[i];
    let size = (w.size.0 as i32, w.size.1 as i32);
    let from = w.pos;
    let spot = self.slot_for(Some(i), size, from);
    self.hide_slot();
    self.list[i].pos = spot;
    FLOAT.store(false, Ordering::Relaxed);
    unsafe {
      let _ = SetWindowPos(
        self.list[i].popup.hwnd,
        Some(HWND_NOTOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
    }
    let c = self.center(i);
    let root = self.list[i].popup.root().clone();
    let _ = motion::hover_pop(&self.comp, &root, c, 1.0);
    self.finish_snap();
    self.snap = Some((self.list[i].popup.hwnd, from, spot, std::time::Instant::now()));
    unsafe {
      SetTimer(Some(self.ctl), T_SNAP, 15, None);
    }
    self.save();
  }

  pub(super) fn step_snap(&mut self) {
    let Some((h, from, to, t0)) = self.snap else {
      unsafe {
        let _ = KillTimer(Some(self.ctl), T_SNAP);
      }
      return;
    };
    let t = (t0.elapsed().as_secs_f32() * 1000.0 / SNAP_MS).min(1.0);
    let (c1, c3) = (1.1f32, 2.1f32);
    let e = 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2);
    let m = (MARGIN * self.dpi) as i32;
    let x = from.0 as f32 + (to.0 - from.0) as f32 * e;
    let y = from.1 as f32 + (to.1 - from.1) as f32 * e;
    unsafe {
      let _ = SetWindowPos(
        h,
        None,
        x.round() as i32 - m,
        y.round() as i32 - m,
        0,
        0,
        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
      );
    }
    if t >= 1.0 {
      self.finish_snap();
    }
  }

  pub(super) fn finish_snap(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_SNAP);
    }
    if let Some((h, _, to, _)) = self.snap.take() {
      let m = (MARGIN * self.dpi) as i32;
      unsafe {
        let _ = SetWindowPos(
          h,
          Some(HWND_BOTTOM),
          to.0 - m,
          to.1 - m,
          0,
          0,
          SWP_NOSIZE | SWP_NOACTIVATE,
        );
      }
    }
  }
}
