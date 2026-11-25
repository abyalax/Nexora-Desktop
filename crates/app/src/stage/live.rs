use super::*;

impl St {
  pub(super) fn sync_live(&mut self) {
    if !capture::supported() {
      return;
    }
    let wanted: Vec<String> = self
      .model
      .strip
      .iter()
      .flat_map(|g| g.iter().take(3).cloned())
      .collect();
    self.live.retain(|a, _| wanted.contains(a));
    if wanted.is_empty() {
      return;
    }
    if self.live_gfx.is_none() {
      self.live_gfx = Gfx::new_hardware(&self.comp)
        .ok()
        .or_else(|| Gfx::shared(&self.comp).ok());
    }
    let Some(lg) = self.live_gfx.as_ref() else {
      return;
    };
    for app in wanted {
      if self.live.contains_key(&app) {
        continue;
      }
      let Some(hwnd) = self.parked.iter().find(|(_, a)| *a == app).map(|(p, _)| p.hwnd) else {
        continue;
      };
      match Capture::start(lg, hwnd) {
        Ok(c) => {
          self.live.insert(app, c);
        }
        Err(e) => log::line("stage", &format!("tangkapan hidup gagal untuk {app}: {e}")),
      }
    }
    self.apply_live_brushes();
  }
  pub(super) fn apply_live_brushes(&self) {
    let Some(strip) = self.strip.as_ref() else {
      return;
    };
    for sl in &strip.slots {
      for (a, br) in &sl.brushes {
        if let Some(c) = self.live.get(a) {
          if c.has_frame {
            let _ = br.SetSurface(&c.surface);
          }
        }
      }
    }
  }
  pub(super) fn poll_live(&mut self) {
    if self.live.is_empty() || !self.strip.as_ref().is_some_and(|s| s.shown) {
      return;
    }
    let Some(lg) = self.live_gfx.as_ref() else {
      return;
    };
    let mut first = false;
    for c in self.live.values_mut() {
      let had = c.has_frame;
      if c.poll(lg) && !had {
        first = true;
      }
    }
    if first {
      self.rebuild();
    }
  }
  pub(super) fn check_overlap(&mut self) {
    let fg = focus::foreground();
    if fg == 0 || is_own(fg) || is_desktop(fg) {
      return;
    }
    let Some(strip) = self.strip.as_ref() else {
      return;
    };
    let mut r = RECT::default();
    unsafe {
      if GetWindowRect(HWND(fg as *mut _), &mut r).is_err() {
        return;
      }
    }
    let overlap = if strip.right {
      (r.right as f32) > nexora_platform::sys::screen_size().0 as f32 - strip.w + 6.0 * strip.dpi
    } else {
      (r.left as f32) < strip.w - 6.0 * strip.dpi
    };
    let want_shown = !overlap || self.peek;
    self.set_strip_shown(want_shown);
  }
  pub(super) fn refresh_shots(&mut self) {
    if self.strip.is_none() || self.fly.is_some() {
      return;
    }
    let mut need_rebuild = false;
    let apps: Vec<String> = self
      .model
      .strip
      .iter()
      .flat_map(|g| g.iter().take(3).cloned())
      .collect();
    for app in apps {
      let Some(hwnd) = self.parked.iter().find(|(_, a)| *a == app).map(|(p, _)| p.hwnd) else {
        continue;
      };
      let Some(shot) = snapshot::capture(hwnd, 2) else {
        continue;
      };
      if blank(&shot.bmp) || self.live.get(&app).is_some_and(|c| c.has_frame) {
        continue;
      }
      let old = self.shots.get(&app).map(|b| (b.w, b.h));
      let same_shape = old
        .map(|(w, h)| ((w as f32 / h.max(1) as f32) - (shot.bmp.w as f32 / shot.bmp.h.max(1) as f32)).abs() < 0.02)
        .unwrap_or(false);
      let Some(gfx) = self.gfx.as_ref() else {
        continue;
      };
      let mut done = false;
      if same_shape {
        if let (Some(strip), Ok(surf)) = (
          self.strip.as_ref(),
          gfx.surface_from_bgra(shot.bmp.w, shot.bmp.h, &shot.bmp.bgra),
        ) {
          for sl in &strip.slots {
            for (a, br) in &sl.brushes {
              if *a == app {
                let _ = br.SetSurface(&surf);
                done = true;
              }
            }
          }
        }
      }
      if !done {
        need_rebuild = true;
      }
      self.shots.insert(app, shot.bmp);
    }
    if need_rebuild {
      self.rebuild();
    }
  }
}
