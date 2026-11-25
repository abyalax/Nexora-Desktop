use super::*;

impl Win {
  pub(super) fn render(&mut self) {
    let content = self.popup.content.clone();
    if let Ok(k) = content.Children() {
      let _ = k.RemoveAll();
    }
    self.regions.clear();
    let d = self.dpi;
    let look = prefs::look(Role::Themed);
    let c = &content;

    for (i, rgb) in [[255u8, 95, 87], [254, 188, 46], [40, 200, 64]].iter().enumerate() {
      let (cx, cy) = ((26.0 + i as f32 * 20.0) * d, 27.0 * d);
      self.rect(
        c,
        cx - 6.0 * d,
        cy - 6.0 * d,
        12.0 * d,
        12.0 * d,
        6.0 * d,
        col(*rgb, 255),
      );
    }
    self.region_fixed(18.0 * d, 17.0 * d, 20.0 * d, 20.0 * d, Action::Close);

    let (fx, fy, fw, fh) = (16.0 * d, 48.0 * d, (SIDEBAR_W - 32.0) * d, 30.0 * d);
    self.rect(
      c,
      fx,
      fy,
      fw,
      fh,
      9.0 * d,
      col(look.fg, if self.search_focus { 36 } else { 22 }),
    );
    self.glyph_icon(c, "\u{E721}", fx + 15.0 * d, fy + fh / 2.0, 11.0, look.fg);
    let (shown, op) = if self.search.is_empty() && !self.search_focus {
      (prefs::tr("Cari").to_string(), 0.45)
    } else {
      (
        format!("{}{}", self.search, if self.search_focus { "|" } else { "" }),
        0.95,
      )
    };
    self.text(
      c,
      &shown,
      fx + 29.0 * d,
      fy + (fh - 17.0 * d) / 2.0,
      13.0,
      400,
      look.fg,
      op,
      0,
    );
    self.region_fixed(fx, fy, fw, fh, Action::Search);

    let (px0, py0, av) = (20.0 * d, 90.0 * d, 38.0 * d);
    self.rect(c, px0, py0, av, av, av / 2.0, crate::prefs::accent_c(255));
    let user = user_name();
    let initial: String = user.chars().take(1).collect::<String>().to_uppercase();
    self.text(
      c,
      &initial,
      px0 + av / 2.0,
      py0 + (av - 20.0 * d) / 2.0,
      15.0,
      600,
      [255, 255, 255],
      1.0,
      1,
    );
    self.text(
      c,
      &user,
      px0 + av + 10.0 * d,
      py0 + 2.0 * d,
      13.5,
      600,
      look.fg,
      0.95,
      0,
    );
    self.text(
      c,
      prefs::tr("Akun Windows"),
      px0 + av + 10.0 * d,
      py0 + 20.0 * d,
      11.0,
      400,
      look.fg,
      0.6,
      0,
    );

    let pages: [(&'static str, &'static str, [u8; 3], u8); 6] = [
      (
        Page::General.name(),
        Page::General.icon(),
        [142, 142, 147],
        Page::General.index(),
      ),
      (Page::Dock.name(), Page::Dock.icon(), [10, 132, 255], Page::Dock.index()),
      (
        Page::Island.name(),
        Page::Island.icon(),
        [58, 58, 60],
        Page::Island.index(),
      ),
      (
        Page::Stage.name(),
        Page::Stage.icon(),
        [48, 209, 88],
        Page::Stage.index(),
      ),
      (
        Page::Appearance.name(),
        Page::Appearance.icon(),
        [88, 86, 214],
        Page::Appearance.index(),
      ),
      (
        Page::Advanced.name(),
        Page::Advanced.icon(),
        [175, 82, 222],
        Page::Advanced.index(),
      ),
    ];
    let mut sy = 140.0 * d;
    self.side.clear();
    let (view_top, view_bot) = (134.0 * d, (H - 96.0) * d);
    let side_body = match (self.comp.CreateContainerVisual(), self.comp.CreateContainerVisual()) {
      (Ok(v), Ok(b)) => {
        let _ = v.SetOffset(Vector3 {
          X: 0.0,
          Y: view_top,
          Z: 0.0,
        });
        let _ = v.SetSize(Vector2 {
          X: SIDEBAR_W * d,
          Y: view_bot - view_top,
        });
        if let Ok(clip) = self.comp.CreateInsetClip() {
          let _ = v.SetClip(&clip);
        }
        let _ = b.SetOffset(Vector3 {
          X: 0.0,
          Y: -view_top - self.side_scroll,
          Z: 0.0,
        });
        let _ = c.Children().and_then(|k| k.InsertAtTop(&v));
        let _ = v.Children().and_then(|k| k.InsertAtTop(&b));
        b
      }
      _ => c.clone(),
    };
    let parent = &side_body;
    for (name, glyph, color, idx) in pages {
      if !self.page_matches(idx, name) {
        continue;
      }
      let sel = idx == self.tab;
      let (rx, rw, rh) = (16.0 * d, (SIDEBAR_W - 32.0) * d, 36.0 * d);
      let Ok(ic) = self.comp.CreateContainerVisual() else {
        continue;
      };
      let _ = parent.Children().and_then(|k| k.InsertAtTop(&ic));
      if self.side_hover == Some(idx) {
        let _ = ic.SetCenterPoint(Vector3 {
          X: rx + rw / 2.0,
          Y: sy + rh / 2.0,
          Z: 0.0,
        });
        let _ = ic.SetScale(Vector3 {
          X: crate::motion::HOVER_ROW,
          Y: crate::motion::HOVER_ROW,
          Z: 1.0,
        });
      }
      self.side.push((idx, (rx, sy, rw, rh), ic.clone()));
      let c = &ic;
      if sel {
        self.rect(c, rx, sy, rw, rh, 10.0 * d, crate::prefs::accent_c(235));
      }
      self.rect(
        c,
        rx + 8.0 * d,
        sy + 6.0 * d,
        24.0 * d,
        24.0 * d,
        7.0 * d,
        col(color, 255),
      );
      self.glyph_icon(c, glyph, rx + 20.0 * d, sy + 18.0 * d, 13.0, [255, 255, 255]);
      self.text(
        c,
        name,
        rx + 42.0 * d,
        sy + (rh - 17.0 * d) / 2.0,
        13.5,
        if sel { 600 } else { 400 },
        if sel { [255, 255, 255] } else { look.fg },
        1.0,
        0,
      );
      let vis_top = (sy - self.side_scroll).max(view_top);
      let vis_bot = (sy - self.side_scroll + rh).min(view_bot);
      if vis_bot > vis_top {
        self.region_fixed(rx, vis_top, rw, vis_bot - vis_top, Action::Tab(idx));
      }
      sy += 40.0 * d;
    }
    let side_max = (sy - 4.0 * d - view_bot).max(0.0);
    self.side_view = (view_top, side_max);
    if self.side_scroll > side_max {
      self.side_scroll = side_max;
    }
    self.side_bar = None;
    if side_max > 0.5 {
      let (track_top, track_h) = (view_top + 4.0 * d, view_bot - view_top - 8.0 * d);
      let content = view_bot - view_top + side_max;
      let th = (track_h * (view_bot - view_top) / content).max(28.0 * d);
      let ty = track_top + (track_h - th) * (self.side_scroll / side_max);
      let bw = if self.side_drag.is_some() { 7.0 * d } else { 5.0 * d };
      let bx = (SIDEBAR_W - SIDE_INSET - 5.0) * d - bw;
      self.rect(
        c,
        bx,
        ty,
        bw,
        th,
        bw / 2.0,
        col(look.fg, if self.side_drag.is_some() { 110 } else { 70 }),
      );
      self.side_bar = Some(((bx - 6.0 * d, track_top, bw + 12.0 * d, track_h), th));
    }
    let (kx, kw) = (20.0 * d, (SIDEBAR_W - 40.0) * d);
    let ky = (H - 84.0) * d;
    self.text(
      c,
      prefs::tr("Dibuat oleh kreativa.dev"),
      kx + kw / 2.0,
      ky,
      11.0,
      500,
      look.fg,
      0.55,
      1,
    );
    let (kby, kbh) = (ky + 22.0 * d, 32.0 * d);
    self.rect(c, kx, kby, kw, kbh, kbh / 2.0, col([255, 159, 10], 255));
    self.text(
      c,
      prefs::tr("Belikan aku kopi"),
      kx + kw / 2.0,
      kby + (kbh - 17.0 * d) / 2.0,
      13.0,
      600,
      [255, 255, 255],
      1.0,
      1,
    );
    self.region_fixed(kx, kby, kw, kbh, Action::Donate);

    let (bx, by, bw, bh) = (CONTENT_X * d, 11.0 * d, 64.0 * d, 28.0 * d);
    self.rect(c, bx, by, bw, bh, 14.0 * d, col(look.fg, 24));
    let (can_back, can_fwd) = (self.hpos > 0, self.hpos + 1 < self.history.len());
    let dim = |on: bool| {
      if on {
        look.fg
      } else if look.dark {
        [110, 110, 116]
      } else {
        [170, 170, 176]
      }
    };
    self.glyph_icon(c, "\u{E76B}", bx + bw * 0.27, by + bh / 2.0, 11.0, dim(can_back));
    self.glyph_icon(c, "\u{E76C}", bx + bw * 0.73, by + bh / 2.0, 11.0, dim(can_fwd));
    self.rect(
      c,
      bx + bw / 2.0,
      by + 7.0 * d,
      1.0,
      bh - 14.0 * d,
      0.0,
      col(look.fg, 34),
    );
    self.region_fixed(bx, by, bw / 2.0, bh, Action::Back);
    self.region_fixed(bx + bw / 2.0, by, bw / 2.0, bh, Action::Forward);
    let title = pages.iter().find(|p| p.3 == self.tab).map(|p| p.0).unwrap_or("");
    self.text(
      c,
      title,
      bx + bw + 12.0 * d,
      by + (bh - 19.0 * d) / 2.0,
      15.0,
      700,
      look.fg,
      1.0,
      0,
    );

    let view = match (self.comp.CreateContainerVisual(), self.comp.CreateContainerVisual()) {
      (Ok(v), Ok(b)) => Some((v, b)),
      _ => None,
    };
    let Some((viewport, body)) = view else { return };
    let (vt, vh) = (VIEW_TOP * d, (H - VIEW_TOP - 10.0) * d);
    let _ = viewport.SetOffset(Vector3 { X: 0.0, Y: vt, Z: 0.0 });
    let _ = viewport.SetSize(Vector2 { X: W * d, Y: vh });
    if let Ok(clip) = self.comp.CreateInsetClip() {
      let _ = clip.SetLeftInset(0.0);
      let _ = clip.SetRightInset(0.0);
      let _ = clip.SetTopInset(0.0);
      let _ = clip.SetBottomInset(0.0);
      let _ = viewport.SetClip(&clip);
    }
    let _ = body.SetOffset(Vector3 {
      X: 0.0,
      Y: -self.scroll,
      Z: 0.0,
    });
    let _ = c.Children().and_then(|k| k.InsertAtTop(&viewport));
    let _ = viewport.Children().and_then(|k| k.InsertAtTop(&body));

    let mut y = VIEW_TOP * d - vt;
    y += 4.0 * d;
    let groups = self.page(self.tab);
    for (header, rows) in groups {
      if let Some(h) = header {
        self.text_abs(&body, h, CONTENT_X * d + 8.0 * d, y + 2.0 * d, 12.0, 600, look.fg, 0.55);
        y += 24.0 * d;
      }
      let used = self.card_in(&body, y, &rows, &look, vt);
      y += used + 20.0 * d;
    }
    self.content_h = y + 6.0 * d;
    let max_scroll = (self.content_h - vh).max(0.0);
    if self.scroll > max_scroll {
      self.scroll = max_scroll;
      let _ = body.SetOffset(Vector3 {
        X: 0.0,
        Y: -self.scroll,
        Z: 0.0,
      });
    }
    self.size = (W * d, H * d);
    let (x, yy, ww, hh) = self.popup.rect;
    let (nw, nh) = ((W * d) as i32, (H * d) as i32);
    if (nw, nh) != (ww, hh) {
      let _ = self.popup.set_rect(x, yy, nw, nh);
    }
    self.frame();
  }
  pub(super) fn frame(&mut self) {
    let d = self.dpi;
    let (w, h) = ((W * d) as i32, (H * d) as i32);
    let (hx, hy, hw, hh) = (
      SIDE_INSET * d,
      SIDE_INSET * d,
      (SIDEBAR_W - 2.0 * SIDE_INSET) * d,
      (H - 2.0 * SIDE_INSET) * d,
    );
    let _ = self.popup.panel.set_rect(hx, hy, hw, hh);
    let _ = self.popup.panel.set_radius(SIDE_RADIUS * d);
    let dark = prefs::look(Role::Themed).dark;
    if self.frame.is_some() && self.frame_key == (w, h, dark) {
      return;
    }
    let data = frame_pixels(w, h, 24.0 * d, (hx, hy, hw, hh), SIDE_RADIUS * d, dark);
    let go = || -> windows_core::Result<windows::UI::Composition::SpriteVisual> {
      let surf = self.gfx.surface_from_bgra(w as u32, h as u32, &data)?;
      let br = self.comp.CreateSurfaceBrushWithSurface(&surf)?;
      br.SetStretch(CompositionStretch::Fill)?;
      let v = match &self.frame {
        Some(v) => v.clone(),
        None => {
          let v = self.comp.CreateSpriteVisual()?;
          self.popup.root().Children()?.InsertAtBottom(&v)?;
          v
        }
      };
      v.SetBrush(&br)?;
      v.SetSize(Vector2 {
        X: w as f32,
        Y: h as f32,
      })?;
      Ok(v)
    };
    match go() {
      Ok(v) => {
        self.frame = Some(v);
        self.frame_key = (w, h, dark);
      }
      Err(e) => log::line("settings", &format!("bingkai gagal: {e}")),
    }
  }
}
