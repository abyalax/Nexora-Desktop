use super::*;

type Vec3Frame = (f32, Vector3, (f32, f32), (f32, f32));

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
  Minimize,
  Restore,
  Close,
  Open,
}

impl Kind {
  pub(super) fn ms(self) -> u32 {
    let g = genie_on();
    match self {
      Kind::Minimize => {
        if g {
          540
        } else {
          380
        }
      }
      Kind::Restore => {
        if g {
          470
        } else {
          340
        }
      }
      Kind::Close => 270,
      Kind::Open => 300,
    }
  }
}

pub(super) fn span(ms: u32) -> TimeSpan {
  TimeSpan {
    Duration: ms as i64 * 10_000,
  }
}

impl Fx {
  pub(super) fn ease(&self, a: (f32, f32), b: (f32, f32)) -> windows_core::Result<CompositionEasingFunction> {
    self
      .comp
      .CreateCubicBezierEasingFunction(Vector2 { X: a.0, Y: a.1 }, Vector2 { X: b.0, Y: b.1 })?
      .cast()
  }
  pub(super) fn vec3(
    &self,
    obj: &CompositionObject,
    prop: &str,
    from: Vector3,
    frames: &[Vec3Frame],
    ms: u32,
  ) -> windows_core::Result<()> {
    let a = self.comp.CreateVector3KeyFrameAnimation()?;
    a.InsertKeyFrame(0.0, from)?;
    for (t, v, c1, c2) in frames {
      a.InsertKeyFrameWithEasingFunction(*t, *v, &self.ease(*c1, *c2)?)?;
    }
    a.SetDuration(span(ms))?;
    a.SetDelayTime(span(START_DELAY))?;
    a.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
    obj.StartAnimation(&HSTRING::from(prop), &a)
  }
  pub(super) fn scalar(
    &self,
    obj: &CompositionObject,
    prop: &str,
    frames: &[(f32, f32)],
    ms: u32,
  ) -> windows_core::Result<()> {
    let a = self.comp.CreateScalarKeyFrameAnimation()?;
    for (t, v) in frames {
      a.InsertKeyFrame(*t, *v)?;
    }
    a.SetDuration(span(ms))?;
    a.SetDelayTime(span(START_DELAY))?;
    a.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
    obj.StartAnimation(&HSTRING::from(prop), &a)
  }
  pub(super) fn play(
    &mut self,
    surface: CompositionDrawingSurface,
    a: (i32, i32, i32, i32),
    b: (i32, i32, i32, i32),
    kind: Kind,
  ) {
    if matches!(kind, Kind::Minimize | Kind::Restore) && genie_on() {
      let (win, icon) = if kind == Kind::Restore { (b, a) } else { (a, b) };
      match self.play_genie(surface, win, icon, kind == Kind::Restore) {
        Ok(()) => {
          log::line(
            "fx",
            &format!(
              "animasi {} genie ({} ms) jendela {win:?} ikon {icon:?}",
              if kind == Kind::Restore { "restore" } else { "minimize" },
              kind.ms()
            ),
          );
          unsafe {
            let _ = SetWindowPos(
              self.hwnd,
              Some(HWND_TOPMOST),
              0,
              0,
              self.screen.0,
              self.screen.1,
              SWP_NOACTIVATE,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            SetTimer(Some(self.hwnd), T_HIDE, START_DELAY + kind.ms() + 80, None);
          }
        }
        Err(e) => log::line("fx", &format!("genie gagal: {e}")),
      }
      return;
    }
    let mut go = || -> windows_core::Result<()> {
      let (win, icon) = match kind {
        Kind::Restore => (b, a),
        _ => (a, b),
      };
      let (wx, wy, ww, wh) = (win.0 as f32, win.1 as f32, win.2.max(1) as f32, win.3.max(1) as f32);
      let (ix, iy, iw, ih) = (icon.0 as f32, icon.1 as f32, icon.2.max(1) as f32, icon.3.max(1) as f32);
      if let Some(old) = self.sprite.take() {
        let _ = self.surface.root.Children()?.Remove(&old);
      }
      let sprite = self.comp.CreateSpriteVisual()?;
      let brush = self.comp.CreateSurfaceBrushWithSurface(&surface)?;
      brush.SetStretch(CompositionStretch::Fill)?;
      sprite.SetBrush(&brush)?;
      sprite.SetSize(Vector2 { X: ww, Y: wh })?;
      sprite.SetCenterPoint(Vector3 {
        X: ww / 2.0,
        Y: wh / 2.0,
        Z: 0.0,
      })?;
      self.surface.root.Children()?.InsertAtTop(&sprite)?;
      let obj: CompositionObject = sprite.cast()?;
      let at_win = Vector3 { X: wx, Y: wy, Z: 0.0 };
      let at_icon = Vector3 {
        X: ix + iw / 2.0 - ww / 2.0,
        Y: iy + ih / 2.0 - wh / 2.0,
        Z: 0.0,
      };
      let (sx, sy) = (iw / ww, ih / wh);
      let one = Vector3 { X: 1.0, Y: 1.0, Z: 1.0 };
      let ms = kind.ms();
      match kind {
        Kind::Minimize => {
          sprite.SetOffset(at_win)?;
          sprite.SetScale(one)?;
          self.vec3(&obj, "Offset", at_win, &[(1.0, at_icon, (0.45, 0.0), (0.2, 1.0))], ms)?;
          self.vec3(
            &obj,
            "Scale",
            one,
            &[
              (
                0.55,
                Vector3 {
                  X: 0.82,
                  Y: 0.35,
                  Z: 1.0,
                },
                (0.3, 0.0),
                (0.5, 1.0),
              ),
              (1.0, Vector3 { X: sx, Y: sy, Z: 1.0 }, (0.5, 0.0), (0.2, 1.0)),
            ],
            ms,
          )?;
          self.scalar(&obj, "Opacity", &[(0.0, 1.0), (0.82, 1.0), (1.0, 0.0)], ms)?;
        }
        Kind::Restore => {
          let from = Vector3 { X: sx, Y: sy, Z: 1.0 };
          sprite.SetOffset(at_icon)?;
          sprite.SetScale(from)?;
          self.vec3(&obj, "Offset", at_icon, &[(1.0, at_win, (0.2, 0.8), (0.2, 1.0))], ms)?;
          self.vec3(
            &obj,
            "Scale",
            from,
            &[
              (
                0.45,
                Vector3 {
                  X: 0.82,
                  Y: 0.35,
                  Z: 1.0,
                },
                (0.2, 0.7),
                (0.3, 1.0),
              ),
              (1.0, one, (0.2, 0.8), (0.2, 1.0)),
            ],
            ms,
          )?;
          self.scalar(&obj, "Opacity", &[(0.0, 0.0), (0.12, 1.0), (1.0, 1.0)], ms)?;
        }
        Kind::Close => {
          sprite.SetOffset(at_win)?;
          sprite.SetScale(one)?;
          self.vec3(
            &obj,
            "Scale",
            one,
            &[(
              1.0,
              Vector3 {
                X: 0.06,
                Y: 0.06,
                Z: 1.0,
              },
              (0.33, 0.55),
              (0.45, 1.0),
            )],
            ms,
          )?;
          self.scalar(&obj, "Opacity", &[(0.0, 1.0), (0.55, 0.92), (1.0, 0.0)], ms)?;
        }
        Kind::Open => {
          let from = Vector3 {
            X: 0.86,
            Y: 0.86,
            Z: 1.0,
          };
          sprite.SetOffset(at_win)?;
          sprite.SetScale(from)?;
          sprite.SetOpacity(0.0)?;
          self.vec3(&obj, "Scale", from, &[(1.0, one, (0.16, 1.0), (0.3, 1.0))], ms)?;
          self.scalar(&obj, "Opacity", &[(0.0, 0.0), (0.45, 1.0), (1.0, 1.0)], ms)?;
        }
      }
      self.sprite = Some(sprite);
      log::line(
        "fx",
        &format!(
          "animasi {} ({ms} ms) dari {a:?} ke {b:?}",
          match kind {
            Kind::Minimize => "minimize",
            Kind::Restore => "restore",
            Kind::Close => "tutup",
            Kind::Open => "buka",
          }
        ),
      );
      unsafe {
        let _ = SetWindowPos(
          self.hwnd,
          Some(HWND_TOPMOST),
          0,
          0,
          self.screen.0,
          self.screen.1,
          SWP_NOACTIVATE,
        );
        let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        SetTimer(Some(self.hwnd), T_HIDE, START_DELAY + ms + 80, None);
      }
      Ok(())
    };
    if let Err(e) = go() {
      log::line("fx", &format!("animasi gagal: {e}"));
    }
  }
  pub(super) fn play_genie(
    &mut self,
    surface: CompositionDrawingSurface,
    win: (i32, i32, i32, i32),
    icon: (i32, i32, i32, i32),
    reverse: bool,
  ) -> windows_core::Result<()> {
    use nexora_core::genie::{self, Rect as GRect};
    if let Some(old) = self.genie.take() {
      let _ = self.surface.root.Children()?.Remove(&old);
    }
    if let Some(old) = self.sprite.take() {
      let _ = self.surface.root.Children()?.Remove(&old);
    }
    let w = GRect {
      x: win.0 as f32,
      y: win.1 as f32,
      w: win.2.max(1) as f32,
      h: win.3.max(1) as f32,
    };
    let ic = GRect {
      x: icon.0 as f32,
      y: icon.1 as f32,
      w: icon.2.max(1) as f32,
      h: icon.3.max(1) as f32,
    };
    let n = ((w.h / 5.0) as usize).clamp(48, 120);
    let sh = w.h / n as f32;
    let ms = if reverse {
      Kind::Restore.ms()
    } else {
      Kind::Minimize.ms()
    };
    let samples = ((ms / 12) as usize).clamp(24, 48);
    let root = self.comp.CreateContainerVisual()?;
    self.surface.root.Children()?.InsertAtTop(&root)?;
    let (dur, delay) = (span(ms), span(START_DELAY));
    let v3 = |p: (f32, f32), z: f32| Vector3 { X: p.0, Y: p.1, Z: z };
    for i in 0..n {
      let s = self.comp.CreateSpriteVisual()?;
      let b = self.comp.CreateSurfaceBrushWithSurface(&surface)?;
      b.SetStretch(CompositionStretch::Fill)?;
      b.SetTransformMatrix(Matrix3x2 {
        M11: 1.0,
        M12: 0.0,
        M21: 0.0,
        M22: n as f32,
        M31: 0.0,
        M32: -(i as f32) * sh,
      })?;
      s.SetBrush(&b)?;
      s.SetSize(Vector2 { X: w.w, Y: sh })?;
      s.SetCenterPoint(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 })?;
      let (o0, s0) = genie::slice(i, n, genie::tau_at(0.0, reverse), w, ic);
      s.SetOffset(v3(o0, 0.0))?;
      s.SetScale(v3(s0, 1.0))?;
      root.Children()?.InsertAtTop(&s)?;

      let ao = self.comp.CreateVector3KeyFrameAnimation()?;
      let asc = self.comp.CreateVector3KeyFrameAnimation()?;
      for k in 0..samples {
        let t = k as f32 / (samples - 1) as f32;
        let (o, sc) = genie::slice(i, n, genie::tau_at(t, reverse), w, ic);
        ao.InsertKeyFrame(t, v3(o, 0.0))?;
        asc.InsertKeyFrame(t, v3(sc, 1.0))?;
      }
      for a in [&ao, &asc] {
        a.SetDuration(dur)?;
        a.SetDelayTime(delay)?;
        a.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
      }
      let obj: CompositionObject = s.cast()?;
      obj.StartAnimation(&HSTRING::from("Offset"), &ao)?;
      obj.StartAnimation(&HSTRING::from("Scale"), &asc)?;
    }
    let fade = self.comp.CreateScalarKeyFrameAnimation()?;
    if reverse {
      fade.InsertKeyFrame(0.0, 0.0)?;
      fade.InsertKeyFrame(0.08, 1.0)?;
      fade.InsertKeyFrame(1.0, 1.0)?;
    } else {
      fade.InsertKeyFrame(0.0, 1.0)?;
      fade.InsertKeyFrame(0.88, 1.0)?;
      fade.InsertKeyFrame(1.0, 0.0)?;
    }
    fade.SetDuration(dur)?;
    fade.SetDelayTime(delay)?;
    fade.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
    root.SetOpacity(if reverse { 0.0 } else { 1.0 })?;
    root
      .cast::<CompositionObject>()?
      .StartAnimation(&HSTRING::from("Opacity"), &fade)?;
    self.genie = Some(root);
    Ok(())
  }
}
