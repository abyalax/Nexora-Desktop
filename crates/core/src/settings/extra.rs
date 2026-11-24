//! Table-driven settings added after 0.4.0 (ADR-0004).
//!
//! One row per setting. Every row lives in [`Extra`], is parsed with clamping (garbage keeps the old value) and is
//! **written only when it differs from its default**, so existing `settings.cfg` files stay byte-identical.
//! The flat key is the field name. Adding a setting is one row below; [`Extra::SPECS`] lets one generic test cover
//! default, clamp low/high, garbage and round-trip for every row (BASE-07).
//!
//! Ranges and defaults follow `docs/settings_catalog.md`. Percent-like sliders store whole numbers
//! (`font_scale` is 75–150 = 0.75–1.5×). Select rows store an index into their option list; the saved text is the
//! option name, so reordering the UI never changes a file.

use std::fmt::Write as _;

/// What a row stores. Used by tests and by the settings window to build the matching row widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
  Bool {
    default: bool,
  },
  Int {
    min: u16,
    max: u16,
    default: u16,
  },
  Choice {
    names: &'static [&'static str],
    default: u8,
  },
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
  pub key: &'static str,
  pub kind: Kind,
}

macro_rules! extra_settings {
  (
    bool { $($b:ident : $bd:expr,)* }
    int { $($i:ident : $imin:expr, $imax:expr, $idef:expr;)* }
    choice { $($c:ident : [$($cn:literal),+], $cdef:expr;)* }
  ) => {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Extra {
      $(pub $b: bool,)*
      $(pub $i: u16,)*
      $(pub $c: u8,)*
    }

    impl Default for Extra {
      fn default() -> Self {
        Extra { $($b: $bd,)* $($i: $idef,)* $($c: $cdef,)* }
      }
    }

    impl Extra {
      /// Every row, in file order.
      pub const SPECS: &'static [Spec] = &[
        $(Spec { key: stringify!($b), kind: Kind::Bool { default: $bd } },)*
        $(Spec { key: stringify!($i), kind: Kind::Int { min: $imin, max: $imax, default: $idef } },)*
        $(Spec { key: stringify!($c), kind: Kind::Choice { names: &[$($cn),+], default: $cdef } },)*
      ];

      /// Apply one `key=value` line. Returns `false` when `key` is not an extra key.
      pub(crate) fn apply(&mut self, key: &str, v: &str) -> bool {
        $(if key == stringify!($b) {
          if let Some(x) = super::parse_bool(v) {
            self.$b = x;
          }
          return true;
        })*
        $(if key == stringify!($i) {
          if let Ok(n) = v.parse::<i64>() {
            self.$i = n.clamp($imin as i64, $imax as i64) as u16;
          }
          return true;
        })*
        $(if key == stringify!($c) {
          if let Some(p) = [$($cn),+].iter().position(|n| n.eq_ignore_ascii_case(v)) {
            self.$c = p as u8;
          }
          return true;
        })*
        false
      }

      /// Append every row that differs from its default.
      pub(crate) fn write_non_default(&self, out: &mut String) {
        let d = Extra::default();
        $(if self.$b != d.$b {
          let _ = writeln!(out, "{}={}", stringify!($b), self.$b);
        })*
        $(if self.$i != d.$i {
          let _ = writeln!(out, "{}={}", stringify!($i), self.$i);
        })*
        $(if self.$c != d.$c {
          let _ = writeln!(out, "{}={}", stringify!($c), [$($cn),+][self.$c as usize]);
        })*
      }

      /// Current value of a row as a number (bool 0/1, int as is, choice index). `None` for an unknown key.
      pub fn get(&self, key: &str) -> Option<i64> {
        $(if key == stringify!($b) {
          return Some(self.$b as i64);
        })*
        $(if key == stringify!($i) {
          return Some(self.$i as i64);
        })*
        $(if key == stringify!($c) {
          return Some(self.$c as i64);
        })*
        None
      }
    }
  };
}

extra_settings! {
  bool {
    custom_dock_blur: false, // APP-01
    pinned_reflections: false, // APP-04
    custom_icon_labels: false, // APP-09
    custom_context_menus: false, // APP-13
    custom_folder_grid: false, // APP-17
    custom_nexora_menus: false, // APP-22
    custom_finder_blur: false, // APP-26
    launchpad_custom_blur: false, // APP-30
    launchpad_above_windows: false, // APP-31
    launchpad_blur_background: true, // APP-32
    tray_all_finder: false, // TRY-01
    tray_all_dock: false, // TRY-02
    tray_system_finder: true, // TRY-03
    night_light: false, // SCR-02
    stage_icon: true, // STG-01
    stage_blur_icons: true, // STG-03
    stage_native_preview: true, // STG-06
    stage_keep_border: false, // STG-12
    minimize_excluded_windows: true, // WAN-06
    launchpad_follow_dock: true, // DLP-07
    dock_dblclick_folder: true, // DAD-02
    dock_hide_open_folder_button: false, // DAD-03
    dock_prevent_cover: true, // DAD-07
    use_nexora_minimize: true, // DMN-01
    disable_minimize_anim: false, // DMN-02
    shift_slow_anim: true, // DMN-03
    minimize_into_icon: true, // DMN-04
    dynamic_minimize_content: true, // DMN-05
    preview_app_name: true, // DPV-04
    preview_controls: true, // DPV-05
    preview_native: true, // DPV-06
    finder_menu_icon: true, // FLB-01
    finder_auto_hide: false, // FLB-02
    finder_shutdown_confirmation: true, // FLB-03
    finder_maximized_affects_appearance: true, // FLB-04
    finder_close_maximized_on_corner: false, // FLB-05
    finder_lock_tray_dragging: false, // FLB-06
    finder_window_options: true, // FLB-07
    finder_colored_tray_icons: true, // FLB-08
    finder_input_method_menu: true, // FLB-09
    finder_search_icon: true, // FLB-10
    finder_account_icon: true, // FLB-11
    finder_media_playback_icon: true, // FLB-12
    finder_lyrics_menu: true, // FLB-13
    finder_color_player_lyrics: true, // FLB-14
    finder_show_notifications: true, // FNO-01
    finder_notification_sounds: true, // FNO-02
    finder_show_time: true, // FTD-01
    finder_show_date: true, // FTD-02
    finder_show_volume: true, // FAU-01
    finder_show_audio_output: true, // FAU-02
    finder_show_media_controls: true, // FAU-03
    finder_show_brightness: true, // FDI-01
    finder_show_wifi: true, // FHW-01
    finder_show_bluetooth: true, // FHW-02
    finder_show_hardware_icons: true, // FHW-03
    finder_show_battery_menu: true, // FBA-01
    finder_remaining_battery_time: true, // FBA-03
    finder_auto_power_modes: false, // FBA-04
    notch_media: true, // IAC-01
    notch_shelf: true, // IAC-05
    notch_timer: true, // IAC-07
  }
  int {
    font_scale: 75, 150, 100; // GEN-12
    dock_blur: 0, 50, 22; // APP-02
    dock_transparency: 0, 100, 62; // APP-03
    reflection_blur: 0, 30, 5; // APP-05
    reflection_transparency: 0, 100, 35; // APP-06
    reflection_distance: 0, 20, 3; // APP-07
    reflection_gradient_distance: 0, 30, 8; // APP-08
    label_blur: 0, 40, 18; // APP-10
    label_transparency: 0, 100, 70; // APP-11
    label_border_transparency: 0, 100, 12; // APP-12
    context_blur: 0, 50, 22; // APP-14
    context_transparency: 0, 100, 85; // APP-15
    context_border: 0, 100, 12; // APP-16
    folder_blur: 0, 50, 22; // APP-18
    folder_transparency: 0, 100, 85; // APP-19
    folder_border: 0, 100, 12; // APP-20
    folder_icon_size: 24, 96, 48; // APP-21
    menu_blur: 0, 50, 26; // APP-23
    menu_transparency: 0, 100, 90; // APP-24
    menu_border: 0, 100, 10; // APP-25
    finder_blur: 0, 50, 26; // APP-27
    finder_transparency: 0, 100, 88; // APP-28
    wallpaper_tint: 0, 100, 0; // APP-29
    launchpad_blur: 0, 50, 24; // APP-33
    new_tray_icon_seconds: 0, 30, 5; // TRY-04
    screen_brightness: 0, 100, 100; // SCR-01
    night_light_intensity: 0, 100, 25; // SCR-03
    dock_display: 0, 15, 0; // DLB-10
    launchpad_icon_size: 24, 100, 52; // DLP-01
    launchpad_h_spacing: 0, 60, 22; // DLP-02
    launchpad_v_spacing: 0, 60, 24; // DLP-03
    launchpad_display: 0, 17, 0; // DLP-06
    dock_act_bar_width: 1, 100, 4; // DAD-16
    dock_act_bar_height: 1, 30, 2; // DAD-17
    dock_act_bar_distance: 0, 40, 0; // DAD-18
    preview_size: 120, 520, 260; // DPV-02
    finder_notification_seconds: 1, 15, 5; // FNO-03
  }
  choice {
    start_method: ["none", "registry", "task"], 0; // GEN-08
    taskbar_scope: ["none", "all_displays", "dock_display"], 1; // GEN-09
    desktop_icons: ["hide", "auto", "show"], 0; // STG-07
    display_font: ["segoe_ui", "arial", "inter", "roboto", "system_ui"], 0; // GEN-11
    screen_target: ["all", "primary", "active"], 1; // SCR-04
    stage_hidden_mode: ["default", "always_show", "always_hide"], 0; // STG-11
    window_anim_effect: ["scale", "fade", "slide", "zoom"], 0; // WAN-02
    window_anim_speed: ["0.5x", "0.75x", "1.0x", "1.25x", "1.5x"], 2; // WAN-03
    dock_position: ["left", "bottom", "right", "top"], 1; // DLB-09
    launchpad_sources: ["added", "added_start_menu", "added_desktop", "all"], 0; // DLP-04
    dock_detection: ["all", "current"], 0; // DAD-13
    dock_activation_trigger: ["edge", "bar", "edge_and_bar"], 0; // DAD-15
    minimized_icon_position: ["next_to_icon", "independent", "separator"], 0; // DMN-06
    minimize_anim_speed: ["0.5x", "0.75x", "1.0x", "1.25x", "1.5x"], 2; // DMN-08
    finder_date_format: ["short", "long", "iso"], 0; // FTD-04
    finder_power_mode_plugged: ["balanced", "best_performance", "best_power_efficiency"], 0; // FBA-05
    finder_power_mode_battery: ["balanced", "best_performance", "best_power_efficiency"], 0; // FBA-06
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn line(key: &str, v: &str) -> Extra {
    let mut e = Extra::default();
    assert!(e.apply(key, v), "{key} must be a known extra key");
    e
  }

  /// BASE-07: the five Co tests, generated from each registry row.
  #[test]
  fn every_row_defaults_clamps_ignores_garbage_and_round_trips() {
    for s in Extra::SPECS {
      let d = Extra::default();
      match s.kind {
        Kind::Bool { default } => {
          assert_eq!(d.get(s.key), Some(default as i64), "{} default", s.key);
          assert_eq!(line(s.key, "banana"), d, "{} garbage keeps default", s.key);
          let flipped = line(s.key, if default { "off" } else { "on" });
          assert_eq!(flipped.get(s.key), Some(!default as i64), "{} flips", s.key);
          let mut text = String::new();
          flipped.write_non_default(&mut text);
          let mut back = Extra::default();
          for l in text.lines() {
            let (k, v) = l.split_once('=').unwrap();
            back.apply(k, v);
          }
          assert_eq!(back, flipped, "{} round-trip", s.key);
        }
        Kind::Int { min, max, default } => {
          assert_eq!(d.get(s.key), Some(default as i64), "{} default", s.key);
          assert!((min..=max).contains(&default), "{} default inside range", s.key);
          assert_eq!(
            line(s.key, "-99999").get(s.key),
            Some(min as i64),
            "{} clamp low",
            s.key
          );
          assert_eq!(
            line(s.key, "99999").get(s.key),
            Some(max as i64),
            "{} clamp high",
            s.key
          );
          assert_eq!(line(s.key, "abc"), d, "{} garbage keeps default", s.key);
          let mut e = Extra::default();
          e.apply(s.key, &max.to_string());
          let mut text = String::new();
          e.write_non_default(&mut text);
          let mut back = Extra::default();
          for l in text.lines() {
            let (k, v) = l.split_once('=').unwrap();
            back.apply(k, v);
          }
          assert_eq!(back, e, "{} round-trip", s.key);
        }
        Kind::Choice { names, default } => {
          assert_eq!(d.get(s.key), Some(default as i64), "{} default", s.key);
          assert!((default as usize) < names.len(), "{} default is an option", s.key);
          assert_eq!(line(s.key, "no-such-option"), d, "{} garbage keeps default", s.key);
          for (i, n) in names.iter().enumerate() {
            let e = line(s.key, &n.to_ascii_uppercase());
            assert_eq!(e.get(s.key), Some(i as i64), "{} parses {n}", s.key);
            let mut text = String::new();
            e.write_non_default(&mut text);
            let mut back = Extra::default();
            for l in text.lines() {
              let (k, v) = l.split_once('=').unwrap();
              back.apply(k, v);
            }
            assert_eq!(back, e, "{} round-trip {n}", s.key);
          }
        }
      }
    }
  }

  #[test]
  fn keys_are_unique() {
    let mut keys: Vec<_> = Extra::SPECS.iter().map(|s| s.key).collect();
    keys.sort_unstable();
    let n = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), n, "duplicate registry key");
  }

  #[test]
  fn defaults_write_nothing() {
    let mut text = String::new();
    Extra::default().write_non_default(&mut text);
    assert!(text.is_empty(), "ADR-0004: defaults are never written");
  }

  #[test]
  fn unknown_keys_are_not_claimed() {
    assert!(!Extra::default().apply("theme", "dark"));
    assert!(!Extra::default().apply("nonsense", "1"));
  }
}
