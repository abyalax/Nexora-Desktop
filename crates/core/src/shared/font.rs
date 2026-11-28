//! Display font choice and global scale (GEN-11, GEN-12).
//!
//! The file stores an index into [`NAMES`] (`display_font`). Text rendering asks [`resolve`] for a family that is
//! really available, so a missing font never leaves the UI without text.

/// Option names as stored in `settings.cfg` (must match the `display_font` registry row).
pub const NAMES: [&str; 5] = ["segoe_ui", "arial", "inter", "roboto", "system_ui"];

/// Family used when nothing else is available. Ships with every supported Windows version.
pub const LAST_RESORT: &str = "Segoe UI";

/// Fonts that Nexora ships in `assets/fonts`, so they never need to be installed.
pub const BUNDLED: [&str; 1] = ["Inter"];

/// Family name for a stored choice. An unknown index selects the first option.
pub fn family(index: u8) -> &'static str {
  match index {
    1 => "Arial",
    2 => "Inter",
    3 => "Roboto",
    _ => LAST_RESORT,
  }
}

/// Family to draw with. `is_installed` answers for system fonts; bundled fonts are always available.
/// Order: the choice, then Segoe UI, then Arial, then [`LAST_RESORT`].
pub fn resolve(index: u8, is_installed: impl Fn(&str) -> bool) -> &'static str {
  let ok = |f: &str| BUNDLED.contains(&f) || is_installed(f);
  [family(index), "Segoe UI", "Arial"]
    .into_iter()
    .find(|f| ok(f))
    .unwrap_or(LAST_RESORT)
}

/// Text scale: UI scale (percent, 50–200) times font scale (percent, 75–150), as a factor clamped to 0.5–3.0.
pub fn text_scale(ui_scale_pct: u8, font_scale_pct: u16) -> f32 {
  let f = (ui_scale_pct as f32 / 100.0) * (font_scale_pct as f32 / 100.0);
  f.clamp(0.5, 3.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn family_matches_each_stored_option() {
    let f: Vec<_> = (0..5).map(family).collect();
    assert_eq!(f, ["Segoe UI", "Arial", "Inter", "Roboto", "Segoe UI"]);
    assert_eq!(family(200), "Segoe UI");
  }

  #[test]
  fn installed_choice_is_used() {
    assert_eq!(resolve(3, |f| f == "Roboto"), "Roboto");
  }

  #[test]
  fn missing_choice_falls_back_to_segoe_then_arial() {
    assert_eq!(resolve(3, |f| f == "Segoe UI"), "Segoe UI");
    assert_eq!(resolve(3, |f| f == "Arial"), "Arial");
    assert_eq!(
      resolve(1, |_| false),
      LAST_RESORT,
      "nothing installed still yields a family"
    );
  }

  #[test]
  fn bundled_font_needs_no_install() {
    assert_eq!(resolve(2, |_| false), "Inter");
  }

  #[test]
  fn text_scale_multiplies_and_clamps() {
    assert_eq!(text_scale(100, 100), 1.0);
    assert!((text_scale(125, 120) - 1.5).abs() < 1e-6);
    assert_eq!(text_scale(200, 150), 3.0);
    assert_eq!(text_scale(0, 0), 0.5);
  }
}
