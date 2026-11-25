use super::*;

impl St {
  pub(super) fn start_fly(&mut self, items: Vec<(Bitmap, Rect, Rect)>, show: Vec<String>, activate: isize) {
    let comp = self.comp.clone();
    if self.gfx.is_none() {
      self.gfx = Gfx::shared(&comp).ok();
    }
    let Some(gfx) = self.gfx.as_ref() else { return };
    let (sw, sh) = nexora_platform::sys::screen_size();
    let go = || -> windows_core::Result<Popup> {
      let base = nexora_ui::glass::GlassParams::dark((10.0, 10.0), 0.0);
      let mut popup = Popup::new(&comp, w!("NexoraStageFly"), Some(fly_proc), WS_EX_TRANSPARENT, base)?;
      popup.set_rect(0, 0, sw, sh)?;
      popup.panel.set_rect(0.0, 0.0, 0.0, 0.0)?;
      popup
        .content
        .SetTransformMatrix(perspective(sw as f32 / 2.0, sh as f32 / 2.0, 1400.0))?;
      for (bmp, from, to) in &items {
        let surf = gfx.surface_from_bgra(bmp.w, bmp.h, &bmp.bgra)?;
        let brush = comp.CreateSurfaceBrushWithSurface(&surf)?;
        brush.SetStretch(CompositionStretch::Fill)?;
        let v = comp.CreateSpriteVisual()?;
        v.SetBrush(&brush)?;
        v.SetSize(Vector2 { X: to.2, Y: to.3 })?;
        let g = comp.CreateRoundedRectangleGeometry()?;
        g.SetSize(Vector2 { X: to.2, Y: to.3 })?;
        g.SetCornerRadius(Vector2 { X: 10.0, Y: 10.0 })?;
        let clip = comp.CreateGeometricClip()?;
        clip.SetGeometry(&g)?;
        v.SetClip(&clip)?;
        let (cx, cy) = (to.2 / 2.0, to.3 / 2.0);
        v.SetCenterPoint(Vector3 { X: cx, Y: cy, Z: 0.0 })?;
        let (sx, sy) = (from.2 / to.2.max(1.0), from.3 / to.3.max(1.0));
        popup.content.Children()?.InsertAtTop(&v)?;
        let off0 = Vector3 {
          X: from.0 - cx * (1.0 - sx),
          Y: from.1 - cy * (1.0 - sy),
          Z: 0.0,
        };
        motion::vec3(
          &comp,
          &v,
          "Offset",
          off0,
          Vector3 {
            X: to.0,
            Y: to.1,
            Z: 0.0,
          },
          motion::stage_ms(motion::STAGE_SWAP_MS),
        )?;
        let sign = if prefs::get().stage_right { -1.0f32 } else { 1.0 };
        let shrinking = from.2 > to.2;
        v.SetRotationAxis(Vector3 { X: 0.0, Y: 1.0, Z: 0.0 })?;
        let (a0, a1) = if shrinking {
          (0.0, sign * TILT)
        } else {
          (sign * TILT, 0.0)
        };
        motion::scalar(
          &comp,
          &v,
          "RotationAngleInDegrees",
          a0,
          a1,
          motion::stage_ms(motion::STAGE_SWAP_MS),
        )?;
        motion::vec3(
          &comp,
          &v,
          "Scale",
          Vector3 { X: sx, Y: sy, Z: 1.0 },
          Vector3 { X: 1.0, Y: 1.0, Z: 1.0 },
          motion::stage_ms(motion::STAGE_SWAP_MS),
        )?;
      }
      popup.show();
      Ok(popup)
    };
    match go() {
      Ok(popup) => {
        self.fly = Some(Fly { popup, show, activate });
        unsafe {
          SetTimer(
            Some(self.ctl),
            T_FLY,
            motion::stage_ms(motion::STAGE_SWAP_MS) + 30,
            None,
          );
        }
      }
      Err(e) => {
        log::line("stage", &format!("animasi gagal: {e}"));
        self.reveal_hold();
        self.unpark_apps(&show);
        if activate != 0 {
          focus::force_foreground(activate);
        }
      }
    }
  }
  pub(super) fn finish_fly(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_FLY);
    }
    if let Some(f) = self.fly.take() {
      self.reveal_hold();
      self.unpark_apps(&f.show);
      if f.activate != 0 {
        focus::force_foreground(f.activate);
      }
      if let Some(old) = self.dying.take() {
        let h = old.hwnd;
        drop(old);
        unsafe {
          let _ = DestroyWindow(h);
        }
      }
      let _ = motion::scalar(&self.comp, f.popup.root(), "Opacity", 1.0, 0.0, 140);
      let _ = f.popup.root().SetOpacity(0.0);
      self.dying = Some(f.popup);
      unsafe {
        SetTimer(Some(self.ctl), T_FLYEND, 180, None);
      }
    }
  }
  pub(super) fn kill_dying(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_FLYEND);
    }
    if let Some(old) = self.dying.take() {
      let h = old.hwnd;
      drop(old);
      unsafe {
        let _ = DestroyWindow(h);
      }
    }
  }
  pub(super) fn icon(&mut self, app: &str, px: i32) -> Option<Bitmap> {
    if !self.icons.contains_key(app) {
      let b = icons::load(app, px);
      self.icons.insert(app.to_string(), b);
    }
    self.icons.get(app).cloned().flatten()
  }
  pub(super) fn ensure_strip(&mut self) -> bool {
    if self.strip.is_some() {
      return true;
    }
    let mut look = prefs::look(prefs::Role::Dark);
    look.blur_sigma = 0.0;
    let base = nexora_ui::glass::GlassParams::dark((10.0, 10.0), 0.0);
    let Ok(popup) = Popup::new(
      &self.comp,
      w!("NexoraStageStrip"),
      Some(strip_proc),
      WINDOW_EX_STYLE(0),
      prefs::apply(base, &look),
    ) else {
      return false;
    };
    let dpi = unsafe { GetDpiForWindow(popup.hwnd) }.max(96) as f32 / 96.0;
    let _ = popup.panel.set_rect(0.0, 0.0, 0.0, 0.0);
    self.strip = Some(StripUi {
      popup,
      dpi,
      w: 150.0 * dpi,
      slots: vec![],
      hover: None,
      tracking: false,
      shown: true,
      right: false,
    });
    self.edge = crate::strip::Strip::new(self.ctl, WM_EDGE_IN, WM_EDGE_OUT).ok();
    true
  }
}
