use super::*;

pub(super) fn side_glass(mut g: nexora_ui::glass::GlassParams) -> nexora_ui::glass::GlassParams {
  g.blur_sigma = g.blur_sigma.max(16.0);
  g
}

pub(super) fn round_sdf(px: f32, py: f32, r: (f32, f32, f32, f32), rad: f32) -> f32 {
  let (cx, cy) = (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
  let rad = rad.min(r.2 / 2.0).min(r.3 / 2.0);
  let qx = (px - cx).abs() - (r.2 / 2.0 - rad);
  let qy = (py - cy).abs() - (r.3 / 2.0 - rad);
  (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rad
}

pub(super) fn frame_pixels(
  w: i32,
  h: i32,
  radius: f32,
  hole: (f32, f32, f32, f32),
  hole_radius: f32,
  dark: bool,
) -> Vec<u8> {
  let base: [f32; 3] = if dark {
    [16.0, 16.0, 18.0]
  } else {
    [255.0, 255.0, 255.0]
  };
  let (edge, edge_a): ([f32; 3], f32) = if dark {
    ([255.0, 255.0, 255.0], 0.16)
  } else {
    ([0.0, 0.0, 0.0], 0.12)
  };
  let outer = (0.0, 0.0, w as f32, h as f32);
  let blur = hole_radius * 1.1;
  let shade_dy = hole_radius * 0.12;
  let (tone, peak, inner): ([f32; 3], f32, f32) = if dark {
    ([255.0, 255.0, 255.0], 0.0, 0.07)
  } else {
    ([0.0, 0.0, 0.0], 0.10, 0.05)
  };
  let mut out = vec![0u8; (w * h * 4) as usize];
  for y in 0..h {
    for x in 0..w {
      let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
      let d = round_sdf(px, py, outer, radius);
      let cov = (0.5 - d).clamp(0.0, 1.0);
      if cov <= 0.0 {
        continue;
      }
      let a = cov;
      let sd = round_sdf(px, py - shade_dy, hole, hole_radius);
      let shade = if sd <= 0.0 {
        inner
      } else {
        peak * (1.0 - (sd / blur).clamp(0.0, 1.0)).powi(3)
      };
      let ring = (cov - (0.5 - (d + 1.0)).clamp(0.0, 1.0)).clamp(0.0, 1.0) * edge_a;
      let i = ((y * w + x) * 4) as usize;
      for c in 0..3 {
        let shaded = base[c] * (1.0 - shade) + tone[c] * shade;
        let v = shaded * (1.0 - ring) + edge[c] * ring;
        out[i + 2 - c] = (v * a).round() as u8;
      }
      out[i + 3] = (a * 255.0).round() as u8;
    }
  }
  out
}

pub(super) fn recolor(b: &Bitmap, rgb: [u8; 3]) -> Vec<u8> {
  let mut out = b.bgra.clone();
  for px in out.chunks_exact_mut(4) {
    let a = px[3] as u32;
    px[0] = (rgb[2] as u32 * a / 255) as u8;
    px[1] = (rgb[1] as u32 * a / 255) as u8;
    px[2] = (rgb[0] as u32 * a / 255) as u8;
  }
  out
}

pub(super) fn col(rgb: [u8; 3], a: u8) -> Color {
  Color {
    A: a,
    R: rgb[0],
    G: rgb[1],
    B: rgb[2],
  }
}

impl Win {
  pub(super) fn surface(
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
    if self.cache.len() > 160 {
      self.cache.clear();
    }
    let style = Style { px, weight };
    let (tw, th) = measure(s, &style);
    let tw = tw + 2;
    let bmp = render(s, &style, tw, th)?;
    let data = recolor(&bmp, rgb);
    let surf = self.gfx.surface_from_bgra(bmp.w, bmp.h, &data).ok()?;
    let v = (surf, tw, th);
    self.cache.insert(key, v.clone());
    Some(v)
  }
  #[allow(clippy::too_many_arguments)]
  pub(super) fn text(
    &mut self,
    parent: &ContainerVisual,
    s: &str,
    x: f32,
    y: f32,
    size: f32,
    weight: i32,
    rgb: [u8; 3],
    alpha: f32,
    anchor: u8,
  ) -> f32 {
    let px = (size * self.dpi).round() as i32;
    let Some((surf, w, h)) = self.surface(s, px, weight, rgb) else {
      return 0.0;
    };
    let left = match anchor {
      1 => x - w as f32 / 2.0,
      2 => x - w as f32,
      _ => x,
    };
    let make = || -> windows_core::Result<()> {
      let b = self.comp.CreateSurfaceBrushWithSurface(&surf)?;
      b.SetStretch(CompositionStretch::None)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 {
        X: w as f32,
        Y: h as f32,
      })?;
      v.SetOffset(Vector3 { X: left, Y: y, Z: 0.0 })?;
      v.SetOpacity(alpha)?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = make();
    w as f32
  }
  pub(super) fn rect(&self, parent: &ContainerVisual, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let make = || -> windows_core::Result<()> {
      let v = self.comp.CreateSpriteVisual()?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      v.SetBrush(&self.comp.CreateColorBrushWithColor(c)?)?;
      if r > 0.0 {
        let g = self.comp.CreateRoundedRectangleGeometry()?;
        g.SetSize(Vector2 { X: w, Y: h })?;
        g.SetCornerRadius(Vector2 { X: r, Y: r })?;
        let clip = self.comp.CreateGeometricClip()?;
        clip.SetGeometry(&g)?;
        v.SetClip(&clip)?;
      }
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = make();
  }
  pub(super) fn glass_knob(&self, parent: &ContainerVisual, x: f32, y: f32, w: f32, h: f32) {
    let d = self.dpi;
    self.rect(
      parent,
      x - 1.0 * d,
      y + 1.0 * d,
      w + 2.0 * d,
      h + 2.0 * d,
      h / 2.0 + d,
      Color {
        A: 46,
        R: 0,
        G: 0,
        B: 0,
      },
    );
    let make = || -> windows_core::Result<()> {
      let geo = self.comp.CreateRoundedRectangleGeometry()?;
      geo.SetOffset(Vector2 { X: 0.5, Y: 0.5 })?;
      geo.SetSize(Vector2 {
        X: (w - 1.0).max(0.0),
        Y: (h - 1.0).max(0.0),
      })?;
      geo.SetCornerRadius(Vector2 { X: h / 2.0, Y: h / 2.0 })?;
      let shape = self.comp.CreateSpriteShapeWithGeometry(&geo)?;
      shape.SetFillBrush(&self.comp.CreateColorBrushWithColor(Color {
        A: 92,
        R: 255,
        G: 255,
        B: 255,
      })?)?;
      shape.SetStrokeBrush(&self.comp.CreateColorBrushWithColor(Color {
        A: 190,
        R: 255,
        G: 255,
        B: 255,
      })?)?;
      shape.SetStrokeThickness(1.0)?;
      let v = self.comp.CreateShapeVisual()?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      v.Shapes()?.Append(&shape)?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = make();
  }
  pub(super) fn glyph_icon(&mut self, parent: &ContainerVisual, g: &str, cx: f32, cy: f32, size: f32, rgb: [u8; 3]) {
    let px = (size * self.dpi).round() as i32;
    let b = (size * 1.8 * self.dpi).round() as i32;
    let key = format!("ico|{px}|{rgb:?}|{g}");
    let hit = self.cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let Some(bmp) = render_face(g, &Style { px, weight: 400 }, ICON_FACE, b, b) else {
          return;
        };
        let Some((cd, cw, ch)) = nexora_core::symbols::crop_ink(&recolor(&bmp, rgb), bmp.w, bmp.h) else {
          return;
        };
        let Ok(sf) = self.gfx.surface_from_bgra(cw, ch, &cd) else {
          return;
        };
        let v = (sf, cw as i32, ch as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let make = || -> windows_core::Result<()> {
      let br = self.comp.CreateSurfaceBrushWithSurface(&surf)?;
      br.SetStretch(CompositionStretch::None)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&br)?;
      v.SetSize(Vector2 {
        X: w as f32,
        Y: h as f32,
      })?;
      v.SetOffset(Vector3 {
        X: cx - w as f32 / 2.0,
        Y: cy - h as f32 / 2.0,
        Z: 0.0,
      })?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = make();
  }
  pub(super) fn sprite(&self, parent: &ContainerVisual, s: &CompositionDrawingSurface, x: f32, y: f32, w: f32, h: f32) {
    let make = || -> windows_core::Result<()> {
      let b = self.comp.CreateSurfaceBrushWithSurface(s)?;
      b.SetStretch(CompositionStretch::Fill)?;
      let v = self.comp.CreateSpriteVisual()?;
      v.SetBrush(&b)?;
      v.SetSize(Vector2 { X: w, Y: h })?;
      v.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = make();
  }
  pub(super) fn thumb_surface(
    &mut self,
    key: String,
    w: u32,
    h: u32,
    make: impl FnOnce() -> Vec<u8>,
  ) -> Option<CompositionDrawingSurface> {
    if let Some(s) = self.thumbs.get(&key) {
      return Some(s.clone());
    }
    if self.thumbs.len() > 64 {
      self.thumbs.clear();
    }
    let s = self.gfx.surface_from_bgra(w, h, &make()).ok()?;
    self.thumbs.insert(key, s.clone());
    Some(s)
  }
  #[allow(clippy::too_many_arguments)]
  pub(super) fn text_abs(
    &mut self,
    parent: &ContainerVisual,
    s: &str,
    x: f32,
    y: f32,
    size: f32,
    weight: i32,
    rgb: [u8; 3],
    alpha: f32,
  ) {
    self.text(parent, s, x, y, size, weight, rgb, alpha, 0);
  }
}
