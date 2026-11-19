//! Glance rotation, download watching and the calendar agenda.

use super::*;

impl Notch {
  pub(super) fn today() -> views::Date {
    let t = nexora_platform::time::local_now();
    (t.year, t.month, t.day)
  }

  pub(super) fn cal_date(&self) -> views::Date {
    let (y, m, d) = Self::today();
    nexora_core::panels::calendar::add_days(y, m, d, self.cal_sel)
  }

  pub(super) fn ensure_events(&mut self) {
    let date = self.cal_date();
    if self.cal_events.as_ref().is_some_and(|(d, _)| *d == date) {
      return;
    }
    if self.agenda.is_none() {
      let hw = self.hwnd.0 as isize;
      self.agenda = Some(nexora_platform::agenda::spawn(move || post(hw, WM_AGENDA)));
    }
    if let Some(a) = &self.agenda {
      let _ = a.requests.send(date);
    }
  }

  pub(super) fn on_agenda(&mut self) {
    let Some(a) = &self.agenda else { return };
    let mut got = false;
    while let Ok(ans) = a.answers.try_recv() {
      self.cal_events = Some(ans);
      got = true;
    }
    if got && self.mode == Mode::Open(Tab::Timer) {
      self.render();
    }
  }

  pub(super) fn ensure_app_icon(&mut self) {
    let id = self.media_app.clone();
    if id.is_empty() || self.app_icons.contains_key(&id) {
      return;
    }
    let px = (48.0 * self.dpi).round() as i32;
    let bmp = nexora_platform::icons::load(&format!("shell:AppsFolder\\{id}"), px).or_else(|| {
      id.to_ascii_lowercase()
        .ends_with(".exe")
        .then(|| nexora_platform::wins::enumerate::exe_path_of_running(&id))
        .flatten()
        .and_then(|p| nexora_platform::icons::load(&p, px))
    });
    let s = bmp.and_then(|b| self.paint.gfx.surface_from_bgra(b.w, b.h, &b.bgra).ok());
    self.app_icons.insert(id, s);
  }

  pub(super) fn sync_download_watch(&mut self) {
    let on = crate::prefs::get().notch_downloads;
    if on && self.dl_watch.is_none() {
      let hw = self.hwnd.0 as isize;
      self.dl_watch = Some(nexora_platform::dlwatch::spawn(move || post(hw, WM_DOWNLOAD)));
    } else if !on && self.dl_watch.is_some() {
      self.dl_watch = None;
      self.dl = None;
      self.dl_done = None;
    }
  }

  pub(super) fn on_download(&mut self) {
    let Some(w) = &self.dl_watch else { return };
    let mut changed = false;
    while let Ok(s) = w.snapshots.try_recv() {
      self.dl = nexora_core::dynamic_island::download::primary(&s.active).cloned();
      if self.dl.is_some() {
        self.dl_done = None;
      } else if let Some(f) = s.finished {
        log::line("unduhan", &format!("selesai ({:?})", f.source));
        self.dl_done = Some(f);
        timer(
          self.hwnd,
          T_DLDONE,
          nexora_core::dynamic_island::download::DONE_LINGER_MS,
        );
      }
      changed = true;
    }
    if changed {
      let app = self
        .dl
        .as_ref()
        .or(self.dl_done.as_ref())
        .map(|d| d.app.clone())
        .unwrap_or_default();
      if !app.is_empty() && !self.dl_icons.contains_key(&app) {
        let px = (44.0 * self.dpi).round() as i32;
        let s =
          nexora_platform::icons::load(&app, px).and_then(|b| self.paint.gfx.surface_from_bgra(b.w, b.h, &b.bgra).ok());
        self.dl_icons.insert(app, s);
      }
      self.refresh();
    }
  }

  pub(super) fn download_view<'a>(
    dl: &'a Option<nexora_core::dynamic_island::download::Download>,
    dl_done: &'a Option<nexora_core::dynamic_island::download::Download>,
    icons: &'a std::collections::HashMap<String, Option<CompositionDrawingSurface>>,
    send: Option<(f32, bool)>,
  ) -> Option<views::DownloadView<'a>> {
    let (d, done) = match (dl, dl_done) {
      (Some(d), _) => (d, false),
      (None, Some(d)) => (d, true),
      _ => {
        let (fr, done) = send?;
        let pct = (fr * 100.0).floor() as u32;
        return Some(views::DownloadView {
          icon: None,
          label: format!("{}", if done { 100 } else { pct.min(99) }),
          frac: Some(fr),
          done,
          up: true,
        });
      }
    };
    Some(views::DownloadView {
      icon: icons.get(&d.app).and_then(|s| s.as_ref()),
      label: nexora_core::dynamic_island::download::label(d),
      frac: d.percent.map(|p| p / 100.0),
      done,
      up: false,
    })
  }

  pub(super) fn glance_items(&self) -> Vec<glance::Item> {
    glance::items(self.battery.is_some())
  }

  pub(super) fn glance_text(&self, item: glance::Item) -> String {
    let s = crate::prefs::get();
    let now = nexora_platform::time::local_now();
    let lang = s.lang;
    match item {
      glance::Item::Greeting => {
        let name = if s.greet_name.is_empty() {
          std::env::var("USERNAME").unwrap_or_default()
        } else {
          s.greet_name.as_str().to_string()
        };
        glance::greeting_text(s.greet_text.as_str(), &name, now.hour, lang)
      }
      glance::Item::Clock => glance::clock(now.hour, now.minute, s.clock_24h, lang),
      glance::Item::Date => {
        let wd = calendar::weekday(now.year, now.month, now.day) as usize;
        let days = nexora_core::i18n::weekday_labels(lang, false, calendar::weekday_labels(false));
        let month: String = nexora_core::i18n::month_name(lang, now.month, calendar::month_name(now.month))
          .chars()
          .take(3)
          .collect();
        format!("{}, {} {}", days[wd], now.day, month)
      }
      glance::Item::Battery => self.battery.map_or(String::new(), |(p, _)| format!("{p}%")),
    }
  }

  pub(super) fn sync_glance(&self) {
    if self.look == state::look(Mode::Rest, Some(state::Activity::Glance)) {
      let ms = std::env::var("NACRE_GLANCE_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(glance::ROTATE_MS);
      timer(self.hwnd, T_GLANCE, ms);
    } else {
      kill(self.hwnd, T_GLANCE);
    }
  }

  pub(super) fn rotate_glance(&mut self) {
    let n = self.glance_items().len();
    self.glance_idx = glance::next(self.glance_idx, n);
    self.retire_content(120);
    let rise = 6.0 * self.dpi;
    if let Ok(g) = self.body.ghost.cast::<CompositionObject>() {
      let _ = keyframes(&self.paint.comp, &g, "Offset.Y", 0.0, -rise, 160, 0);
    }
    self.render();
    if let Ok(c) = self.body.content.cast::<CompositionObject>() {
      let comp = &self.paint.comp;
      let _ = self.body.content.SetScale(Vector3 { X: 1.0, Y: 1.0, Z: 1.0 });
      let _ = keyframes(comp, &c, "Opacity", 0.0, 1.0, 180, 60);
      let _ = keyframes(comp, &c, "Offset.Y", rise, 0.0, 220, 40);
    }
  }
}
