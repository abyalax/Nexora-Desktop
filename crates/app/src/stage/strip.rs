use super::*;

impl St {
  pub(super) fn rebuild(&mut self) {
    let groups = self.model.strip.clone();
    if groups.is_empty() {
      self.destroy_strip();
      return;
    }
    if !self.ensure_strip() {
      return;
    }
    if self.gfx.is_none() {
      self.gfx = Gfx::shared(&self.comp).ok();
    }
    let dpi = self.strip.as_ref().map(|s| s.dpi).unwrap_or(1.0);
    let icon_px = (prefs::get().stage_size as f32 * 0.22 * dpi) as i32;
    let mut icon_bmps: HashMap<String, Bitmap> = HashMap::new();
    for g in &groups {
      for a in g.iter().take(2) {
        if let Some(b) = self.icon(a, icon_px) {
          icon_bmps.insert(a.clone(), b);
        }
      }
    }
    let comp = self.comp.clone();
    let (sw, sh) = nexora_platform::sys::screen_size();
    let mut ars: HashMap<String, f32> = self
      .shots
      .iter()
      .map(|(k, b)| (k.clone(), b.w as f32 / b.h.max(1) as f32))
      .collect();
    for (k, c) in &self.live {
      if c.has_frame {
        let (w, h) = c.size();
        if h > 0 {
          ars.insert(k.clone(), w as f32 / h as f32);
        }
      }
    }
    let shots = &self.shots;
    let hold = self.hold.clone();
    let Some(gfx) = self.gfx.as_ref() else { return };
    let Some(strip) = self.strip.as_mut() else {
      return;
    };
    let prev: HashMap<String, f32> = strip
      .slots
      .iter()
      .filter_map(|sl| sl.apps.first().map(|a| (a.clone(), sl.rect.1)))
      .collect();
    let slide_ms = motion::stage_ms(motion::STRIP_SLIDE_MS);

    let cfg = prefs::get();
    let right = cfg.stage_right;
    let sign = if right { -1.0f32 } else { 1.0 };
    let (tw, th_max, gap) = (
      cfg.stage_size as f32 * dpi,
      cfg.stage_size as f32 * 0.72 * dpi,
      cfg.stage_gap as f32 * dpi,
    );
    strip.w = tw + 56.0 * dpi;
    strip.right = right;
    let slot_h = th_max + 22.0 * dpi;
    let n = groups.len() as f32;
    let mut total = n * slot_h + (n - 1.0) * gap;
    let mut scale = 1.0;
    let avail = sh as f32 - 120.0 * dpi;
    if total > avail {
      scale = avail / total;
      total = avail;
    }
    let (slot_h, gap, tw, th_max) = (slot_h * scale, gap * scale, tw * scale, th_max * scale);
    let y0 = ((sh as f32 - total) / 2.0).max(40.0 * dpi);
    let pad = 26.0 * dpi;
    let px = if right { sw - strip.w as i32 } else { 0 };
    let _ = strip.popup.set_rect(px, y0 as i32, strip.w as i32, total.ceil() as i32);
    let card_x = if right { strip.w - pad - tw } else { pad };
    let show_titles = cfg.stage_titles;
    let _ = strip.popup.panel.set_rect(0.0, 0.0, 0.0, 0.0);
    if let Ok(k) = strip.popup.content.Children() {
      let _ = k.RemoveAll();
    }
    strip.slots.clear();
    let go = |strip: &mut StripUi| -> windows_core::Result<()> {
      for (i, g) in groups.iter().enumerate() {
        let sy = i as f32 * (slot_h + gap);
        let cont = comp.CreateContainerVisual()?;
        cont.SetSize(Vector2 { X: tw, Y: slot_h })?;
        cont.SetOffset(Vector3 {
          X: card_x,
          Y: sy,
          Z: 0.0,
        })?;
        cont.SetCenterPoint(Vector3 {
          X: tw / 2.0,
          Y: slot_h / 2.0,
          Z: 0.0,
        })?;
        cont.SetTransformMatrix(perspective(tw / 2.0, slot_h / 2.0, 600.0 * dpi))?;
        let mut cards = Vec::new();
        let mut brushes: Vec<(String, CompositionSurfaceBrush)> = Vec::new();
        let mut front: Rect = (card_x, sy, tw, slot_h);
        let count = g.len().min(3);
        for j in (0..count).rev() {
          let app = &g[j];
          let (bw, bh) = gfx_dims(ars.get(app).copied(), tw, th_max);
          let off = j as f32;
          let v = comp.CreateSpriteVisual()?;
          v.SetSize(Vector2 { X: bw, Y: bh })?;
          let hug = if right { tw - bw } else { 0.0 };
          let (ox, oy) = (
            hug + sign * off * 7.0 * dpi * scale,
            slot_h - bh - 14.0 * dpi * scale - off * 9.0 * dpi * scale,
          );
          if j == 0 {
            front = (card_x + ox, sy + oy, bw, bh);
          }
          v.SetOffset(Vector3 { X: ox, Y: oy, Z: 0.0 })?;
          v.SetCenterPoint(Vector3 {
            X: bw / 2.0,
            Y: bh / 2.0,
            Z: 0.0,
          })?;
          v.SetRotationAxis(Vector3 { X: 0.0, Y: 1.0, Z: 0.0 })?;
          v.SetRotationAngleInDegrees(sign * TILT)?;
          if j > 0 {
            v.SetOpacity(0.78)?;
          }
          let brush: CompositionBrush = match shots.get(app) {
            Some(b) => {
              let surf = gfx.surface_from_bgra(b.w, b.h, &b.bgra)?;
              let br = comp.CreateSurfaceBrushWithSurface(&surf)?;
              br.SetStretch(CompositionStretch::Fill)?;
              brushes.push((app.clone(), br.clone()));
              windows_core::Interface::cast::<CompositionBrush>(&br)?
            }
            None => {
              let surf = gfx.surface_from_bgra(1, 1, &[34, 29, 26, 170])?;
              let br = comp.CreateSurfaceBrushWithSurface(&surf)?;
              br.SetStretch(CompositionStretch::Fill)?;
              brushes.push((app.clone(), br.clone()));
              windows_core::Interface::cast::<CompositionBrush>(&br)?
            }
          };
          v.SetBrush(&brush)?;
          let geo = comp.CreateRoundedRectangleGeometry()?;
          geo.SetSize(Vector2 { X: bw, Y: bh })?;
          geo.SetCornerRadius(Vector2 {
            X: 9.0 * dpi * scale,
            Y: 9.0 * dpi * scale,
          })?;
          let clip = comp.CreateGeometricClip()?;
          clip.SetGeometry(&geo)?;
          v.SetClip(&clip)?;
          cont.Children()?.InsertAtTop(&v)?;
          cards.push(v);
        }
        for (k, a) in g.iter().take(2).enumerate() {
          if let Some(b) = icon_bmps.get(a) {
            let surf = gfx.surface_from_bgra(b.w, b.h, &b.bgra)?;
            let br = comp.CreateSurfaceBrushWithSurface(&surf)?;
            br.SetStretch(CompositionStretch::Uniform)?;
            let v = comp.CreateSpriteVisual()?;
            v.SetBrush(&br)?;
            let isz = tw * 0.22;
            v.SetSize(Vector2 { X: isz, Y: isz })?;
            let bx = if right {
              tw - isz + 0.08 * tw - k as f32 * 14.0 * dpi * scale
            } else {
              -0.08 * tw + k as f32 * 14.0 * dpi * scale
            };
            v.SetOffset(Vector3 {
              X: bx,
              Y: slot_h - isz + 2.0 * dpi,
              Z: 0.0,
            })?;
            cont.Children()?.InsertAtTop(&v)?;
          }
        }
        let mut label = None;
        if show_titles {
          if let Some((surf, lw, lh)) = title_surface(gfx, &g[0], (11.0 * dpi) as i32, tw as i32) {
            let br = comp.CreateSurfaceBrushWithSurface(&surf)?;
            br.SetStretch(CompositionStretch::None)?;
            let v = comp.CreateSpriteVisual()?;
            v.SetBrush(&br)?;
            v.SetSize(Vector2 {
              X: lw as f32,
              Y: lh as f32,
            })?;
            v.SetOffset(Vector3 {
              X: (tw - lw as f32) / 2.0,
              Y: 0.0,
              Z: 0.0,
            })?;
            v.SetOpacity(0.0)?;
            cont.Children()?.InsertAtTop(&v)?;
            label = Some(v);
          }
        }
        strip.popup.content.Children()?.InsertAtTop(&cont)?;
        let held = g.iter().any(|a| hold.contains(a));
        match prev.get(&g[0]) {
          _ if held => cont.SetOpacity(0.0)?,
          Some(py) if (py - sy).abs() > 0.5 => {
            motion::vec3(
              &comp,
              &cont,
              "Offset",
              Vector3 {
                X: card_x,
                Y: *py,
                Z: 0.0,
              },
              Vector3 {
                X: card_x,
                Y: sy,
                Z: 0.0,
              },
              slide_ms,
            )?;
          }
          None if !prev.is_empty() => {
            motion::scalar(&comp, &cont, "Opacity", 0.0, 1.0, slide_ms)?;
            motion::vec3(
              &comp,
              &cont,
              "Scale",
              Vector3 { X: 0.9, Y: 0.9, Z: 1.0 },
              Vector3 { X: 1.0, Y: 1.0, Z: 1.0 },
              slide_ms,
            )?;
          }
          _ => {}
        }
        strip.slots.push(Slot {
          apps: g.clone(),
          rect: (card_x, sy, tw, slot_h),
          card: front,
          cont,
          cards,
          brushes,
          label,
        });
      }
      Ok(())
    };
    if let Err(e) = go(strip) {
      log::line("stage", &format!("strip gagal: {e}"));
      return;
    }
    if let Some(e) = &self.edge {
      let ew = (2.0 * dpi) as i32;
      e.place(if right { sw - ew } else { 0 }, 0, ew, sh);
      e.set_visible(true);
    }
    let was_shown = strip.shown;
    strip.hover = None;
    if was_shown {
      strip.popup.show();
    }
    self.sync_live();
  }
  pub(super) fn set_strip_shown(&mut self, show: bool) {
    let comp = self.comp.clone();
    let Some(s) = self.strip.as_mut() else { return };
    if s.shown == show {
      return;
    }
    s.shown = show;
    let out = if s.right { s.w } else { -s.w };
    let from = if show { out } else { 0.0 };
    let to = if show { 0.0 } else { out };
    let _ = motion::vec3(
      &comp,
      &s.popup.content,
      "Offset",
      Vector3 {
        X: from,
        Y: 0.0,
        Z: 0.0,
      },
      Vector3 { X: to, Y: 0.0, Z: 0.0 },
      motion::stage_ms(motion::STRIP_SLIDE_MS),
    );
    unsafe {
      if show {
        let _ = KillTimer(Some(self.ctl), T_HIDE);
        s.popup.show();
      } else {
        SetTimer(
          Some(self.ctl),
          T_HIDE,
          motion::stage_ms(motion::STRIP_SLIDE_MS) + 20,
          None,
        );
      }
    }
  }
  pub(super) fn set_hover(&mut self, i: Option<usize>) {
    let comp = self.comp.clone();
    let Some(s) = self.strip.as_mut() else { return };
    if s.hover == i {
      return;
    }
    let sign = if s.right { -1.0f32 } else { 1.0 };
    let ms = motion::stage_ms(motion::CARD_HOVER_MS);
    let tilt = |sl: &Slot, on: bool| {
      let (a, b, sc) = if on {
        (sign * TILT, sign * TILT_HOVER, 1.06)
      } else {
        (sign * TILT_HOVER, sign * TILT, 1.0)
      };
      for c in &sl.cards {
        let _ = motion::scalar(&comp, c, "RotationAngleInDegrees", a, b, ms);
        let _ = c.SetRotationAngleInDegrees(b);
      }
      let from = if on { 1.0 } else { 1.06 };
      let _ = motion::vec3(
        &comp,
        &sl.cont,
        "Scale",
        Vector3 {
          X: from,
          Y: from,
          Z: 1.0,
        },
        Vector3 { X: sc, Y: sc, Z: 1.0 },
        ms,
      );
      let _ = sl.cont.SetScale(Vector3 { X: sc, Y: sc, Z: 1.0 });
      if let Some(l) = &sl.label {
        let (o0, o1) = if on { (0.0, 1.0) } else { (1.0, 0.0) };
        let _ = motion::scalar(&comp, l, "Opacity", o0, o1, ms);
        let _ = l.SetOpacity(o1);
      }
    };
    if let Some(old) = s.hover.and_then(|o| s.slots.get(o)) {
      tilt(old, false);
    }
    if let Some(new) = i.and_then(|n| s.slots.get(n)) {
      tilt(new, true);
    }
    s.hover = i;
  }
}
