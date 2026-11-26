use crate::popup::Popup;
use crate::{motion, paths, prefs};
use nexora_core::appcat::{self, Cat};
use nexora_core::search;
use nexora_core::spotlight::{self as model, ACTIONS, Mode};
use nexora_core::symbols::{self, Sym};
use nexora_platform::apps::{self, App};
use nexora_platform::filesearch::{self, Hit};
use nexora_platform::icons::{self, Bitmap};
use nexora_platform::text::{ICON_FACE, Style, measure, render, render_face};
use nexora_platform::{clipboard, keys, launch, log, sys, time};
use nexora_ui::gfx::Gfx;
use nexora_ui::glass::{GlassPanel, GlassParams};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::UI::Color;
use windows::UI::Composition::{
  CompositionDrawingSurface, CompositionObject, CompositionStretch, Compositor, ContainerVisual, InsetClip,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateRectRgn, SetWindowRgn};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetKeyState, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
  UnregisterHotKey, VK_BACK, VK_CONTROL, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_core::{HSTRING, Interface};
use windows_numerics::{Vector2, Vector3};

mod actions;
mod canvas;
mod frame;
mod geometry;
mod grid;
mod helpers;
mod lifecycle;
mod procs;
mod rowlist;
mod rows_data;

pub use self::actions::{clip_time, ctrl_down, perform};
pub use self::helpers::{action_glyph, action_title, hit, quick_file, recolor, spring, union, url_encode};
pub use self::lifecycle::{close, finish_closing, init, toggle, toggle_mode};
pub use self::procs::{ctl_proc, spot_proc};

const WM_APPS: u32 = WM_APP + 80;
const WM_CLOSE_SPOT: u32 = WM_APP + 81;
const WM_FILES: u32 = WM_APP + 84;
const WM_THUMBS: u32 = WM_APP + 85;
const WM_MOUSELEAVE_SPOT: u32 = 0x02A3;
const T_MORPH: usize = 1;
const T_REGION: usize = 2;
const T_FINISH: usize = 9;
const HOTKEY_ID: i32 = 0xAB01;
const HOTKEY_APPS: i32 = 0xAB02;
const COLS: usize = 5;
const FTYPES: [(&str, &[&str]); 4] = [
  (
    "Gambar",
    &["png", "jpg", "jpeg", "gif", "webp", "heic", "bmp", "svg", "tif", "tiff"],
  ),
  ("Video", &["mp4", "mov", "mkv", "avi", "webm", "wmv", "m4v"]),
  (
    "Dokumen",
    &[
      "pdf", "doc", "docx", "txt", "md", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx",
    ],
  ),
  ("Musik", &["mp3", "wav", "flac", "m4a", "aac", "ogg", "wma"]),
];

fn of_type(path: &str, t: Option<usize>) -> bool {
  let Some(t) = t else { return true };
  let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
  FTYPES[t].1.contains(&ext.as_str())
}
const GRID_ROWS: usize = 4;

type Rect = (f32, f32, f32, f32);

#[derive(Clone, Copy, PartialEq, Debug)]
enum Filter {
  Apps,
  Files,
  Actions,
}

#[derive(Clone, PartialEq)]
enum RKind {
  App,
  Calc,
  Web,
  Action(usize),
  File,
  Clip,
}

#[derive(Clone)]
struct Row {
  kind: RKind,
  title: String,
  sub: String,
  path: String,
}

#[derive(Clone, Copy, PartialEq)]
enum HitKind {
  Mode(Mode),
  Filter(Filter),
  #[allow(dead_code)]
  Cat(Option<Cat>),
  Row(usize),
  Cell(usize),
  ShowMore(usize),
  More,
  MoreItem,
  CopyRow(usize),
  FType(usize),
}

pub(super) enum Run {
  Nothing,
  Open(String),
  Action(&'static str),
  Copy(String),
}

#[allow(dead_code)]
struct Spot {
  popup: Popup,
  gfx: Gfx,
  comp: Compositor,
  dpi: f32,
  screen: (i32, i32),
  mode: Mode,
  query: String,
  rows: Vec<Row>,
  sel: usize,
  scroll: usize,
  apps: Vec<App>,
  names: Vec<String>,
  cat: Option<Cat>,
  filter: Option<Filter>,
  files: Vec<Hit>,
  files_gen: u64,
  shared_files: Arc<Mutex<(u64, Vec<Hit>)>>,
  quick: HashMap<String, String>,
  editing: Option<usize>,
  edit_buf: String,
  hits: Vec<(Rect, HitKind)>,
  circles: Vec<GlassPanel>,
  text_cache: HashMap<String, (CompositionDrawingSurface, i32, i32)>,
  icon_cache: HashMap<String, Option<CompositionDrawingSurface>>,
  geo: Geo,
  cur: Rect,
  buttons: bool,
  buttons_intro: bool,
  side_fading: bool,
  hover_mode: Option<Mode>,
  popped: Option<String>,
  tracking: bool,
  clip: InsetClip,
  side: ContainerVisual,
  sections: Vec<(String, std::ops::Range<usize>)>,
  expanded: Vec<bool>,
  pscroll: f32,
  content_h: f32,
  view: (f32, f32),
  cell_y: Vec<(usize, f32, f32)>,
  more_open: bool,
  headers: Vec<(usize, String)>,
  recents: Arc<Mutex<Vec<(String, String)>>>,
  thumbs: Arc<Mutex<HashMap<String, Bitmap>>>,
  thumb_surf: HashMap<String, (CompositionDrawingSurface, u32, u32)>,
  recents_requested: bool,
  morphing: bool,
  ftype: Option<usize>,
}

#[derive(Clone, Copy)]
struct Geo {
  h: f32,
  ws: f32,
  cap: f32,
  btn: f32,
  gap: f32,
  gw: f32,
  max_h: f32,
  win: (i32, i32, i32, i32),
}

fn geo(dpi: f32, screen: (i32, i32)) -> Geo {
  let d = dpi;
  let (sw, sh) = (screen.0 as f32, screen.1 as f32);
  let h = (54.0 * d).round();
  let btn = h;
  let gap = (12.0 * d).round();
  let ws = (620.0 * d).min(sw * 0.55).round();
  let cap = (ws * 0.85).round();
  let gw = cap + 4.0 * (btn + gap);
  let max_h = (sh * 0.6).min(600.0 * d).max(h * 4.0).round();
  let top = (sh * 0.23 - h / 2.0).round();
  Geo {
    h,
    ws,
    cap,
    btn,
    gap,
    gw,
    max_h,
    win: (
      ((sw - gw) / 2.0).round() as i32,
      top as i32,
      gw.ceil() as i32,
      max_h.ceil() as i32,
    ),
  }
}

struct Ctl {
  hwnd: HWND,
  comp: Compositor,
}

thread_local! {
    static SPOT: RefCell<Option<Box<Spot>>> = const { RefCell::new(None) };
    static CLOSING: RefCell<Option<Box<Spot>>> = const { RefCell::new(None) };
    static CTL: RefCell<Option<Ctl>> = const { RefCell::new(None) };
}

fn with_spot<R>(f: impl FnOnce(&mut Spot) -> R) -> Option<R> {
  SPOT.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(|b| f(b))))
}
