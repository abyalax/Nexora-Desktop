mod activity;
mod body;
mod draw;
mod drop;
mod ears;
mod lyric_sync;
mod placement;
mod playback;
mod pointer;
mod present;
mod reconfigure;
mod send;
mod shadow;
mod shelf;
mod timers;
mod views;
mod wndproc;

use crate::menu::{Item as MItem, MENU_CLOSE, MENU_PICK, Menu};
use crate::strip::Strip;
use body::Body;
use draw::Painter;
use nexora_core::dynamic_island::glance;
use nexora_core::dynamic_island::state::{self, Event, Live, Look, Mode, OpenOn, Size, Tab, View, motion};
use nexora_core::dynamic_island::timer::{Phase, Timer};
use nexora_core::panels::calendar;
use nexora_core::visibility::policy::{Inputs, Mode as Vis, should_hide};
use nexora_platform::loopback::{self, Loopback};
use nexora_platform::media::{self, ArtUpdate, Command, Media};
use nexora_platform::wins::{overlap, tracker};
use nexora_platform::{log, sys};
use nexora_ui::gfx::Gfx;
use nexora_ui::glass::Tier;
use nexora_ui::host::{Host, Surface};
use shadow::Shadow;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::Instant;
use views::{Action, Data, Frame, LyrState, LyricsView, MediaView, Region};
use windows::UI::Composition::{
  AnimationDelayBehavior, CompositionDrawingSurface, CompositionObject, Compositor, Visual,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateRectRgn, SetWindowRgn};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use windows_core::{HSTRING, Interface};
use windows_numerics::{Vector2, Vector3};
use wndproc::proc;

const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_DISPLAYCHANGE: u32 = 0x007E;
const WM_DPICHANGED: u32 = 0x02E0;
const WM_POWERBROADCAST: u32 = 0x0218;
const WM_SETTINGCHANGE: u32 = 0x001A;
const STRIP_ENTER: u32 = WM_APP + 14;
const STRIP_LEAVE: u32 = WM_APP + 15;
const WM_TOPMOST: u32 = WM_APP + 72;
const WM_OPEN: u32 = WM_APP + 73;
const WM_MEDIA: u32 = WM_APP + 10;
const WM_BANDS: u32 = WM_APP + 11;
const WM_LYRICS: u32 = WM_APP + 18;
const WM_DND: u32 = WM_APP + 19;
const WM_AFTERDROP: u32 = WM_APP + 20;
const WM_DOWNLOAD: u32 = WM_APP + 21;
const WM_AGENDA: u32 = WM_APP + 23;
const WM_SEND: u32 = WM_APP + 24;

const T_DWELL: usize = 1;
const T_LEAVE: usize = 2;
const T_GHOST: usize = 3;
const T_REGION: usize = 4;
const T_TICK: usize = 5;
const T_CHARGE: usize = 6;
const T_WINCHG: usize = 7;
const T_RECONF: usize = 8;
const T_EDGE: usize = 9;
const T_GRACE: usize = 10;
const T_START: usize = 11;
const T_HIDEWIN: usize = 12;
const T_GLANCE: usize = 13;
const T_MINUTE: usize = 14;
const T_PROGRESS: usize = 15;
const T_LYRIC: usize = 16;
const T_PRIVACY: usize = 17;
const T_DLDONE: usize = 18;
const T_SEND: usize = 19;
const T_SENDDONE: usize = 20;
const T_TESTSEND: usize = 21;
const SEND_IDLE: std::time::Duration = std::time::Duration::from_secs(15 * 60);

struct SendSess {
  server: nexora_platform::sendsrv::Server,
  qr: Option<CompositionDrawingSurface>,
  qr_px: f32,
  names: Vec<String>,
  prog: nexora_platform::sendsrv::Progress,
  last: Instant,
  done_at: Option<Instant>,
}

const WIN_W: f32 = 640.0;
const WIN_H: f32 = 222.0;
const SHADOW_PAD: f32 = 36.0;

static NOTCH_HWND: AtomicIsize = AtomicIsize::new(0);
static BUSY: AtomicBool = AtomicBool::new(false);

pub struct Options {
  pub glass: Option<Tier>,
  pub mode: Vis,
  pub reserve: bool,
}

pub(crate) fn spring(comp: &Compositor, obj: &CompositionObject, prop: &str, v: f32, ratio: f32, period_ms: u32) {
  let go = || -> windows_core::Result<()> {
    let a = comp.CreateSpringScalarAnimation()?;
    a.SetDampingRatio(ratio)?;
    a.SetPeriod(windows::Foundation::TimeSpan {
      Duration: period_ms as i64 * 10_000,
    })?;
    a.SetFinalValue(
      &windows::Foundation::PropertyValue::CreateSingle(v)?.cast::<windows::Foundation::IReference<f32>>()?,
    )?;
    obj.StartAnimation(&HSTRING::from(prop), &a)
  };
  let _ = go();
}

fn ts(ms: u32) -> windows::Foundation::TimeSpan {
  windows::Foundation::TimeSpan {
    Duration: ms as i64 * 10_000,
  }
}

fn keyframes(
  comp: &Compositor,
  obj: &CompositionObject,
  prop: &str,
  from: f32,
  to: f32,
  ms: u32,
  delay: u32,
) -> windows_core::Result<()> {
  let e = crate::motion::ease(comp)?;
  let a = comp.CreateScalarKeyFrameAnimation()?;
  a.InsertKeyFrame(0.0, from)?;
  a.InsertKeyFrameWithEasingFunction(1.0, to, &e)?;
  a.SetDuration(ts(ms))?;
  if delay > 0 {
    a.SetDelayTime(ts(delay))?;
    a.SetDelayBehavior(AnimationDelayBehavior::SetInitialValueBeforeDelay)?;
  }
  obj.StartAnimation(&HSTRING::from(prop), &a)
}

fn with<R>(hwnd: HWND, f: impl FnOnce(&mut Notch) -> R) -> Option<R> {
  let p = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut Notch;
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

fn post(hwnd: isize, msg: u32) {
  unsafe {
    let _ = PostMessageW(Some(HWND(hwnd as *mut _)), msg, WPARAM(0), LPARAM(0));
  }
}

fn post_w(hwnd: isize, msg: u32, w: usize) {
  unsafe {
    let _ = PostMessageW(Some(HWND(hwnd as *mut _)), msg, WPARAM(w), LPARAM(0));
  }
}

fn on_windows_changed() {
  let h = NOTCH_HWND.load(Ordering::Relaxed);
  if h != 0 {
    timer(HWND(h as *mut _), T_WINCHG, 150);
  }
}

fn auto_tier(over: Option<Tier>) -> Tier {
  over.unwrap_or_else(|| {
    let c = sys::conditions();
    if !c.transparency || c.battery_saver || c.remote_session {
      Tier::Solid
    } else {
      Tier::Full
    }
  })
}

static ISLAND_GAP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn island_gap(dpi: f32) -> f32 {
  if crate::prefs::get().notch_island {
    (7.0 * dpi).round()
  } else {
    0.0
  }
}

fn win_size(dpi: f32, screen_w: i32) -> (i32, i32) {
  (
    ((WIN_W * dpi) as i32).min(screen_w),
    (WIN_H * dpi + island_gap(dpi)).ceil() as i32,
  )
}

pub struct Notch {
  hwnd: HWND,
  paint: Painter,
  _surface: Surface,
  body: Body,
  shadow: Option<Shadow>,
  dpi: f32,
  win: (i32, i32),
  screen_w: i32,

  mode: Mode,
  look: Look,
  size: Size,
  first: bool,
  tracking: bool,
  hover_act: Option<Action>,
  regions: Vec<Region>,

  timer: Timer,
  t0: Instant,
  last_sec: i64,
  battery: Option<(u8, bool)>,
  charging_fresh: bool,
  glance_idx: usize,

  media: Media,
  has_media: bool,
  title: String,
  artist: String,
  playing: bool,
  pos_ms: i64,
  dur_ms: i64,
  pos_at: Instant,
  can_seek: bool,
  art: Option<CompositionDrawingSurface>,
  tint: Option<[u8; 3]>,
  loopback: Option<Loopback>,
  live_vis: views::Live,
  seek_drag: Option<(f32, f32, f32)>,

  album: String,
  lyr_worker: nexora_platform::lyrics::Lyrics,
  lyr: Option<nexora_core::dynamic_island::lyrics::Lyrics>,
  lyr_state: LyrState,
  lyr_lrclib: bool,
  lyr_key: String,
  lyr_track: String,
  lyr_idx: Option<usize>,

  shelf: nexora_core::dynamic_island::shelf::Shelf,
  shelf_icons: std::collections::HashMap<String, CompositionDrawingSurface>,
  dnd: drop::Shared,
  _drop_target: Option<windows::Win32::System::Ole::IDropTarget>,
  drop_zone: Option<usize>,
  pending: Vec<String>,
  menu: Option<Menu>,
  menu_item: Option<usize>,
  privacy: (bool, bool),
  dl_watch: Option<nexora_platform::dlwatch::Watch>,
  dl: Option<nexora_core::dynamic_island::download::Download>,
  dl_done: Option<nexora_core::dynamic_island::download::Download>,
  dl_icons: std::collections::HashMap<String, Option<CompositionDrawingSurface>>,
  cal_sel: i32,
  agenda: Option<nexora_platform::agenda::Agenda>,
  cal_events: Option<(views::Date, Option<Vec<nexora_platform::agenda::Event>>)>,
  send: Option<SendSess>,
  send_err: bool,
  media_app: String,
  app_icons: std::collections::HashMap<String, Option<CompositionDrawingSurface>>,
  vis: Vis,
  reserved: bool,
  off: bool,
  shown: bool,
  starting: bool,
  overlap: overlap::Overlap,
  grace: bool,
  edge_armed: bool,
  strip: Option<Strip>,
  tier: Tier,
  glass_override: Option<Tier>,
}

impl Notch {
  pub fn create(host: &Host, opts: Options) -> windows_core::Result<HWND> {
    unsafe {
      let hi: HINSTANCE = GetModuleHandleW(None)?.into();
      let class = w!("NexoraNotch");
      RegisterClassW(&WNDCLASSW {
        lpfnWndProc: Some(proc),
        hInstance: hi,
        lpszClassName: class,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        ..Default::default()
      });
      let hwnd = CreateWindowExW(
        WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        class,
        w!("nexora-notch"),
        WS_POPUP,
        0,
        0,
        400,
        200,
        None,
        None,
        Some(hi),
        None,
      )?;
      let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.0 * crate::prefs::ui_scale();
      let screen_w = sys::screen_size().0;
      let win = win_size(dpi, screen_w);
      let _ = SetWindowPos(
        hwnd,
        None,
        (screen_w - win.0) / 2,
        0,
        win.0,
        win.1,
        SWP_NOACTIVATE | SWP_NOZORDER,
      );
      let comp = host.compositor.clone();
      let surface = host.attach(hwnd, (win.0 as f32, win.1 as f32))?;
      let tier = auto_tier(opts.glass);
      let gfx = Gfx::shared(&comp)?;
      let body = Body::new(&comp, &gfx, &surface.root, win, dpi, tier)?;
      let pad = (SHADOW_PAD * dpi).round();
      let shadow = match Shadow::new(&comp, (win.0 + 2 * pad as i32, win.1 + pad as i32), pad) {
        Ok(s) => Some(s),
        Err(e) => {
          log::line("notch", &format!("bayangan gagal: {e}"));
          None
        }
      };
      let rest = state::look(Mode::Rest, None);
      let mut n = Box::new(Notch {
        hwnd,
        paint: Painter::new(comp, gfx, dpi),
        _surface: surface,
        body,
        shadow,
        dpi,
        win,
        screen_w,
        mode: Mode::Rest,
        look: rest,
        size: state::size(rest, dpi),
        first: true,
        tracking: false,
        hover_act: None,
        regions: Vec::new(),
        timer: Timer::new(
          std::env::var("NACRE_TIMER_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(25 * 60),
        ),
        t0: Instant::now(),
        last_sec: -1,
        battery: sys::battery(),
        charging_fresh: false,
        glance_idx: 0,
        media: {
          let hw = hwnd.0 as isize;
          if std::env::var_os("NACRE_NO_MEDIA").is_some() {
            let (commands, _) = std::sync::mpsc::channel();
            let (_, snapshots) = std::sync::mpsc::channel();
            Media { commands, snapshots }
          } else if std::env::var_os("NACRE_FAKE_MEDIA").is_some() {
            nexora_platform::media_fake::spawn(move || post(hw, WM_MEDIA))
          } else {
            media::spawn(move || post(hw, WM_MEDIA))
          }
        },
        has_media: false,
        title: String::new(),
        artist: String::new(),
        playing: false,
        pos_ms: 0,
        dur_ms: 0,
        pos_at: Instant::now(),
        can_seek: false,
        art: None,
        tint: None,
        loopback: None,
        live_vis: views::Live::default(),
        seek_drag: None,
        album: String::new(),
        lyr_worker: {
          let hw = hwnd.0 as isize;
          nexora_platform::lyrics::spawn(crate::paths::config_dir().join("lyrics"), move || post(hw, WM_LYRICS))
        },
        lyr: None,
        lyr_state: LyrState::Idle,
        lyr_lrclib: false,
        lyr_key: String::new(),
        lyr_track: String::new(),
        lyr_idx: None,
        shelf: nexora_core::dynamic_island::shelf::Shelf::new(24),
        shelf_icons: std::collections::HashMap::new(),
        dnd: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
        _drop_target: None,
        drop_zone: None,
        pending: Vec::new(),
        menu: None,
        menu_item: None,
        privacy: (false, false),
        dl_watch: None,
        dl: None,
        dl_done: None,
        dl_icons: std::collections::HashMap::new(),
        cal_sel: 0,
        agenda: None,
        send: None,
        send_err: false,
        cal_events: None,
        media_app: String::new(),
        app_icons: std::collections::HashMap::new(),
        vis: opts.mode,
        reserved: false,
        off: !crate::prefs::get().notch_enabled,
        shown: true,
        starting: true,
        overlap: overlap::Overlap::default(),
        grace: false,
        edge_armed: false,
        strip: None,
        tier,
        glass_override: opts.glass,
      });
      n.refresh();
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(n) as isize);
      NOTCH_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
      crate::services::register_island(hwnd);
      tracker::install(on_windows_changed);
      let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
      timer(hwnd, T_START, 1600);
      timer(hwnd, T_MINUTE, nexora_platform::time::ms_to_next_minute());
      timer(hwnd, T_PRIVACY, 1500);
      with(hwnd, |d| {
        d._drop_target = drop::register(hwnd, WM_DND, d.dnd.clone());
        if d._drop_target.is_none() {
          log::line("notch", "seret-taruh tidak tersedia");
        }
        d.place_shadow_window();
        if d.vis != Vis::Always {
          d.strip = Strip::new(hwnd, STRIP_ENTER, STRIP_LEAVE).ok();
          d.place_strip();
        }
        if opts.reserve && !d.off && !crate::prefs::get().menubar {
          d.set_reserved(true);
        }
        d.publish_notch();
        d.update_visibility();
        d.sync_download_watch();
      });
      crate::prefs::listen(hwnd);
      if std::env::var_os("NACRE_TEST_SEND").is_some() {
        timer(hwnd, T_TESTSEND, 2500);
      }
      log::line(
        "notch",
        &format!("dibuat tier={tier:?} dpi={dpi} jendela={win:?} layar_w={screen_w}"),
      );
      Ok(hwnd)
    }
  }

  fn now_ms(&self) -> i64 {
    self.t0.elapsed().as_millis() as i64
  }

  fn x0(&self) -> f32 {
    ((self.win.0 as f32 - self.size.w) / 2.0).floor()
  }

  fn live(&self) -> Live {
    let running = self.timer.phase == Phase::Running;
    Live {
      media: self.has_media,
      playing: self.playing,
      lyric_line: self.lyr.as_ref().is_some_and(|l| l.synced),
      lyrics_on: crate::prefs::get().lyrics_compact,
      timer_left_s: running.then(|| self.timer.remaining_secs(self.now_ms())),
      charging_fresh: self.charging_fresh,
      shelf: self.shelf.len(),
      glance: crate::prefs::get().notch_idle == 0,
      download: (crate::prefs::get().notch_downloads && (self.dl.is_some() || self.dl_done.is_some()))
        || self.send_compact().is_some(),
      wide_music: crate::prefs::get().notch_idle == 2,
    }
  }

  fn dispatch(&mut self, ev: Event) {
    let open_on = if crate::prefs::get().notch_hover_open {
      OpenOn::Hover
    } else {
      OpenOn::Click
    };
    let next = state::step(self.mode, ev, open_on, self.tracking);
    if next != self.mode {
      log::line("notch", &format!("{:?} --{ev:?}--> {next:?}", self.mode));
      self.mode = next;
      self.refresh();
      self.update_visibility();
    }
  }

  fn refresh(&mut self) {
    let look = state::look(self.mode, state::activity(&self.live()));
    if look != self.look || self.first {
      self.transition(look);
    } else {
      self.render();
    }
  }
}

fn hover_scale(a: Action) -> f32 {
  if matches!(a, Action::ShelfItem(_)) {
    crate::motion::HOVER_ROW
  } else {
    crate::motion::HOVER_BUTTON
  }
}
