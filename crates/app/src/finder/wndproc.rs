//! Finder bar window procedures and the low-level mouse hook.

use super::*;


pub(super) fn xy(lp: LPARAM) -> (f32, f32) {
    ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32)
}

pub(super) unsafe extern "system" fn bar_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_MOUSEMOVE => {
            let (x, _) = xy(lp);
            with_bar(|b| {
                if !b.tracking {
                    let mut t = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    if TrackMouseEvent(&mut t).is_ok() {
                        b.tracking = true;
                    }
                }
                let id = b.id_at(x);
                if id != b.hover {
                    b.hover = id;
                    if let (Some(open), Some(new)) = (b.open, id) {
                        if new != open && Bar::has_menu(new) {
                            b.open_menu(new);
                        }
                    }
                    b.update_capsule();
                }
            });
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_bar(|b| {
                b.tracking = false;
                b.hover = None;
                b.update_capsule();
            });
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let (x, _) = xy(lp);
            with_bar(|b| {
                if let Some(id) = b.id_at(x) {
                    b.press_feedback();
                    b.click(id);
                }
            });
            LRESULT(0)
        }
        MENU_PICK => {
            with_bar(|b| b.pick(wp.0));
            LRESULT(0)
        }
        MENU_CLOSE => {
            with_bar(|b| b.close_menu());
            LRESULT(0)
        }
        WM_NOTCH => {
            with_bar(|b| {
                b.relayout();
                b.render();
            });
            LRESULT(0)
        }
        WM_TEST_NET => {
            if wp.0 == 3 {
                crate::netmenu::test_toggle_others();
            } else if wp.0 == 4 || wp.0 == 8 {
                let h = with_bar(|b| b.h).unwrap_or(30);
                crate::controlcenter::toggle(h);
            } else if wp.0 == 5 {
                crate::controlcenter::test_open_picker();
            } else if wp.0 == 9 {
                crate::controlcenter::test_enter_edit();
            } else if wp.0 == 7 {
                crate::netmenu::test_click_audio();
            } else {
                let which = if wp.0 == 1 || wp.0 == 6 {
                    crate::netmenu::Which::Bluetooth
                } else {
                    crate::netmenu::Which::Wifi
                };
                with_bar(|b| {
                    if let Some((x, _)) = b.rect_of(WIFI) {
                        crate::netmenu::show(
                            &b.comp,
                            which,
                            crate::netmenu::Anchor::Below(x as i32, b.h),
                            b.dpi,
                        );
                    }
                });
            }
            LRESULT(0)
        }
        m if m == WM_APP + 22 => {
            with_bar(|b| {
                if let Some(r) = &b.radios {
                    while let Ok(s) = r.snapshots.try_recv() {
                        b.wifi = s.wifi;
                        b.bt = s.bluetooth;
                    }
                }
                b.relayout();
                b.render();
            });
            LRESULT(0)
        }
        WM_TIMER if wp.0 == T_SIG => {
            with_bar(|b| {
                let v = nexora_platform::wifi::link().map(|l| l.1);
                if v != b.wifi_bars {
                    b.wifi_bars = v;
                    b.render();
                }
            });
            LRESULT(0)
        }
        WM_TIMER => {
            let id = wp.0;
            if id == T_CLOCK {
                let _ = SetTimer(
                    Some(hwnd),
                    T_CLOCK,
                    time::ms_to_next_minute().max(200),
                    None,
                );
            }
            with_bar(|b| {
                b.relayout();
                b.render();
            });
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub(super) unsafe extern "system" fn ctl_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        prefs::WM_PREFS => {
            apply_prefs();
            LRESULT(0)
        }
        WM_APPCHG => {
            SetTimer(Some(hwnd), T_APP, 90, None);
            LRESULT(0)
        }
        WM_CLICKWIN => {
            let root = wp.0 as isize;
            let mut cls = [0u16; 64];
            let n = GetClassNameW(HWND(root as *mut _), &mut cls);
            let cls = String::from_utf16_lossy(&cls[..n.max(0) as usize]);
            let mut pid = 0u32;
            GetWindowThreadProcessId(HWND(root as *mut _), Some(&mut pid));
            let shell = matches!(
                cls.as_str(),
                "Progman"
                    | "WorkerW"
                    | "Shell_TrayWnd"
                    | "Shell_SecondaryTrayWnd"
                    | "NotifyIconOverflowWindow"
                    | "TopLevelWindowForOverflowXamlIsland"
            );
            if pid != std::process::id() && !shell {
                with_bar(|b| {
                    if b.refresh_app_for(root) {
                        b.relayout();
                        b.render();
                    }
                });
            }
            LRESULT(0)
        }
        WM_TIMER if wp.0 == T_FS => {
            let _ = KillTimer(Some(hwnd), T_FS);
            with_bar(|b| {
                b.check_fullscreen();
                b.recheck_color();
            });
            LRESULT(0)
        }
        WM_TIMER if wp.0 == T_APP => {
            let _ = KillTimer(Some(hwnd), T_APP);
            with_bar(|b| {
                if b.refresh_app() {
                    b.relayout();
                    b.render();
                } else {
                    b.recheck_color();
                }
            });
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub(super) unsafe extern "system" fn ll_mouse(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        let m = wp.0 as u32;
        if m == WM_LBUTTONDOWN || m == WM_RBUTTONDOWN || m == WM_MBUTTONDOWN {
            let ctl = BAR_CTL.load(std::sync::atomic::Ordering::Relaxed);
            if ctl != 0 {
                let pt = (*(lp.0 as *const MSLLHOOKSTRUCT)).pt;
                let h = WindowFromPoint(pt);
                if !h.0.is_null() {
                    let root = GetAncestor(h, GA_ROOT);
                    if !root.0.is_null() {
                        let _ = PostMessageW(
                            Some(HWND(ctl as *mut _)),
                            WM_CLICKWIN,
                            WPARAM(root.0 as usize),
                            LPARAM(0),
                        );
                    }
                }
            }
        }
    }
    CallNextHookEx(None, code, wp, lp)
}

pub(super) fn loc_hook(ev: u32, hwnd: isize) {
    if ev == tracker::EV_LOCATION && hwnd != 0 && hwnd == focus::foreground() {
        CTL.with(|c| {
            if let Some(ctl) = c.borrow().as_ref() {
                unsafe {
                    SetTimer(Some(ctl.hwnd), T_FS, 200, None);
                }
            }
        });
    }
}
