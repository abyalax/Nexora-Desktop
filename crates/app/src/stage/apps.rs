use super::*;

pub(super) fn running_apps(wins: &[nexora_core::dock::items::Win], refused: &[String]) -> Vec<String> {
  let mut running: Vec<String> = Vec::new();
  for w in wins {
    let k = key_of(&w.path);
    if !k.is_empty() && !running.contains(&k) && !excluded(&k) && !refused.contains(&k) {
      running.push(k);
    }
  }
  running
}

pub(crate) fn blank(b: &Bitmap) -> bool {
  if b.w < 4 || b.h < 4 {
    return true;
  }
  let first = &b.bgra[0..3];
  for sy in 0..12u32 {
    for sx in 0..12u32 {
      let (x, y) = ((sx * (b.w - 1)) / 11, (sy * (b.h - 1)) / 11);
      let i = ((y * b.w + x) * 4) as usize;
      let d = (0..3)
        .map(|c| (b.bgra[i + c] as i32 - first[c] as i32).abs())
        .max()
        .unwrap_or(0);
      if d > 6 {
        return false;
      }
    }
  }
  true
}

pub(super) fn icons_visible(stage_active: bool, stage_empty: bool) -> bool {
  let p = prefs::get();
  if p.hide_desktop_icons {
    return false;
  }
  if !stage_active {
    return true;
  }
  match p.stage_icons {
    0 => false,
    1 => stage_empty,
    _ => true,
  }
}

pub(super) fn parked_file() -> std::path::PathBuf {
  paths::config_dir().join("parked.txt")
}

pub fn shutdown() {
  with(|s| s.deactivate());
  nexora_platform::desktopicons::set_visible(true);
}

#[allow(dead_code)]
pub fn is_active() -> bool {
  with(|s| s.active).unwrap_or(false)
}

pub fn shot_for(app: &str) -> Option<Bitmap> {
  with(|s| s.shots.get(&key_of(app)).cloned()).flatten()
}

pub fn bring_app(path: &str) {
  log::line("stage", &format!("bring {path}"));
  with(|s| s.focus_app(&key_of(path), 0));
}

pub(super) fn sync() {
  let p = prefs::get();
  let want = p.stage;
  with(|s| {
    if want && !s.active {
      s.activate();
    } else if !want && s.active {
      s.deactivate();
    } else if want && s.active {
      let cfg = (
        p.stage_right,
        p.stage_size,
        p.stage_gap,
        p.stage_count,
        p.stage_titles as u8,
      );
      if cfg != s.cfg {
        s.cfg = cfg;
        s.model.max = p.stage_count as usize;
        s.edge = None;
        s.destroy_strip();
        s.rebuild();
        s.check_overlap();
      }
    }
    s.apply_icons();
  });
}

pub(super) fn key_of(path: &str) -> String {
  path.to_ascii_lowercase()
}

pub(super) fn is_desktop(hwnd: isize) -> bool {
  unsafe {
    let mut cls = [0u16; 64];
    let n = GetClassNameW(HWND(hwnd as *mut _), &mut cls);
    let c = String::from_utf16_lossy(&cls[..n.max(0) as usize]);
    c == "Progman" || c == "WorkerW"
  }
}

pub(super) fn is_own(hwnd: isize) -> bool {
  unsafe {
    let mut pid = 0u32;
    GetWindowThreadProcessId(HWND(hwnd as *mut _), Some(&mut pid));
    pid == std::process::id()
  }
}

pub(super) fn perspective(cx: f32, cy: f32, d: f32) -> Matrix4x4 {
  Matrix4x4 {
    M11: 1.0,
    M12: 0.0,
    M13: 0.0,
    M14: 0.0,
    M21: 0.0,
    M22: 1.0,
    M23: 0.0,
    M24: 0.0,
    M31: -cx / d,
    M32: -cy / d,
    M33: 1.0,
    M34: -1.0 / d,
    M41: 0.0,
    M42: 0.0,
    M43: 0.0,
    M44: 1.0,
  }
}

pub(super) fn title_surface(
  gfx: &Gfx,
  app: &str,
  px: i32,
  maxw: i32,
) -> Option<(windows::UI::Composition::CompositionDrawingSurface, i32, i32)> {
  use nexora_platform::text::{Style, measure, render};
  let stem = std::path::Path::new(app).file_stem()?.to_string_lossy().to_string();
  let mut chars = stem.chars();
  let name: String = chars.next()?.to_uppercase().chain(chars).collect();
  let style = Style { px, weight: 600 };
  let (tw, th) = measure(&name, &style);
  let tw = (tw + 2).min(maxw.max(8));
  let bmp = render(&name, &style, tw, th)?;
  let mut data = bmp.bgra.clone();
  for p in data.chunks_exact_mut(4) {
    let a = p[3] as u32;
    p[0] = (255 * a / 255) as u8;
    p[1] = (255 * a / 255) as u8;
    p[2] = (255 * a / 255) as u8;
  }
  let surf = gfx.surface_from_bgra(bmp.w, bmp.h, &data).ok()?;
  Some((surf, tw, th))
}

pub(super) fn gfx_dims(ar: Option<f32>, tw: f32, th_max: f32) -> (f32, f32) {
  let ar = ar.unwrap_or(1.6);
  let (mut w, mut h) = (tw, tw / ar);
  if h > th_max {
    h = th_max;
    w = h * ar;
  }
  (w, h)
}
