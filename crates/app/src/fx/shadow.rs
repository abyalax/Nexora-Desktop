use super::*;

pub(super) struct ShadowWin {
  pub(super) hwnd: HWND,
  pub(super) surface: Surface,
  pub(super) sprite: SpriteVisual,
}

pub(super) fn shadow_bitmap(w: i32, h: i32, m: i32, dy: f32, radius: f32, k: i32) -> (u32, u32, Vec<u8>) {
  let (fw, fh) = (w + 2 * m, h + 2 * m);
  let (bw, bh) = (((fw / k).max(2)) as u32, ((fh / k).max(2)) as u32);
  let (hx, hy) = (w as f32 / 2.0, h as f32 / 2.0);
  let (cx, cy) = (fw as f32 / 2.0, fh as f32 / 2.0 + dy);
  let r = radius.min(hx).min(hy);
  let blur = 30.0f32;
  let peak = 0.26f32;
  let mut out = vec![0u8; (bw * bh * 4) as usize];
  for y in 0..bh {
    for x in 0..bw {
      let (px, py) = ((x as f32 + 0.5) * k as f32, (y as f32 + 0.5) * k as f32);
      let (qx, qy) = ((px - cx).abs() - (hx - r), (py - cy).abs() - (hy - r));
      let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
      let t = (d / blur).clamp(0.0, 1.0);
      let a = peak * (1.0 - t) * (1.0 - t) * (1.0 - t);
      let i = ((y * bw + x) * 4) as usize;
      out[i + 3] = (a * 255.0).round() as u8;
    }
  }
  (bw, bh, out)
}

impl Fx {
  pub(super) fn hide_shadow(&mut self) {
    self.shadow_target = 0;
    unsafe {
      let _ = KillTimer(Some(self.hwnd), T_SHADOW_WATCH);
    }
    if let Some(s) = &self.shadow {
      unsafe {
        let _ = ShowWindow(s.hwnd, SW_HIDE);
      }
    }
  }
  pub(super) fn watch_shadow(&mut self) {
    let t = self.shadow_target;
    if t == 0 {
      self.hide_shadow();
      return;
    }
    let fg = focus::foreground();
    if fg != t {
      if eligible(fg) {
        self.update_shadow(fg, true);
      } else {
        self.hide_shadow();
      }
      return;
    }
    let gone = unsafe { IsIconic(HWND(t as *mut _)).as_bool() } || enumerate::cloaked(t);
    if gone {
      self.hide_shadow();
      return;
    }
    if snapshot::frame_rect(t) != Some(self.shadow_rect) {
      self.update_shadow(t, true);
    }
  }
  pub(super) fn on_look_event(&mut self, event: u32, h: isize) {
    match event {
      tracker::EV_FOREGROUND => {
        if eligible(h) {
          self.update_shadow(h, true);
        } else {
          self.hide_shadow();
        }
      }
      tracker::EV_LOCATION if h == self.shadow_target => {
        self.update_shadow(h, false);
        unsafe {
          SetTimer(Some(self.hwnd), T_SHADOW, 120, None);
        }
      }
      tracker::EV_MOVESIZE_END if h == self.shadow_target => {
        self.update_shadow(h, true);
      }
      tracker::EV_MINIMIZE_START | tracker::EV_DESTROY if h == self.shadow_target => self.hide_shadow(),
      _ => {}
    }
  }
  pub(super) fn ensure_shadow_window(&mut self) -> windows_core::Result<()> {
    if self.shadow.is_some() {
      return Ok(());
    }
    unsafe {
      let hi: HINSTANCE = GetModuleHandleW(None)?.into();
      let class = w!("NexoraShadow");
      RegisterClassW(&WNDCLASSW {
        lpfnWndProc: Some(shadow_proc),
        hInstance: hi,
        lpszClassName: class,
        ..Default::default()
      });
      let hwnd = CreateWindowExW(
        WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED,
        class,
        w!("nexora-shadow"),
        WS_POPUP,
        0,
        0,
        100,
        100,
        None,
        None,
        Some(hi),
        None,
      )?;
      let _ = SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), 255, LWA_ALPHA);
      let surface = attach(&self.comp, hwnd, (100.0, 100.0))?;
      let sprite = self.comp.CreateSpriteVisual()?;
      surface.root.Children()?.InsertAtTop(&sprite)?;
      self.shadow = Some(ShadowWin { hwnd, surface, sprite });
    }
    Ok(())
  }
  pub(super) fn update_shadow(&mut self, h: isize, full: bool) {
    let p = prefs::get();
    if !p.win_shadow || h == 0 || !eligible(h) {
      self.hide_shadow();
      return;
    }
    unsafe {
      let hw = HWND(h as *mut _);
      if IsZoomed(hw).as_bool() || IsIconic(hw).as_bool() {
        self.hide_shadow();
        return;
      }
    }
    let Some((x, y, w, ht)) = snapshot::frame_rect(h) else {
      return;
    };
    if w < 80 || ht < 60 {
      self.hide_shadow();
      return;
    }
    if self.ensure_shadow_window().is_err() {
      return;
    }
    let scale = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() }.max(96) as f32 / 96.0;
    let (m, dy, radius) = ((SHADOW_MARGIN as f32 * scale) as i32, 10.0 * scale, 8.0 * scale);
    let (fw, fh) = (w + 2 * m, ht + 2 * m);
    let key = (fw, fh, radius as i32);
    let go = |this: &mut Fx| -> windows_core::Result<()> {
      let Some(sh) = &this.shadow else {
        return Ok(());
      };
      if full && key != this.shadow_key {
        let (bw, bh, data) = shadow_bitmap(w, ht, m, dy, radius, 4);
        let surface = this.gfx.surface_from_bgra(bw, bh, &data)?;
        let brush = this.comp.CreateSurfaceBrushWithSurface(&surface)?;
        brush.SetStretch(CompositionStretch::Fill)?;
        sh.sprite.SetBrush(&brush)?;
        sh.sprite.SetSize(Vector2 {
          X: fw as f32,
          Y: fh as f32,
        })?;
        sh.surface.root.SetSize(Vector2 {
          X: fw as f32,
          Y: fh as f32,
        })?;
        this.shadow_key = key;
      }
      unsafe {
        let below = GetWindow(HWND(h as *mut _), GW_HWNDNEXT).unwrap_or(HWND_BOTTOM);
        let _ = SetWindowPos(
          sh.hwnd,
          Some(below),
          x - m,
          y - m,
          fw,
          fh,
          SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
      }
      Ok(())
    };
    if go(self).is_ok() {
      self.shadow_target = h;
      self.shadow_rect = (x, y, w, ht);
      unsafe {
        SetTimer(Some(self.hwnd), T_SHADOW_WATCH, 250, None);
      }
    }
  }
  pub(super) fn hide_overlay(&mut self) {
    if let Some(old) = self.genie.take() {
      if let Ok(k) = self.surface.root.Children() {
        let _ = k.Remove(&old);
      }
    }
    if let Some(old) = self.sprite.take() {
      if let Ok(k) = self.surface.root.Children() {
        let _ = k.Remove(&old);
      }
    }
    unsafe {
      let _ = ShowWindow(self.hwnd, SW_HIDE);
    }
  }
}

pub(super) unsafe extern "system" fn shadow_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      WM_NCHITTEST => LRESULT(-1),
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
