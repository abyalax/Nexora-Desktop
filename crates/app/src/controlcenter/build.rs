use super::*;

impl Cc {
  pub(super) fn open(&mut self, top: i32) {
    self.top = top;
    self.edit = false;
    if self.gfx.is_none() {
      self.gfx = Gfx::shared(&self.comp).ok();
    }
    let ctl = self.ctl.0 as isize;
    if self.radios.is_none() {
      self.radios = Some(radio::spawn(move || unsafe {
        let _ = PostMessageW(Some(HWND(ctl as *mut _)), WM_RADIO, WPARAM(0), LPARAM(0));
      }));
    }
    if self.media.is_none() && self.layout.contains(&Kind::NowPlaying) {
      self.media = Some(media::spawn(move || unsafe {
        let _ = PostMessageW(Some(HWND(ctl as *mut _)), WM_MEDIA, WPARAM(0), LPARAM(0));
      }));
    }
    self.ssid = wifi::ssid();
    self.intro = true;
    self.build();
    self.intro = false;
    if let Some(p) = &self.panel {
      let _ = p.popup.root().SetOpacity(0.0);
      p.popup.show();
    }
    unsafe {
      SetTimer(Some(self.ctl), T_INTRO, 60, None);
    }
  }
  pub(super) fn start_intro(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_INTRO);
    }
    let items = std::mem::take(&mut self.intro_items);
    if self.closing {
      return;
    }
    let Some(p) = self.panel.as_ref() else { return };
    let _ = motion::pop_in(
      &self.comp,
      p.popup.root(),
      Vector3 {
        X: pad() + p.w - 24.0 * p.dpi,
        Y: pad(),
        Z: 0.0,
      },
      0.45,
      1.025,
      0,
      motion::CC_OPEN_MS,
    );
    for (v, center, delay) in items {
      let _ = motion::pop_in(&self.comp, &v, center, 0.7, 1.04, delay, 360);
    }
  }
  pub(super) fn close(&mut self) {
    if self.closing || self.panel.is_none() {
      return;
    }
    nexora_platform::input::end_dismiss();
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_INTRO);
    }
    self.intro_items.clear();
    self.edit = false;
    if let Some(g) = self.gallery.take() {
      let h = g.popup.hwnd;
      drop(g);
      unsafe {
        let _ = DestroyWindow(h);
      }
    }
    let Some(p) = self.panel.as_ref() else { return };
    if motion::pop_out(
      &self.comp,
      p.popup.root(),
      Vector3 {
        X: pad() + p.w - 24.0 * p.dpi,
        Y: pad(),
        Z: 0.0,
      },
      0.55,
      motion::CC_CLOSE_MS,
    )
    .is_ok()
    {
      self.closing = true;
      unsafe {
        SetTimer(Some(self.ctl), T_CLOSE, motion::CC_CLOSE_MS + 20, None);
      }
    } else {
      self.finish_close();
    }
  }
  pub(super) fn finish_close(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_CLOSE);
    }
    self.closing = false;
    self.edit = false;
    if let Some(p) = self.panel.take() {
      let h = p.popup.hwnd;
      drop(p);
      unsafe {
        let _ = DestroyWindow(h);
      }
      self.cache.clear();
      nexora_platform::sys::trim_later(5000);
    }
  }
  pub(super) fn build(&mut self) {
    let old = self.panel.take();
    let dpi;
    let popup = match old {
      Some(mut p) => {
        if let Ok(k) = p.popup.content.Children() {
          let _ = k.RemoveAll();
        }
        p.panels.clear();
        dpi = p.dpi;
        p.popup
      }
      None => {
        let base = GlassParams::dark((10.0, 10.0), 0.0);
        let Ok(popup) = Popup::new(
          &self.comp,
          w!("NexoraControlCenter"),
          Some(panel_proc),
          WINDOW_EX_STYLE(0),
          base,
        ) else {
          return;
        };
        dpi = unsafe { GetDpiForWindow(popup.hwnd) }.max(96) as f32 / 96.0 * prefs::ui_scale();
        popup
      }
    };
    let mut popup = popup;
    let (cell, gap) = (82.0 * dpi, 14.0 * dpi);
    let sizes = self.sizes.clone();
    let (placed, rows) = controls::layout_sized(&self.layout, &|k| *sizes.get(&k).unwrap_or(&k.default_size()));
    let rows = if self.edit { rows + 2 } else { rows };
    let (gw, gh) = (
      4.0 * cell + 3.0 * gap,
      rows as f32 * cell + (rows.max(1) - 1) as f32 * gap,
    );
    let btn_h = 34.0 * dpi;
    let inuse = if self.edit {
      Vec::new()
    } else {
      nexora_platform::privacy::active()
    };
    let top = if inuse.is_empty() { 0.0 } else { 36.0 * dpi };
    let (pw, ph) = (gw, top + gh + 14.0 * dpi + btn_h + 4.0 * dpi);
    let (sw, _) = nexora_platform::sys::screen_size();
    let x = (sw as f32 - pw - 10.0 * dpi) as i32;
    let y = self.top + (6.0 * dpi) as i32;
    let m = (8.0 * dpi).round();
    PAD.store(m.to_bits(), std::sync::atomic::Ordering::Relaxed);
    let _ = popup.set_rect(
      x - m as i32,
      y - m as i32,
      (pw + 2.0 * m).ceil() as i32,
      (ph + 2.0 * m).ceil() as i32,
    );
    let _ = popup.panel.set_rect(0.0, 0.0, 0.0, 0.0);
    let _ = popup.content.SetOffset(Vector3 { X: m, Y: m, Z: 0.0 });
    let content = popup.content.clone();

    let mut mods = Vec::new();
    let mut panels = Vec::new();
    let mut groups: Vec<(ContainerVisual, Rect)> = Vec::new();
    let mut occupied: Vec<(usize, usize)> = Vec::new();
    for p in &placed {
      let r: Rect = grid_rect(p, cell, gap, top);
      for dy in 0..p.h {
        for dx in 0..p.w {
          occupied.push((p.col + dx, p.row + dy));
        }
      }
      let radius = if p.w == 1 && p.h == 1 {
        r.2 / 2.0
      } else if p.h >= 2 || p.w >= 3 {
        CARD_R * cell
      } else {
        r.3 / 2.0
      };
      let mc = self.comp.CreateContainerVisual().ok();
      if let Some(mc) = &mc {
        let _ = content.Children().and_then(|k| k.InsertAtTop(mc));
        groups.push((mc.clone(), r));
      }
      let parent = mc.as_ref().unwrap_or(&content);
      if let Some(g) = self.glass(parent, r, radius) {
        panels.push(g);
      }
      let mut m = self.draw_module(parent, p, r, dpi, radius);
      m.cont = mc.clone();
      mods.push(m);
    }
    let mut slots = Vec::new();
    if self.edit {
      for row in 0..rows {
        for col in 0..4 {
          if occupied.contains(&(col, row)) {
            continue;
          }
          let r: Rect = (col as f32 * (cell + gap), row as f32 * (cell + gap), cell, cell);
          let _ = self.rrect(
            &content,
            r,
            cell / 2.0,
            Color {
              A: 40,
              R: 255,
              G: 255,
              B: 255,
            },
          );
          slots.push(r);
        }
      }
    }
    let label = if self.edit {
      prefs::tr("Selesai")
    } else {
      prefs::tr("Edit Kontrol…")
    };
    let bw = 128.0 * dpi;
    let bx = (pw - bw) / 2.0;
    let edit_btn: Rect = (bx, top + gh + 14.0 * dpi, bw, btn_h);
    let ec = self.comp.CreateContainerVisual().ok();
    if let Some(ec) = &ec {
      let _ = content.Children().and_then(|k| k.InsertAtTop(ec));
      groups.push((ec.clone(), edit_btn));
    }
    let eparent = ec.as_ref().unwrap_or(&content);
    if let Some(g) = self.glass(eparent, edit_btn, btn_h / 2.0) {
      panels.push(g);
    }
    let (lw, _) = measure(
      label,
      &Style {
        px: (13.0 * dpi) as i32,
        weight: 600,
      },
    );
    self.text(
      eparent,
      label,
      (13.0 * dpi) as i32,
      600,
      WHITE,
      bx + ((bw - lw as f32) / 2.0).max(8.0 * dpi),
      edit_btn.1 + 8.0 * dpi,
      bw - 16.0 * dpi,
      1.0,
    );
    let chip = self.draw_chip(&content, &inuse, pw, dpi, &mut groups, &mut panels);
    self.intro_items.clear();
    if self.intro {
      let icon = (pw - 24.0 * dpi, 0.0f32);
      let mut order: Vec<usize> = (0..groups.len()).collect();
      let dist = |r: &Rect| ((r.0 + r.2 / 2.0 - icon.0).powi(2) + (r.1 + r.3 / 2.0 - icon.1).powi(2)).sqrt();
      order.sort_by(|a, b| dist(&groups[*a].1).total_cmp(&dist(&groups[*b].1)));
      for (rank, i) in order.into_iter().enumerate() {
        let (v, r) = &groups[i];
        let center = Vector3 {
          X: r.0 + r.2 / 2.0,
          Y: r.1 + r.3 / 2.0,
          Z: 0.0,
        };
        let _ = v.SetOpacity(0.0);
        self
          .intro_items
          .push((v.clone(), center, 40 + rank as u32 * motion::CC_STAGGER_MS));
      }
    }

    let hwnd = popup.hwnd;
    let rect = popup.rect;
    self.panel = Some(Panel {
      popup,
      dpi,
      cell,
      gap,
      mods,
      slots,
      edit_btn,
      panels,
      drag: None,
      slider: None,
      w: pw,
      chip,
      hover: None,
      preview: Vec::new(),
      shift: HashMap::new(),
      lift: (0.0, 0.0),
      resize: None,
      ghost: None,
    });
    if self.edit {
      nexora_platform::input::end_dismiss();
    } else {
      nexora_platform::input::begin_dismiss(
        (rect.0, rect.1, rect.0 + rect.2, rect.1 + rect.3),
        hwnd.0 as isize,
        WM_CLOSE_CC,
      );
    }
  }
}
