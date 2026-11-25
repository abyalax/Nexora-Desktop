use super::*;

impl Mgr {
  pub(super) fn text_surf(
    &mut self,
    s: &str,
    px: i32,
    weight: i32,
    rgb: [u8; 3],
  ) -> Option<(CompositionDrawingSurface, i32, i32)> {
    let key = format!("{px}|{weight}|{rgb:?}|{s}");
    if let Some(v) = self.cache.get(&key) {
      return Some(v.clone());
    }
    let st = Style { px, weight };
    let (tw, th) = measure(s, &st);
    let bmp = render(s, &st, tw + 2, th)?;
    let surf = self
      .gfx
      .as_ref()?
      .surface_from_bgra(bmp.w, bmp.h, &recolor(&bmp, rgb))
      .ok()?;
    if self.cache.len() > 500 {
      self.cache.clear();
    }
    let v = (surf, bmp.w as i32, bmp.h as i32);
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
    let surf = self
      .gfx
      .as_ref()?
      .surface_from_bgra(bmp.w, bmp.h, &recolor(&bmp, rgb))
      .ok()?;
    let v = (surf, bmp.w as i32, bmp.h as i32);
    self.cache.insert(key, v.clone());
    Some(v)
  }

  pub(super) fn put(
    &self,
    p: &ContainerVisual,
    s: &CompositionDrawingSurface,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    op: f32,
  ) -> Option<SpriteVisual> {
    let go = || -> windows_core::Result<SpriteVisual> {
      let b = self.comp.CreateSurfaceBrushWithSurface(s)?;
      b.SetStretch(CompositionStretch::None)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 {
        X: x.round(),
        Y: y.round(),
        Z: 0.0,
      })?;
      v.SetOpacity(op)?;
      p.Children()?.InsertAtTop(&v)?;
      Ok(v)
    };
    go().ok()
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn text(
    &mut self,
    p: &ContainerVisual,
    s: &str,
    px: f32,
    weight: i32,
    rgb: [u8; 3],
    x: f32,
    y: f32,
    anchor: u8,
    op: f32,
  ) -> f32 {
    let Some((surf, w, h)) = self.text_surf(s, px.round() as i32, weight, rgb) else {
      return 0.0;
    };
    let x = match anchor {
      1 => x - w as f32 / 2.0,
      2 => x - w as f32,
      _ => x,
    };
    self.put(p, &surf, x, y, w as f32, h as f32, op);
    w as f32
  }

  pub(super) fn glyph(&mut self, p: &ContainerVisual, g: &str, px: f32, rgb: [u8; 3], cx: f32, cy: f32, op: f32) {
    let Some((surf, w, h)) = self.glyph_surf(g, px.round() as i32, rgb) else {
      return;
    };
    self.put(
      p,
      &surf,
      cx - w as f32 / 2.0,
      cy - h as f32 / 2.0,
      w as f32,
      h as f32,
      op,
    );
  }

  pub(super) fn rr(&self, p: &ContainerVisual, r: Rect, radius: f32, c: Color) -> Option<SpriteVisual> {
    let go = || -> windows_core::Result<SpriteVisual> {
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
      p.Children()?.InsertAtTop(&v)?;
      Ok(v)
    };
    go().ok()
  }

  pub(super) fn shadow(&self, p: &ContainerVisual, w: f32, h: f32, radius: f32, op: f32) {
    let d = self.dpi;
    let s = nexora_core::dynamic_island::shade::rounded(radius, 16.0 * d, 5.0 * d);
    let Some(gfx) = self.gfx.as_ref() else { return };
    let go = || -> windows_core::Result<()> {
      let surf = gfx.surface_from_bgra(s.w, s.h, &s.bgra)?;
      let src = self.comp.CreateSurfaceBrushWithSurface(&surf)?;
      src.SetStretch(CompositionStretch::Fill)?;
      let ng = self.comp.CreateNineGridBrush()?;
      ng.SetSource(&src)?;
      let (l, t, r, b) = s.insets;
      ng.SetInsetsWithValues(l, t, r, b)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&ng)?;
      let mg = s.margin as f32;
      v.SetOffset(Vector3 { X: -mg, Y: -mg, Z: 0.0 })?;
      v.SetSize(Vector2 {
        X: w + 2.0 * mg,
        Y: h + 2.0 * mg + s.dy as f32,
      })?;
      v.SetOpacity(op)?;
      p.Children()?.InsertAtTop(&v)?;
      Ok(())
    };
    let _ = go();
  }
}
