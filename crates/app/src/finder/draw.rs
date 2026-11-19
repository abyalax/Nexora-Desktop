//! Finder bar drawing: text, glyphs, logo, status icons and hover capsule.

use super::*;

impl Bar {
  pub(super) fn sprite(
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
      let (dy, om) = if self.shadow {
        (1.0 * self.dpi, 0.38)
      } else {
        (0.0, 1.0)
      };
      v.SetOffset(Vector3 {
        X: x,
        Y: y + dy,
        Z: 0.0,
      })?;
      v.SetOpacity(op * om)?;
      parent.Children()?.InsertAtTop(&v)
    };
    let _ = go();
  }

  pub(super) fn draw_text(&mut self, text: &str, weight: i32, x: f32, w: f32, op: f32) {
    let px = (13.0 * self.dpi).round() as i32;
    let rgb = self.fg();
    let key = format!("t|{px}|{weight}|{rgb:?}|{text}");
    let hit = self.cache.get(&key).cloned();
    let (surf, tw, th) = match hit {
      Some(v) => v,
      None => {
        let style = Style { px, weight };
        let (tw, th) = measure(text, &style);
        let tw = tw + 2;
        let Some(bmp) = render(text, &style, tw, th) else {
          return;
        };
        let data = recolor(&bmp, rgb);
        let Ok(s) = self.gfx.surface_from_bgra(bmp.w, bmp.h, &data) else {
          return;
        };
        if self.cache.len() > 120 {
          self.cache.clear();
        }
        let v = (s, tw, th);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let y = (self.h as f32 - th as f32) / 2.0;
    let layer = self.layer.clone();
    self.sprite(&layer, &surf, x + (w - tw as f32) / 2.0, y, tw as f32, th as f32, op);
  }

  pub(super) fn draw_glyph(&mut self, glyph: &str, cx: f32, size: f32, op: f32) {
    let px = (size * self.dpi).round() as i32;
    let rgb = self.fg();
    if let Some(sym) = nexora_core::symbols::for_glyph(glyph) {
      let n = (size * 1.25 * self.dpi).round().max(8.0) as u32;
      let key = format!("sym|{sym:?}|{n}|{rgb:?}");
      let (surf, w, h) = match self.cache.get(&key).cloned() {
        Some(v) => v,
        None => {
          let data = nexora_core::symbols::render(sym, n, rgb);
          let Ok(s) = self.gfx.surface_from_bgra(n, n, &data) else {
            return;
          };
          let v = (s, n as i32, n as i32);
          self.cache.insert(key, v.clone());
          v
        }
      };
      let layer = self.layer.clone();
      self.sprite(
        &layer,
        &surf,
        (cx - w as f32 / 2.0).round(),
        ((self.h as f32 - h as f32) / 2.0).round(),
        w as f32,
        h as f32,
        op,
      );
      return;
    }
    let key = format!("g|{px}|{rgb:?}|{glyph}");
    let hit = self.cache.get(&key).cloned();
    let (surf, w, h) = match hit {
      Some(v) => v,
      None => {
        let b = (size * 1.6 * self.dpi).round() as i32;
        let Some(bmp) = render_face(glyph, &Style { px, weight: 400 }, ICON_FACE, b, b) else {
          return;
        };
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
          return;
        }
        let (gw, gh) = (x1 - x0 + 1, y1 - y0 + 1);
        let mut data = vec![0u8; gw * gh * 4];
        let rec = recolor(&bmp, rgb);
        for y in 0..gh {
          for x in 0..gw {
            let s = ((y0 + y) * w + x0 + x) * 4;
            let d = (y * gw + x) * 4;
            data[d..d + 4].copy_from_slice(&rec[s..s + 4]);
          }
        }
        let Ok(s) = self.gfx.surface_from_bgra(gw as u32, gh as u32, &data) else {
          return;
        };
        let v = (s, gw as i32, gh as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let layer = self.layer.clone();
    self.sprite(
      &layer,
      &surf,
      cx - w as f32 / 2.0,
      (self.h as f32 - h as f32) / 2.0,
      w as f32,
      h as f32,
      op,
    );
  }

  pub(super) fn draw_logo(&mut self, x: f32, w: f32) {
    let size = (self.h as f32 * 0.54).round() as u32;
    let rgb = self.fg();
    let custom = crate::finder::logo::path().is_some();
    let size = if custom {
      (self.h as f32 * 0.5).round() as u32
    } else {
      size
    };
    let key = format!("logo|{size}|{rgb:?}|{}|{custom}", crate::finder::logo::generation());
    let hit = self.cache.get(&key).cloned();
    let (surf, lw, lh) = match hit {
      Some(v) => v,
      None => {
        let (bw, bh, data) = match crate::finder::logo::bitmap(size, rgb).filter(|_| custom) {
          Some(b) => (b.w, b.h, b.bgra),
          None => (size, size, nexora_core::logo::render_badge(size, rgb)),
        };
        let Ok(s) = self.gfx.surface_from_bgra(bw, bh, &data) else {
          return;
        };
        let v = (s, bw as i32, bh as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let layer = self.layer.clone();
    self.sprite(
      &layer,
      &surf,
      x + (w - lw as f32) / 2.0,
      (self.h as f32 - lh as f32) / 2.0,
      lw as f32,
      lh as f32,
      1.0,
    );
  }

  pub(super) fn small_w(&self, text: &str, size: f32, weight: i32) -> f32 {
    measure(
      text,
      &Style {
        px: (size * self.dpi).round() as i32,
        weight,
      },
    )
    .0 as f32
      + 2.0
  }

  pub(super) fn small_text(&mut self, text: &str, size: f32, weight: i32, rgb: [u8; 3], cx: f32, cy: f32, op: f32) {
    let px = (size * self.dpi).round() as i32;
    let key = format!("st|{px}|{weight}|{rgb:?}|{text}");
    let (surf, tw, th) = match self.cache.get(&key).cloned() {
      Some(v) => v,
      None => {
        let style = Style { px, weight };
        let (tw, th) = measure(text, &style);
        let Some(bmp) = render(text, &style, tw + 2, th) else {
          return;
        };
        let Ok(s) = self.gfx.surface_from_bgra(bmp.w, bmp.h, &recolor(&bmp, rgb)) else {
          return;
        };
        let v = (s, bmp.w as i32, bmp.h as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let layer = self.layer.clone();
    self.sprite(
      &layer,
      &surf,
      (cx - tw as f32 / 2.0).round(),
      (cy - th as f32 / 2.0).round(),
      tw as f32,
      th as f32,
      op,
    );
  }

  pub(super) fn draw_wifi(&mut self, cx: f32) {
    let d = self.dpi;
    let rgb = self.fg();
    let n = (17.0 * d).round() as u32;
    let level = if self.wifi == Some(false) {
      0
    } else {
      nexora_core::logo::wifi_level(self.wifi_bars)
    };
    let key = format!("wifi|{n}|{level}|{rgb:?}");
    let (surf, sw, sh) = match self.cache.get(&key).cloned() {
      Some(v) => v,
      None => {
        let data = nexora_core::logo::render_wifi(n, level, rgb, 0.32);
        let Ok(s) = self.gfx.surface_from_bgra(n, n, &data) else {
          return;
        };
        let v = (s, n as i32, n as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let layer = self.layer.clone();
    let y = ((self.h as f32 - sh as f32) / 2.0 + 1.0 * d).round();
    self.sprite(
      &layer,
      &surf,
      (cx - sw as f32 / 2.0).round(),
      y,
      sw as f32,
      sh as f32,
      1.0,
    );
  }

  pub(super) fn draw_battery(&mut self, x: f32, w: f32) {
    let Some((pct, charging)) = sys::battery() else {
      return;
    };
    let d = self.dpi;
    let rgb = self.fg();
    let mode = prefs::get().battery_pct;
    let inside = mode == 1;
    let (bw, bh) = if inside {
      ((29.0 * d).round() as u32, (14.0 * d).round() as u32)
    } else {
      ((25.0 * d).round() as u32, (12.0 * d).round() as u32)
    };
    let fill = if charging {
      [52, 199, 89]
    } else if pct <= 10 {
      [255, 69, 58]
    } else {
      rgb
    };
    let key = format!("batt3|{bw}|{bh}|{pct}|{rgb:?}|{fill:?}|{inside}");
    let (surf, sw, sh) = match self.cache.get(&key).cloned() {
      Some(v) => v,
      None => {
        let mut data = nexora_core::logo::render_battery_pill(bw, bh, pct, rgb, fill, inside);
        if inside {
          let style = Style {
            px: (9.5 * d).round() as i32,
            weight: BAR_W,
          };
          let label = pct.to_string();
          let (tw, th) = measure(&label, &style);
          if let Some(tb) = render(&label, &style, tw + 4, th + 2) {
            knock_out(&mut data, bw, bh, &tb);
          }
        }
        let Ok(s) = self.gfx.surface_from_bgra(bw, bh, &data) else {
          return;
        };
        let v = (s, bw as i32, bh as i32);
        self.cache.insert(key, v.clone());
        v
      }
    };
    let text_w = if mode == 2 {
      self.small_w(&format!("{pct}%"), 12.0, BAR_W) + 5.0 * d
    } else {
      0.0
    };
    let bx = (x + text_w + (w - text_w - sw as f32) / 2.0).round();
    let by = ((self.h as f32 - sh as f32) / 2.0).round();
    let layer = self.layer.clone();
    self.sprite(&layer, &surf, bx, by, sw as f32, sh as f32, 1.0);
    let cy = self.h as f32 / 2.0;
    match mode {
      2 => {
        self.small_text(&format!("{pct}%"), 12.0, BAR_W, rgb, x + text_w / 2.0, cy, 0.95);
      }
      _ => {}
    }
  }

  #[allow(dead_code)]
  pub(super) fn rect(&self, parent: &ContainerVisual, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let go = || -> windows_core::Result<()> {
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
    let _ = go();
  }

  pub(super) fn draw_items(&mut self) {
    let lefts = self.left.clone();
    let rights = self.right.clone();
    let (app, clock) = (self.app_label(), self.clock_text());
    for p in &lefts {
      match p.id {
        LOGO => self.draw_logo(p.x, p.w),
        APP => self.draw_text(&app, APP_W, p.x, p.w, 1.0),
        M_FILE => self.draw_text(&mt("Berkas"), BAR_W, p.x, p.w, 1.0),
        M_EDIT => self.draw_text(&mt("Edit"), BAR_W, p.x, p.w, 1.0),
        M_VIEW => self.draw_text(&mt("Menu:Tampilan"), BAR_W, p.x, p.w, 1.0),
        M_WIN => self.draw_text(&mt("Menu:Jendela"), BAR_W, p.x, p.w, 1.0),
        M_HELP => self.draw_text(&mt("Bantuan"), BAR_W, p.x, p.w, 1.0),
        n if n >= NATIVE0 && n < NATIVE0 + 20 => {
          let title = self
            .native
            .get((n - NATIVE0) as usize)
            .map(|t| t.title.clone())
            .unwrap_or_default();
          self.draw_text(&title, BAR_W, p.x, p.w, 1.0)
        }
        _ => {}
      }
    }
    for p in &rights {
      let cx = p.x + p.w / 2.0;
      match p.id {
        CLOCK => self.draw_text(&clock, BAR_W, p.x, p.w, 1.0),
        CTRL => self.draw_glyph("\u{E9E9}", cx, 13.0, 0.95),
        SEARCH => self.draw_glyph("\u{E721}", cx, 13.0, 0.95),
        STAGEI => self.draw_glyph("\u{E8A0}", cx, 13.0, 0.95),
        WIFI => self.draw_wifi(cx),
        BATT => {
          if !self.shadow {
            self.draw_battery(p.x, p.w)
          }
        }
        _ => {}
      }
    }
  }

  pub(super) fn render(&mut self) {
    if let Ok(k) = self.layer.Children() {
      let _ = k.RemoveAll();
    }
    self.update_text_color();
    if !self.dark_text && !prefs::get().menubar_glass && !prefs::get().menubar_material {
      self.shadow = true;
      self.draw_items();
      self.shadow = false;
    }
    self.draw_items();
    self.update_capsule();
  }

  pub(super) fn rect_of(&self, id: u32) -> Option<(f32, f32)> {
    self
      .left
      .iter()
      .chain(self.right.iter())
      .find(|p| p.id == id)
      .map(|p| (p.x, p.w))
  }

  pub(super) fn update_capsule(&self) {
    use windows::Foundation::TimeSpan;
    use windows::UI::Composition::{CompositionEasingFunction, CompositionObject};
    use windows_core::{HSTRING, Interface};
    let cc = if crate::controlcenter::is_open() {
      Some(CTRL)
    } else {
      None
    };
    let active = self
      .open
      .or(cc)
      .or(self.hover)
      .filter(|id| *id != CLOCK || self.open.is_none());
    let d = self.dpi;
    let ms = |v: u32| TimeSpan {
      Duration: v as i64 * 10_000,
    };
    match active.and_then(|id| self.rect_of(id)) {
      Some((x, w)) => {
        let (ch, cy) = (self.h as f32 - 4.0 * d, 2.0 * d);
        let nw = w - 4.0 * d;
        let to = Vector3 {
          X: x + 2.0 * d,
          Y: cy,
          Z: 0.0,
        };
        let visible = self.capsule.Opacity().unwrap_or(0.0) > 0.5;
        let (old_off, old_w) = (
          self.capsule.Offset().unwrap_or(to),
          self.capsule.Size().map(|s| s.X).unwrap_or(nw),
        );
        let moved = visible && ((old_off.X - to.X).abs() > 0.5 || (old_w - nw).abs() > 0.5);
        let _ = self.capsule.SetSize(Vector2 { X: nw, Y: ch });
        let _ = self.capsule.SetCenterPoint(Vector3 {
          X: nw / 2.0,
          Y: ch / 2.0,
          Z: 0.0,
        });
        let _ = self.capsule.SetOffset(to);
        let rgb = self.fg();
        let (a_top, a_bot) = if self.open.is_some() {
          (76u8, 46u8)
        } else {
          (52u8, 26u8)
        };
        if let Ok(gb) = self.comp.CreateLinearGradientBrush() {
          let _ = gb.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 });
          let _ = gb.SetEndPoint(Vector2 { X: 0.0, Y: 1.0 });
          if let (Ok(s0), Ok(s1), Ok(stops)) = (
            self.comp.CreateColorGradientStopWithOffsetAndColor(
              0.0,
              Color {
                A: a_top,
                R: rgb[0],
                G: rgb[1],
                B: rgb[2],
              },
            ),
            self.comp.CreateColorGradientStopWithOffsetAndColor(
              1.0,
              Color {
                A: a_bot,
                R: rgb[0],
                G: rgb[1],
                B: rgb[2],
              },
            ),
            gb.ColorStops(),
          ) {
            let _ = stops.Append(&s0);
            let _ = stops.Append(&s1);
            let _ = self.capsule.SetBrush(&gb);
          }
        }
        if let Ok(g) = self.comp.CreateRoundedRectangleGeometry() {
          let _ = g.SetSize(Vector2 { X: nw, Y: ch });
          let _ = g.SetCornerRadius(Vector2 {
            X: ch / 2.0,
            Y: ch / 2.0,
          });
          if let Ok(clip) = self.comp.CreateGeometricClip() {
            let _ = clip.SetGeometry(&g);
            let _ = self.capsule.SetClip(&clip);
          }
        }
        let ease = || -> windows_core::Result<CompositionEasingFunction> {
          self
            .comp
            .CreateCubicBezierEasingFunction(Vector2 { X: 0.2, Y: 0.0 }, Vector2 { X: 0.0, Y: 1.0 })?
            .cast()
        };
        if !visible {
          let go = || -> windows_core::Result<()> {
            let e = ease()?;
            let o = self.comp.CreateScalarKeyFrameAnimation()?;
            o.InsertKeyFrame(0.0, 0.0)?;
            o.InsertKeyFrameWithEasingFunction(1.0, 1.0, &e)?;
            o.SetDuration(ms(150))?;
            self
              .capsule
              .cast::<CompositionObject>()?
              .StartAnimation(&HSTRING::from("Opacity"), &o)?;
            let sc = self.comp.CreateVector3KeyFrameAnimation()?;
            sc.InsertKeyFrame(
              0.0,
              Vector3 {
                X: 0.90,
                Y: 0.86,
                Z: 1.0,
              },
            )?;
            sc.InsertKeyFrameWithEasingFunction(1.0, Vector3 { X: 1.0, Y: 1.0, Z: 1.0 }, &e)?;
            sc.SetDuration(ms(190))?;
            self
              .capsule
              .cast::<CompositionObject>()?
              .StartAnimation(&HSTRING::from("Scale"), &sc)
          };
          let _ = go();
        } else if moved {
          let from_x = old_off.X + old_w / 2.0 - nw / 2.0;
          let _ = crate::motion::vec3(
            &self.comp,
            &self.capsule,
            "Offset",
            Vector3 {
              X: from_x,
              Y: cy,
              Z: 0.0,
            },
            to,
            crate::motion::SWITCHER_MOVE_MS + 50,
          );
          let _ = crate::motion::vec3(
            &self.comp,
            &self.capsule,
            "Scale",
            Vector3 {
              X: (old_w / nw).clamp(0.5, 2.0),
              Y: 1.0,
              Z: 1.0,
            },
            Vector3 { X: 1.0, Y: 1.0, Z: 1.0 },
            crate::motion::SWITCHER_MOVE_MS + 50,
          );
        }
        let _ = self.capsule.SetOpacity(1.0);
      }
      None => {
        if self.capsule.Opacity().unwrap_or(0.0) > 0.01 {
          let _ = crate::motion::scalar(&self.comp, &self.capsule, "Opacity", 1.0, 0.0, 150);
          let _ = crate::motion::vec3(
            &self.comp,
            &self.capsule,
            "Scale",
            Vector3 { X: 1.0, Y: 1.0, Z: 1.0 },
            Vector3 {
              X: 0.94,
              Y: 0.9,
              Z: 1.0,
            },
            150,
          );
          let _ = self.capsule.SetOpacity(0.0);
        }
      }
    }
  }

  pub(super) fn press_feedback(&self) {
    use windows::Foundation::TimeSpan;
    use windows::UI::Composition::{CompositionEasingFunction, CompositionObject};
    use windows_core::{HSTRING, Interface};
    if self.capsule.Opacity().unwrap_or(0.0) < 0.5 {
      return;
    }
    let go = || -> windows_core::Result<()> {
      let e: CompositionEasingFunction = self
        .comp
        .CreateCubicBezierEasingFunction(Vector2 { X: 0.2, Y: 0.0 }, Vector2 { X: 0.0, Y: 1.0 })?
        .cast()?;
      let sc = self.comp.CreateVector3KeyFrameAnimation()?;
      sc.InsertKeyFrame(0.0, Vector3 { X: 1.0, Y: 1.0, Z: 1.0 })?;
      sc.InsertKeyFrameWithEasingFunction(
        0.25,
        Vector3 {
          X: 0.93,
          Y: 0.88,
          Z: 1.0,
        },
        &e,
      )?;
      sc.InsertKeyFrameWithEasingFunction(
        0.65,
        Vector3 {
          X: 1.035,
          Y: 1.04,
          Z: 1.0,
        },
        &e,
      )?;
      sc.InsertKeyFrameWithEasingFunction(1.0, Vector3 { X: 1.0, Y: 1.0, Z: 1.0 }, &e)?;
      sc.SetDuration(TimeSpan { Duration: 300 * 10_000 })?;
      self
        .capsule
        .cast::<CompositionObject>()?
        .StartAnimation(&HSTRING::from("Scale"), &sc)
    };
    let _ = go();
  }
}

pub(super) fn knock_out(data: &mut [u8], w: u32, h: u32, text: &nexora_platform::icons::Bitmap) {
  let (tw, th) = (text.w as usize, text.h as usize);
  let (mut x0, mut y0, mut x1, mut y1) = (tw, th, 0usize, 0usize);
  for y in 0..th {
    for x in 0..tw {
      if text.bgra[(y * tw + x) * 4 + 3] > 24 {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
      }
    }
  }
  if x0 > x1 {
    return;
  }
  let nub = (w as f32 * 0.06).max(1.5);
  let body = w as f32 - nub - 1.0;
  let (iw, ih) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
  let ox = ((body - iw) / 2.0).round() as i32 - x0 as i32;
  let oy = ((h as f32 - ih) / 2.0).round() as i32 - y0 as i32;
  for y in 0..th {
    for x in 0..tw {
      let m = text.bgra[(y * tw + x) * 4 + 3] as u32;
      if m == 0 {
        continue;
      }
      let (dx, dy) = (x as i32 + ox, y as i32 + oy);
      if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
        continue;
      }
      let i = ((dy as u32 * w + dx as u32) * 4) as usize;
      for c in 0..4 {
        data[i + c] = (data[i + c] as u32 * (255 - m) / 255) as u8;
      }
    }
  }
}
