use super::*;

pub(super) unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      WM_NCHITTEST => {
        let mut r = windows::Win32::Foundation::RECT::default();
        let _ = GetWindowRect(hwnd, &mut r);
        let (px, py) = ((lp.0 as i16) as i32, ((lp.0 >> 16) as i16) as i32);
        let drag = with_win(|w| {
          let d = w.dpi;
          py - r.top < (VIEW_TOP * d) as i32 && px - r.left > ((CONTENT_X + 70.0) * d) as i32
        })
        .unwrap_or(false);
        LRESULT(if drag { HTCAPTION } else { HTCLIENT } as isize)
      }
      WM_MOUSEWHEEL => {
        let delta = ((wp.0 >> 16) as i16) as f32;
        with_win(|w| {
          let d = w.dpi;
          let vh = (H - VIEW_TOP - 10.0) * d;
          let mut p = windows::Win32::Foundation::POINT::default();
          let _ = GetCursorPos(&mut p);
          if ((p.x - w.popup.rect.0) as f32) < SIDEBAR_W * d {
            let max = w.side_view.1;
            w.side_scroll = (w.side_scroll - delta / 120.0 * 40.0 * d).clamp(0.0, max);
            w.render();
            return;
          }
          let max = (w.content_h - vh).max(0.0);
          w.scroll = (w.scroll - delta / 120.0 * 48.0 * d).clamp(0.0, max);
          w.render();
        });
        LRESULT(0)
      }
      WM_LBUTTONDOWN => {
        let (x, y) = xy(lp);
        let grabbed = with_win(|w| {
          let Some((r, th)) = w.side_bar else {
            return false;
          };
          if !(x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3) {
            return false;
          }
          let max = w.side_view.1;
          let ty = r.1 + (r.3 - th) * (w.side_scroll / max.max(0.001));
          let grab = if y >= ty && y < ty + th { y - ty } else { th / 2.0 };
          w.side_drag = Some(grab);
          w.side_scroll = (((y - grab - r.1) / (r.3 - th).max(1.0)) * max).clamp(0.0, max);
          w.render();
          true
        })
        .unwrap_or(false);
        if grabbed {
          SetCapture(hwnd);
          return LRESULT(0);
        }
        let started = with_win(|w| match w.hit(x, y) {
          Some(Action::Slider(id, sx, sw, scale)) => {
            w.dragging = Some((id, sx, sw));
            w.slide(id, x, sx, sw, false, scale);
            w.render();
            true
          }
          Some(_) => {
            w.press = Some((x, y));
            w.render();
            false
          }
          None => false,
        })
        .unwrap_or(false);
        if started {
          let _ = SetCapture(hwnd);
        }
        LRESULT(0)
      }
      WM_MOUSEMOVE => {
        let (x, y) = xy(lp);
        with_win(|w| {
          if let (Some(grab), Some((r, th))) = (w.side_drag, w.side_bar) {
            let max = w.side_view.1;
            w.side_scroll = (((y - grab - r.1) / (r.3 - th).max(1.0)) * max).clamp(0.0, max);
            w.render();
            return;
          }
          if let Some((id, sx, sw)) = w.dragging {
            w.slide(id, x, sx, sw, false, None);
            return;
          }
          let mut t = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
          };
          let _ = TrackMouseEvent(&mut t);
          let (top, s) = (w.side_view.0, w.side_scroll);
          let hit = if y < top || y > (H - 96.0) * w.dpi {
            None
          } else {
            w.side
              .iter()
              .find(|(_, r, _)| x >= r.0 && x < r.0 + r.2 && y + s >= r.1 && y + s < r.1 + r.3)
              .map(|e| e.0)
          };
          w.hover_side(hit);
        });
        LRESULT(0)
      }
      WM_MOUSELEAVE => {
        with_win(|w| w.hover_side(None));
        LRESULT(0)
      }
      WM_CHAR => {
        let ch = wp.0 as u32;
        with_win(|w| {
          if w.edit.is_some() {
            if ch >= 32 && ch != 127 && w.edit_buf.chars().count() < 60 {
              if let Some(c) = char::from_u32(ch) {
                w.edit_buf.push(c);
                w.render();
              }
            }
            return;
          }
          if w.search_focus && ch >= 32 && ch != 127 {
            if let Some(c) = char::from_u32(ch) {
              w.search.push(c);
              w.render();
            }
          }
        });
        LRESULT(0)
      }
      WM_KEYDOWN => {
        let vk = wp.0 as u16;
        with_win(|w| {
          if w.edit.is_some() {
            match vk {
              0x08 => {
                w.edit_buf.pop();
                w.render();
              }
              0x1B => {
                w.edit = None;
                w.render();
              }
              0x0D => w.commit_edit(),
              _ => {}
            }
            return;
          }
          if !w.search_focus {
            return;
          }
          match vk {
            0x08 => {
              w.search.pop();
            }
            0x1B => {
              w.search.clear();
              w.search_focus = false;
            }
            0x0D => {
              if let Some(t) = (0u8..8).find(|i| w.page_matches(*i, "")) {
                w.go(t, true);
                return;
              }
            }
            _ => return,
          }
          w.render();
        });
        LRESULT(0)
      }
      WM_LBUTTONUP => {
        let (x, y) = xy(lp);
        let _ = ReleaseCapture();
        if with_win(|w| w.side_drag.take().is_some()).unwrap_or(false) {
          with_win(|w| w.render());
          return LRESULT(0);
        }
        let act = with_win(|w| {
          let was_pressed = w.press.take().is_some();
          if let Some((id, sx, sw)) = w.dragging.take() {
            w.slide(id, x, sx, sw, true, None);
            w.render();
            return None;
          }
          if was_pressed {
            w.render();
          }
          w.hit(x, y)
        })
        .flatten();
        if !matches!(act, Some(Action::Search)) {
          with_win(|w| {
            if w.search_focus {
              w.search_focus = false;
              w.render();
            }
          });
        }
        if !matches!(act, Some(Action::EditText(_))) {
          with_win(|w| {
            if w.edit.is_some() {
              w.commit_edit();
            }
          });
        }
        dispatch(hwnd, act);
        LRESULT(0)
      }
      WM_PREFS => {
        with_win(|w| w.render());
        let (themed, _) = (prefs::look(Role::Themed), ());
        with_win(|w| {
          let base = nexora_ui::glass::GlassParams::dark((10.0, 10.0), 24.0 * w.dpi);
          let _ = w.popup.panel.set_params(side_glass(prefs::apply(base, &themed)));
        });
        LRESULT(0)
      }
      WM_CLOSE_SELF => {
        prefs::unlisten(hwnd);
        let win = WIN.with(|s| s.borrow_mut().take());
        drop(win);
        let _ = DestroyWindow(hwnd);
        nexora_platform::sys::trim_later(5000);
        log::line("settings", "ditutup");
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
