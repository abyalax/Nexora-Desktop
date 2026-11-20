mod commands;
mod custom_icons;
mod hover;
mod input;
mod pinning;
mod pins;
mod preview;
mod render;
mod timers;
mod tooltip;
mod visibility;
mod window;
mod wndproc;

use crate::menu::{MENU_CLOSE, MENU_PICK, Menu};
use crate::paths;
use crate::strip::Strip;
use nexora_core::dock::entries::{self, Entry, LAUNCHPAD_KEY, START_KEY, TRASH_PATH};
use nexora_core::dock::geometry;
use nexora_core::dock::items::{Item, Pin, key_of, merge, package_family};
use nexora_core::dock::layout::{Layout, Metrics, Slot, hit, layout};
use nexora_core::dock::recents;
use nexora_core::dock::reorder;
use nexora_core::dock::ring::next_in_ring;
use nexora_core::visibility::policy::Mode;
use nexora_platform::icons;
use nexora_platform::wins::overlap::{self, Overlap};
use nexora_platform::wins::{enumerate, focus, tracker};
use nexora_platform::{launch, log, sys};
use nexora_ui::features::dock::view::{Appearance, Kind, View, spring};
use nexora_ui::gfx::Gfx;
use nexora_ui::glass::{GlassPanel, GlassParams, Tier};
use nexora_ui::host::{Host, Surface};
use preview::{PREVIEW_CLICK, PREVIEW_ENTER, PREVIEW_LEAVE, Preview};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::{Duration, Instant};
use tooltip::Tooltip;
use windows::UI::Composition::{
  CompositionDrawingSurface, CompositionObject, CompositionPropertySet, Compositor, ContainerVisual,
  ExpressionAnimation, SpringScalarNaturalMotionAnimation,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateRectRgn, SetWindowRgn};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_core::{HSTRING, Interface};
use windows_numerics::Vector3;

pub const WM_ICONS: u32 = WM_APP + 3;
pub const WM_APPBAR_SYNC: u32 = WM_APP + 4;
const STRIP_ENTER: u32 = WM_APP + 5;
const STRIP_LEAVE: u32 = WM_APP + 6;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_SETTINGCHANGE: u32 = 0x001A;
const WM_POWERBROADCAST: u32 = 0x0218;

const TIMER_WINDOWS: usize = 100;
const TIMER_APPBAR: usize = 101;
const TIMER_TIP: usize = 102;
const TIMER_PREVIEW: usize = 103;
const TIMER_GRACE: usize = 104;
const TIMER_DWELL: usize = 105;
const TIMER_PCLOSE: usize = 106;
const TIMER_HIDEWIN: usize = 107;
const TIMER_START: usize = 108;
const TIMER_REGION: usize = 109;
const TIMER_RECONF: usize = 110;
const TIMER_TRASH: usize = 111;
const TIMER_REVEAL: usize = 112;
const TRASH_POLL_MS: u32 = 4000;
const WM_DISPLAYCHANGE: u32 = 0x007E;
const WM_DPICHANGED: u32 = 0x02E0;

static DOCK_HWND: AtomicIsize = AtomicIsize::new(0);
static BUSY: AtomicBool = AtomicBool::new(false);

const CMD_OPEN: usize = 1;
const CMD_PIN: usize = 2;
const CMD_CLOSE: usize = 3;
const CMD_QUIT: usize = 4;
const CMD_SETTINGS: usize = 5;
const CMD_ICON: usize = 6;
const CMD_ICON_RESET: usize = 7;
const CMD_ADD: usize = 8;
const CMD_TRASH_OPEN: usize = 9;
const CMD_TRASH_EMPTY: usize = 10;
const CMD_STACK_FOLDER: usize = 11;
const CMD_FORGET: usize = 12;
const CMD_STACK: usize = 100;
const DOWNLOADS_KEY: &str = "\u{0}downloads";

fn downloads_key() -> String {
  nexora_platform::downloads::folder()
    .map(|p| p.to_string_lossy().to_string())
    .unwrap_or_default()
}

pub struct Options {
  pub appbar: bool,
  pub mode: Mode,
  pub glass: Option<Tier>,
}

struct MenuInfo {
  item: Option<Item>,
  trash: bool,
  special: Option<&'static str>,
}

struct Press {
  slot: usize,
  x: f32,
  y: f32,
  pinned_item: Option<usize>,
}

pub struct Dock {
  hwnd: HWND,
  main_hwnd: HWND,
  comp: Compositor,
  gfx: Gfx,
  _surface: Surface,
  stage: ContainerVisual,
  stage_anim: SpringScalarNaturalMotionAnimation,
  panel: GlassPanel,
  content: ContainerVisual,
  props: CompositionPropertySet,
  k_anim: SpringScalarNaturalMotionAnimation,
  exprs_ok: bool,
  k_on: bool,

  dpi: f32,
  m: Metrics,
  screen: (i32, i32),
  win: (i32, i32),
  win_pos: (i32, i32),
  card_y: f32,
  card_h: f32,
  margin: f32,
  off: bool,
  icon_px: i32,
  icon_y: f32,
  dot_y: f32,
  pad_side: f32,
  opts: Options,

  pins: Vec<Pin>,
  pins_path: PathBuf,
  recents: Vec<Pin>,
  recents_path: PathBuf,
  stack_files: Vec<String>,
  items: Vec<Item>,
  signature: Vec<(String, bool, usize, String)>,
  icon_key: (nexora_core::settings::IconStyle, u8, bool, bool),
  seen: Vec<String>,

  entries: Vec<Entry>,
  slots: Vec<Slot>,
  views: Vec<View>,
  layout: Layout,

  surfaces: HashMap<String, CompositionDrawingSurface>,
  requested: HashSet<String>,
  trash_full: bool,
  worker: icons::Worker,
  plates: bool,

  tracking: bool,
  region_full: bool,
  cursor: Option<f32>,
  last_apply: Instant,
  last_card: (f32, f32, f32, f32),
  start_open_at_press: bool,
  hover: Option<usize>,
  press: Option<Press>,
  dragging: Option<usize>,

  shown: bool,
  starting: bool,
  revealing: bool,
  overlap: Overlap,
  grace: bool,
  edge_armed: bool,
  menu_open: bool,
  preview_hover: bool,
  tier: Tier,
  strip: Option<Strip>,
  tooltip: Option<Tooltip>,
  preview: Option<Preview>,
  menu: Option<Menu>,
  pending_menu: Option<MenuInfo>,
}

fn f32_ref(v: f32) -> windows_core::Result<windows::Foundation::IReference<f32>> {
  windows::Foundation::PropertyValue::CreateSingle(v)?.cast()
}

fn with_dock<R>(hwnd: HWND, f: impl FnOnce(&mut Dock) -> R) -> Option<R> {
  let p = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut Dock;
  if p.is_null() || BUSY.swap(true, Ordering::Acquire) {
    return None;
  }
  let r = f(unsafe { &mut *p });
  BUSY.store(false, Ordering::Release);
  Some(r)
}

fn timer(hwnd: HWND, id: usize, ms: u32) {
  unsafe {
    SetTimer(Some(hwnd), id, ms, None);
  }
}

fn kill(hwnd: HWND, id: usize) {
  unsafe {
    let _ = KillTimer(Some(hwnd), id);
  }
}

pub fn request_appbar_resync() {
  let h = DOCK_HWND.load(Ordering::Relaxed);
  if h != 0 {
    unsafe {
      let _ = PostMessageW(Some(HWND(h as *mut _)), WM_APPBAR_SYNC, WPARAM(0), LPARAM(0));
    }
  }
}

fn on_windows_changed() {
  let h = DOCK_HWND.load(Ordering::Relaxed);
  if h != 0 {
    timer(HWND(h as *mut _), TIMER_WINDOWS, 120);
  }
}

pub fn reveal_for(ms: u32) {
  let h = HWND(DOCK_HWND.load(Ordering::Relaxed) as *mut _);
  with_dock(h, |d| {
    d.revealing = true;
    d.update_visibility();
    timer(d.hwnd, TIMER_REVEAL, ms);
  });
}

pub fn icon_rect_for(hwnd: isize) -> Option<(i32, i32, i32, i32)> {
  let h = HWND(DOCK_HWND.load(Ordering::Relaxed) as *mut _);
  with_dock(h, |d| d.icon_rect(hwnd)).flatten()
}

pub fn cli_commands(hwnd: HWND, add: Option<String>, icon: Option<(String, String)>) {
  with_dock(hwnd, |d| {
    if let Some(p) = add {
      d.add_app(&p);
    }
    if let Some((app, img)) = icon {
      let app = if app.eq_ignore_ascii_case("start") {
        START_KEY.to_string()
      } else if app.eq_ignore_ascii_case("launchpad") {
        LAUNCHPAD_KEY.to_string()
      } else {
        app
      };
      if custom_icons::set(&app, &img) {
        d.reload_icon(&app);
        if app == START_KEY || app == LAUNCHPAD_KEY {
          d.signature.clear();
          d.refresh_windows();
        }
      }
    }
  });
}

fn metrics_from_prefs(dpi: f32) -> Metrics {
  let s = crate::prefs::get();
  Metrics::custom(s.dock_size as f32, s.dock_gap as f32, s.dock_zoom as f32 / 100.0, dpi)
}

use wndproc::proc;

fn auto_tier(c: &sys::Conditions) -> Tier {
  if !c.transparency || c.battery_saver || c.remote_session {
    Tier::Solid
  } else {
    Tier::Full
  }
}
