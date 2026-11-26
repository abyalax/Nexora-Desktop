use crate::menu::{Item as MItem, MENU_CLOSE, MENU_PICK, Menu};
use crate::popup::Popup;
use crate::{motion, paths, prefs};
use nexora_core::panels::calendar;
use nexora_core::widgets::{self, Category, Kind, Placed};
use nexora_platform::text::{ICON_FACE, Style, measure, render, render_face};
use nexora_platform::{launch, log, sys, time};
use nexora_ui::gfx::Gfx;
use nexora_ui::glass::{GlassParams, Tier};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Foundation::TimeSpan;
use windows::UI::Color;
use windows::UI::Composition::{
  CompositionDrawingSurface, CompositionObject, CompositionStretch, Compositor, ContainerVisual, SpriteVisual,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_BACK, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_core::{HSTRING, Interface};
use windows_numerics::{Vector2, Vector3};

mod draw;
mod gallery;
mod helpers;
mod interact;
mod layout;
mod manage;
mod procs;
mod surfaces;
mod widget;

#[allow(unused_imports)]
use self::{draw::*, gallery::*, helpers::*, interact::*, layout::*, manage::*, procs::*, surfaces::*, widget::*};

type SnapState = (HWND, (i32, i32), (i32, i32), std::time::Instant);

const WM_MOUSELEAVE: u32 = 0x02A3;
const T_TICK: usize = 1;
const T_REMOVE: usize = 2;
const T_GCLOSE: usize = 3;
const T_SNAP: usize = 4;
const SNAP_MS: f32 = 220.0;
const CMD_EDIT: usize = 1;
const CMD_REMOVE: usize = 2;
const MARGIN: f32 = 26.0;
const RADIUS: f32 = 32.0;
const PREVIEW: f32 = 0.6;
const RED: [u8; 3] = [255, 59, 48];
const ORANGE: [u8; 3] = [255, 149, 0];
const GREEN: [u8; 3] = [52, 199, 89];
const BLUE: [u8; 3] = [10, 132, 255];

static FLOAT: AtomicBool = AtomicBool::new(false);

type Rect = (f32, f32, f32, f32);

struct Hand {
  v: SpriteVisual,
  which: u8,
  offset: Option<i32>,
}

struct Widget {
  kind: Kind,
  popup: Popup,
  pos: (i32, i32),
  size: (f32, f32),
  hands: Vec<Hand>,
  sig: String,
  hover: bool,
  tracking: bool,
}

struct Drag {
  idx: usize,
  grab: (i32, i32),
  start: (i32, i32),
  moved: bool,
}

struct Gallery {
  popup: Popup,
  cat: Option<Category>,
  query: String,
  cards: Vec<(Kind, Rect)>,
  cats: Vec<(Option<Category>, Rect)>,
  done: Rect,
  conts: Vec<(Rect, ContainerVisual, f32)>,
  hover: Option<usize>,
  tracking: bool,
  press: Option<(Kind, (i32, i32))>,
  closing: bool,
}

struct Mgr {
  ctl: HWND,
  comp: Compositor,
  gfx: Option<Gfx>,
  dpi: f32,
  list: Vec<Widget>,
  cache: HashMap<String, (CompositionDrawingSurface, i32, i32)>,
  edit: bool,
  gallery: Option<Gallery>,
  drag: Option<Drag>,
  menu: Option<Menu>,
  menu_for: Option<usize>,
  cpu_prev: Option<(u64, u64)>,
  cpu: u8,
  ram: u8,
  ticks: u32,
  pending_remove: Option<HWND>,
  slot: Option<Popup>,
  slot_at: Option<(i32, i32, i32, i32)>,
  snap: Option<SnapState>,
}

thread_local! {
    static MGR: RefCell<Option<Mgr>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut Mgr) -> R) -> Option<R> {
  MGR.with(|c| c.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

pub fn init(comp: &Compositor) -> windows_core::Result<()> {
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraWidgetsCtl");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(ctl_proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let ctl = CreateWindowExW(
      WINDOW_EX_STYLE(0),
      class,
      w!("nexora-widgets"),
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
    let dpi = windows::Win32::UI::HiDpi::GetDpiForSystem().max(96) as f32 / 96.0;
    MGR.with(|c| {
      *c.borrow_mut() = Some(Mgr {
        ctl,
        comp: comp.clone(),
        gfx: Gfx::shared(comp).ok(),
        dpi,
        list: Vec::new(),
        cache: HashMap::new(),
        edit: false,
        gallery: None,
        drag: None,
        menu: None,
        menu_for: None,
        cpu_prev: sys::cpu_times(),
        cpu: 0,
        ram: sys::memory_load().unwrap_or(0),
        ticks: 0,
        pending_remove: None,
        slot: None,
        slot_at: None,
        snap: None,
      })
    });
    prefs::listen(ctl);
    SetTimer(Some(ctl), T_TICK, 1000, None);
  }
  with(|m| m.load());
  Ok(())
}

pub fn open_gallery() {
  with(|m| m.open_gallery());
}
