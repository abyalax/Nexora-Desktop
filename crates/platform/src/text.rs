use crate::icons::Bitmap;
use windows::core::HSTRING;
use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW,
    GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextColor, ANTIALIASED_QUALITY, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS, DIB_RGB_COLORS,
    DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FONT_CHARSET, HGDIOBJ,
    OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
};

pub const ICON_FACE: &str = "Segoe Fluent Icons";

pub fn text_face() -> &'static str {
    static FACE: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
    FACE.get_or_init(|| {
        let bundled = load_bundled_fonts();
        let segoe_var = font_exists("Segoe UI Variable Text");
        let face = pick_face(bundled, segoe_var);
        crate::log::line(
            "font",
            &format!(
                "selected font: {} (bundled: {}, segoe_var: {})",
                face, bundled, segoe_var
            ),
        );
        face
    })
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
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{AddFontResourceExW, FONT_RESOURCE_CHARACTERISTICS};
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
            let w: Vec<u16> = p
                .to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let added = unsafe {
                AddFontResourceExW(
                    PCWSTR(w.as_ptr()),
                    FONT_RESOURCE_CHARACTERISTICS(0x10),
                    None,
                )
            };
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
    unsafe extern "system" fn found(
        _: *const LOGFONTW,
        _: *const TEXTMETRICW,
        _: u32,
        lp: LPARAM,
    ) -> i32 {
        *(lp.0 as *mut bool) = true;
        0
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
        EnumFontFamiliesExW(
            dc,
            &lf,
            Some(found),
            LPARAM(&mut hit as *mut bool as isize),
            0,
        );
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

pub fn measure(text: &str, style: &Style) -> (i32, i32) {
    unsafe {
        let dc = CreateCompatibleDC(None);
        let big = Style {
            px: style.px * SS,
            weight: style.weight,
        };
        let f = font(&big, text_face());
        let old = SelectObject(dc, HGDIOBJ(f.0));
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(dc, &wide, &mut sz);
        SelectObject(dc, old);
        let _ = DeleteObject(HGDIOBJ(f.0));
        let _ = DeleteDC(dc);
        ((sz.cx + SS - 1) / SS, (sz.cy + SS - 1) / SS)
    }
}

pub fn render(text: &str, style: &Style, w: i32, h: i32) -> Option<Bitmap> {
    render_face(text, style, text_face(), w, h)
}

pub fn render_face(text: &str, style: &Style, face: &str, w: i32, h: i32) -> Option<Bitmap> {
    if w <= 0 || h <= 0 || text.is_empty() {
        return None;
    }
    let big = render_raw(
        text,
        &Style {
            px: style.px * SS,
            weight: style.weight,
        },
        face,
        w * SS,
        h * SS,
    )?;
    let (w, h, s) = (w as usize, h as usize, SS as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0u32;
            for dy in 0..s {
                let row = ((y * s + dy) * w * s + x * s) * 4;
                for dx in 0..s {
                    sum += big.bgra[row + dx * 4 + 3] as u32;
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
        for f in [
            pick_face(true, false),
            pick_face(false, true),
            pick_face(false, false),
        ] {
            assert!(!f.contains("SF Pro"));
        }
    }
}
