use super::*;

impl Cc {
  pub(super) fn text_surf(
    &mut self,
    text: &str,
    px: i32,
    weight: i32,
    rgb: [u8; 3],
    maxw: i32,
  ) -> Option<(CompositionDrawingSurface, i32, i32)> {
    let key = format!("t|{px}|{weight}|{rgb:?}|{maxw}|{text}");
    if let Some(v) = self.cache.get(&key) {
      return Some(v.clone());
    }
    let style = Style { px, weight };
    let (tw, th) = measure(text, &style);
    let tw = (tw + 2).min(maxw.max(8));
    let bmp = render(text, &style, tw, th)?;
    let s = self
      .gfx
      .as_ref()?
      .surface_from_bgra(bmp.w, bmp.h, &recolor(&bmp, rgb))
      .ok()?;
    let v = (s, tw, th);
    self.cache.insert(key, v.clone());
    Some(v)
  }
  pub(super) fn glyph_surf(&mut self, g: &str, px: i32, rgb: [u8; 3]) -> Option<(CompositionDrawingSurface, i32, i32)> {
    let key = format!("g|{px}|{rgb:?}|{g}");
    if let Some(v) = self.cache.get(&key) {
      return Some(v.clone());
    }
    let b = (px as f32 * 1.6) as i32;
    let bmp = render_face(g, &Style { px, weight: 400 }, ICON_FACE, b, b)?;
    let (w, h) = (bmp.w as usize, bmp.h as usize);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
    for y in 0..h {
      for x in 0..w {
        if bmp.bgra[(y * w + x) * 4 + 3] > 8 {
          x0 = x0.min(x);
          x1 = x1.max(x);
          y0 = y0.min(y);
          y1 = y1.max(y);
        }
      }
    }
    if x0 > x1 {
      return None;
    }
    let (gw, gh) = (x1 - x0 + 1, y1 - y0 + 1);
    let rec = recolor(&bmp, rgb);
    let mut data = vec![0u8; gw * gh * 4];
    for y in 0..gh {
      for x in 0..gw {
        let s = ((y0 + y) * w + x0 + x) * 4;
        let d = (y * gw + x) * 4;
        data[d..d + 4].copy_from_slice(&rec[s..s + 4]);
      }
    }
    let s = self.gfx.as_ref()?.surface_from_bgra(gw as u32, gh as u32, &data).ok()?;
    let v = (s, gw as i32, gh as i32);
    self.cache.insert(key, v.clone());
    Some(v)
  }
  pub(super) fn symbol(&mut self, parent: &ContainerVisual, s: Sym, px: i32, rgb: [u8; 3], cx: f32, cy: f32, op: f32) {
    let n = px.max(8) as u32;
    let key = format!("sym|{s:?}|{n}|{rgb:?}");
    let hit = self.cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let data = symbols::render(s, n, rgb);
        let Some((cd, cw, ch)) = symbols::crop_ink(&data, n, n) else {
          return;
        };
        let Some(Ok(sf)) = self.gfx.as_ref().map(|g| g.surface_from_bgra(cw, ch, &cd)) else {
          return;
        };
        let v = (sf, cw as i32, ch as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    self.put(
      parent,
      &surf,
      cx - w as f32 / 2.0,
      cy - h as f32 / 2.0,
      w as f32,
      h as f32,
      op,
    );
  }
  pub(super) fn put(
    &self,
    parent: &ContainerVisual,
    s: &CompositionDrawingSurface,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    op: f32,
  ) {
    let go = || -> windows_core::Result<()> {
      let b = self.comp.CreateSurfaceBrushWithSurface(s)?;
      b.SetStretch(CompositionStretch::None)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      v.SetOpacity(op)?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = go();
  }
  pub(super) fn text(
    &mut self,
    parent: &ContainerVisual,
    t: &str,
    px: i32,
    weight: i32,
    rgb: [u8; 3],
    x: f32,
    y: f32,
    maxw: f32,
    op: f32,
  ) {
    if let Some((s, w, h)) = self.text_surf(t, px, weight, rgb, maxw as i32) {
      self.put(parent, &s, x, y, w as f32, h as f32, op);
    }
  }
  pub(super) fn glyph(&mut self, parent: &ContainerVisual, g: &str, px: i32, rgb: [u8; 3], cx: f32, cy: f32, op: f32) {
    if let Some((s, w, h)) = self.glyph_surf(g, px, rgb) {
      self.put(
        parent,
        &s,
        cx - w as f32 / 2.0,
        cy - h as f32 / 2.0,
        w as f32,
        h as f32,
        op,
      );
    }
  }
  pub(super) fn rrect(&self, parent: &ContainerVisual, r: Rect, radius: f32, c: Color) -> Option<SpriteVisual> {
    let go = || -> windows_core::Result<SpriteVisual> {
      let v = self.comp.CreateSpriteVisual()?;
      v.SetSize(Vector2 { X: r.2, Y: r.3 })?;
      v.SetOffset(Vector3 { X: r.0, Y: r.1, Z: 0.0 })?;
      v.SetBrush(&self.comp.CreateColorBrushWithColor(c)?)?;
      let g = self.comp.CreateRoundedRectangleGeometry()?;
      g.SetSize(Vector2 { X: r.2, Y: r.3 })?;
      g.SetCornerRadius(Vector2 { X: radius, Y: radius })?;
      let clip = self.comp.CreateGeometricClip()?;
      clip.SetGeometry(&g)?;
      v.SetClip(&clip)?;
      parent.Children()?.InsertAtTop(&v)?;
      Ok(v)
    };
    go().ok()
  }
  pub(super) fn glass(&self, parent: &ContainerVisual, r: Rect, radius: f32) -> Option<GlassPanel> {
    let look = prefs::look(prefs::Role::Dark);
    let base = GlassParams::dark((r.2, r.3), radius);
    let p = GlassPanel::new(&self.comp, parent, prefs::apply(base, &look)).ok()?;
    let _ = p.set_rect(r.0, r.1, r.2, r.3);
    let _ = p.set_radius(radius);
    Some(p)
  }
  pub(super) fn is_on(&self, k: Kind) -> bool {
    match k {
      Kind::Wifi => self.wifi == Some(true),
      Kind::Bluetooth => self.bt == Some(true),
      Kind::Stage => prefs::get().stage,
      Kind::DarkMode => prefs::look(prefs::Role::Dark).dark && prefs::get().theme == nexora_core::settings::Theme::Dark,
      _ => false,
    }
  }
  pub(super) fn status(&self, k: Kind) -> String {
    let on = prefs::tr("Aktif");
    let off = prefs::tr("Mati");
    match k {
      Kind::Wifi => match (self.wifi, &self.ssid) {
        (Some(true), Some(s)) => s.clone(),
        (Some(true), None) => on.to_string(),
        (Some(false), _) => off.to_string(),
        _ => "-".to_string(),
      },
      Kind::Bluetooth => match self.bt {
        Some(true) => on.to_string(),
        Some(false) => off.to_string(),
        _ => "-".to_string(),
      },
      Kind::Hotspot | Kind::Share => prefs::tr("Buka Pengaturan").to_string(),
      Kind::Battery => match nexora_platform::sys::battery() {
        Some((p, ch)) => format!("{p}%{}", if ch { " \u{26A1}" } else { "" }),
        None => "-".to_string(),
      },
      Kind::Focus => prefs::tr("Fokus").to_string(),
      _ => String::new(),
    }
  }
  pub(super) fn draw_chip(
    &mut self,
    content: &ContainerVisual,
    inuse: &[nexora_platform::privacy::InUse],
    pw: f32,
    dpi: f32,
    groups: &mut Vec<(ContainerVisual, Rect)>,
    panels: &mut Vec<GlassPanel>,
  ) -> Option<(Rect, bool)> {
    if inuse.is_empty() {
      return None;
    }
    let mut apps: Vec<&str> = Vec::new();
    for u in inuse {
      if !apps.contains(&u.app.as_str()) {
        apps.push(&u.app);
      }
    }
    let (mic, cam) = (inuse.iter().any(|u| !u.camera), inuse.iter().any(|u| u.camera));
    let label = apps.iter().take(2).copied().collect::<Vec<_>>().join(", ");
    let style = Style {
      px: (12.5 * dpi) as i32,
      weight: 600,
    };
    let (lw, _) = measure(&label, &style);
    let ch = 28.0 * dpi;
    let dots = (mic as u32 + cam as u32) as f32 * 13.0 * dpi;
    let cw = (12.0 * dpi + dots + 3.0 * dpi + lw as f32 + 14.0 * dpi).min(pw);
    let cr: Rect = (pw - cw, 0.0, cw, ch);
    let cc = self.comp.CreateContainerVisual().ok()?;
    let _ = content.Children().and_then(|k| k.InsertAtTop(&cc));
    groups.push((cc.clone(), cr));
    if let Some(g) = self.glass(&cc, cr, ch / 2.0) {
      panels.push(g);
    }
    let mut x = cr.0 + 12.0 * dpi;
    let d = 8.0 * dpi;
    for (on, col) in [
      (
        mic,
        Color {
          A: 255,
          R: 255,
          G: 149,
          B: 0,
        },
      ),
      (
        cam,
        Color {
          A: 255,
          R: 52,
          G: 199,
          B: 89,
        },
      ),
    ] {
      if on {
        let _ = self.rrect(&cc, (x, cr.1 + (ch - d) / 2.0, d, d), d / 2.0, col);
        x += 13.0 * dpi;
      }
    }
    self.text(
      &cc,
      &label,
      style.px,
      600,
      WHITE,
      x + 3.0 * dpi,
      cr.1 + 6.0 * dpi,
      lw as f32 + 4.0,
      1.0,
    );
    Some((cr, cam && !mic))
  }
  pub(super) fn glass_ring(&self, c: &ContainerVisual, b: Rect, dpi: f32) {
    let n = b.2.round().max(4.0) as u32;
    let data = nexora_core::widgets::ring(n, 1.4 * dpi, 1.0, [255, 255, 255], [0, 0, 0, 0]);
    if let Some(Ok(sf)) = self.gfx.as_ref().map(|g| g.surface_from_bgra(n, n, &data)) {
      self.put(c, &sf, b.0, b.1, n as f32, n as f32, 0.6);
    }
  }
  pub(super) fn art_at(&self, c: &ContainerVisual, x: f32, y: f32, art: f32, dpi: f32) {
    let Some(a) = self.art.clone() else {
      let _ = self.rrect(
        c,
        (x, y, art, art),
        (art * 0.18).min(12.0 * dpi),
        Color {
          A: 60,
          R: 255,
          G: 255,
          B: 255,
        },
      );
      return;
    };
    let Some(gfx) = self.gfx.as_ref() else { return };
    let go = || -> windows_core::Result<()> {
      let s = gfx.surface_from_bgra(a.w, a.h, &a.bgra)?;
      let b = self.comp.CreateSurfaceBrushWithSurface(&s)?;
      b.SetStretch(CompositionStretch::UniformToFill)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 { X: art, Y: art })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      let g = self.comp.CreateRoundedRectangleGeometry()?;
      g.SetSize(Vector2 { X: art, Y: art })?;
      let rad = (art * 0.18).min(14.0 * dpi);
      g.SetCornerRadius(Vector2 { X: rad, Y: rad })?;
      let cl = self.comp.CreateGeometricClip()?;
      cl.SetGeometry(&g)?;
      v.SetClip(&cl)?;
      c.Children()?.InsertAtTop(&v)
    };
    let _ = go();
  }
}
