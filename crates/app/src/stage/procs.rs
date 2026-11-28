use super::*;

pub(super) unsafe extern "system" fn ctl_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      prefs::WM_PREFS => {
        sync();
        LRESULT(0)
      }
      WM_FG => {
        SetTimer(Some(hwnd), T_FG, 140, None);
        LRESULT(0)
      }
      WM_LOC => {
        SetTimer(Some(hwnd), T_OVERLAP, 160, None);
        LRESULT(0)
      }
      WM_EDGE_IN => {
        with(|s| {
          s.peek = true;
          s.set_strip_shown(true);
        });
        LRESULT(0)
      }
      WM_EDGE_OUT => {
        SetTimer(Some(hwnd), T_PEEK, 700, None);
        LRESULT(0)
      }
      WM_TIMER => {
        match wp.0 {
          T_FG => {
            let _ = KillTimer(Some(hwnd), T_FG);
            with(|s| {
              if s.active {
                s.refresh();
              }
            });
          }
          T_FLY => {
            with(|s| s.finish_fly());
          }
          T_FLYEND => {
            with(|s| s.kill_dying());
          }
          T_LIVE => {
            with(|s| {
              if s.active {
                s.poll_live();
              }
            });
          }
          T_SHOT => {
            with(|s| {
              if s.active {
                s.refresh_shots();
              }
            });
          }
          T_OVERLAP => {
            let _ = KillTimer(Some(hwnd), T_OVERLAP);
            with(|s| s.check_overlap());
          }
          T_HIDE => {
            let _ = KillTimer(Some(hwnd), T_HIDE);
            with(|s| {
              if let Some(st) = &s.strip {
                if !st.shown {
                  st.popup.hide();
                }
              }
            });
          }
          T_PEEK => {
            let _ = KillTimer(Some(hwnd), T_PEEK);
            with(|s| {
              let over = s.strip.as_ref().map(|st| st.hover.is_some()).unwrap_or(false);
              if !over {
                s.peek = false;
                s.check_overlap();
              }
            });
          }
          _ => {}
        }
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}

pub(super) unsafe extern "system" fn fly_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}

pub(super) unsafe extern "system" fn strip_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    let (x, y) = ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32);
    match msg {
      WM_MOUSEMOVE => {
        with(|s| {
          let Some(st) = s.strip.as_mut() else { return };
          if !st.tracking {
            let mut t = TRACKMOUSEEVENT {
              cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
              dwFlags: TME_LEAVE,
              hwndTrack: hwnd,
              dwHoverTime: 0,
            };
            if TrackMouseEvent(&mut t).is_ok() {
              st.tracking = true;
            }
          }
          let i = st.slots.iter().position(|sl| {
            x >= sl.rect.0 - 8.0 && x < sl.rect.0 + sl.rect.2 + 8.0 && y >= sl.rect.1 && y < sl.rect.1 + sl.rect.3
          });
          s.set_hover(i);
        });
        LRESULT(0)
      }
      WM_MOUSELEAVE => {
        with(|s| {
          if let Some(st) = s.strip.as_mut() {
            st.tracking = false;
          }
          s.set_hover(None);
          if s.peek {
            SetTimer(Some(s.ctl), T_PEEK, 500, None);
          }
        });
        LRESULT(0)
      }
      WM_LBUTTONUP => {
        with(|s| {
          let Some(st) = s.strip.as_ref() else { return };
          let Some(sl) = st.slots.iter().find(|sl| {
            x >= sl.rect.0 - 8.0 && x < sl.rect.0 + sl.rect.2 + 8.0 && y >= sl.rect.1 && y < sl.rect.1 + sl.rect.3
          }) else {
            return;
          };
          let shift = windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(
            windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT.0 as i32,
          ) < 0;
          let apps = sl.apps.clone();
          if shift {
            let ch = s.model.add_to_stage(&apps[0]);
            let wins = enumerate::list(std::process::id());
            s.apply(ch, &wins, 0);
          } else {
            s.focus_app(&apps[0], 0);
          }
        });
        LRESULT(0)
      }
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
