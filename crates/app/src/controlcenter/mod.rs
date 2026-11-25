use crate::menu::{Item as MItem, MENU_CLOSE, MENU_PICK, Menu};

use crate::popup::Popup;

use crate::{motion, paths, prefs, services};

use nexora_core::controls::{self, Category, Kind, Placed, Size};

use nexora_core::symbols::{self, Sym};

use nexora_platform::icons::Bitmap;

use nexora_platform::media::{self, ArtUpdate, Command};

use nexora_platform::radio::{self, Cmd as RadioCmd, Kind as RK};

use nexora_platform::text::{ICON_FACE, Style, measure, render, render_face};

use nexora_platform::{audio, brightness, keys, launch, wifi};

use nexora_ui::gfx::Gfx;

use nexora_ui::glass::{GlassPanel, GlassParams, Tier};

use std::cell::RefCell;

use std::collections::HashMap;

use windows::UI::Color;

use windows::UI::Composition::{
  CompositionDrawingSurface, CompositionStretch, Compositor, ContainerVisual, SpriteVisual,
};

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};

use windows::Win32::System::LibraryLoader::GetModuleHandleW;

use windows::Win32::UI::HiDpi::GetDpiForWindow;

use windows::Win32::UI::Input::KeyboardAndMouse::{
  ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_BACK, VK_ESCAPE,
};

use windows::Win32::UI::WindowsAndMessaging::*;

use windows::core::w;

mod build;
mod draw;
mod edit;
mod gallery;
mod module;
mod procs;
mod resize;
use procs::*;
use resize::*;

const WM_MOUSELEAVE: u32 = 0x02A3;

use windows_core::Interface;

use windows_numerics::{Vector2, Vector3};

const WM_CLOSE_CC: u32 = WM_APP + 130;

static PAD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn pad() -> f32 {
  f32::from_bits(PAD.load(std::sync::atomic::Ordering::Relaxed))
}

fn pxy(lp: LPARAM) -> (f32, f32) {
  let (x, y) = xy(lp);
  (x - pad(), y - pad())
}

const WM_RADIO: u32 = WM_APP + 131;

const BTN_DISC: u8 = 20;

const WM_MEDIA: u32 = WM_APP + 132;

const WM_VALUES: u32 = WM_APP + 133;

type Rect = (f32, f32, f32, f32);

fn r_sep_visible(cy: f32, gh: f32, dpi: f32) -> bool {
  cy > 0.0 && cy < gh - 60.0 * dpi
}

fn hit(r: Rect, x: f32, y: f32) -> bool {
  x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

fn glyph_of(k: Kind) -> &'static str {
  match k {
    Kind::Wifi => "\u{E701}",
    Kind::Bluetooth => "\u{E702}",
    Kind::Hotspot => "\u{E704}",
    Kind::Share => "\u{E72D}",
    Kind::Airplane => "\u{E709}",
    Kind::NowPlaying => "\u{E8D6}",
    Kind::Stage => "\u{E8A0}",
    Kind::Mirror => "\u{EC15}",
    Kind::Focus => "\u{E708}",
    Kind::DarkMode => "\u{E790}",
    Kind::Screenshot => "\u{E722}",
    Kind::Display => "\u{E706}",
    Kind::Sound => "\u{E767}",
    Kind::Battery => "\u{E83F}",
    Kind::Lock => "\u{E72E}",
    Kind::Mission => "\u{E7C4}",
    Kind::Apps => "\u{E71D}",
    Kind::Calculator => "\u{E8EF}",
    Kind::Timer => "\u{E916}",
  }
}

struct Mod {
  placed: Placed,
  rect: Rect,
  track: Option<Rect>,
  fill: Option<SpriteVisual>,
  thumb: Vec<SpriteVisual>,
  btns: Vec<(Rect, u8)>,
  badge: Rect,
  grip: Rect,
  vertical: bool,
  cont: Option<ContainerVisual>,
}

#[allow(dead_code)]
struct Panel {
  popup: Popup,
  dpi: f32,
  cell: f32,
  gap: f32,
  mods: Vec<Mod>,
  slots: Vec<Rect>,
  edit_btn: Rect,
  panels: Vec<GlassPanel>,
  drag: Option<(usize, f32, f32)>,
  slider: Option<usize>,
  w: f32,
  chip: Option<(Rect, bool)>,
  hover: Option<usize>,
  preview: Vec<Kind>,
  shift: HashMap<Kind, (f32, f32)>,
  lift: (f32, f32),
  resize: Option<(usize, Size)>,
  ghost: Option<ContainerVisual>,
}

fn grid_rect(p: &Placed, cell: f32, gap: f32, top: f32) -> Rect {
  (
    p.col as f32 * (cell + gap),
    top + p.row as f32 * (cell + gap),
    p.w as f32 * cell + (p.w - 1) as f32 * gap,
    p.h as f32 * cell + (p.h - 1) as f32 * gap,
  )
}

fn center_of(r: Rect) -> Vector3 {
  Vector3 {
    X: r.0 + r.2 / 2.0,
    Y: r.1 + r.3 / 2.0,
    Z: 0.0,
  }
}

#[allow(dead_code)]
struct Gallery {
  popup: Popup,
  dpi: f32,
  cat: Option<Category>,
  query: String,
  cards: Vec<(Kind, Rect)>,
  cats: Vec<(Option<Category>, Rect)>,
  done: Rect,
  scroll: f32,
  content_h: f32,
  panels: Vec<GlassPanel>,
  conts: Vec<(Rect, ContainerVisual, f32)>,
  hover: Option<usize>,
  tracking: bool,
}

struct Cc {
  ctl: HWND,
  comp: Compositor,
  gfx: Option<Gfx>,
  layout: Vec<Kind>,
  sizes: HashMap<Kind, Size>,
  menu: Option<Menu>,
  menu_kind: Option<Kind>,
  edit: bool,
  panel: Option<Panel>,
  gallery: Option<Gallery>,
  radios: Option<radio::Radios>,
  wifi: Option<bool>,
  bt: Option<bool>,
  ssid: Option<String>,
  media: Option<media::Media>,
  track: Option<(String, String, bool, bool)>,
  art: Option<Bitmap>,
  cache: HashMap<String, (CompositionDrawingSurface, i32, i32)>,
  top: i32,
  intro: bool,
  intro_items: Vec<(ContainerVisual, Vector3, u32)>,
  closing: bool,
  pending_remove: Option<Kind>,
}

const T_CLOSE: usize = 1;

const T_INTRO: usize = 2;

const T_REMOVE: usize = 3;

thread_local! {
    static CC: RefCell<Option<Cc>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut Cc) -> R) -> Option<R> {
  CC.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn layout_file() -> std::path::PathBuf {
  paths::config_dir().join("cc_layout.txt")
}

pub fn init(comp: &Compositor) -> windows_core::Result<()> {
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraCcCtl");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(ctl_proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let hwnd = CreateWindowExW(
      WINDOW_EX_STYLE(0),
      class,
      w!("nexora-cc-ctl"),
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
    let (layout, sizes) = controls::from_text_sized(&std::fs::read_to_string(layout_file()).unwrap_or_default());
    CC.with(|c| {
      *c.borrow_mut() = Some(Cc {
        ctl: hwnd,
        comp: comp.clone(),
        gfx: None,
        layout,
        sizes,
        menu: None,
        menu_kind: None,
        edit: false,
        panel: None,
        gallery: None,
        radios: None,
        wifi: None,
        bt: None,
        ssid: None,
        media: None,
        track: None,
        art: None,
        cache: HashMap::new(),
        top: 30,
        intro: false,
        intro_items: Vec::new(),
        closing: false,
        pending_remove: None,
      })
    });
  }
  Ok(())
}

pub fn toggle(top: i32) {
  let open = with(|c| c.panel.is_some() && !c.closing).unwrap_or(false);
  if open {
    close();
  } else {
    with(|c| {
      c.finish_close();
      c.open(top)
    });
  }
}

pub fn is_open() -> bool {
  with(|c| c.panel.is_some() && !c.closing).unwrap_or(false)
}

pub fn close() {
  with(|c| c.close());
}

pub fn test_open_picker() {
  with(|c| {
    let i = c
      .panel
      .as_ref()
      .and_then(|p| p.mods.iter().position(|m| m.placed.kind == Kind::Wifi));
    if let Some(i) = i {
      c.open_picker(i);
    }
  });
}

pub fn test_enter_edit() {
  with(|c| c.enter_edit());
}

pub fn refresh_radios() {
  with(|c| {
    if let Some(r) = &c.radios {
      let _ = r.commands.send(RadioCmd::Refresh);
    }
  });
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

const WHITE: [u8; 3] = [255, 255, 255];

impl Cc {
  fn save_layout(&self) {
    let _ = std::fs::write(layout_file(), controls::to_text_sized(&self.layout, &self.sizes));
  }
}
