use super::*;

pub(super) struct Opening {
  pub(super) hwnd: isize,
  pub(super) target: (i32, i32, i32, i32),
  pub(super) since: std::time::Instant,
  pub(super) playing: bool,
}

pub(super) fn open_file() -> std::path::PathBuf {
  crate::paths::config_dir().join("opening.txt")
}

pub(super) fn hold_parked(p: Option<Parked>) {
  let mut g = OPEN_PARKED.lock().unwrap_or_else(|e| e.into_inner());
  *g = p;
  match p {
    Some(p) => park::save(&open_file(), &[p]),
    None => {
      let _ = std::fs::remove_file(open_file());
    }
  }
}

pub(super) fn release_parked() {
  let p = OPEN_PARKED.lock().unwrap_or_else(|e| e.into_inner()).take();
  if let Some(p) = p {
    if park::is_parked(p.hwnd) {
      park::unpark(&p);
    }
  }
  let _ = std::fs::remove_file(open_file());
}

impl Fx {
  pub(super) fn begin_open(&mut self, h: isize) {
    if self.opened.contains(&h) || self.opening.as_ref().is_some_and(|o| o.hwnd == h) {
      return;
    }
    if !openable(h, self.screen) {
      return;
    }
    self.opened.insert(h);
    if self.opened.len() > 600 {
      self.opened.retain(|x| focus::is_alive(*x));
    }
    self.end_open();
    let Some(target) = snapshot::frame_rect(h) else {
      return;
    };
    flag(h);
    let Some(p) = park::park(h) else { return };
    if !park::is_parked(h) {
      park::unpark(&p);
      return;
    }
    hold_parked(Some(p));
    self.opening = Some(Opening {
      hwnd: h,
      target,
      since: std::time::Instant::now(),
      playing: false,
    });
    unsafe {
      SetTimer(Some(self.hwnd), T_OPEN_POLL, 16, None);
    }
  }
  pub(super) fn poll_open(&mut self) {
    let Some((h, target, waited, playing)) = self
      .opening
      .as_ref()
      .map(|o| (o.hwnd, o.target, o.since.elapsed().as_millis(), o.playing))
    else {
      unsafe {
        let _ = KillTimer(Some(self.hwnd), T_OPEN_POLL);
      }
      return;
    };
    if playing {
      return;
    }
    if !focus::is_alive(h) || !park::is_parked(h) {
      self.opening = None;
      hold_parked(None);
      unsafe {
        let _ = KillTimer(Some(self.hwnd), T_OPEN_POLL);
      }
      return;
    }
    let shot = snapshot::capture(h, 1).filter(|s| !crate::stage::blank(&s.bmp));
    match shot {
      Some(shot) => {
        unsafe {
          let _ = KillTimer(Some(self.hwnd), T_OPEN_POLL);
        }
        match self.gfx.surface_from_bgra(shot.bmp.w, shot.bmp.h, &shot.bmp.bgra) {
          Ok(surface) => {
            if let Some(o) = self.opening.as_mut() {
              o.playing = true;
            }
            self.play(surface.clone(), target, target, Kind::Open);
            self.snap = Some(Snap {
              hwnd: h,
              rect: target,
              surface,
            });
            unsafe {
              SetTimer(Some(self.hwnd), T_OPEN_END, START_DELAY + Kind::Open.ms() - 16, None);
            }
          }
          Err(_) => self.end_open(),
        }
      }
      None if waited > OPEN_WAIT_MS => self.end_open(),
      None => {}
    }
  }
  pub(super) fn end_open(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.hwnd), T_OPEN_POLL);
      let _ = KillTimer(Some(self.hwnd), T_OPEN_END);
    }
    if self.opening.take().is_some() {
      release_parked();
    }
  }
  pub(super) fn capture_current(&mut self) {
    let h = self.current;
    if h == 0 || !enabled() || !eligible(h) || focus::is_minimized(h) || park::is_parked(h) {
      return;
    }
    let Some(shot) = snapshot::capture(h, 1) else {
      return;
    };
    if let Ok(surface) = self.gfx.surface_from_bgra(shot.bmp.w, shot.bmp.h, &shot.bmp.bgra) {
      log::line(
        "fx",
        &format!(
          "tangkap hwnd={h} {}x{} (px {}x{})",
          shot.rect.2, shot.rect.3, shot.bmp.w, shot.bmp.h
        ),
      );
      self.snap = Some(Snap {
        hwnd: h,
        rect: shot.rect,
        surface,
      });
    }
  }
  pub(super) fn icon_rect(&self, hwnd: isize) -> (i32, i32, i32, i32) {
    crate::dock::icon_rect_for(hwnd).unwrap_or((self.screen.0 / 2 - 24, self.screen.1 - 70, 48, 48))
  }
  pub(super) fn on_event(&mut self, event: u32, h: isize) {
    self.on_look_event(event, h);
    if !enabled() {
      return;
    }
    match event {
      tracker::EV_SHOW | tracker::EV_UNCLOAKED => self.begin_open(h),
      tracker::EV_FOREGROUND => {
        if eligible(h) {
          flag(h);
          self.seen.insert(h);
          self.current = h;
          unsafe {
            SetTimer(Some(self.hwnd), T_CAPTURE, 350, None);
          }
        }
      }
      tracker::EV_MOVESIZE_END if h == self.current => unsafe {
        SetTimer(Some(self.hwnd), T_CAPTURE, 350, None);
      },
      tracker::EV_MINIMIZE_START => {
        if let Some(s) = &self.snap {
          if s.hwnd == h && self.pending_restore.is_none() {
            let (surface, rect) = (s.surface.clone(), s.rect);
            crate::dock::reveal_for(START_DELAY + Kind::Minimize.ms() + 450);
            let icon = self.icon_rect(h);
            let keep = s.clone();
            self.play(surface, rect, icon, Kind::Minimize);
            self.parked.retain(|p| p.hwnd != h);
            self.parked.push(keep);
            if self.parked.len() > 2 {
              self.parked.remove(0);
            }
          }
        }
      }
      tracker::EV_DESTROY => {
        self.parked.retain(|p| p.hwnd != h);
        self.opened.remove(&h);
        if self.opening.as_ref().is_some_and(|o| o.hwnd == h) {
          self.opening = None;
          hold_parked(None);
        }
        if let Some(s) = &self.snap {
          if s.hwnd == h && !focus::is_alive(h) {
            let (surface, rect) = (s.surface.clone(), s.rect);
            self.play(surface, rect, rect, Kind::Close);
            self.snap = None;
            if self.current == h {
              self.current = 0;
            }
          }
        }
      }
      _ => {}
    }
  }
}
