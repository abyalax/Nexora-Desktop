use super::*;

pub(super) unsafe extern "system" fn ctl_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_HOTKEY => {
      if wp.0 as i32 == HOTKEY_APPS {
        toggle_mode(Mode::Apps);
      } else {
        toggle();
      }
      LRESULT(0)
    }
    WM_APPS => {
      with_spot(|s| {
        s.apps = apps::snapshot();
        s.names = s.apps.iter().map(|a| a.name.clone()).collect();
        s.rebuild_rows();
        s.render();
      });
      LRESULT(0)
    }
    WM_TIMER if wp.0 == T_FINISH => {
      let _ = KillTimer(Some(hwnd), T_FINISH);
      finish_closing();
      LRESULT(0)
    }
    WM_THUMBS => {
      with_spot(|s| {
        if s.mode == Mode::Files && s.query.trim().is_empty() && !s.morphing {
          let sel = s.sel;
          s.rebuild_rows();
          s.sel = sel.min(s.rows.len().saturating_sub(1));
          s.render();
        }
      });
      LRESULT(0)
    }
    WM_FILES => {
      with_spot(|s| {
        if s.take_files() && matches!(s.mode, Mode::Search | Mode::Files) {
          let (sel, scroll) = (s.sel, s.scroll);
          s.rebuild_rows();
          s.sel = sel.min(s.rows.len().saturating_sub(1));
          s.scroll = scroll;
          s.render();
        }
      });
      LRESULT(0)
    }
    m if m == prefs::WM_PREFS => {
      clipboard::set_enabled(prefs::get().clipboard_history);
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}

pub(super) unsafe extern "system" fn spot_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_CHAR => {
      let c = wp.0 as u32;
      if c >= 32 && c != 127 && !ctrl_down() {
        if let Some(ch) = char::from_u32(c) {
          with_spot(|s| {
            if s.editing.is_some() {
              if ch.is_ascii_alphanumeric() && s.edit_buf.len() < 6 {
                s.edit_buf.push(ch.to_ascii_lowercase());
                s.render();
              }
              return;
            }
            s.query.push(ch);
            if matches!(s.mode, Mode::Search | Mode::Files) {
              s.request_files();
            }
            s.rebuild_rows();
            s.render();
          });
        }
      }
      LRESULT(0)
    }
    WM_KEYDOWN => {
      let vk = wp.0 as u16;
      if ctrl_down() && (0x31..=0x34).contains(&vk) {
        if let Some(m) = Mode::from_digit((vk - 0x30) as u32) {
          with_spot(|s| s.set_mode(m));
        }
        return LRESULT(0);
      }
      if vk == VK_ESCAPE.0 {
        let handled = with_spot(|s| {
          if s.editing.is_some() {
            s.finish_edit_key(false);
            return true;
          }
          if !s.query.is_empty() {
            s.query.clear();
            s.files.clear();
            s.rebuild_rows();
            s.render();
            return true;
          }
          if s.mode != Mode::Search {
            s.set_mode(Mode::Search);
            return true;
          }
          false
        })
        .unwrap_or(false);
        if !handled {
          close();
        }
      } else if vk == VK_BACK.0 {
        with_spot(|s| {
          if s.editing.is_some() {
            s.edit_buf.pop();
            s.render();
            return;
          }
          s.query.pop();
          if matches!(s.mode, Mode::Search | Mode::Files) {
            s.request_files();
          }
          s.rebuild_rows();
          s.render();
        });
      } else if vk == VK_TAB.0 {
        with_spot(|s| {
          if s.mode == Mode::Actions || s.mode == Mode::Search {
            s.start_edit_key();
          }
        });
      } else if vk == VK_DOWN.0 || vk == VK_UP.0 || vk == VK_LEFT.0 || vk == VK_RIGHT.0 {
        with_spot(|s| {
          let (dx, dy) = match vk {
            v if v == VK_DOWN.0 => (0, 1),
            v if v == VK_UP.0 => (0, -1),
            v if v == VK_LEFT.0 => (-1, 0),
            _ => (1, 0),
          };
          if (dx != 0 && s.mode != Mode::Apps) || s.editing.is_some() {
            return;
          }
          s.move_sel(dx, dy);
        });
      } else if vk == VK_RETURN.0 {
        let run = with_spot(|s| {
          if s.editing.is_some() {
            s.finish_edit_key(true);
            return Run::Nothing;
          }
          s.take_run()
        })
        .unwrap_or(Run::Nothing);
        if !matches!(run, Run::Nothing) {
          close();
          perform(run);
        }
      }
      LRESULT(0)
    }
    WM_MOUSEWHEEL => {
      let delta = (wp.0 >> 16) as i16;
      with_spot(|s| {
        if matches!(s.mode, Mode::Apps | Mode::Files) && s.query.trim().is_empty() {
          let max = (s.content_h - (s.view.1 - s.view.0)).max(0.0);
          s.pscroll = (s.pscroll + if delta < 0 { 72.0 } else { -72.0 } * s.dpi).clamp(0.0, max);
          s.render();
          return;
        }
        let step = if s.mode == Mode::Apps { 1 } else { 2 };
        let per = if s.mode == Mode::Apps { COLS } else { 1 };
        let max_scroll = if s.mode == Mode::Apps {
          s.rows.len().div_ceil(per).saturating_sub(GRID_ROWS)
        } else {
          s.rows.len().saturating_sub(8)
        };
        s.scroll = if delta < 0 {
          (s.scroll + step).min(max_scroll)
        } else {
          s.scroll.saturating_sub(step)
        };
        s.render();
      });
      LRESULT(0)
    }
    WM_MOUSEMOVE => {
      let (x, y) = ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32);
      with_spot(|s| {
        if !s.tracking {
          let mut t = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
          };
          if TrackMouseEvent(&mut t).is_ok() {
            s.tracking = true;
          }
        }
        if s.morphing {
          return;
        }
        if s.compact() {
          if !s.buttons {
            s.buttons = true;
            s.buttons_intro = true;
            s.hover_mode = None;
            s.render();
            return;
          }
          let hm = s
            .hits
            .iter()
            .find(|(r, k)| hit(*r, x, y) && matches!(k, HitKind::Mode(_)))
            .and_then(|(_, k)| match k {
              HitKind::Mode(m) => Some(*m),
              _ => None,
            });
          if hm != s.hover_mode {
            s.hover_mode = hm;
            s.render();
          }
          return;
        }
        let h = s
          .hits
          .iter()
          .find(|(r, k)| hit(*r, x, y) && matches!(k, HitKind::Row(_) | HitKind::Cell(_)))
          .map(|(_, k)| *k);
        let idx = match h {
          Some(HitKind::Row(i)) | Some(HitKind::Cell(i)) => Some(i),
          _ => None,
        };
        if let Some(i) = idx {
          if i != s.sel {
            s.sel = i;
            s.render();
          }
        }
      });
      LRESULT(0)
    }
    WM_MOUSELEAVE_SPOT => {
      with_spot(|s| {
        s.tracking = false;
        if s.compact() && s.buttons && !s.morphing {
          s.buttons = false;
          s.hover_mode = None;
          s.render();
        }
      });
      LRESULT(0)
    }
    WM_TIMER => {
      let id = wp.0;
      if id == T_MORPH {
        with_spot(|s| s.finish_morph());
      } else if id == T_REGION {
        let _ = KillTimer(Some(hwnd), T_REGION);
        with_spot(|s| s.set_region(s.cur));
      }
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let (x, y) = ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32);
      let run = with_spot(|s| {
        let Some(k) = s.hits.iter().find(|(r, _)| hit(*r, x, y)).map(|(_, k)| *k) else {
          return Run::Nothing;
        };
        match k {
          HitKind::Mode(m) => {
            s.set_mode(m);
            Run::Nothing
          }
          HitKind::Filter(f) => {
            s.filter = if s.filter == Some(f) { None } else { Some(f) };
            s.rebuild_rows();
            s.render();
            Run::Nothing
          }
          HitKind::Cat(c) => {
            s.cat = c;
            s.rebuild_rows();
            s.render();
            Run::Nothing
          }
          HitKind::Row(i) | HitKind::Cell(i) => {
            s.sel = i;
            s.take_run()
          }
          HitKind::ShowMore(si) => {
            if let Some(e) = s.expanded.get_mut(si) {
              *e = !*e;
            }
            s.render();
            Run::Nothing
          }
          HitKind::More => {
            s.more_open = !s.more_open;
            s.render();
            Run::Nothing
          }
          HitKind::MoreItem => {
            s.more_open = false;
            match s.mode {
              Mode::Clipboard => {
                clipboard::clear();
                s.rebuild_rows();
                s.render();
                Run::Nothing
              }
              Mode::Apps => {
                let all = s.expanded.iter().all(|e| *e);
                for e in s.expanded.iter_mut() {
                  *e = !all;
                }
                s.render();
                Run::Nothing
              }
              _ => Run::Open("shell:recent".to_string()),
            }
          }
          HitKind::FType(i) => {
            s.ftype = if s.ftype == Some(i) { None } else { Some(i) };
            s.rebuild_rows();
            s.render();
            Run::Nothing
          }
          HitKind::CopyRow(i) => s.rows.get(i).map(|r| Run::Copy(r.path.clone())).unwrap_or(Run::Nothing),
        }
      })
      .unwrap_or(Run::Nothing);
      if !matches!(run, Run::Nothing) {
        close();
        perform(run);
      }
      LRESULT(0)
    }
    WM_ACTIVATE => {
      if (wp.0 & 0xFFFF) == 0 {
        let _ = PostMessageW(Some(hwnd), WM_CLOSE_SPOT, WPARAM(0), LPARAM(0));
      }
      LRESULT(0)
    }
    m if m == WM_CLOSE_SPOT => {
      close();
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}
