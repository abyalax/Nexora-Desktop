use crate::icons::Bitmap;
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
  ANTIALIASED_QUALITY, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLIP_DEFAULT_PRECIS, CreateCompatibleDC, CreateDIBSection,
  CreateFontW, DIB_RGB_COLORS, DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject,
  DrawTextW, FF_DONTCARE, FONT_CHARSET, GetTextExtentPoint32W, HGDIOBJ, OUT_DEFAULT_PRECIS, SelectObject, SetBkMode,
  SetTextColor, TRANSPARENT, VARIABLE_PITCH,
};
use windows::core::HSTRING;

pub const ICON_FACE: &str = "Segoe Fluent Icons";

pub fn text_face() -> &'static str {
  text_face_with_index(0)
}

pub fn text_face_with_index(display_index: u8) -> &'static str {
  static FACE: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
  FACE.get_or_init(|| {
    let bundled = load_bundled_fonts();
    let face = resolve_display_font(display_index, bundled);
    crate::log::line("font", &format!("selected font: {} (bundled: {})", face, bundled));
    face
  })
}

fn resolve_display_font(display_index: u8, bundled_inter: bool) -> &'static str {
  use nexora_core::shared::font;
  font::resolve(display_index, |f| font_exists(f) || (bundled_inter && f == "Inter"))
}

pub fn pick_face(bundled_inter: bool, segoe_variable: bool) -> &'static str {
  if bundled_inter {
    "Inter"
  } else if segoe_variable {
    "Segoe UI Variable Text"
  } else {
    "Segoe UI"
  }
}

fn load_bundled_fonts() -> bool {
  use windows::Win32::Graphics::Gdi::{AddFontResourceExW, FONT_RESOURCE_CHARACTERISTICS};
  use windows::core::PCWSTR;
  let Ok(exe) = std::env::current_exe() else {
    return false;
  };
  let Some(dir) = exe.parent().map(|d| d.join("fonts")) else {
    return false;
  };
  let Ok(rd) = std::fs::read_dir(dir) else {
    return false;
  };
  let mut inter = false;
  for e in rd.flatten() {
    let p = e.path();
    let ok = p
      .extension()
      .and_then(|x| x.to_str())
      .map(|x| x.eq_ignore_ascii_case("ttf") || x.eq_ignore_ascii_case("otf"))
      .unwrap_or(false);
    if ok {
      let w: Vec<u16> = p.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
      let added = unsafe { AddFontResourceExW(PCWSTR(w.as_ptr()), FONT_RESOURCE_CHARACTERISTICS(0x10), None) };
      let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
      // Load all Inter font variants, not just regular
      if added > 0 && (name.starts_with("inter-") || name.starts_with("inter")) {
        inter = true;
        crate::log::line("font", &format!("loaded: {}", name));
      }
    }
  }
  if !inter {
    crate::log::line("font", "no Inter fonts loaded from bundled fonts directory");
  }
  inter
}

fn font_exists(face: &str) -> bool {
  use windows::Win32::Foundation::LPARAM;
  use windows::Win32::Graphics::Gdi::{EnumFontFamiliesExW, LOGFONTW, TEXTMETRICW};
  unsafe extern "system" fn found(_: *const LOGFONTW, _: *const TEXTMETRICW, _: u32, lp: LPARAM) -> i32 {
    unsafe {
      *(lp.0 as *mut bool) = true;
      0
    }
  }
  unsafe {
    let dc = CreateCompatibleDC(None);
    let mut lf = LOGFONTW {
      lfCharSet: FONT_CHARSET(0),
      ..Default::default()
    }; // ANSI_CHARSET to check for Latin font specifically
    for (i, c) in face.encode_utf16().take(31).enumerate() {
      lf.lfFaceName[i] = c;
    }
    let mut hit = false;
    EnumFontFamiliesExW(dc, &lf, Some(found), LPARAM(&mut hit as *mut bool as isize), 0);
    let _ = DeleteDC(dc);
    hit
  }
}

pub struct Style {
  pub px: i32,
  pub weight: i32,
}

fn font(style: &Style, face: &str) -> windows::Win32::Graphics::Gdi::HFONT {
  unsafe {
    CreateFontW(
      -style.px,
      0,
      0,
      0,
      style.weight,
      0,
      0,
      0,
      FONT_CHARSET(0), // ANSI_CHARSET instead of DEFAULT_CHARSET to prevent Chinese font fallback
      OUT_DEFAULT_PRECIS,
      CLIP_DEFAULT_PRECIS,
      ANTIALIASED_QUALITY,
      (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
      &HSTRING::from(face),
    )
  }
}

fn measure_scaled_with_face(text: &str, style: &Style, face: &str, scale: f32) -> (i32, i32) {
  unsafe {
    let dc = CreateCompatibleDC(None);
    let scaled_px = (style.px as f32 * scale).round() as i32;
    let big = Style {
      px: scaled_px * SS,
      weight: style.weight,
    };
    let f = font(&big, face);
    let old = SelectObject(dc, HGDIOBJ(f.0));
    let wide: Vec<u16> = text.encode_utf16().collect();
    let mut sz = SIZE::default();
    let _ = GetTextExtentPoint32W(dc, &wide, &mut sz);
    SelectObject(dc, old);
    let _ = DeleteObject(HGDIOBJ(f.0));
    let _ = DeleteDC(dc);
    let scaled_w = ((sz.cx as f32 / scale) + SS as f32 - 1.0) / SS as f32;
    let scaled_h = ((sz.cy as f32 / scale) + SS as f32 - 1.0) / SS as f32;
    (scaled_w as i32, scaled_h as i32)
  }
}

pub fn measure(text: &str, style: &Style) -> (i32, i32) {
  measure_scaled_with_face(text, style, text_face(), 1.0)
}

pub fn render(text: &str, style: &Style, w: i32, h: i32) -> Option<Bitmap> {
  render_with_scale(text, style, text_face(), 1.0, w, h)
}

pub fn render_with_font_and_scale(
  text: &str,
  style: &Style,
  display_index: u8,
  scale: f32,
  w: i32,
  h: i32,
) -> Option<Bitmap> {
  render_with_scale(text, style, text_face_with_index(display_index), scale, w, h)
}

pub fn measure_with_font_and_scale(text: &str, style: &Style, display_index: u8, scale: f32) -> (i32, i32) {
  let face = text_face_with_index(display_index);
  measure_scaled_with_face(text, style, face, scale)
}

pub fn render_with_scale(text: &str, style: &Style, face: &str, scale: f32, w: i32, h: i32) -> Option<Bitmap> {
  if w <= 0 || h <= 0 || text.is_empty() {
    return None;
  }
  let scaled_px = (style.px as f32 * scale).round() as i32;

  // For scale < 1.0, render directly without supersampling
  if scale < 1.0 {
    return render_raw(
      text,
      &Style {
        px: scaled_px,
        weight: style.weight,
      },
      face,
      w,
      h,
    );
  }

  // For scale >= 1.0, use supersampling for better quality
  let big = render_raw(
    text,
    &Style {
      px: scaled_px * SS,
      weight: style.weight,
    },
    face,
    (w as f32 * scale).round() as i32,
    (h as f32 * scale).round() as i32,
  )?;
  let (w, h, s) = (w as usize, h as usize, SS as usize);
  let big_w = big.w as usize;
  let mut out = vec![0u8; w * h * 4];
  for y in 0..h {
    for x in 0..w {
      let mut sum = 0u32;
      for dy in 0..s {
        let src_y = y * s + dy;
        if src_y >= big.h as usize {
          continue;
        }
        let row = (src_y * big_w + x * s) * 4;
        for dx in 0..s {
          let src_x = x * s + dx;
          if src_x >= big_w {
            continue;
          }
          let idx = row + src_x * 4 + 3;
          if idx < big.bgra.len() {
            sum += big.bgra[idx] as u32;
          }
        }
      }
      let a = (sum / (s * s) as u32) as u8;
      out[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[a, a, a, a]);
    }
  }
  Some(Bitmap {
    w: w as u32,
    h: h as u32,
    bgra: out,
  })
}

pub fn render_face(text: &str, style: &Style, face: &str, w: i32, h: i32) -> Option<Bitmap> {
  render_with_scale(text, style, face, 1.0, w, h)
}

const SS: i32 = 3;

fn render_raw(text: &str, style: &Style, face: &str, w: i32, h: i32) -> Option<Bitmap> {
  if w <= 0 || h <= 0 || text.is_empty() {
    return None;
  }
  unsafe {
    let dc = CreateCompatibleDC(None);
    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let dib = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
    if bits.is_null() {
      let _ = DeleteDC(dc);
      return None;
    }
    let old_bmp = SelectObject(dc, HGDIOBJ(dib.0));
    let f = font(style, face);
    let old_font = SelectObject(dc, HGDIOBJ(f.0));
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, COLORREF(0x00FF_FFFF));
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut rc = RECT {
      left: 0,
      top: 0,
      right: w,
      bottom: h,
    };
    DrawTextW(
      dc,
      &mut wide,
      &mut rc,
      DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
    );

    let src = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize);
    let mut out = vec![0u8; src.len()];
    for (s, d) in src.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
      let a = s[0].max(s[1]).max(s[2]);
      d.copy_from_slice(&[a, a, a, a]);
    }
    SelectObject(dc, old_font);
    SelectObject(dc, old_bmp);
    let _ = DeleteObject(HGDIOBJ(f.0));
    let _ = DeleteObject(HGDIOBJ(dib.0));
    let _ = DeleteDC(dc);
    Some(Bitmap {
      w: w as u32,
      h: h as u32,
      bgra: out,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::pick_face;

  #[test]
  fn never_picks_fonts_installed_by_the_user() {
    assert_eq!(pick_face(true, true), "Inter");
    assert_eq!(pick_face(false, true), "Segoe UI Variable Text");
    assert_eq!(pick_face(false, false), "Segoe UI");
    for f in [pick_face(true, false), pick_face(false, true), pick_face(false, false)] {
      assert!(!f.contains("SF Pro"));
    }
  }
}
