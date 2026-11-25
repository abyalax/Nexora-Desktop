use super::*;

pub(super) fn union(a: Rect, b: Rect) -> Rect {
  let (l, t) = (a.0.min(b.0), a.1.min(b.1));
  let (r, bt) = ((a.0 + a.2).max(b.0 + b.2), (a.1 + a.3).max(b.1 + b.3));
  (l, t, r - l, bt - t)
}

pub(super) fn spring(comp: &Compositor, obj: &CompositionObject, prop: &str, v: f32, ratio: f32, period_ms: u32) {
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

pub(super) fn recolor(b: &Bitmap, rgb: [u8; 3]) -> Vec<u8> {
  let mut out = b.bgra.clone();
  for px in out.chunks_exact_mut(4) {
    let a = px[3] as u32;
    px[0] = (rgb[2] as u32 * a / 255) as u8;
    px[1] = (rgb[1] as u32 * a / 255) as u8;
    px[2] = (rgb[0] as u32 * a / 255) as u8;
  }
  out
}

pub(super) fn url_encode(s: &str) -> String {
  let mut out = String::new();
  for b in s.bytes() {
    if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
      out.push(b as char);
    } else if b == b' ' {
      out.push('+');
    } else {
      out.push_str(&format!("%{b:02X}"));
    }
  }
  out
}

pub(super) fn hit(r: Rect, x: f32, y: f32) -> bool {
  x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

pub(super) fn quick_file() -> std::path::PathBuf {
  paths::config_dir().join("actions.txt")
}

pub(super) fn action_title(a: &model::ActionDef) -> String {
  prefs::tr(a.title).to_string()
}

pub(super) fn action_glyph(id: &str) -> (&'static str, [u8; 3]) {
  match id {
    "note" => ("\u{E70F}", [255, 204, 0]),
    "timer" => ("\u{E916}", [255, 149, 0]),
    "email" => ("\u{E715}", [10, 132, 255]),
    "screenshot" => ("\u{E722}", [99, 99, 108]),
    "lock" => ("\u{E72E}", [99, 99, 108]),
    "desktop" => ("\u{E7F4}", [90, 200, 250]),
    "dark" => ("\u{E790}", [88, 86, 214]),
    "stage" => ("\u{E8A0}", [48, 209, 88]),
    "mission" => ("\u{E7C4}", [10, 132, 255]),
    "calc" => ("\u{E8EF}", [255, 149, 0]),
    "taskmgr" => ("\u{E9D9}", [99, 99, 108]),
    "settings" => ("\u{E713}", [142, 142, 147]),
    _ => ("\u{E74D}", [255, 69, 58]),
  }
}
