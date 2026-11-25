use super::*;

impl Spot {
  pub(super) fn request_recents(&mut self) {
    if self.recents_requested {
      return;
    }
    self.recents_requested = true;
    let (rec, th) = (self.recents.clone(), self.thumbs.clone());
    let h = CTL
      .with(|c| c.borrow().as_ref().map(|c| c.hwnd.0 as isize))
      .unwrap_or(0);
    let px = (180.0 * self.dpi) as i32;
    std::thread::spawn(move || unsafe {
      use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
      let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
      let list = nexora_platform::recent::list(15);
      *rec.lock().unwrap() = list.clone();
      let _ = PostMessageW(Some(HWND(h as *mut _)), WM_THUMBS, WPARAM(0), LPARAM(0));
      for (_, p) in list {
        if let Some(b) = icons::thumbnail(&p, px) {
          th.lock().unwrap().insert(p, b);
          let _ = PostMessageW(Some(HWND(h as *mut _)), WM_THUMBS, WPARAM(0), LPARAM(0));
        }
      }
      if com {
        CoUninitialize();
      }
    });
  }

  pub(super) fn thumb_for(&mut self, path: &str, box_px: f32) -> Option<(CompositionDrawingSurface, f32, f32)> {
    if !self.thumb_surf.contains_key(path) {
      let b = self.thumbs.lock().unwrap().get(path).cloned()?;
      let s = self.gfx.surface_from_bgra(b.w, b.h, &b.bgra).ok()?;
      self.thumb_surf.insert(path.to_string(), (s, b.w, b.h));
    }
    let (s, w, h) = self.thumb_surf.get(path)?.clone();
    let k = (box_px / w as f32).min(box_px / h as f32);
    Some((s, w as f32 * k, h as f32 * k))
  }

  pub(super) fn suggestions(&self) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    let key = |p: &str| p.replace('/', "\\").to_lowercase();
    for f in ["pins.txt", "recents.txt"] {
      let text = std::fs::read_to_string(paths::config_dir().join(f)).unwrap_or_default();
      for line in text.lines() {
        let p = key(line.split('\t').next().unwrap_or("").trim());
        if p.is_empty() {
          continue;
        }
        if let Some(i) = self.apps.iter().position(|a| key(&a.path) == p) {
          if !out.contains(&i) {
            out.push(i);
          }
        }
        if out.len() >= COLS {
          return out;
        }
      }
    }
    let mut rest: Vec<usize> = (0..self.apps.len()).filter(|i| !out.contains(i)).collect();
    rest.sort_by_key(|&i| self.apps[i].name.to_lowercase());
    out.extend(rest.into_iter().take(COLS - out.len()));
    out
  }

  pub(super) fn rebuild_rows(&mut self) {
    self.rows.clear();
    self.headers.clear();
    self.sections.clear();
    self.sel = 0;
    self.scroll = 0;
    let q = self.query.trim().to_string();
    match self.mode {
      Mode::Search => {
        if q.is_empty() {
          return;
        }
        let f = self.filter;
        if f.is_none() {
          if let Some(v) = search::eval(&q) {
            self.rows.push(Row {
              kind: RKind::Calc,
              title: format!("= {}", search::format_number(v)),
              sub: prefs::tr("Hasil kalkulator").to_string(),
              path: String::new(),
            });
          }
        }
        if f.is_none() || f == Some(Filter::Apps) {
          for i in search::rank(&q, &self.names, if f.is_some() { 8 } else { 5 }) {
            self.rows.push(Row {
              kind: RKind::App,
              title: self.apps[i].name.clone(),
              sub: prefs::tr("Aplikasi").to_string(),
              path: self.apps[i].path.clone(),
            });
          }
        }
        if f.is_none() || f == Some(Filter::Actions) {
          let titles = |a: &model::ActionDef| action_title(a);
          for i in model::rank_actions(&q, &self.quick, &titles)
            .into_iter()
            .take(if f.is_some() { 8 } else { 2 })
          {
            self.rows.push(self.action_row(i));
          }
        }
        if f.is_none() || f == Some(Filter::Files) {
          for h in self.files.iter().take(if f.is_some() { 8 } else { 3 }) {
            self.rows.push(Row {
              kind: RKind::File,
              title: h.name.clone(),
              sub: h.path.clone(),
              path: h.path.clone(),
            });
          }
        }
        if f.is_none() {
          self.rows.push(Row {
            kind: RKind::Web,
            title: format!("{} \"{}\"", prefs::tr("Cari di web:"), q),
            sub: prefs::tr("Pencarian web").to_string(),
            path: format!("https://www.google.com/search?q={}", url_encode(&q)),
          });
        }
        self.rows.truncate(9);
      }
      Mode::Apps if q.is_empty() => {
        let push = |rows: &mut Vec<Row>,
                    sections: &mut Vec<(String, std::ops::Range<usize>)>,
                    title: String,
                    idxs: Vec<usize>,
                    apps: &[App]| {
          if idxs.is_empty() {
            return;
          }
          let start = rows.len();
          for i in idxs {
            rows.push(Row {
              kind: RKind::App,
              title: apps[i].name.clone(),
              sub: String::new(),
              path: apps[i].path.clone(),
            });
          }
          sections.push((title, start..rows.len()));
        };
        let (mut rows, mut sections) = (Vec::new(), Vec::new());
        let sugg = self.suggestions();
        push(
          &mut rows,
          &mut sections,
          prefs::tr("Saran").to_string(),
          sugg,
          &self.apps,
        );
        for c in appcat::ALL {
          let mut v: Vec<usize> = (0..self.apps.len())
            .filter(|&i| appcat::guess(&self.apps[i].name, &self.apps[i].path) == c)
            .collect();
          v.sort_by_key(|&i| self.apps[i].name.to_lowercase());
          push(
            &mut rows,
            &mut sections,
            prefs::tr(c.title()).to_string(),
            v,
            &self.apps,
          );
        }
        self.rows = rows;
        self.sections = sections;
        if self.expanded.len() != self.sections.len() {
          self.expanded = vec![false; self.sections.len()];
        }
      }
      Mode::Apps => {
        for i in search::rank(&q, &self.names, 200) {
          let a = &self.apps[i];
          self.rows.push(Row {
            kind: RKind::App,
            title: a.name.clone(),
            sub: String::new(),
            path: a.path.clone(),
          });
        }
      }
      Mode::Files if q.is_empty() => {
        let rec = self.recents.lock().unwrap().clone();
        for (n, p) in rec.into_iter().filter(|(_, p)| of_type(p, self.ftype)) {
          self.rows.push(Row {
            kind: RKind::File,
            title: n,
            sub: p.clone(),
            path: p,
          });
        }
        self.sections = vec![(prefs::tr("Terbaru").to_string(), 0..self.rows.len())];
        self.expanded = vec![true];
      }
      Mode::Files => {
        for h in self.files.iter().filter(|h| of_type(&h.path, self.ftype)) {
          self.rows.push(Row {
            kind: RKind::File,
            title: h.name.clone(),
            sub: h.path.clone(),
            path: h.path.clone(),
          });
        }
      }
      Mode::Actions => {
        if q.is_empty() {
          self.headers.push((0, prefs::tr("Saran").to_string()));
          let n = ACTIONS.len().min(5);
          for i in 0..n {
            self.rows.push(self.action_row(i));
          }
          let mut rest: Vec<usize> = (n..ACTIONS.len()).collect();
          rest.sort_by_key(|&i| prefs::tr(ACTIONS[i].app).to_lowercase());
          let mut last = String::new();
          for i in rest {
            let app = prefs::tr(ACTIONS[i].app).to_string();
            if app != last {
              self.headers.push((self.rows.len(), app.clone()));
              last = app;
            }
            self.rows.push(self.action_row(i));
          }
        } else {
          let titles = |a: &model::ActionDef| action_title(a);
          for i in model::rank_actions(&q, &self.quick, &titles) {
            self.rows.push(self.action_row(i));
          }
        }
      }
      Mode::Clipboard => {
        let ql = q.to_lowercase();
        for e in clipboard::history() {
          if ql.is_empty() || e.text.to_lowercase().contains(&ql) {
            let first = e.text.lines().next().unwrap_or("").trim().to_string();
            let sub = format!(
              "{} \u{00B7} {} {}",
              prefs::tr("teks"),
              prefs::tr("Disalin"),
              clip_time(e.at)
            );
            self.rows.push(Row {
              kind: RKind::Clip,
              title: first,
              sub,
              path: e.text.clone(),
            });
          }
        }
      }
    }
  }

  pub(super) fn request_files(&mut self) {
    let q = self.query.trim().to_string();
    self.files_gen += 1;
    if q.chars().count() < 2 {
      self.files.clear();
      return;
    }
    let (gen, shared) = (self.files_gen, self.shared_files.clone());
    let h = CTL
      .with(|c| c.borrow().as_ref().map(|c| c.hwnd.0 as isize))
      .unwrap_or(0);
    std::thread::spawn(move || {
      let hits = filesearch::search(&q);
      {
        let mut g = shared.lock().unwrap();
        if gen >= g.0 {
          *g = (gen, hits);
        }
      }
      unsafe {
        let _ = PostMessageW(Some(HWND(h as *mut _)), WM_FILES, WPARAM(0), LPARAM(0));
      }
    });
  }

  pub(super) fn take_files(&mut self) -> bool {
    let g = self.shared_files.lock().unwrap();
    if g.0 == self.files_gen && g.0 != 0 {
      self.files = g.1.clone();
      true
    } else {
      false
    }
  }

  pub(super) fn action_row(&self, i: usize) -> Row {
    let a = &ACTIONS[i];
    Row {
      kind: RKind::Action(i),
      title: action_title(a),
      sub: prefs::tr(a.app).to_string(),
      path: a.id.to_string(),
    }
  }
}
