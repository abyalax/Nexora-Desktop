use crate::popup::Popup;

use crate::prefs::{self, Role, WM_PREFS};

use nexora_core::hud::slider;

use nexora_core::i18n::Lang;

use nexora_core::settings::{Behavior, HotAction, ListKind, Material, Theme};

use nexora_platform::icons::Bitmap;

use nexora_platform::text::{ICON_FACE, Style, measure, render, render_face};

use nexora_platform::{log, sys};

use nexora_ui::gfx::Gfx;

use std::cell::RefCell;

use std::collections::HashMap;

use windows::UI::Color;

use windows::UI::Composition::{CompositionDrawingSurface, CompositionStretch, Compositor, ContainerVisual};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

use windows::Win32::UI::HiDpi::GetDpiForWindow;

type SideItem = (u8, (f32, f32, f32, f32), ContainerVisual);

use windows::Win32::UI::Input::KeyboardAndMouse::{
  ReleaseCapture, SetCapture, SetFocus, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};

use windows::Win32::UI::WindowsAndMessaging::*;

use windows::core::w;

mod actions;
mod card;
mod gfx;
mod interact;
mod model;
mod pages;
mod pages_adv;
mod pages_dock;
mod pages_look;
mod render;
mod wndproc;
use actions::*;
use gfx::*;
use interact::*;
use model::*;
use pages::*;
use wndproc::*;

const WM_MOUSELEAVE: u32 = 0x02A3;

use windows_numerics::{Vector2, Vector3};

const WM_CLOSE_SELF: u32 = WM_APP + 50;

const W: f32 = 780.0;

const H: f32 = 560.0;

const SIDEBAR_W: f32 = 214.0;

const CONTENT_X: f32 = SIDEBAR_W + 24.0;

const CONTENT_W: f32 = W - SIDEBAR_W - 48.0;

const VIEW_TOP: f32 = 56.0;

const SIDE_INSET: f32 = 8.0;

const SIDE_RADIUS: f32 = 18.0;

const DONATE_URL: &str = "https://saweria.co/kreativadev";

struct Win {
  popup: Popup,
  gfx: Gfx,
  comp: Compositor,
  dpi: f32,
  size: (f32, f32),
  regions: Vec<Region>,
  cache: HashMap<String, (CompositionDrawingSurface, i32, i32)>,
  dragging: Option<(u8, f32, f32)>,
  side: Vec<SideItem>,
  side_hover: Option<u8>,
  side_scroll: f32,
  side_view: (f32, f32),
  side_bar: Option<((f32, f32, f32, f32), f32)>,
  side_drag: Option<f32>,
  press: Option<(f32, f32)>,
  tab: u8,
  scroll: f32,
  content_h: f32,
  history: Vec<u8>,
  hpos: usize,
  search: String,
  search_focus: bool,
  thumbs: HashMap<String, CompositionDrawingSurface>,
  edit: Option<u8>,
  edit_buf: String,
  frame: Option<windows::UI::Composition::SpriteVisual>,
  frame_key: (i32, i32, bool),
}

thread_local! {
    static WIN: RefCell<Option<Box<Win>>> = const { RefCell::new(None) };
    static AGENT_COMP: RefCell<Option<Compositor>> = const { RefCell::new(None) };
}

#[cfg(all(feature = "agent-debug", debug_assertions))]
pub fn bind_agent_compositor(comp: &Compositor) {
  AGENT_COMP.with(|c| *c.borrow_mut() = Some(comp.clone()));
}

#[cfg(all(feature = "agent-debug", debug_assertions))]
pub fn open_agent(page: Option<u8>) {
  AGENT_COMP.with(|c| {
    if let Some(comp) = c.borrow().as_ref() {
      open(comp, page);
    }
  });
}

pub fn open(comp: &Compositor, page: Option<u8>) {
  let existing = WIN.with(|w| w.borrow().as_ref().map(|b| b.popup.hwnd));
  if let Some(h) = existing {
    unsafe {
      let _ = SetWindowPos(
        h,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
    }
    if let Some(p) = page {
      WIN.with(|w| {
        if let Some(win) = w.borrow_mut().as_mut() {
          win.go(p, false);
        }
      });
    }
    return;
  }
  match create(comp, page) {
    Ok(()) => log::line("settings", "dibuka"),
    Err(e) => log::line("settings", &format!("gagal: {e}")),
  }
}

fn create(comp: &Compositor, page: Option<u8>) -> windows_core::Result<()> {
  let glass = side_glass(prefs::glass((10.0, 10.0), SIDE_RADIUS, Role::Themed));
  let popup = Popup::new(comp, w!("NexoraSettings"), Some(proc), WINDOW_EX_STYLE(0), glass)?;
  let dpi = unsafe { GetDpiForWindow(popup.hwnd) }.max(96) as f32 / 96.0;
  let gfx = Gfx::shared(comp)?;
  let tier = {
    let c = sys::conditions();
    if !c.transparency || c.battery_saver || c.remote_session {
      nexora_ui::glass::Tier::Solid
    } else {
      nexora_ui::glass::Tier::Full
    }
  };
  let _ = popup.panel.set_tier(tier);
  let _ = popup.panel.set_radius(24.0 * dpi);
  let mut win = Box::new(Win {
    popup,
    gfx,
    comp: comp.clone(),
    dpi,
    size: (0.0, 0.0),
    regions: vec![],
    cache: HashMap::new(),
    dragging: None,
    side: vec![],
    side_hover: None,
    side_scroll: 0.0,
    side_view: (0.0, 0.0),
    side_bar: None,
    side_drag: None,
    press: None,
    tab: page.unwrap_or(0),
    scroll: 0.0,
    content_h: 0.0,
    history: vec![page.unwrap_or(0)],
    hpos: 0,
    search: String::new(),
    search_focus: false,
    thumbs: HashMap::new(),
    edit: None,
    edit_buf: String::new(),
    frame: None,
    frame_key: (0, 0, false),
  });
  win.render();
  let (sw, sh) = sys::screen_size();
  let (w, h) = (win.size.0 as i32, win.size.1 as i32);
  let top = ((sh - h) / 2).clamp(12, (sh - h - 12).max(12));
  log::line("settings", &format!("layar={sw}x{sh} jendela={w}x{h} atas={top}"));
  win.popup.set_rect((sw - w) / 2, top, w, h)?;
  win.frame();
  win.popup.show();
  prefs::listen(win.popup.hwnd);
  WIN.with(|s| *s.borrow_mut() = Some(win));
  Ok(())
}
