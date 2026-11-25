use super::*;

impl Spot {
  pub(super) fn text(
    &mut self,
    parent: &ContainerVisual,
    s: &str,
    x: f32,
    cy: f32,
    px: i32,
    weight: i32,
    rgb: [u8; 3],
    op: f32,
    max_w: f32,
  ) -> f32 {
    if s.is_empty() {
      return 0.0;
    }
    let key = format!("{px}|{weight}|{rgb:?}|{}|{s}", max_w as i32);
    let hit = self.text_cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let style = Style { px, weight };
        let (tw, th) = measure(s, &style);
        let tw = (tw + 2).min(max_w as i32).max(2);
        let Some(bmp) = render(s, &style, tw, th) else {
          return 0.0;
        };
        let Ok(sf) = self.gfx.surface_from_bgra(bmp.w, bmp.h, &recolor(&bmp, rgb)) else {
          return 0.0;
        };
        if self.text_cache.len() > 300 {
          self.text_cache.clear();
        }
        let v = (sf, tw, th);
        self.text_cache.insert(key, v.clone());
        v
      }
    };
    self.sprite(parent, &surf, x, cy - h as f32 / 2.0, w as f32, h as f32, op);
    w as f32
  }

  pub(super) fn text_w(&self, s: &str, px: i32, weight: i32) -> f32 {
    measure(s, &Style { px, weight }).0 as f32 + 2.0
  }

  pub(super) fn sprite(&self, parent: &ContainerVisual, s: &CompositionDrawingSurface, x: f32, y: f32, w: f32, h: f32, op: f32) {
    let go = || -> windows_core::Result<()> {
      let b = self.comp.CreateSurfaceBrushWithSurface(s)?;
      b.SetStretch(CompositionStretch::Uniform)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      v.SetOpacity(op)?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = go();
  }

  pub(super) fn rect(&self, parent: &ContainerVisual, r: Rect, radius: f32, c: Color) {
    let go = || -> windows_core::Result<()> {
      let v = self.comp.CreateSpriteVisual()?;
      v.SetSize(Vector2 { X: r.2, Y: r.3 })?;
      v.SetOffset(Vector3 { X: r.0, Y: r.1, Z: 0.0 })?;
      v.SetBrush(&self.comp.CreateColorBrushWithColor(c)?)?;
      if radius > 0.0 {
        let g = self.comp.CreateRoundedRectangleGeometry()?;
        g.SetSize(Vector2 { X: r.2, Y: r.3 })?;
        g.SetCornerRadius(Vector2 { X: radius, Y: radius })?;
        let clip = self.comp.CreateGeometricClip()?;
        clip.SetGeometry(&g)?;
        v.SetClip(&clip)?;
      }
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = go();
  }

  pub(super) fn icon_surface(&mut self, path: &str, px: i32) -> Option<CompositionDrawingSurface> {
    let key = format!("{px}|{path}");
    if let Some(c) = self.icon_cache.get(&key) {
      return c.clone();
    }
    let plates = prefs::get().dock_plates;
    let s = icons::load(path, px).and_then(|mut b| {
      if !icons::is_themed(path) {
        if plates && b.w == b.h {
          b.bgra = nexora_core::iconplate::plate(&b.bgra, b.w);
        }
        let p = prefs::get();
        b.bgra = nexora_core::iconplate::style_icon_v(&b.bgra, p.icon_style, p.icon_tint, prefs::icon_dark());
      }
      self.gfx.surface_from_bgra(b.w, b.h, &b.bgra).ok()
    });
    if self.icon_cache.len() > 200 {
      self.icon_cache.clear();
    }
    self.icon_cache.insert(key, s.clone());
    s
  }

  pub(super) fn sym(&mut self, parent: &ContainerVisual, s: Sym, cx: f32, cy: f32, size: f32, rgb: [u8; 3], op: f32) {
    let n = size.round().max(8.0) as u32;
    let key = format!("sym|{s:?}|{n}|{rgb:?}");
    let hit = self.text_cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let data = symbols::render(s, n, rgb);
        let Some((cd, cw, ch)) = symbols::crop_ink(&data, n, n) else {
          return;
        };
        let Ok(sf) = self.gfx.surface_from_bgra(cw, ch, &cd) else {
          return;
        };
        let v = (sf, cw as i32, ch as i32);
        self.text_cache.insert(key, v.clone());
        v
      }
    };
    self.sprite(
      parent,
      &surf,
      cx - w as f32 / 2.0,
      cy - h as f32 / 2.0,
      w as f32,
      h as f32,
      op,
    );
  }

  pub(super) fn glyph(&mut self, parent: &ContainerVisual, g: &str, cx: f32, cy: f32, size: f32, rgb: [u8; 3], op: f32) {
    let px = size.round() as i32;
    let b = (size * 1.6).round() as i32;
    let key = format!("g|{px}|{rgb:?}|{g}");
    let hit = self.text_cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let Some(bmp) = render_face(g, &Style { px, weight: 400 }, ICON_FACE, b, b) else {
          return;
        };
        let Some((cd, cw, ch)) = symbols::crop_ink(&recolor(&bmp, rgb), bmp.w, bmp.h) else {
          return;
        };
        let Ok(sf) = self.gfx.surface_from_bgra(cw, ch, &cd) else {
          return;
        };
        let v = (sf, cw as i32, ch as i32);
        self.text_cache.insert(key, v.clone());
        v
      }
    };
    self.sprite(
      parent,
      &surf,
      cx - w as f32 / 2.0,
      cy - h as f32 / 2.0,
      w as f32,
      h as f32,
      op,
    );
  }

  pub(super) fn mode_icon(&mut self, parent: &ContainerVisual, m: Mode, cx: f32, cy: f32, size: f32, rgb: [u8; 3], op: f32) {
    match m {
      Mode::Search => self.sym(parent, Sym::Search, cx, cy, size, rgb, op),
      Mode::Apps => self.sym(parent, Sym::Grid, cx, cy, size * 0.95, rgb, op),
      Mode::Files => self.glyph(parent, "\u{E8B7}", cx, cy, size * 0.95, rgb, op),
      Mode::Actions => self.sym(parent, Sym::Windows, cx, cy, size, rgb, op),
      Mode::Clipboard => self.glyph(parent, "\u{E77F}", cx, cy, size * 0.95, rgb, op),
    }
  }

  pub(super) fn glass_circle(&self, parent: &ContainerVisual, r: Rect) -> Option<GlassPanel> {
    let base = GlassParams::dark((r.2, r.3), r.2 / 2.0);
    let p = GlassPanel::new(&self.comp, parent, prefs::apply(base, &prefs::look(prefs::Role::Dock))).ok()?;
    let _ = p.set_rect(r.0, r.1, r.2, r.3);
    let _ = p.set_radius(r.2 / 2.0);
    Some(p)
  }
}
