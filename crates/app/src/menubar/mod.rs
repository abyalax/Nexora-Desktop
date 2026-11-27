mod actions;
mod appearance;
mod draw;
mod layout;
pub mod logo;
mod menus;
mod wndproc;

use crate::menu::{Item, MENU_CLOSE, MENU_PICK, Menu};
use crate::popup::Popup;
use crate::{prefs, services};
use nexora_core::menubar::items::{
  self, APP, BATT, BarInputs, CLOCK, CTRL, ITEM_MARGIN, LOGO, M_EDIT, M_FILE, M_HELP, M_VIEW, M_WIN, Menus, NATIVE0,
  SEARCH, STAGEI, Show, TextMetrics, WIFI,
};
use nexora_core::menubar::layout::{self as lay, Gap, Placed};
use nexora_platform::icons::Bitmap;
use nexora_platform::text::{ICON_FACE, Style, measure, render, render_face};
use nexora_platform::wins::{enumerate, focus, tracker};
use nexora_platform::{keys, launch, log, radio, shell, sys, time};
use nexora_ui::gfx::Gfx;
use nexora_ui::glass::Tier;
use std::cell::RefCell;
use std::collections::HashMap;
use windows::UI::Color;
use windows::UI::Composition::{
  CompositionDrawingSurface, CompositionStretch, Compositor, ContainerVisual, SpriteVisual,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_numerics::{Vector2, Vector3};

const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_APPCHG: u32 = WM_APP + 70;
const WM_CLICKWIN: u32 = WM_APP + 74;
static MOUSE_HOOK: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static BAR_CTL: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
const WM_NOTCH: u32 = WM_APP + 71;
const WM_TEST_NET: u32 = WM_APP + 75;
const T_CLOCK: usize = 1;
const T_STATUS: usize = 2;
const T_APP: usize = 3;
const T_FS: usize = 4;
const T_SIG: usize = 5;
const BAR_W: i32 = 600;
const APP_W: i32 = 800;
const NATIVE_CMD: usize = 100_000;

struct Bar {
  popup: Popup,
  comp: Compositor,
  gfx: Gfx,
  dpi: f32,
  screen: (i32, i32),
  h: i32,
  layer: ContainerVisual,
  capsule: SpriteVisual,
  bg: SpriteVisual,
  hair: SpriteVisual,
  cache: HashMap<String, (CompositionDrawingSurface, i32, i32)>,
  left: Vec<Placed>,
  right: Vec<Placed>,
  app: String,
  hidden: Vec<isize>,
  menu: Menu,
  open: Option<u32>,
  hover: Option<u32>,
  tracking: bool,
  wifi: Option<bool>,
  wifi_bars: Option<u8>,
  bt: Option<bool>,
  radios: Option<radio::Radios>,
  dark_text: bool,
  shadow: bool,
  native: Vec<nexora_platform::menus::Top>,
  native_hwnd: isize,
  fullscreen: bool,
}

struct Ctl {
  hwnd: HWND,
  comp: Compositor,
}

thread_local! {
    static BAR: RefCell<Option<Box<Bar>>> = const { RefCell::new(None) };
    static CTL: RefCell<Option<Ctl>> = const { RefCell::new(None) };
}

fn with_bar<R>(f: impl FnOnce(&mut Bar) -> R) -> Option<R> {
  BAR.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(|b| f(b))))
}

pub fn refresh_radios() {
  with_bar(|b| {
    if let Some(r) = &b.radios {
      let _ = r.commands.send(radio::Cmd::Refresh);
    }
  });
}

fn mt(key: &'static str) -> String {
  let t = prefs::tr(key);
  t.strip_prefix("Menu:").unwrap_or(t).to_string()
}

fn recolor(b: &Bitmap, rgb: [u8; 3]) -> Vec<u8> {
  let mut out = b.bgra.clone();
  for px in out.chunks_exact_mut(4) {
    let a = px[3] as u32;
    px[0] = (rgb[2] as u32 * a / 255) as u8;
    px[1] = (rgb[1] as u32 * a / 255) as u8;
    px[2] = (rgb[0] as u32 * a / 255) as u8;
  }
  out
}

fn noop_fg() {
  CTL.with(|c| {
    if let Some(ctl) = c.borrow().as_ref() {
      unsafe {
        let _ = PostMessageW(Some(ctl.hwnd), WM_APPCHG, WPARAM(0), LPARAM(0));
      }
    }
  });
}

fn is_fullscreen(hwnd: isize) -> bool {
  unsafe {
    let h = HWND(hwnd as *mut _);
    let mut cls = [0u16; 64];
    let n = GetClassNameW(h, &mut cls);
    let cls = String::from_utf16_lossy(&cls[..n.max(0) as usize]);
    if cls == "Progman" || cls == "WorkerW" || cls == "Shell_TrayWnd" {
      return false;
    }
    let mut r = windows::Win32::Foundation::RECT::default();
    if GetWindowRect(h, &mut r).is_err() {
      return false;
    }
    let (sw, sh) = sys::screen_size();
    r.left <= 0 && r.top <= 0 && r.right >= sw && r.bottom >= sh
  }
}

pub fn init(comp: &Compositor) -> windows_core::Result<()> {
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraBarCtl");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(ctl_proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let hwnd = CreateWindowExW(
      WINDOW_EX_STYLE(0),
      class,
      w!("nexora-bar-ctl"),
      WS_POPUP,
      0,
      0,
      0,
      0,
      Some(HWND_MESSAGE),
      None,
      Some(hi),
      None,
    )?;
    CTL.with(|c| {
      *c.borrow_mut() = Some(Ctl {
        hwnd,
        comp: comp.clone(),
      })
    });
    prefs::listen(hwnd);
    tracker::install(noop_fg);
    tracker::install_fx(loc_hook);
    BAR_CTL.store(hwnd.0 as isize, std::sync::atomic::Ordering::Relaxed);
    if let Ok(h) = SetWindowsHookExW(WH_MOUSE_LL, Some(ll_mouse), None, 0) {
      MOUSE_HOOK.store(h.0 as isize, std::sync::atomic::Ordering::Relaxed);
    }
    apply_prefs();
  }
  Ok(())
}

fn apply_prefs() {
  let want = prefs::get().menubar;
  let have = BAR.with(|c| c.borrow().is_some());
  if want && !have {
    let comp = CTL.with(|c| c.borrow().as_ref().map(|c| c.comp.clone()));
    if let Some(comp) = comp {
      match create(&comp) {
        Ok(b) => {
          let hwnd = b.popup.hwnd;
          BAR.with(|c| *c.borrow_mut() = Some(b));
          services::register_bar(hwnd);
          with_bar(|b| {
            b.refresh_app();
            b.relayout();
            b.render();
          });
          services::post_island(WM_APP + 72);
          CTL.with(|c| {
            if let Some(ctl) = c.borrow().as_ref() {
              unsafe {
                SetTimer(Some(ctl.hwnd), T_APP, 900, None);
              }
            }
          });
          log::line("menubar", "dibuat");
          crate::hotcorners::raise();
        }
        Err(e) => log::line("menubar", &format!("gagal: {e}")),
      }
    }
  } else if !want && have {
    let b = BAR.with(|c| c.borrow_mut().take());
    if let Some(b) = b {
      b.menu.hide();
      unsafe {
        let _ = DestroyWindow(b.popup.hwnd);
      }
    }
    shell::appbar::release_top();
    log::line("menubar", "dihancurkan");
    services::post_island(crate::prefs::WM_PREFS);
  } else if want {
    let rescale = with_bar(|b| {
      let want_dpi = unsafe { GetDpiForWindow(b.popup.hwnd) }.max(96) as f32 / 96.0 * prefs::ui_scale();
      (want_dpi - b.dpi).abs() > 0.01
    })
    .unwrap_or(false);
    if rescale {
      let b = BAR.with(|c| c.borrow_mut().take());
      if let Some(b) = b {
        b.menu.hide();
        unsafe {
          let _ = DestroyWindow(b.popup.hwnd);
        }
      }
      shell::appbar::release_top();
      apply_prefs();
      return;
    }
    with_bar(|b| {
      b.apply_look();
      b.relayout();
      b.render();
    });
  }
}

fn create(comp: &Compositor) -> windows_core::Result<Box<Bar>> {
  let screen = sys::screen_size();
  let glass = prefs::apply(
    nexora_ui::glass::GlassParams::dark((10.0, 10.0), 0.0),
    &prefs::bar_look(),
  );
  let mut popup = Popup::new(comp, w!("NexoraMenuBar"), Some(bar_proc), WINDOW_EX_STYLE(0), glass)?;
  let dpi = unsafe { GetDpiForWindow(popup.hwnd) }.max(96) as f32 / 96.0 * prefs::ui_scale();
  let notch_h = services::notch().map(|(_, _, h)| h as f32 / dpi);
  let h = (notch_h.unwrap_or(26.0) * dpi).round() as i32;
  let tier = {
    let c = sys::conditions();
    if !c.transparency || c.battery_saver || c.remote_session {
      Tier::Solid
    } else {
      Tier::Full
    }
  };
  let _ = popup.panel.set_tier(tier);
  popup.set_rect(0, 0, screen.0, h)?;
  popup.panel.set_radius(0.0)?;
  popup.panel.set_rect(0.0, 0.0, 0.0, 0.0)?;
  let bg = comp.CreateSpriteVisual()?;
  popup.content.Children()?.InsertAtTop(&bg)?;
  let hair = comp.CreateSpriteVisual()?;
  hair.SetOpacity(0.0)?;
  popup.content.Children()?.InsertAtTop(&hair)?;
  let capsule = comp.CreateSpriteVisual()?;
  capsule.SetOpacity(0.0)?;
  popup.content.Children()?.InsertAtTop(&capsule)?;
  let layer = comp.CreateContainerVisual()?;
  popup.content.Children()?.InsertAtTop(&layer)?;
  let gfx = Gfx::shared(comp)?;
  let menu = Menu::new(comp, popup.hwnd, dpi, screen, tier)?;
  popup.show();
  shell::appbar::reserve_top(popup.hwnd, screen.0, h);
  unsafe {
    SetTimer(Some(popup.hwnd), T_CLOCK, time::ms_to_next_minute().max(200), None);
    SetTimer(Some(popup.hwnd), T_STATUS, 60_000, None);
    SetTimer(Some(popup.hwnd), T_SIG, 4_000, None);
  }
  let hw = popup.hwnd.0 as isize;
  let radios = radio::spawn(move || unsafe {
    let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_APP + 22, WPARAM(0), LPARAM(0));
  });
  if let Ok(v) = std::env::var("NACRE_TEST_NETMENU") {
    std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_millis(2500));
      let code = match v.as_str() {
        "bt" => 1,
        "wifi-others" => 2,
        "cc" => 4,
        "bt-click" => 6,
        "cc-edit" => 8,
        _ => 0,
      };
      unsafe {
        let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_TEST_NET, WPARAM(code), LPARAM(0));
      }
      if code == 2 || code == 4 || code == 6 || code == 8 {
        std::thread::sleep(std::time::Duration::from_millis(2500));
        unsafe {
          let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_TEST_NET, WPARAM(code + 1), LPARAM(0));
        }
      }
    });
  }
  let mut bar = Box::new(Bar {
    popup,
    comp: comp.clone(),
    gfx,
    dpi,
    screen,
    h,
    layer,
    capsule,
    bg,
    hair,
    cache: HashMap::new(),
    left: vec![],
    right: vec![],
    app: String::new(),
    hidden: vec![],
    menu,
    open: None,
    hover: None,
    tracking: false,
    wifi: None,
    wifi_bars: nexora_platform::wifi::link().map(|l| l.1),
    bt: None,
    radios: Some(radios),
    dark_text: false,
    shadow: false,
    native: vec![],
    native_hwnd: 0,
    fullscreen: false,
  });
  bar.apply_bg();
  Ok(bar)
}

use wndproc::{bar_proc, ctl_proc, ll_mouse, loc_hook};
