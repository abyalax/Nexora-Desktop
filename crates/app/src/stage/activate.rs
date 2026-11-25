use super::*;

impl St {
  pub(super) fn activate(&mut self) {
    self.active = true;
    let p = prefs::get();
    self.model = Stage {
      max: p.stage_count as usize,
      ..Default::default()
    };
    self.cfg = (
      p.stage_right,
      p.stage_size,
      p.stage_gap,
      p.stage_count,
      p.stage_titles as u8,
    );
    log::line("stage", "aktif");
    self.refresh();
    self.apply_icons();
    unsafe {
      SetTimer(Some(self.ctl), T_SHOT, 3000, None);
      SetTimer(Some(self.ctl), T_LIVE, 50, None);
    }
  }
  pub(super) fn deactivate(&mut self) {
    self.refused.clear();
    if !self.active && self.parked.is_empty() {
      return;
    }
    self.active = false;
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_SHOT);
      let _ = KillTimer(Some(self.ctl), T_LIVE);
    }
    self.live.clear();
    self.live_gfx = None;
    self.finish_fly();
    self.hold.clear();
    self.kill_dying();
    for (p, _) in self.parked.drain(..) {
      park::unpark(&p);
    }
    for ids in take_paused(None) {
      nexora_platform::media::resume_ids(ids);
    }
    let _ = std::fs::remove_file(parked_file());
    self.destroy_strip();
    self.shots.clear();
    self.model = Stage::default();
    log::line("stage", "nonaktif");
  }
  pub(super) fn apply_icons(&self) {
    nexora_platform::desktopicons::set_visible(icons_visible(self.active, self.model.current.is_empty()));
  }
  pub(super) fn destroy_strip(&mut self) {
    self.edge = None;
    if let Some(s) = self.strip.take() {
      let h = s.popup.hwnd;
      drop(s);
      unsafe {
        let _ = DestroyWindow(h);
      }
    }
  }
  pub(super) fn save(&self) {
    let list: Vec<Parked> = self.parked.iter().map(|(p, _)| *p).collect();
    park::save(&parked_file(), &list);
  }
  pub(super) fn refresh(&mut self) {
    let now = std::time::Instant::now();
    if self.fly.is_some() || now < self.busy_until {
      let wait = self.busy_until.saturating_duration_since(now).as_millis() as u32 + 60;
      unsafe {
        SetTimer(Some(self.ctl), T_FG, wait.max(80), None);
      }
      return;
    }
    let fg = focus::foreground();
    if fg == 0 || is_own(fg) {
      return;
    }
    if self.want_fg != 0 {
      let w = std::mem::take(&mut self.want_fg);
      if fg != w && unsafe { IsWindow(Some(HWND(w as *mut _))).as_bool() } {
        focus::force_foreground(w);
        self.busy_until = std::time::Instant::now() + std::time::Duration::from_millis(350);
        unsafe {
          SetTimer(Some(self.ctl), T_FG, 420, None);
        }
        return;
      }
    }
    let wins = enumerate::list(std::process::id());
    let running = running_apps(&wins, &self.refused);
    self.model.prune(&running);
    let ch = if is_desktop(fg) {
      if !prefs::get().stage_desktop_click {
        return;
      }
      self.model.desktop()
    } else {
      let Some(w) = wins.iter().find(|w| w.hwnd == fg) else {
        return;
      };
      let app = key_of(&w.path);
      if app.is_empty() || excluded(&app) || self.refused.contains(&app) {
        return;
      }
      self.model.focus(&app, &running)
    };
    self.apply(ch, &wins, 0);
  }
  pub(super) fn focus_app(&mut self, app: &str, _from: isize) {
    let wins = enumerate::list(std::process::id());
    let running = running_apps(&wins, &self.refused);
    let target = self
      .parked
      .iter()
      .find(|(_, a)| a == app)
      .map(|(p, _)| p.hwnd)
      .or_else(|| wins.iter().find(|w| key_of(&w.path) == app).map(|w| w.hwnd))
      .unwrap_or(0);
    let ch = self.model.focus(app, &running);
    self.apply(ch, &wins, target);
  }
  pub(super) fn slot_rect_of(&self, app: &str) -> Option<Rect> {
    let s = self.strip.as_ref()?;
    let base = s.popup.rect;
    let (i, sl) = s
      .slots
      .iter()
      .enumerate()
      .find(|(_, sl)| sl.apps.iter().any(|a| a == app))?;
    let k = if s.hover == Some(i) { 1.06 } else { 1.0 };
    let (cx, cy) = (sl.rect.0 + sl.rect.2 / 2.0, sl.rect.1 + sl.rect.3 / 2.0);
    let (x, y, w, h) = sl.card;
    Some((
      base.0 as f32 + cx + (x - cx) * k,
      base.1 as f32 + cy + (y - cy) * k,
      w * k,
      h * k,
    ))
  }
  pub(super) fn reveal_hold(&mut self) {
    let hold = std::mem::take(&mut self.hold);
    let comp = self.comp.clone();
    let Some(s) = self.strip.as_ref() else { return };
    for sl in s.slots.iter().filter(|sl| sl.apps.iter().any(|a| hold.contains(a))) {
      let _ = motion::scalar(&comp, &sl.cont, "Opacity", 0.0, 1.0, 150);
      let _ = sl.cont.SetOpacity(1.0);
    }
  }
  pub(super) fn apply(&mut self, ch: Change, wins: &[nexora_core::dock::items::Win], activate: isize) {
    self.finish_fly();
    if activate != 0 {
      self.want_fg = activate;
    }
    self.busy_until = std::time::Instant::now()
      + std::time::Duration::from_millis(motion::stage_ms(motion::STAGE_SWAP_MS) as u64 + 350);
    let mut items: Vec<(Bitmap, Rect, Rect)> = Vec::new();

    let show_from: Option<(String, Rect)> = ch
      .show
      .iter()
      .find_map(|a| self.slot_rect_of(a).map(|r| (a.clone(), r)));

    let mut outs: Vec<(String, Rect)> = Vec::new();
    let mut refused_now = false;
    let media_on = prefs::get().stage_media;
    for app in &ch.show {
      for ids in take_paused(Some(app)) {
        nexora_platform::media::resume_ids(ids);
      }
    }
    for app in &ch.show {
      if let Some((p, _)) = self.parked.iter().find(|(_, a)| a == app) {
        if let Some(shot) = snapshot::capture(p.hwnd, 2) {
          if !blank(&shot.bmp) || !self.shots.contains_key(app) {
            self.shots.insert(app.clone(), shot.bmp);
          }
        }
      }
    }
    for app in &ch.park {
      if excluded(app) {
        continue;
      }
      if media_on {
        let stem = std::path::Path::new(app)
          .file_stem()
          .map(|x| x.to_string_lossy().to_string())
          .unwrap_or_default();
        if !stem.is_empty() {
          let key = app.clone();
          nexora_platform::media::pause_app(stem, move |ids| {
            if !ids.is_empty() {
              if let Ok(mut g) = PAUSED.lock() {
                g.push((key, ids));
              }
            }
          });
        }
      }
      for w in wins.iter().filter(|w| key_of(&w.path) == *app) {
        if focus::is_minimized(w.hwnd) || park::is_parked(w.hwnd) || self.parked.iter().any(|(p, _)| p.hwnd == w.hwnd) {
          continue;
        }
        if let Some(shot) = snapshot::capture(w.hwnd, 2) {
          let first_of_app = !outs.iter().any(|(a, _)| a == app);
          let good = !blank(&shot.bmp);
          if first_of_app && good && outs.len() < 4 {
            outs.push((
              app.clone(),
              (
                shot.rect.0 as f32,
                shot.rect.1 as f32,
                shot.rect.2 as f32,
                shot.rect.3 as f32,
              ),
            ));
          }
          if (first_of_app && good) || !self.shots.contains_key(app) {
            self.shots.insert(app.clone(), shot.bmp);
          }
        }
        if let Some(p) = park::park(w.hwnd) {
          if park::is_parked(w.hwnd) {
            self.parked.push((p, app.clone()));
          } else {
            park::unpark(&p);
            if !self.refused.contains(app) {
              log::line("stage", &format!("{app} menolak diparkir; tidak dikelola"));
              self.refused.push(app.clone());
              refused_now = true;
            }
          }
        }
      }
    }
    if refused_now {
      let running = running_apps(wins, &self.refused);
      self.model.prune(&running);
    }
    self.save();
    self.hold = outs.iter().map(|(a, _)| a.clone()).collect();
    self.rebuild();

    for (app, from) in outs {
      if let (Some(bmp), Some(to)) = (self.shots.get(&app).cloned(), self.slot_rect_of(&app)) {
        items.push((bmp, from, to));
      }
    }

    let mut fly_in = false;
    if let Some((app, from)) = show_from {
      let to = self.parked.iter().find(|(_, a)| *a == app).map(|(p, _)| {
        let (l, t, r, b) = p.rect;
        (l as f32, t as f32, (r - l) as f32, (b - t) as f32)
      });
      if let (Some(bmp), Some(to)) = (self.shots.get(&app).cloned(), to) {
        items.push((bmp, from, to));
        fly_in = true;
      }
    }
    if items.is_empty() {
      self.reveal_hold();
    } else {
      self.start_fly(items, if fly_in { ch.show.clone() } else { vec![] }, activate);
    }
    if !fly_in {
      self.unpark_apps(&ch.show);
      if activate != 0 {
        focus::force_foreground(activate);
      }
    }
    self.apply_icons();
    self.check_overlap();
  }
  pub(super) fn unpark_apps(&mut self, apps: &[String]) {
    let mut keep = Vec::new();
    for (p, a) in self.parked.drain(..) {
      if apps.contains(&a) {
        let mut q = p;
        let sw = nexora_platform::sys::screen_size().0;
        let right = self.strip.as_ref().map(|s| s.right).unwrap_or(false);
        let edge = self.strip.as_ref().map(|s| (s.w + 8.0 * s.dpi) as i32).unwrap_or(0);
        if !q.maximized && right && q.rect.2 > sw - edge {
          let dx = (q.rect.2 - (sw - edge)).min(q.rect.0.max(0));
          q.rect = (q.rect.0 - dx, q.rect.1, q.rect.2 - dx, q.rect.3);
        } else if !q.maximized && !right && q.rect.0 < edge {
          let dx = edge - q.rect.0;
          q.rect = (q.rect.0 + dx, q.rect.1, q.rect.2 + dx, q.rect.3);
          if q.rect.2 > sw {
            q.rect.2 = sw;
          }
        }
        park::unpark(&q);
      } else {
        keep.push((p, a));
      }
    }
    self.parked = keep;
    self.save();
  }
}
