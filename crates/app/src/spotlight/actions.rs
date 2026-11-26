use super::*;

impl Spot {
  pub(super) fn take_run(&mut self) -> Run {
    let Some(r) = self.rows.get(self.sel).cloned() else {
      return Run::Nothing;
    };
    match r.kind {
      RKind::App | RKind::Web | RKind::File => Run::Open(r.path),
      RKind::Calc => Run::Nothing,
      RKind::Action(i) => Run::Action(ACTIONS[i].id),
      RKind::Clip => Run::Copy(r.path),
    }
  }

  pub(super) fn move_sel(&mut self, dx: i32, dy: i32) {
    let n = self.rows.len();
    if n == 0 {
      return;
    }
    let empty_q = self.query.trim().is_empty();
    if matches!(self.mode, Mode::Apps | Mode::Files) && empty_q {
      let vis: Vec<usize> = self.cell_y.iter().map(|c| c.0).collect();
      if vis.is_empty() {
        return;
      }
      let pos = vis.iter().position(|&i| i == self.sel).unwrap_or(0) as i32;
      let np = (pos + dx + dy * COLS as i32).clamp(0, vis.len() as i32 - 1) as usize;
      self.sel = vis[np];
      if let Some(&(_, y, h)) = self.cell_y.iter().find(|c| c.0 == self.sel) {
        let (top, bottom) = self.view;
        let yv = y - self.pscroll;
        if yv < top {
          self.pscroll -= top - yv + 6.0 * self.dpi;
        } else if yv + h > bottom {
          self.pscroll += yv + h - bottom + 6.0 * self.dpi;
        }
        self.pscroll = self.pscroll.clamp(0.0, (self.content_h - (bottom - top)).max(0.0));
      }
    } else if self.mode == Mode::Apps {
      let cur = self.sel as i32;
      let next = (cur + dx + dy * COLS as i32).clamp(0, n as i32 - 1) as usize;
      self.sel = next;
      let row = self.sel / COLS;
      if row < self.scroll {
        self.scroll = row;
      } else if row >= self.scroll + GRID_ROWS {
        self.scroll = row + 1 - GRID_ROWS;
      }
    } else {
      let step = if dy != 0 { dy } else { dx };
      self.sel = ((self.sel as i32 + step).rem_euclid(n as i32)) as usize;
      if self.sel < self.scroll {
        self.scroll = self.sel;
      } else if self.sel >= self.scroll + 8 {
        self.scroll = self.sel + 1 - 8;
      }
    }
    self.render();
  }

  pub(super) fn start_edit_key(&mut self) {
    if let Some(RKind::Action(i)) = self.rows.get(self.sel).map(|r| r.kind.clone()) {
      self.editing = Some(i);
      self.edit_buf = model::key_of(&ACTIONS[i], &self.quick).to_string();
      self.render();
    }
  }

  pub(super) fn finish_edit_key(&mut self, save: bool) {
    if let (Some(i), true) = (self.editing, save) {
      let k = self.edit_buf.trim().to_lowercase();
      self.quick.insert(ACTIONS[i].id.to_string(), k);
      let _ = std::fs::write(quick_file(), model::quick_keys_to_text(&self.quick));
    }
    self.editing = None;
    self.edit_buf.clear();
    self.rebuild_rows();
    self.render();
  }
}

pub fn run_action(id: &str) {
  match id {
    "note" => launch::run("notepad.exe", ""),
    "timer" => launch::open("ms-clock:"),
    "email" => launch::open("mailto:"),
    "screenshot" => keys::chord(&[keys::WIN, keys::SHIFT, 0x53]),
    "lock" => launch::run("rundll32.exe", "user32.dll,LockWorkStation"),
    "desktop" => keys::chord(&[keys::WIN, 0x44]),
    "dark" => prefs::update(|s| {
      s.theme = if s.theme == nexora_core::settings::Theme::Dark {
        nexora_core::settings::Theme::Light
      } else {
        nexora_core::settings::Theme::Dark
      }
    }),
    "stage" => prefs::update(|s| s.stage = !s.stage),
    "mission" => crate::mission::toggle(),
    "calc" => launch::run("calc.exe", ""),
    "taskmgr" => launch::run("taskmgr.exe", ""),
    "settings" => launch::open("ms-settings:"),
    "clearclip" => clipboard::clear(),
    _ => {}
  }
}

pub fn clip_time(at: i64) -> String {
  let now_local = time::local_now();
  let now_secs = (now_local.hour * 3600 + now_local.minute * 60 + now_local.second) as i64;
  let age = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0)
    - at;
  let t = (now_secs - age).rem_euclid(86_400);
  let (h, m) = ((t / 3600) as u32, ((t % 3600) / 60) as u32);
  if prefs::get().clock_24h {
    format!("{h:02}.{m:02}")
  } else {
    format!(
      "{}:{m:02} {}",
      if h % 12 == 0 { 12 } else { h % 12 },
      if h < 12 { "AM" } else { "PM" }
    )
  }
}

pub fn perform(r: Run) {
  match r {
    Run::Nothing => {}
    Run::Open(p) => launch::open(&p),
    Run::Action(id) => run_action(id),
    Run::Copy(t) => {
      clipboard::copy_text(&t);
    }
  }
}

pub fn ctrl_down() -> bool {
  unsafe { (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 }
}
