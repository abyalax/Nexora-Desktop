use crate::paths;
use nexora_core::settings::{self, ListKind, Look, ProgramLists, Settings};
use nexora_platform::{log, sys};
use nexora_ui::glass::GlassParams;
use std::sync::{Arc, Mutex};
use windows::UI::Color;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

pub const WM_PREFS: u32 = WM_APP + 40;

static CUR: Mutex<Option<Settings>> = Mutex::new(None);
static DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LISTENERS: Mutex<Vec<isize>> = Mutex::new(Vec::new());
static LISTS: Mutex<Option<Arc<ProgramLists>>> = Mutex::new(None);

fn file() -> std::path::PathBuf {
  paths::config_dir().join("settings.cfg")
}

fn lists_file() -> std::path::PathBuf {
  paths::config_dir().join("lists.cfg")
}

pub fn init() {
  let s = std::fs::read_to_string(file())
    .map(|t| Settings::parse(&t))
    .unwrap_or_default();
  log::line("prefs", &format!("dimuat: {s:?}"));
  *CUR.lock().unwrap() = Some(s);

  let lists = std::fs::read_to_string(lists_file())
    .map(|t| ProgramLists::parse(&t))
    .unwrap_or_default();
  let total: usize = ListKind::ALL.iter().map(|k| lists.get(*k).len()).sum();
  log::line("prefs", &format!("lists dimuat: {} entries", total));
  *LISTS.lock().unwrap() = Some(Arc::new(lists));
}

pub fn get() -> Settings {
  let mut g = CUR.lock().unwrap();
  *g.get_or_insert_with(Settings::default)
}

pub fn lists() -> Arc<ProgramLists> {
  let mut g = LISTS.lock().unwrap();
  g.get_or_insert_with(|| Arc::new(ProgramLists::default())).clone()
}

pub fn update_lists(f: impl FnOnce(&mut ProgramLists)) {
  let current = lists();
  let mut lists = (*current).clone();
  f(&mut lists);

  let _ = std::fs::create_dir_all(paths::config_dir());
  let text = lists.serialize();

  let tmp_path = lists_file().with_extension("tmp");
  if let Err(e) = std::fs::write(&tmp_path, text) {
    log::line("prefs", &format!("lists simpan gagal: {e}"));
    return;
  }

  if let Err(e) = std::fs::rename(&tmp_path, lists_file()) {
    log::line("prefs", &format!("lists rename gagal: {e}"));
    let _ = std::fs::remove_file(&tmp_path);
    return;
  }

  *LISTS.lock().unwrap() = Some(Arc::new(lists));
  log::line("prefs", "lists disimpan");
}

pub fn listen(h: HWND) {
  LISTENERS.lock().unwrap().push(h.0 as isize);
}

pub fn unlisten(h: HWND) {
  LISTENERS.lock().unwrap().retain(|x| *x != h.0 as isize);
}

pub fn update(f: impl FnOnce(&mut Settings)) {
  update_with(true, f);
}

pub fn update_with(save: bool, f: impl FnOnce(&mut Settings)) {
  let before = get();
  let mut s = before;
  f(&mut s);
  let dirty = DIRTY.load(std::sync::atomic::Ordering::Relaxed);
  if s == before && !(save && dirty) {
    return;
  }
  *CUR.lock().unwrap() = Some(s);
  DIRTY.store(!save, std::sync::atomic::Ordering::Relaxed);
  if save {
    let _ = std::fs::create_dir_all(paths::config_dir());
    if let Err(e) = std::fs::write(file(), s.serialize()) {
      log::line("prefs", &format!("simpan gagal: {e}"));
    }
  }
  broadcast();
}

pub fn broadcast() {
  for h in LISTENERS.lock().unwrap().iter() {
    unsafe {
      let _ = PostMessageW(Some(HWND(*h as *mut _)), WM_PREFS, WPARAM(0), LPARAM(0));
    }
  }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Role {
  Dock,
  Notch,
  Themed,
  Dark,
}

/// Override blur and tint alpha with user values: blur is a sigma-like 0-50 value, transparency is 0-100
/// where 100 is fully see-through.
fn apply_custom(l: &mut Look, blur: u16, transparency: u16) {
  l.blur_sigma = blur.min(50) as f32;
  l.tint[0] = (255.0 * (1.0 - transparency.min(100) as f32 / 100.0)).round() as u8;
}

pub use nexora_core::settings::Surface;

/// `look(role)` with the surface's own blur and transparency applied while its "custom values" switch is on.
/// Dock and Finder have their own wiring in [`look`] and [`bar_look`]; use this for Labels, ContextMenus,
/// NexoraMenus, FolderGrid and Launchpad.
pub fn surface_look(role: Role, surface: Surface) -> Look {
  let mut l = look(role);
  if let Some(c) = get().extra.custom_glass(surface) {
    match c.transparency {
      Some(t) => apply_custom(&mut l, c.blur, t),
      None => l.blur_sigma = c.blur.min(50) as f32,
    }
  }
  l
}

pub fn bar_look() -> Look {
  let s = get();
  let mut l = settings::bar_look(&s, sys::apps_dark());
  if s.extra.custom_finder_blur {
    apply_custom(&mut l, s.extra.finder_blur, s.extra.finder_transparency);
  }
  l
}

pub fn look(role: Role) -> Look {
  let mut s = get();
  let dark_sys = sys::apps_dark();
  match role {
    Role::Dock => {
      let mut l = settings::look(&s, dark_sys, s.dock_material);
      if s.extra.custom_dock_blur {
        apply_custom(&mut l, s.extra.dock_blur, s.extra.dock_transparency);
      }
      l
    }
    Role::Notch => {
      s.theme = settings::Theme::Dark;
      settings::look(&s, dark_sys, s.notch_material)
    }
    Role::Themed => settings::look(&s, dark_sys, s.dock_material),
    Role::Dark => {
      s.theme = settings::Theme::Dark;
      settings::look(&s, dark_sys, s.dock_material)
    }
  }
}

fn c(a: [u8; 4]) -> Color {
  Color {
    A: a[0],
    R: a[1],
    G: a[2],
    B: a[3],
  }
}

pub fn apply(base: GlassParams, l: &Look) -> GlassParams {
  GlassParams {
    blur_sigma: l.blur_sigma,
    saturation: l.saturation,
    tint: c(l.tint),
    rim: c(l.rim),
    solid: c(l.solid),
    highlight: l.highlight,
    liquify: l.liquify,
    ..base
  }
}

pub fn ui_scale() -> f32 {
  get().ui_scale.clamp(50, 200) as f32 / 100.0
}

pub fn text_scale() -> f32 {
  let s = get();
  nexora_core::shared::font::text_scale(s.ui_scale.clamp(50, 200), s.extra.font_scale)
}

pub fn round(role: Role) -> f32 {
  let s = get();
  settings::round_factor(if role == Role::Notch {
    s.notch_round
  } else {
    s.dock_round
  })
}

pub fn glass(size: (f32, f32), radius: f32, role: Role) -> GlassParams {
  apply(GlassParams::dark(size, radius), &look(role))
}

pub fn tr(src: &'static str) -> &'static str {
  let lang = get().lang;
  if lang == nexora_core::i18n::Lang::En && !nexora_core::i18n::is_known(src) {
    note_missing(src);
  }
  nexora_core::i18n::tr(lang, src)
}

fn note_missing(src: &'static str) {
  static SEEN: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
  let mut g = SEEN.lock().unwrap_or_else(|e| e.into_inner());
  if !g.contains(&src) && g.len() < 400 {
    g.push(src);
    nexora_platform::log::line("i18n", &format!("belum diterjemahkan: {src:?}"));
  }
}

pub fn accent() -> [u8; 3] {
  let a = get().accent;
  if a == settings::ACCENT_MULTI {
    return nexora_platform::sys::accent_color().unwrap_or(settings::ACCENTS[0]);
  }
  settings::ACCENTS[(a as usize).min(settings::ACCENTS.len() - 1)]
}

pub fn icon_dark() -> bool {
  match get().icon_variant {
    1 => false,
    2 => true,
    _ => look(Role::Themed).dark,
  }
}

pub fn accent_c(a: u8) -> Color {
  let c = accent();
  Color {
    A: a,
    R: c[0],
    G: c[1],
    B: c[2],
  }
}

#[allow(dead_code)]
pub fn fg(a: u8) -> Color {
  let l = look(Role::Themed);
  Color {
    A: a,
    R: l.fg[0],
    G: l.fg[1],
    B: l.fg[2],
  }
}
