//! Window procedure: translates Win32 messages into `Notch` calls.

use super::shelf::after_drop;
use super::*;

pub(super) fn xy(lp: LPARAM) -> (f32, f32) {
  let gap = f32::from_bits(ISLAND_GAP.load(std::sync::atomic::Ordering::Relaxed));
  ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32 - gap)
}

pub(super) unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      WM_LBUTTONDOWN => {
        let (x, y) = xy(lp);
        if with(hwnd, |d| d.on_down(x, y)).unwrap_or(false) {
          let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetCapture(hwnd);
        }
        LRESULT(0)
      }
      WM_MOUSEMOVE => {
        let (x, y) = xy(lp);
        with(hwnd, |d| {
          d.on_drag(x);
          d.on_move();
          d.on_hover(x, y);
        });
        LRESULT(0)
      }
      WM_MEDIA => {
        with(hwnd, |d| d.on_media());
        LRESULT(0)
      }
      WM_BANDS => {
        with(hwnd, |d| d.on_bands());
        LRESULT(0)
      }
      WM_LYRICS => {
        with(hwnd, |d| d.on_lyrics());
        LRESULT(0)
      }
      WM_DOWNLOAD => {
        with(hwnd, |d| d.on_download());
        LRESULT(0)
      }
      WM_AGENDA => {
        with(hwnd, |d| d.on_agenda());
        LRESULT(0)
      }
      WM_SEND => {
        with(hwnd, |d| d.on_send());
        LRESULT(0)
      }
      WM_DND => {
        with(hwnd, |d| d.on_dnd(wp.0, lp));
        LRESULT(0)
      }
      WM_AFTERDROP => {
        let pending = with(hwnd, |d| std::mem::take(&mut d.pending)).unwrap_or_default();
        after_drop(&pending, wp.0);
        LRESULT(0)
      }
      WM_RBUTTONUP => {
        let (x, y) = xy(lp);
        with(hwnd, |d| d.on_right_click(x, y));
        LRESULT(0)
      }
      m if m == MENU_PICK => {
        let id = wp.0;
        with(hwnd, |d| d.menu_pick(id));
        LRESULT(0)
      }
      m if m == MENU_CLOSE => {
        with(hwnd, |d| {
          if let Some(mn) = &d.menu {
            mn.hide();
          }
          if !d.tracking && matches!(d.mode, Mode::Open(_)) {
            timer(hwnd, T_LEAVE, motion::LEAVE_GRACE_MS);
          }
        });
        LRESULT(0)
      }
      WM_MOUSELEAVE => {
        with(hwnd, |d| d.on_leave());
        LRESULT(0)
      }
      WM_LBUTTONUP => {
        let (x, y) = xy(lp);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
        if with(hwnd, |d| d.on_up()).unwrap_or(false) {
          return LRESULT(0);
        }
        with(hwnd, |d| d.on_click(x, y));
        LRESULT(0)
      }
      WM_MOUSEWHEEL => {
        let delta = (wp.0 >> 16) as i16;
        with(hwnd, |d| d.on_wheel(delta));
        LRESULT(0)
      }
      STRIP_ENTER => {
        with(hwnd, |d| {
          if d.vis != Vis::Always {
            timer(hwnd, T_EDGE, 200);
          }
        });
        LRESULT(0)
      }
      STRIP_LEAVE => {
        with(hwnd, |d| {
          kill(hwnd, T_EDGE);
          if d.edge_armed {
            d.edge_armed = false;
            d.grace = true;
            timer(hwnd, T_GRACE, 800);
            d.update_visibility();
          }
        });
        LRESULT(0)
      }
      WM_TIMER => {
        with(hwnd, |d| d.on_timer(wp.0));
        LRESULT(0)
      }
      crate::prefs::WM_PREFS => {
        with(hwnd, |d| {
          d.apply_prefs();
          d.on_reconfigure();
        });
        LRESULT(0)
      }
      WM_TOPMOST => {
        let _ = SetWindowPos(
          hwnd,
          Some(HWND_TOPMOST),
          0,
          0,
          0,
          0,
          SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        with(hwnd, |d| d.place_shadow_window());
        LRESULT(0)
      }
      WM_OPEN => {
        with(hwnd, |d| {
          if !d.off && !matches!(d.mode, Mode::Open(_)) {
            d.dispatch(Event::Click);
          }
        });
        LRESULT(0)
      }
      WM_DISPLAYCHANGE | WM_DPICHANGED => {
        timer(hwnd, T_RECONF, 500);
        DefWindowProcW(hwnd, msg, wp, lp)
      }
      WM_POWERBROADCAST => {
        with(hwnd, |d| {
          d.refresh_tier();
          d.on_power();
        });
        DefWindowProcW(hwnd, msg, wp, lp)
      }
      WM_SETTINGCHANGE => {
        with(hwnd, |d| d.refresh_tier());
        DefWindowProcW(hwnd, msg, wp, lp)
      }
      WM_DESTROY => {
        NOTCH_HWND.store(0, Ordering::Relaxed);
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
