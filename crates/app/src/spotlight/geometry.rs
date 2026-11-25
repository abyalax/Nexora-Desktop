use super::*;

impl Spot {
  pub(super) fn look(&self) -> nexora_core::settings::Look {
    prefs::look(prefs::Role::Dock)
  }

  pub(super) fn idle_rect(&self) -> Rect {
    let g = self.geo;
    ((g.gw - g.ws) / 2.0, 0.0, g.ws, g.h)
  }

  pub(super) fn hover_rect(&self) -> Rect {
    let g = self.geo;
    (0.0, 0.0, g.cap, g.h)
  }

  pub(super) fn panel_h(&self) -> f32 {
    let (g, d) = (self.geo, self.dpi);
    if self.mode != Mode::Search {
      return g.max_h;
    }
    let n = self.rows.len().clamp(1, 8) as f32;
    (g.h + 42.0 * d + n * 54.0 * d + 12.0 * d).min(g.max_h)
  }

  pub(super) fn panel_rect(&self) -> Rect {
    let h = self.panel_h();
    if self.mode == Mode::Search {
      let i = self.idle_rect();
      (i.0, 0.0, i.2, h)
    } else {
      (0.0, 0.0, self.geo.gw, h)
    }
  }

  pub(super) fn set_region(&self, r: Rect) {
    let g = self.geo;
    let r = if r.3 <= g.h + 0.5 { (0.0, 0.0, g.gw, g.h) } else { r };
    unsafe {
      let rgn = CreateRectRgn(
        r.0.floor() as i32,
        r.1.floor() as i32,
        (r.0 + r.2).ceil() as i32,
        (r.1 + r.3).ceil() as i32,
      );
      SetWindowRgn(self.popup.hwnd, Some(rgn), true);
    }
  }

  pub(super) fn place(&mut self, t: Rect, anim: Option<(f32, u32)>) {
    let g = self.geo;
    let old = self.cur;
    let changed =
      (old.0 - t.0).abs() > 0.5 || (old.1 - t.1).abs() > 0.5 || (old.2 - t.2).abs() > 0.5 || (old.3 - t.3).abs() > 0.5;
    if !changed {
      return;
    }
    let radius = if t.3 <= g.h + 0.5 { t.3 / 2.0 } else { 26.0 * self.dpi };
    let (ww, wh) = (g.win.2 as f32, g.win.3 as f32);
    let ins = if t.3 <= g.h + 0.5 {
      (0.0, 0.0, 0.0, 0.0)
    } else {
      (t.0, t.1, (ww - t.0 - t.2).max(0.0), (wh - t.1 - t.3).max(0.0))
    };
    match anim {
      Some((ratio, period)) => {
        let _ = self.popup.panel.animate_rect(t.0, t.1, t.2, t.3, radius, ratio, period);
        if let Ok(o) = self.clip.cast::<CompositionObject>() {
          for (p, v) in [
            ("LeftInset", ins.0),
            ("TopInset", ins.1),
            ("RightInset", ins.2),
            ("BottomInset", ins.3),
          ] {
            spring(&self.comp, &o, p, v, ratio, period);
          }
        }
        self.set_region(union(old, t));
        unsafe {
          SetTimer(Some(self.popup.hwnd), T_REGION, 380, None);
        }
      }
      None => {
        let _ = self.popup.panel.set_rect(t.0, t.1, t.2, t.3);
        let _ = self.popup.panel.set_radius(radius);
        let _ = self.clip.SetLeftInset(ins.0);
        let _ = self.clip.SetTopInset(ins.1);
        let _ = self.clip.SetRightInset(ins.2);
        let _ = self.clip.SetBottomInset(ins.3);
        self.set_region(t);
      }
    }
    self.cur = t;
  }

  pub(super) fn reset_state(&mut self, mode: Mode) {
    self.mode = mode;
    self.query.clear();
    self.sel = 0;
    self.scroll = 0;
    self.cat = None;
    self.filter = None;
    self.editing = None;
    self.edit_buf.clear();
    self.files.clear();
    self.pscroll = 0.0;
    self.more_open = false;
    self.ftype = None;
    if mode == Mode::Files {
      self.request_recents();
    }
    self.rebuild_rows();
  }

  pub(super) fn set_mode(&mut self, mode: Mode) {
    if self.compact() && mode != Mode::Search {
      self.begin_morph(mode);
      return;
    }
    self.reset_state(mode);
    self.render();
  }

  pub(super) fn compact(&self) -> bool {
    self.mode == Mode::Search && self.query.trim().is_empty()
  }

  pub(super) fn begin_morph(&mut self, mode: Mode) {
    self.reset_state(mode);
    self.buttons = false;
    self.hover_mode = None;
    let t = self.panel_rect();
    self.place((t.0, 0.0, t.2, self.geo.h), Some((1.0, 55)));
    let content = self.popup.content.clone();
    let _ = crate::motion::scalar(&self.comp, &content, "Opacity", 1.0, 0.0, 90);
    let _ = content.SetOpacity(0.0);
    let _ = crate::motion::scalar(&self.comp, &self.side, "Opacity", 1.0, 0.0, 90);
    let _ = self.side.SetOpacity(0.0);
    self.morphing = true;
    unsafe {
      SetTimer(Some(self.popup.hwnd), T_MORPH, 100, None);
    }
  }

  pub(super) fn finish_morph(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.popup.hwnd), T_MORPH);
    }
    if !self.morphing {
      return;
    }
    self.morphing = false;
    if let Ok(k) = self.side.Children() {
      let _ = k.RemoveAll();
    }
    self.render_with(true);
  }
}
