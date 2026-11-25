use super::*;

pub(super) fn list_file() -> std::path::PathBuf {
  paths::config_dir().join("widgets.txt")
}

pub(super) fn col(rgb: [u8; 3], a: u8) -> Color {
  Color {
    A: a,
    R: rgb[0],
    G: rgb[1],
    B: rgb[2],
  }
}

pub(super) fn recolor(b: &nexora_platform::icons::Bitmap, rgb: [u8; 3]) -> Vec<u8> {
  let mut out = b.bgra.clone();
  for px in out.chunks_exact_mut(4) {
    let a = px[3] as u32;
    px[0] = (rgb[2] as u32 * a / 255) as u8;
    px[1] = (rgb[1] as u32 * a / 255) as u8;
    px[2] = (rgb[0] as u32 * a / 255) as u8;
  }
  out
}

pub(super) fn cursor() -> (i32, i32) {
  let mut p = POINT::default();
  unsafe {
    let _ = GetCursorPos(&mut p);
  }
  (p.x, p.y)
}

pub(super) fn hit(r: Rect, x: f32, y: f32) -> bool {
  x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

pub(super) fn en() -> bool {
  prefs::get().lang == nexora_core::i18n::Lang::En
}

pub(super) fn month_label(m: u32) -> String {
  const EN: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  if en() {
    EN[((m + 11) % 12) as usize].to_string()
  } else {
    calendar::month_name(m).to_string()
  }
}

pub(super) fn weekday_label(wd: u32) -> String {
  const ID: [&str; 7] = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
  const EN: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
  ];
  if en() {
    EN[(wd % 7) as usize]
  } else {
    ID[(wd % 7) as usize]
  }
  .to_string()
}

pub(super) fn hhmm(h: u32, m: u32) -> String {
  if prefs::get().clock_24h {
    format!("{h:02}.{m:02}")
  } else {
    format!("{}:{m:02}", if h % 12 == 0 { 12 } else { h % 12 })
  }
}

pub(super) fn utc_minutes() -> i32 {
  let n = time::local_now();
  (n.hour * 60 + n.minute) as i32 - sys::utc_offset_min()
}

pub(super) fn area(dpi: f32) -> widgets::Rect {
  let (sw, sh) = sys::screen_size();
  let top = (44.0 * dpi) as i32;
  let side = (20.0 * dpi) as i32;
  (side, top, sw - 2 * side, sh - top - (120.0 * dpi) as i32)
}

pub(super) fn style() -> u8 {
  prefs::get().widget_style
}

pub(super) fn palette(kind: Kind) -> (Option<Color>, [u8; 3]) {
  if style() == 1 {
    return (None, [255, 255, 255]);
  }
  let dark = prefs::look(prefs::Role::Themed).dark;
  if dark || kind.dark_card() {
    (
      Some(Color {
        A: 240,
        R: 28,
        G: 28,
        B: 30,
      }),
      [255, 255, 255],
    )
  } else if kind.frosted_card() {
    (
      Some(Color {
        A: 205,
        R: 246,
        G: 246,
        B: 248,
      }),
      [22, 22, 24],
    )
  } else {
    (
      Some(Color {
        A: 248,
        R: 255,
        G: 255,
        B: 255,
      }),
      [22, 22, 24],
    )
  }
}

pub(super) fn sweep(comp: &Compositor, v: &SpriteVisual, from: f32, to: f32) -> windows_core::Result<()> {
  let a = comp.CreateScalarKeyFrameAnimation()?;
  a.InsertKeyFrame(0.0, from)?;
  a.InsertKeyFrameWithEasingFunction(1.0, to, &comp.CreateLinearEasingFunction()?)?;
  a.SetDuration(TimeSpan {
    Duration: 1000 * 10_000,
  })?;
  v.cast::<CompositionObject>()?
    .StartAnimation(&HSTRING::from("RotationAngleInDegrees"), &a)
}

pub(super) fn xy(lp: LPARAM) -> (f32, f32) {
  ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32)
}
