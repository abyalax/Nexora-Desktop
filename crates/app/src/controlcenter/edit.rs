use super::*;

impl Cc {
  pub(super) fn origins(&self) -> HashMap<Kind, (f32, f32)> {
    let Some(p) = self.panel.as_ref() else {
      return HashMap::new();
    };
    let dragged = p.drag.map(|(i, _, _)| p.mods[i].placed.kind);
    p.mods
      .iter()
      .map(|m| {
        let k = m.placed.kind;
        let (dx, dy) = if Some(k) == dragged {
          p.lift
        } else {
          p.shift.get(&k).copied().unwrap_or((0.0, 0.0))
        };
        (k, (m.rect.0 + dx, m.rect.1 + dy))
      })
      .collect()
  }
  pub(super) fn build_flip(&mut self, from: HashMap<Kind, (f32, f32)>, pop: Option<Kind>, lifted: Option<Kind>) {
    self.build();
    let comp = self.comp.clone();
    let Some(p) = self.panel.as_ref() else { return };
    for m in &p.mods {
      let Some(c) = &m.cont else { continue };
      let k = m.placed.kind;
      let center = center_of(m.rect);
      if Some(k) == pop || (!from.is_empty() && !from.contains_key(&k)) {
        let _ = motion::pop_in(&comp, c, center, 0.5, 1.06, 0, 360);
        continue;
      }
      if let Some(o) = from.get(&k) {
        let (dx, dy) = (o.0 - m.rect.0, o.1 - m.rect.1);
        if dx.abs() > 0.5 || dy.abs() > 0.5 {
          let _ = motion::slide_pop(
            &comp,
            c,
            Vector3 { X: dx, Y: dy, Z: 0.0 },
            Vector3 { X: 0.0, Y: 0.0, Z: 0.0 },
          );
        }
      }
      if Some(k) == lifted {
        let _ = c.SetCenterPoint(center);
        let _ = c.SetScale(Vector3 {
          X: 1.08,
          Y: 1.08,
          Z: 1.0,
        });
        let _ = motion::hover_pop(&comp, c, center, 1.0);
      }
    }
  }
  pub(super) fn pop_all(&self) {
    let Some(p) = self.panel.as_ref() else { return };
    for (rank, m) in p.mods.iter().enumerate() {
      if let Some(c) = &m.cont {
        let _ = c.SetOpacity(0.0);
        let _ = motion::pop_in(&self.comp, c, center_of(m.rect), 0.86, 1.05, rank as u32 * 14, 340);
      }
    }
  }
  pub(super) fn start_drag(&mut self, i: usize, x: f32, y: f32) {
    let comp = self.comp.clone();
    let layout = self.layout.clone();
    let Some(p) = self.panel.as_mut() else { return };
    p.drag = Some((i, x, y));
    p.lift = (0.0, 0.0);
    p.preview = layout;
    p.shift.clear();
    let content = p.popup.content.clone();
    if let Some(m) = p.mods.get(i) {
      if let Some(c) = &m.cont {
        if let Ok(k) = content.Children() {
          let _ = k.Remove(c);
          let _ = k.InsertAtTop(c);
        }
        let _ = motion::hover_pop(&comp, c, center_of(m.rect), 1.08);
      }
    }
  }
  pub(super) fn drag_move(&mut self, x: f32, y: f32) {
    let comp = self.comp.clone();
    let sizes = self.sizes.clone();
    let size_of = |k: Kind| *sizes.get(&k).unwrap_or(&k.default_size());
    let Some(p) = self.panel.as_mut() else { return };
    let Some((i, x0, y0)) = p.drag else { return };
    let Some(kind) = p.mods.get(i).map(|m| m.placed.kind) else {
      return;
    };
    p.lift = (x - x0, y - y0);
    if let Some(c) = &p.mods[i].cont {
      if let Ok(o) = c.cast::<windows::UI::Composition::CompositionObject>() {
        let _ = o.StopAnimation(&windows_core::HSTRING::from("Offset"));
      }
      let _ = c.SetOffset(Vector3 {
        X: p.lift.0,
        Y: p.lift.1,
        Z: 0.0,
      });
    }
    let (cell, gap) = (p.cell, p.gap);
    let (placed, _) = controls::layout_sized(&p.preview, &size_of);
    let Some(over) = placed
      .iter()
      .filter(|q| q.kind != kind)
      .find(|q| hit(grid_rect(q, cell, gap, 0.0), x, y))
      .map(|q| q.kind)
    else {
      return;
    };
    let (Some(a), Some(b)) = (
      p.preview.iter().position(|k| *k == kind),
      p.preview.iter().position(|k| *k == over),
    ) else {
      return;
    };
    let k = p.preview.remove(a);
    p.preview.insert(b, k);
    let (placed, _) = controls::layout_sized(&p.preview, &size_of);
    for m in &p.mods {
      let k = m.placed.kind;
      if k == kind {
        continue;
      }
      let Some(q) = placed.iter().find(|q| q.kind == k) else {
        continue;
      };
      let r = grid_rect(q, cell, gap, 0.0);
      let to = (r.0 - m.rect.0, r.1 - m.rect.1);
      let cur = p.shift.get(&k).copied().unwrap_or((0.0, 0.0));
      if (to.0 - cur.0).abs() > 0.5 || (to.1 - cur.1).abs() > 0.5 {
        if let Some(c) = &m.cont {
          let _ = motion::slide_pop(
            &comp,
            c,
            Vector3 {
              X: cur.0,
              Y: cur.1,
              Z: 0.0,
            },
            Vector3 {
              X: to.0,
              Y: to.1,
              Z: 0.0,
            },
          );
        }
        p.shift.insert(k, to);
      }
    }
  }
  pub(super) fn end_drag(&mut self) {
    let from = self.origins();
    let Some(p) = self.panel.as_mut() else { return };
    let Some((i, _, _)) = p.drag.take() else {
      return;
    };
    let kind = p.mods[i].placed.kind;
    let preview = std::mem::take(&mut p.preview);
    p.shift.clear();
    if !preview.is_empty() && preview != self.layout {
      self.layout = preview;
      self.save_layout();
    }
    self.build_flip(from, None, Some(kind));
  }
  pub(super) fn finish_remove(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_REMOVE);
    }
    let Some(kind) = self.pending_remove.take() else {
      return;
    };
    let from = self.origins();
    self.layout.retain(|k| *k != kind);
    self.build_flip(from, None, None);
    self.refresh_gallery();
  }
  pub(super) fn set_gallery_hover(&mut self, i: Option<usize>) {
    let comp = self.comp.clone();
    let Some(g) = self.gallery.as_mut() else {
      return;
    };
    if g.hover == i {
      return;
    }
    for (k, on) in [(g.hover, false), (i, true)] {
      if let Some((r, c, to)) = k.and_then(|k| g.conts.get(k)) {
        let _ = motion::hover_pop(&comp, c, center_of(*r), if on { *to } else { 1.0 });
      }
    }
    g.hover = i;
  }
  pub(super) fn open_picker(&mut self, i: usize) {
    let Some(p) = self.panel.as_ref() else { return };
    let Some(m) = p.mods.get(i) else { return };
    let which = match m.placed.kind {
      Kind::Wifi => crate::netmenu::Which::Wifi,
      Kind::Bluetooth => crate::netmenu::Which::Bluetooth,
      _ => return,
    };
    let (rect, dpi) = (p.popup.rect, p.dpi);
    self.close();
    let m = pad() as i32;
    crate::netmenu::show(
      &self.comp,
      which,
      crate::netmenu::Anchor::Corner(rect.0 + rect.2 - m, rect.1 + m),
      dpi,
    );
  }
  pub(super) fn click(&mut self, i: usize, x: f32, y: f32) {
    let Some(p) = self.panel.as_ref() else { return };
    let m = &p.mods[i];
    let kind = m.placed.kind;
    let btn = m.btns.iter().find(|(r, _)| hit(*r, x, y)).map(|(_, b)| *b);
    let send = |cmd: RadioCmd, s: &Cc| {
      if let Some(r) = &s.radios {
        let _ = r.commands.send(cmd);
      }
    };
    let small = m.placed.w == 1 && m.placed.h == 1;
    match kind {
      Kind::Wifi | Kind::Bluetooth if !small && btn != Some(BTN_DISC) => self.open_picker(i),
      Kind::Wifi => send(RadioCmd::Set(RK::Wifi, self.wifi != Some(true)), self),
      Kind::Bluetooth => send(RadioCmd::Set(RK::Bluetooth, self.bt != Some(true)), self),
      Kind::NowPlaying => {
        if let (Some(b), Some(md)) = (btn, &self.media) {
          let _ = md.commands.send(match b {
            0 => Command::Prev,
            1 => Command::PlayPause,
            _ => Command::Next,
          });
        }
      }
      Kind::Stage => {
        prefs::update(|s| s.stage = !s.stage);
        self.build();
      }
      Kind::DarkMode => {
        prefs::update(|s| {
          s.theme = if s.theme == nexora_core::settings::Theme::Dark {
            nexora_core::settings::Theme::Light
          } else {
            nexora_core::settings::Theme::Dark
          }
        });
        self.build();
      }
      Kind::Display | Kind::Sound => {
        if btn == Some(10) {
          self.close();
          if kind == Kind::Sound {
            keys::chord(&[0x5B, 0x11, 0x56]);
          } else {
            launch::open("ms-settings:display");
          }
        }
      }
      other => {
        self.close();
        match other {
          Kind::Hotspot => launch::open("ms-settings:network-mobilehotspot"),
          Kind::Share => launch::open("ms-settings:crossdevice"),
          Kind::Airplane => launch::open("ms-settings:network-airplanemode"),
          Kind::Mirror => keys::chord(&[keys::WIN, 0x4B]),
          Kind::Focus => launch::open("ms-settings:notifications"),
          Kind::Screenshot => keys::chord(&[keys::WIN, keys::SHIFT, 0x53]),
          Kind::Battery => launch::open("ms-settings:batterysaver"),
          Kind::Lock => launch::run("rundll32.exe", "user32.dll,LockWorkStation"),
          Kind::Mission => crate::mission::toggle(),
          Kind::Apps => crate::launchpad::toggle(),
          Kind::Calculator => launch::run("calc.exe", ""),
          Kind::Timer => launch::open("ms-clock:"),
          _ => {}
        }
      }
    }
  }
  pub(super) fn enter_edit(&mut self) {
    self.edit = true;
    self.build();
    self.pop_all();
    self.open_gallery();
  }
  pub(super) fn leave_edit(&mut self) {
    self.save_layout();
    self.edit = false;
    if let Some(g) = self.gallery.take() {
      let h = g.popup.hwnd;
      drop(g);
      unsafe {
        let _ = DestroyWindow(h);
      }
    }
    self.build();
    self.pop_all();
  }
  pub(super) fn remove(&mut self, kind: Kind) {
    if self.pending_remove.is_some() {
      self.finish_remove();
    }
    if let Some(m) = self
      .panel
      .as_ref()
      .and_then(|p| p.mods.iter().find(|m| m.placed.kind == kind))
    {
      if let Some(c) = &m.cont {
        let _ = motion::pop_out(&self.comp, c, center_of(m.rect), 0.4, 180);
      }
    }
    self.pending_remove = Some(kind);
    unsafe {
      SetTimer(Some(self.ctl), T_REMOVE, 190, None);
    }
  }
  pub(super) fn add(&mut self, kind: Kind) {
    if !self.layout.contains(&kind) {
      let from = self.origins();
      self.layout.push(kind);
      if kind == Kind::NowPlaying && self.media.is_none() {
        let ctl = self.ctl.0 as isize;
        self.media = Some(media::spawn(move || unsafe {
          let _ = PostMessageW(Some(HWND(ctl as *mut _)), WM_MEDIA, WPARAM(0), LPARAM(0));
        }));
      }
      self.build_flip(from, Some(kind), None);
      self.refresh_gallery();
    }
  }
  #[allow(dead_code)]
  pub(super) fn move_to(&mut self, from: Kind, over: Kind) {
    let (Some(a), Some(b)) = (
      self.layout.iter().position(|k| *k == from),
      self.layout.iter().position(|k| *k == over),
    ) else {
      return;
    };
    if a == b {
      return;
    }
    let k = self.layout.remove(a);
    self.layout.insert(b, k);
    self.build();
  }
}
