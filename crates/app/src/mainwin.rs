use crate::paths;
use crate::prefs;
use crate::startup::single_instance;
use nexora_platform::shell::taskbar;
use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
  NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

pub const WM_TRAY: u32 = WM_APP + 1;
const ID_TOGGLE_TASKBAR: usize = 1;
pub const ID_QUIT: usize = 2;
const TIMER_EXIT: usize = 1;

static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

pub fn create() -> windows::core::Result<HWND> {
  unsafe {
    let hi = GetModuleHandleW(None)?.into();
    let class = w!("NacreMain");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    TASKBAR_CREATED.store(RegisterWindowMessageW(w!("TaskbarCreated")), Ordering::Relaxed);
    CreateWindowExW(
      WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
      class,
      w!("nexora"),
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      Some(hi),
      None,
    )
  }
}

pub fn set_exit_timer(hwnd: HWND, seconds: u32) {
  unsafe {
    SetTimer(Some(hwnd), TIMER_EXIT, seconds * 1000, None);
  }
}

fn tray_icon() -> HICON {
  unsafe {
    let hi = GetModuleHandleW(None).map(|h| HINSTANCE(h.0)).ok();
    let (cx, cy) = (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON));
    match LoadImageW(hi, PCWSTR(1 as *const u16), IMAGE_ICON, cx, cy, LR_SHARED) {
      Ok(h) if !h.is_invalid() => HICON(h.0),
      _ => LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
    }
  }
}

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
  let mut nid = NOTIFYICONDATAW {
    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
    hWnd: hwnd,
    uID: 1,
    uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
    uCallbackMessage: WM_TRAY,
    hIcon: tray_icon(),
    ..Default::default()
  };
  for (i, c) in "nexora".encode_utf16().enumerate() {
    nid.szTip[i] = c;
  }
  nid
}

pub fn add_tray(hwnd: HWND) {
  let nid = tray_data(hwnd);
  unsafe {
    let _ = Shell_NotifyIconW(NIM_ADD, &nid);
  }
}

fn remove_tray(hwnd: HWND) {
  let nid = tray_data(hwnd);
  unsafe {
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
  }
}

pub static TAKEOVER_ALLOWED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn apply_autostart() {
  use nexora_platform::startup;
  let method = crate::prefs::get().extra.start_method.min(2);
  let Ok(exe) = std::env::current_exe() else {
    return;
  };
  // Keep the two supported mechanisms mutually exclusive, including when the user switches
  // methods or disables startup after a previous version created the other entry.
  let registry = startup::set(method == 1, &exe);
  let scheduled = startup::set_scheduled_task(method == 2, &exe);
  nexora_platform::log::line(
    "startup",
    &format!("method={method} registry={registry:?} scheduled={scheduled:?}"),
  );
}

pub fn set_taskbar_hidden(on: bool) {
  if on {
    let displays = nexora_platform::monitors::list();
    let domain_displays: Vec<_> = displays
      .iter()
      .map(|monitor| nexora_core::shared::monitors::Monitor {
        id: &monitor.id,
        bounds: nexora_core::shared::geometry::Rect::new(
          monitor.bounds.0 as f32,
          monitor.bounds.1 as f32,
          (monitor.bounds.2 - monitor.bounds.0) as f32,
          (monitor.bounds.3 - monitor.bounds.1) as f32,
        ),
        primary: monitor.primary,
      })
      .collect();
    let dock_monitor = displays
      .iter()
      .find(|monitor| monitor.primary)
      .map_or("", |monitor| monitor.id.as_str());
    let scope = nexora_core::window_management::taskbar::Scope::from_index(prefs::get().extra.taskbar_scope as u8);
    let hidden_monitor_ids = nexora_core::window_management::taskbar::to_hide(scope, &domain_displays, dock_monitor)
      .into_iter()
      .filter_map(|index| displays.get(index).map(|monitor| monitor.id.clone()))
      .collect();
    if let Err(e) = taskbar::hide(&paths::marker(), &paths::guard_exe(), hidden_monitor_ids) {
      nexora_platform::log::line("taskbar", &format!("hide gagal: {e}"));
    }
  } else {
    taskbar::restore();
  }
  crate::dock::request_appbar_resync();
}

fn show_menu(hwnd: HWND) {
  unsafe {
    let Ok(menu) = CreatePopupMenu() else { return };
    let check = if taskbar::is_hidden() { MF_CHECKED } else { MF_UNCHECKED };
    let _ = AppendMenuW(
      menu,
      MF_STRING | check,
      ID_TOGGLE_TASKBAR,
      w!("Sembunyikan taskbar (uji)"),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, ID_QUIT, w!("Keluar dari nexora"));
    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_BOTTOMALIGN, pt.x, pt.y, Some(0), hwnd, None);
    let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
  }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    let tc = TASKBAR_CREATED.load(Ordering::Relaxed);
    if tc != 0 && msg == tc {
      add_tray(hwnd);
      taskbar::reassert();
      crate::dock::request_appbar_resync();
      nexora_platform::log::line("shell", "TaskbarCreated");
      return LRESULT(0);
    }
    match msg {
      #[cfg(all(feature = "agent-debug", debug_assertions))]
      crate::agent::WM_AGENT_OPEN => {
        match wp.0 {
          1 => crate::settings_win::open_agent((lp.0 > 0).then_some(lp.0 as u8)),
          2 => crate::spotlight::toggle(),
          3 => crate::launchpad::toggle(),
          4 => crate::controlcenter::toggle(0),
          _ => {}
        }
        LRESULT(0)
      }
      crate::prefs::WM_PREFS => {
        apply_autostart();
        if TAKEOVER_ALLOWED.load(Ordering::Relaxed) {
          let p = crate::prefs::get();
          let want = p.takeover && p.dock_enabled;
          if want != taskbar::is_hidden() {
            nexora_platform::log::line("taskbar", &format!("pengaturan mode penuh → {want}"));
            set_taskbar_hidden(want);
          }
        }
        LRESULT(0)
      }
      WM_TRAY => {
        if (lp.0 as u32 & 0xFFFF) == WM_RBUTTONUP || (lp.0 as u32 & 0xFFFF) == WM_CONTEXTMENU {
          show_menu(hwnd);
        }
        LRESULT(0)
      }
      WM_COMMAND => {
        match wp.0 & 0xFFFF {
          ID_TOGGLE_TASKBAR => set_taskbar_hidden(!taskbar::is_hidden()),
          ID_QUIT => {
            let _ = DestroyWindow(hwnd);
          }
          _ => {}
        }
        LRESULT(0)
      }
      single_instance::WM_WAKE => {
        nexora_platform::log::line("instance", "peluncuran kedua terdeteksi");
        LRESULT(0)
      }
      WM_TIMER if wp.0 == TIMER_EXIT => {
        let _ = DestroyWindow(hwnd);
        LRESULT(0)
      }
      WM_ENDSESSION => {
        if wp.0 != 0 {
          taskbar::restore();
        }
        LRESULT(0)
      }
      WM_DESTROY => {
        crate::fx::restore_all();
        taskbar::restore();
        remove_tray(hwnd);
        PostQuitMessage(0);
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
