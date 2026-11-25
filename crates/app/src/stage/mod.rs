use crate::popup::Popup;

use crate::{motion, paths, prefs};

use nexora_core::stage::{Change, Stage};

use nexora_platform::icons::{self, Bitmap};

use nexora_platform::log;

use nexora_platform::wins::park::{self, Parked};

use nexora_platform::wins::{enumerate, focus, snapshot, tracker};

use nexora_ui::capture::{self, Capture};

use nexora_ui::gfx::Gfx;

use std::cell::RefCell;

use std::collections::HashMap;

use windows::UI::Composition::{
  CompositionBrush, CompositionStretch, CompositionSurfaceBrush, Compositor, ContainerVisual, SpriteVisual,
};

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};

use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use windows::Win32::UI::HiDpi::GetDpiForWindow;

use windows::Win32::UI::Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent};

use windows::Win32::UI::WindowsAndMessaging::*;

use windows::core::w;

use windows_numerics::{Matrix4x4, Vector2, Vector3};

mod activate;
mod apps;
mod fly;
mod live;
mod procs;
mod strip;
pub use apps::*;
use procs::*;

const WM_FG: u32 = WM_APP + 110;

const WM_LOC: u32 = WM_APP + 111;

const WM_EDGE_IN: u32 = WM_APP + 112;

const WM_EDGE_OUT: u32 = WM_APP + 113;

const WM_MOUSELEAVE: u32 = 0x02A3;

const T_FG: usize = 1;

const T_FLY: usize = 2;

const T_SHOT: usize = 3;

const T_OVERLAP: usize = 4;

const T_HIDE: usize = 5;

const T_PEEK: usize = 6;

const T_FLYEND: usize = 7;

const T_LIVE: usize = 8;

const TILT: f32 = 22.0;

const TILT_HOVER: f32 = 8.0;

type Rect = (f32, f32, f32, f32);

struct Slot {
  apps: Vec<String>,
  rect: Rect,
  card: Rect,
  cont: ContainerVisual,
  cards: Vec<SpriteVisual>,
  brushes: Vec<(String, CompositionSurfaceBrush)>,
  label: Option<SpriteVisual>,
}

struct StripUi {
  popup: Popup,
  dpi: f32,
  w: f32,
  slots: Vec<Slot>,
  hover: Option<usize>,
  tracking: bool,
  shown: bool,
  right: bool,
}

struct Fly {
  popup: Popup,
  show: Vec<String>,
  activate: isize,
}

struct St {
  ctl: HWND,
  comp: Compositor,
  gfx: Option<Gfx>,
  live_gfx: Option<Gfx>,
  live: HashMap<String, Capture>,
  active: bool,
  model: Stage,
  parked: Vec<(Parked, String)>,
  shots: HashMap<String, Bitmap>,
  icons: HashMap<String, Option<Bitmap>>,
  strip: Option<StripUi>,
  edge: Option<crate::strip::Strip>,
  fly: Option<Fly>,
  dying: Option<Popup>,
  hold: Vec<String>,
  #[allow(dead_code)]
  sig: String,
  peek: bool,
  want_fg: isize,
  refused: Vec<String>,
  busy_until: std::time::Instant,
  cfg: (bool, u8, u8, u8, u8),
}

static PAUSED: std::sync::Mutex<Vec<(String, Vec<String>)>> = std::sync::Mutex::new(Vec::new());

fn take_paused(app: Option<&str>) -> Vec<Vec<String>> {
  let Ok(mut g) = PAUSED.lock() else {
    return vec![];
  };
  let mut out = Vec::new();
  g.retain(|(a, ids)| {
    if app.map(|x| x == a).unwrap_or(true) {
      out.push(ids.clone());
      false
    } else {
      true
    }
  });
  out
}

fn excluded(app: &str) -> bool {
  let Ok(t) = std::fs::read_to_string(paths::config_dir().join("stage_exclude.txt")) else {
    return false;
  };
  t.lines()
    .map(|l| l.trim().to_ascii_lowercase())
    .any(|l| !l.is_empty() && !l.starts_with('#') && app.contains(&l))
}

thread_local! {
    static ST: RefCell<Option<St>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut St) -> R) -> Option<R> {
  ST.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn hook(ev: u32, hwnd: isize) {
  let target = ST.with(|c| c.try_borrow().ok().and_then(|b| b.as_ref().map(|s| (s.ctl, s.active))));
  let Some((ctl, true)) = target else { return };
  unsafe {
    if ev == tracker::EV_FOREGROUND {
      let _ = PostMessageW(Some(ctl), WM_FG, WPARAM(0), LPARAM(0));
    } else if ev == tracker::EV_LOCATION || ev == tracker::EV_MOVESIZE_END {
      let _ = PostMessageW(Some(ctl), WM_LOC, WPARAM(hwnd as usize), LPARAM(0));
    }
  }
}

pub fn init(comp: &Compositor) -> windows_core::Result<()> {
  let n = park::restore_file(&parked_file());
  if n > 0 {
    log::line("stage", &format!("{n} jendela terparkir dari sesi lalu dikembalikan"));
  }
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraStageCtl");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(ctl_proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let hwnd = CreateWindowExW(
      WINDOW_EX_STYLE(0),
      class,
      w!("nexora-stage"),
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
    ST.with(|c| {
      *c.borrow_mut() = Some(St {
        ctl: hwnd,
        comp: comp.clone(),
        gfx: None,
        live_gfx: None,
        live: HashMap::new(),
        active: false,
        model: Stage::default(),
        parked: Vec::new(),
        shots: HashMap::new(),
        icons: HashMap::new(),
        strip: None,
        edge: None,
        fly: None,
        dying: None,
        hold: Vec::new(),
        sig: String::new(),
        peek: false,
        want_fg: 0,
        refused: Vec::new(),
        busy_until: std::time::Instant::now(),
        cfg: (false, 0, 0, 0, 0),
      })
    });
    prefs::listen(hwnd);
    tracker::install(noop);
    tracker::install_fx(hook);
  }
  sync();
  with(|s| s.apply_icons());
  Ok(())
}

fn noop() {}
