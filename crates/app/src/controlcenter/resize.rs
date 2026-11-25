use super::*;

impl Cc {
  pub(super) fn start_resize(&mut self, i: usize) {
    let sizes = self.sizes.clone();
    let layout = self.layout.clone();
    let Some(p) = self.panel.as_mut() else { return };
    let Some(m) = p.mods.get(i) else { return };
    let k = m.placed.kind;
    let cur = k.clamp(m.placed.w, m.placed.h);
    let _ = sizes;
    p.resize = Some((i, cur));
    p.preview = layout;
    p.shift.clear();
    self.show_ghost(i, cur);
  }
  pub(super) fn show_ghost(&mut self, i: usize, size: Size) {
    let comp = self.comp.clone();
    let Some(p) = self.panel.as_ref() else { return };
    let (cell, gap, dpi) = (p.cell, p.gap, p.dpi);
    let Some(m) = p.mods.get(i) else { return };
    let origin = (
      m.rect.0 + p.shift.get(&m.placed.kind).map(|s| s.0).unwrap_or(0.0),
      m.rect.1 + p.shift.get(&m.placed.kind).map(|s| s.1).unwrap_or(0.0),
    );
    let (w, h) = (
      size.w as f32 * cell + (size.w - 1) as f32 * gap,
      size.h as f32 * cell + (size.h - 1) as f32 * gap,
    );
    let radius = if size.w == 1 && size.h == 1 {
      w / 2.0
    } else if size.h >= 2 || size.w >= 3 {
      CARD_R * cell
    } else {
      h / 2.0
    };
    let content = p.popup.content.clone();
    let old = p.ghost.clone();
    if let Some(g) = old {
      if let Ok(k) = content.Children() {
        let _ = k.Remove(&g);
      }
    }
    let Ok(g) = self.comp.CreateContainerVisual() else {
      return;
    };
    let _ = self.rrect(
      &g,
      (origin.0, origin.1, w, h),
      radius,
      Color {
        A: 60,
        R: 255,
        G: 255,
        B: 255,
      },
    );
    let t = 2.0 * dpi;
    let _ = self.rrect(
      &g,
      (origin.0 + t, origin.1 + t, w - 2.0 * t, h - 2.0 * t),
      (radius - t).max(0.0),
      Color {
        A: 30,
        R: 0,
        G: 0,
        B: 0,
      },
    );
    let _ = content.Children().and_then(|k| k.InsertAtTop(&g));
    let center = Vector3 {
      X: origin.0 + w / 2.0,
      Y: origin.1 + h / 2.0,
      Z: 0.0,
    };
    let _ = g.SetCenterPoint(center);
    let _ = g.SetScale(Vector3 {
      X: 0.93,
      Y: 0.93,
      Z: 1.0,
    });
    let _ = motion::hover_pop(&comp, &g, center, 1.0);
    if let Some(p) = self.panel.as_mut() {
      p.ghost = Some(g);
    }
  }
  pub(super) fn resize_move(&mut self, x: f32, y: f32) {
    let comp = self.comp.clone();
    let Some(p) = self.panel.as_ref() else { return };
    let Some((i, cur)) = p.resize else { return };
    let Some(m) = p.mods.get(i) else { return };
    let k = m.placed.kind;
    let (cell, gap) = (p.cell, p.gap);
    let origin = (
      m.rect.0 + p.shift.get(&k).map(|s| s.0).unwrap_or(0.0),
      m.rect.1 + p.shift.get(&k).map(|s| s.1).unwrap_or(0.0),
    );
    let want = k.clamp(
      controls::cells_for(x - origin.0, cell, gap),
      controls::cells_for(y - origin.1, cell, gap),
    );
    if want == cur {
      return;
    }
    let mut sizes = self.sizes.clone();
    sizes.insert(k, want);
    let preview = p.preview.clone();
    let (placed, _) = controls::layout_sized(&preview, &|kk| *sizes.get(&kk).unwrap_or(&kk.default_size()));
    let mut moves = Vec::new();
    for mm in &p.mods {
      let kk = mm.placed.kind;
      let Some(q) = placed.iter().find(|q| q.kind == kk) else {
        continue;
      };
      let r = grid_rect(q, cell, gap, 0.0);
      let to = (r.0 - mm.rect.0, r.1 - mm.rect.1);
      let from = p.shift.get(&kk).copied().unwrap_or((0.0, 0.0));
      if (to.0 - from.0).abs() > 0.5 || (to.1 - from.1).abs() > 0.5 {
        moves.push((kk, mm.cont.clone(), from, to));
      }
    }
    if let Some(p) = self.panel.as_mut() {
      p.resize = Some((i, want));
      for (kk, cont, from, to) in moves {
        if let Some(c) = cont {
          let _ = motion::slide_pop(
            &comp,
            &c,
            Vector3 {
              X: from.0,
              Y: from.1,
              Z: 0.0,
            },
            Vector3 {
              X: to.0,
              Y: to.1,
              Z: 0.0,
            },
          );
        }
        p.shift.insert(kk, to);
      }
    }
    self.show_ghost(i, want);
  }
  pub(super) fn end_resize(&mut self) {
    let from = self.origins();
    let Some(p) = self.panel.as_mut() else { return };
    let Some((i, want)) = p.resize.take() else {
      return;
    };
    let k = p.mods[i].placed.kind;
    p.preview.clear();
    p.shift.clear();
    p.ghost = None;
    if want != k.clamp(p.mods[i].placed.w, p.mods[i].placed.h) {
      self.sizes.insert(k, want);
      self.save_layout();
    }
    self.build_flip(from, Some(k), None);
  }
  pub(super) fn set_hover(&mut self, hit: Option<usize>) {
    let comp = self.comp.clone();
    let Some(p) = self.panel.as_mut() else { return };
    if p.hover == hit {
      return;
    }
    for (i, on) in [(p.hover, false), (hit, true)] {
      let Some(m) = i.and_then(|i| p.mods.get(i)) else {
        continue;
      };
      let Some(c) = &m.cont else { continue };
      let r = m.rect;
      let to = if !on {
        1.0
      } else if m.placed.w == 1 && m.placed.h == 1 {
        motion::HOVER_BUTTON
      } else {
        motion::HOVER_ROW
      };
      let _ = motion::hover_pop(
        &comp,
        c,
        Vector3 {
          X: r.0 + r.2 / 2.0,
          Y: r.1 + r.3 / 2.0,
          Z: 0.0,
        },
        to,
      );
    }
    p.hover = hit;
  }
  pub(super) fn set_slider(&mut self, i: usize, x: f32, y: f32, commit: bool) {
    let Some(p) = self.panel.as_ref() else { return };
    let Some(m) = p.mods.get(i) else { return };
    let Some(t) = m.track else { return };
    let frac = if m.vertical {
      (1.0 - (y - t.1) / t.3).clamp(0.0, 1.0)
    } else {
      ((x - t.0) / t.2).clamp(0.0, 1.0)
    };
    if let Some(f) = &m.fill {
      if m.vertical {
        place_slider_v(f, &m.thumb, t, frac, p.dpi);
      } else {
        place_slider(f, &m.thumb, t, frac, p.dpi);
      }
    }
    let kind = m.placed.kind;
    if commit {
      match kind {
        Kind::Display => services::set_brightness(frac),
        _ => services::set_volume(frac),
      }
    }
  }
  pub(super) fn open_size_menu(&mut self, i: usize, x: f32, y: f32) {
    let Some(p) = self.panel.as_ref() else { return };
    let kind = p.mods[i].placed.kind;
    if !kind.resizable() {
      return;
    }
    let cur = *self.sizes.get(&kind).unwrap_or(&kind.default_size());
    let (rect, dpi, hwnd) = (p.popup.rect, p.dpi, p.popup.hwnd);
    let Some(gfx) = self.gfx.as_ref() else { return };
    if self.menu.is_none() {
      match Menu::new(&self.comp, hwnd, dpi, nexora_platform::sys::screen_size(), Tier::Full) {
        Ok(m) => self.menu = Some(m),
        Err(_) => return,
      }
    }
    let mk = |id: usize, label: &'static str, s: Size| {
      MItem::action(
        id,
        &format!("{}{}", if cur == s { "\u{2713} " } else { "" }, prefs::tr(label)),
        true,
      )
    };
    let items = vec![
      mk(1, "Kecil", Size::S),
      mk(2, "Sedang", Size::M),
      mk(3, "Besar", Size::L),
    ];
    self.menu_kind = Some(kind);
    if let Some(m) = self.menu.as_mut() {
      let _ = m.show(
        &self.comp,
        gfx,
        items,
        rect.0 + (x + pad()) as i32,
        rect.1 + (y + pad()) as i32,
      );
    }
  }
  pub(super) fn pick_size(&mut self, id: usize) {
    if let Some(m) = &self.menu {
      m.hide();
    }
    let Some(kind) = self.menu_kind.take() else {
      return;
    };
    let s = match id {
      1 => Size::S,
      2 => Size::M,
      _ => Size::L,
    };
    let from = self.origins();
    self.sizes.insert(kind, s);
    self.build_flip(from, Some(kind), None);
    self.save_layout();
  }
}

pub(super) fn place_slider(fill: &SpriteVisual, thumb: &[SpriteVisual], t: Rect, frac: f32, dpi: f32) {
  let th = t.3 - 20.0 * dpi;
  let (kw, kh) = (KNOB.0 * dpi, KNOB.1 * dpi);
  let kx = (t.0 + t.2 * frac).clamp(t.0 + kw / 2.0, t.0 + t.2 - kw / 2.0);
  let cy = t.1 + t.3 / 2.0;
  let _ = fill.SetSize(Vector2 {
    X: (kx - t.0).max(th),
    Y: th,
  });
  for (j, s) in thumb.iter().enumerate() {
    let inset = [-1.0, 0.0, 1.5].get(j).copied().unwrap_or(0.0) * dpi;
    let _ = s.SetOffset(Vector3 {
      X: kx - kw / 2.0 + inset,
      Y: cy - kh / 2.0 + inset,
      Z: 0.0,
    });
  }
}

pub(super) fn place_slider_v(fill: &SpriteVisual, thumb: &[SpriteVisual], t: Rect, frac: f32, dpi: f32) {
  let tw = t.2 - 28.0 * dpi;
  let (kw, kh) = (KNOB.1 * dpi, KNOB.0 * dpi);
  let ky = (t.1 + t.3 * (1.0 - frac)).clamp(t.1 + kh / 2.0, t.1 + t.3 - kh / 2.0);
  let cx = t.0 + t.2 / 2.0;
  let _ = fill.SetOffset(Vector3 {
    X: cx - tw / 2.0,
    Y: ky,
    Z: 0.0,
  });
  let _ = fill.SetSize(Vector2 {
    X: tw,
    Y: (t.1 + t.3 - ky).max(tw),
  });
  for (j, s) in thumb.iter().enumerate() {
    let inset = [-1.0, 0.0, 1.5].get(j).copied().unwrap_or(0.0) * dpi;
    let _ = s.SetOffset(Vector3 {
      X: cx - kw / 2.0 + inset,
      Y: ky - kh / 2.0 + inset,
      Z: 0.0,
    });
  }
}
