use crate::prefs;
use nexora_platform::wins::park::{self, Parked};
use nexora_platform::wins::{enumerate, focus, snapshot, tracker};
use nexora_platform::{log, sys};
use nexora_ui::gfx::Gfx;
use nexora_ui::host::{Host, Surface, attach};
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};
use windows::Foundation::TimeSpan;
use windows::UI::Composition::{
  AnimationDelayBehavior, CompositionDrawingSurface, CompositionEasingFunction, CompositionObject, CompositionStretch,
  Compositor, ContainerVisual, SpriteVisual,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_core::{HSTRING, Interface};
use windows_numerics::{Matrix3x2, Vector2, Vector3};

mod open;
mod play;
mod shadow;

use open::*;
use play::*;
use shadow::*;

const WM_FX: u32 = WM_APP + 60;
const T_CAPTURE: usize = 1;
const T_REFRESH: usize = 2;
const T_HIDE: usize = 3;
const T_RESTORE: usize = 4;
const T_OPEN_POLL: usize = 7;
const T_OPEN_END: usize = 8;
const OPEN_WAIT_MS: u128 = 320;

static FX_HWND: AtomicIsize = AtomicIsize::new(0);
static FLAGGED: Mutex<Vec<isize>> = Mutex::new(Vec::new());
static OPEN_PARKED: Mutex<Option<Parked>> = Mutex::new(None);

#[derive(Clone)]
struct Snap {
  hwnd: isize,
  rect: (i32, i32, i32, i32),
  surface: CompositionDrawingSurface,
}

struct Fx {
  hwnd: HWND,
  comp: Compositor,
  gfx: Gfx,
  surface: Surface,
  screen: (i32, i32),
  snap: Option<Snap>,
  parked: Vec<Snap>,
  current: isize,
  sprite: Option<SpriteVisual>,
  genie: Option<ContainerVisual>,
  pending_restore: Option<(isize, isize)>,
  seen: HashSet<isize>,
  opened: HashSet<isize>,
  opening: Option<Opening>,
  shadow: Option<ShadowWin>,
  shadow_target: isize,
  shadow_key: (i32, i32, i32),
  shadow_rect: (i32, i32, i32, i32),
}

const T_SHADOW: usize = 5;

const T_SHADOW_WATCH: usize = 6;

const SHADOW_MARGIN: i32 = 44;

thread_local! {
    static FX: RefCell<Option<Fx>> = const { RefCell::new(None) };
}

fn with_fx<R>(f: impl FnOnce(&mut Fx) -> R) -> Option<R> {
  FX.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn enabled() -> bool {
  prefs::get().anim
}

fn genie_on() -> bool {
  prefs::get().min_effect == 0
}

const START_DELAY: u32 = 40;

fn noop() {}

fn hook(event: u32, hwnd: isize) {
  let h = FX_HWND.load(Ordering::Relaxed);
  if h != 0 {
    unsafe {
      let _ = PostMessageW(Some(HWND(h as *mut _)), WM_FX, WPARAM(event as usize), LPARAM(hwnd));
    }
  }
}

pub fn init(host: &Host) -> windows_core::Result<()> {
  let n = park::restore_file(&open_file());
  if n > 0 {
    log::line("fx", &format!("{n} jendela yang sedang dibuka sesi lalu dikembalikan"));
  }
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraFx");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let screen = sys::screen_size();
    let hwnd = CreateWindowExW(
      WS_EX_NOREDIRECTIONBITMAP
        | WS_EX_TOPMOST
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE
        | WS_EX_TRANSPARENT
        | WS_EX_LAYERED,
      class,
      w!("nexora-fx"),
      WS_POPUP,
      0,
      0,
      screen.0,
      screen.1,
      None,
      None,
      Some(hi),
      None,
    )?;
    let _ = SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), 255, LWA_ALPHA);
    let surface = attach(&host.compositor, hwnd, (screen.0 as f32, screen.1 as f32))?;
    let gfx = Gfx::shared(&host.compositor)?;
    FX_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
    FX.with(|c| {
      *c.borrow_mut() = Some(Fx {
        hwnd,
        comp: host.compositor.clone(),
        gfx,
        surface,
        screen,
        snap: None,
        parked: Vec::new(),
        current: 0,
        sprite: None,
        genie: None,
        pending_restore: None,
        seen: HashSet::new(),
        opened: HashSet::new(),
        opening: None,
        shadow: None,
        shadow_target: 0,
        shadow_key: (0, 0, 0),
        shadow_rect: (0, 0, 0, 0),
      })
    });
    prefs::listen(hwnd);
    tracker::install(noop);
    tracker::install_fx(hook);
    if enabled() {
      SetTimer(Some(hwnd), T_REFRESH, 4000, None);
      let fg = focus::foreground();
      if eligible(fg) {
        flag(fg);
        with_fx(|fx| fx.current = fg);
        SetTimer(Some(hwnd), T_CAPTURE, 600, None);
      }
    }
    {
      let fg = focus::foreground();
      if eligible(fg) {
        with_fx(|fx| fx.update_shadow(fg, true));
      }
    }
    log::line("fx", "siap");
  }
  Ok(())
}

pub fn restore_all() {
  release_parked();
  let list: Vec<isize> = std::mem::take(&mut *FLAGGED.lock().unwrap_or_else(|e| e.into_inner()));
  for h in list {
    snapshot::set_transitions_disabled(h, false);
  }
}

fn flag(h: isize) {
  let mut g = FLAGGED.lock().unwrap_or_else(|e| e.into_inner());
  if !g.contains(&h) {
    if snapshot::set_transitions_disabled(h, true) {
      g.push(h);
      log::line("fx", &format!("transisi DWM dimatikan hwnd={h}"));
    }
    if g.len() > 400 {
      g.retain(|x| focus::is_alive(*x));
    }
  }
}

fn openable(h: isize, screen: (i32, i32)) -> bool {
  if !eligible(h) || enumerate::cloaked(h) || park::is_parked(h) {
    return false;
  }
  unsafe {
    let hw = HWND(h as *mut _);
    if GetAncestor(hw, GA_ROOT) != hw || GetWindow(hw, GW_OWNER).is_ok_and(|o| !o.0.is_null()) {
      return false;
    }
    if IsZoomed(hw).as_bool() || IsIconic(hw).as_bool() {
      return false;
    }
  }
  match snapshot::frame_rect(h) {
    Some((x, y, w, hh)) => w >= 120 && hh >= 80 && !(w >= screen.0 && hh >= screen.1) && x > -2000 && y > -2000,
    None => false,
  }
}

fn eligible(h: isize) -> bool {
  unsafe {
    let hw = HWND(h as *mut _);
    if !IsWindow(Some(hw)).as_bool() || !IsWindowVisible(hw).as_bool() {
      return false;
    }
    let style = GetWindowLongPtrW(hw, GWL_STYLE) as u32;
    let ex = GetWindowLongPtrW(hw, GWL_EXSTYLE) as u32;
    if style & WS_CAPTION.0 != WS_CAPTION.0 || ex & WS_EX_TOOLWINDOW.0 != 0 {
      return false;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hw, Some(&mut pid));
    pid != std::process::id()
  }
}

pub fn restore(target: isize, icon: (i32, i32, i32, i32), dock: isize) -> bool {
  if !enabled() {
    return false;
  }
  with_fx(|fx| {
    if fx.pending_restore.is_some() {
      return false;
    }
    let Some(pos) = fx.parked.iter().position(|p| p.hwnd == target) else {
      return false;
    };
    let s = fx.parked.remove(pos);
    let (x, y, w, h) = s.rect;
    fx.play(s.surface.clone(), icon, (x, y, w, h), Kind::Restore);
    fx.snap = Some(s);
    fx.pending_restore = Some((target, dock));
    unsafe {
      SetTimer(Some(fx.hwnd), T_RESTORE, START_DELAY + Kind::Restore.ms() - 30, None);
    }
    true
  })
  .unwrap_or(false)
}

impl Fx {
  fn apply_prefs(&mut self) {
    let fg = focus::foreground();
    if eligible(fg) {
      self.update_shadow(fg, true);
    } else if !prefs::get().win_shadow {
      self.hide_shadow();
    }
    unsafe {
      if enabled() {
        SetTimer(Some(self.hwnd), T_REFRESH, 4000, None);
        if self.current == 0 {
          let fg = focus::foreground();
          if eligible(fg) {
            flag(fg);
            self.current = fg;
            SetTimer(Some(self.hwnd), T_CAPTURE, 200, None);
          }
        }
      } else {
        let _ = KillTimer(Some(self.hwnd), T_REFRESH);
        self.end_open();
        self.snap = None;
        self.hide_overlay();
        restore_all();
      }
    }
  }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      WM_FX => {
        with_fx(|fx| fx.on_event(wp.0 as u32, lp.0));
        LRESULT(0)
      }
      prefs::WM_PREFS => {
        with_fx(|fx| fx.apply_prefs());
        LRESULT(0)
      }
      WM_TIMER => {
        let id = wp.0;
        match id {
          T_HIDE | T_CAPTURE | T_RESTORE | T_SHADOW | T_OPEN_END => {
            let _ = KillTimer(Some(hwnd), id);
          }
          _ => {}
        }
        with_fx(|fx| match id {
          T_SHADOW => {
            let t = fx.shadow_target;
            if t != 0 {
              fx.update_shadow(t, true);
            }
          }
          T_SHADOW_WATCH => fx.watch_shadow(),
          T_OPEN_POLL => fx.poll_open(),
          T_OPEN_END => fx.end_open(),
          T_CAPTURE | T_REFRESH => fx.capture_current(),
          T_HIDE => {
            fx.hide_overlay();
            fx.pending_restore = None;
          }
          T_RESTORE => {
            if let Some((t, dock)) = fx.pending_restore {
              let ok = focus::activate(t, dock);
              log::line("fx", &format!("restore hwnd={t} ok={ok}"));
            }
          }
          _ => {}
        });
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
