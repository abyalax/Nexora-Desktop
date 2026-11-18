//! Dock window procedure: raw Win32 messages routed to `Dock` handlers.

use super::*;

fn xy(lp: LPARAM) -> (f32, f32) {
    ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32)
}

pub(super) unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_MOUSEMOVE => {
            let (x, y) = xy(lp);
            with_dock(hwnd, |d| {
                if !d.tracking {
                    let mut t = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    if TrackMouseEvent(&mut t).is_ok() {
                        d.tracking = true;
                        d.update_visibility();
                    }
                }
                d.on_move(x, y);
            });
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_dock(hwnd, |d| d.on_leave());
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let (x, y) = xy(lp);
            with_dock(hwnd, |d| d.on_down(x, y));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let (x, y) = xy(lp);
            with_dock(hwnd, |d| d.on_up(x, y));
            LRESULT(0)
        }
        WM_CAPTURECHANGED => {
            with_dock(hwnd, |d| {
                if d.dragging.take().is_some() {
                    d.press = None;
                    d.apply(None, false);
                    d.bind_expressions();
                }
            });
            LRESULT(0)
        }
        WM_MBUTTONUP => {
            let (x, _) = xy(lp);
            with_dock(hwnd, |d| d.on_middle_click(x));
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            let (x, _) = xy(lp);
            with_dock(hwnd, |d| d.open_menu(x));
            LRESULT(0)
        }
        MENU_PICK => {
            with_dock(hwnd, |d| d.on_menu_pick(wp.0));
            LRESULT(0)
        }
        MENU_CLOSE => {
            with_dock(hwnd, |d| d.on_menu_close());
            LRESULT(0)
        }
        WM_TIMER => {
            with_dock(hwnd, |d| d.on_timer(wp.0));
            LRESULT(0)
        }
        WM_ICONS => {
            with_dock(hwnd, |d| d.on_icons());
            LRESULT(0)
        }
        WM_APPBAR_SYNC => {
            timer(hwnd, TIMER_APPBAR, 400);
            LRESULT(0)
        }
        WM_DROPFILES => {
            let h = windows::Win32::UI::Shell::HDROP(wp.0 as *mut _);
            with_dock(hwnd, |d| d.on_drop(h));
            LRESULT(0)
        }
        crate::prefs::WM_PREFS => {
            with_dock(hwnd, |d| {
                d.apply_prefs();
                d.on_reconfigure();
            });
            LRESULT(0)
        }
        STRIP_ENTER => {
            with_dock(hwnd, |d| d.on_strip_enter());
            LRESULT(0)
        }
        STRIP_LEAVE => {
            with_dock(hwnd, |d| d.on_strip_leave());
            LRESULT(0)
        }
        PREVIEW_ENTER => {
            with_dock(hwnd, |d| {
                d.preview_hover = true;
                kill(hwnd, TIMER_PCLOSE);
            });
            LRESULT(0)
        }
        PREVIEW_LEAVE => {
            with_dock(hwnd, |d| {
                d.preview_hover = false;
                timer(hwnd, TIMER_PCLOSE, 200);
            });
            LRESULT(0)
        }
        PREVIEW_CLICK => {
            let x = wp.0 as isize as i32;
            with_dock(hwnd, |d| d.on_preview_click(x));
            LRESULT(0)
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => {
            timer(hwnd, TIMER_RECONF, 500);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SETTINGCHANGE | WM_POWERBROADCAST => {
            with_dock(hwnd, |d| d.refresh_tier());
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_DESTROY => {
            nacre_platform::shell::appbar::release();
            DOCK_HWND.store(0, Ordering::Relaxed);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
