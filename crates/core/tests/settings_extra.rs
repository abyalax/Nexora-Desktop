//! ADR-0004 / ENG-02: new keys are written only when they differ from their default, and never disturb 0.4.0 files.

use nexora_core::settings::{Extra, Settings};

const FIXTURE: &str = include_str!("fixtures/settings_0_5_0_activities.cfg");

#[test]
fn a_default_settings_file_contains_no_extra_key() {
  let text = Settings::default().serialize();
  for spec in Extra::SPECS {
    assert!(
      !text.lines().any(|l| l.starts_with(&format!("{}=", spec.key))),
      "{} must not be written while default",
      spec.key
    );
  }
}

#[test]
fn extra_keys_do_not_collide_with_legacy_keys() {
  let legacy = Settings::default().serialize();
  let legacy_keys: Vec<&str> = legacy
    .lines()
    .filter_map(|l| l.split_once('=').map(|(k, _)| k))
    .collect();
  for spec in Extra::SPECS {
    assert!(
      !legacy_keys.contains(&spec.key),
      "{} collides with a legacy key",
      spec.key
    );
  }
}

#[test]
fn the_0_5_0_fixture_parses_and_round_trips_stably() {
  let s = Settings::parse(FIXTURE);
  assert!(!s.extra.notch_media && !s.extra.notch_shelf && !s.extra.notch_timer);
  assert_eq!(s.extra.dock_position, 0, "left");
  assert_eq!(s.extra.font_scale, 125);
  assert!(s.extra.finder_auto_hide);
  assert_eq!(s.extra.window_anim_speed, 4, "1.5x");
  let once = s.serialize();
  let twice = Settings::parse(&once).serialize();
  assert_eq!(once, twice);
  assert_eq!(Settings::parse(&once), s);
}

#[test]
fn legacy_values_are_untouched_by_extra_keys() {
  let with = Settings::parse(&format!("blur=33\n{FIXTURE}"));
  assert_eq!(with.blur, 33);
  let legacy_part: String = with
    .serialize()
    .lines()
    .take_while(|l| !l.starts_with("notch_media"))
    .collect::<Vec<_>>()
    .join("\n");
  assert!(legacy_part.contains("blur=33"));
}

#[test]
fn new_boot_taskbar_and_icon_keys_win_and_derive_the_legacy_bools() {
  let s =
    Settings::parse("autostart=false\ntakeover=true\nstart_method=task\ntaskbar_scope=none\ndesktop_icons=auto\n");
  assert!(s.autostart, "task method means autostart");
  assert!(!s.takeover, "scope none means no takeover");
  assert!(s.hide_desktop_icons, "auto still hides while running");
  let back = Settings::parse(&s.serialize());
  assert_eq!(back, s);
  assert!(s.serialize().contains("start_method=task"));
}

#[test]
fn legacy_only_files_derive_the_new_values_and_do_not_write_them_back() {
  let s = Settings::parse("autostart=true\ntakeover=false\nhide_desktop_icons=false\n");
  assert_eq!(
    (s.extra.start_method, s.extra.taskbar_scope, s.extra.desktop_icons),
    (1, 0, 2)
  );
  let text = s.serialize();
  for k in ["start_method=", "taskbar_scope=", "desktop_icons="] {
    assert!(
      !text.lines().any(|l| l.starts_with(k)),
      "{k} is implied by the legacy bool"
    );
  }
  assert_eq!(Settings::parse(&text), s);
}

#[test]
fn setters_keep_legacy_bools_in_step() {
  let mut s = Settings::default();
  s.set_start_method(2);
  s.set_taskbar_scope(2);
  s.set_desktop_icons(1);
  assert!(s.autostart && s.takeover && s.hide_desktop_icons);
  s.set_start_method(0);
  s.set_taskbar_scope(0);
  s.set_desktop_icons(2);
  assert!(!s.autostart && !s.takeover && !s.hide_desktop_icons);
  assert_eq!(Settings::parse(&s.serialize()), s);
}

#[test]
fn stage_size_never_wraps_around_u8() {
  // `clamp(70, 260) as u8` used to turn 999 into 4. The field is a u8, so 255 is the real ceiling.
  assert_eq!(Settings::parse("stage_size=999\n").stage_size, 255);
  assert_eq!(Settings::parse("stage_size=1\n").stage_size, 70);
  assert_eq!(Settings::parse("stage_size=200\n").stage_size, 200);
}

#[test]
fn dock_look_and_behavior_rows_use_the_catalogue_defaults() {
  let e = Settings::default().extra;
  assert_eq!(e.get("dock_icon_effect"), Some(1), "Scaling");
  assert_eq!(e.get("dock_icon_light_intensity"), Some(35));
  assert_eq!(e.get("dock_icon_background_light"), Some(25));
  assert_eq!(e.get("dock_icon_background_color"), Some(0x63c7dc));
  assert_eq!(e.get("dock_activation_delay"), Some(150));
  assert_eq!(e.get("preview_delay"), Some(450));
  assert_eq!(e.get("dock_show_labels"), Some(0), "off by default (design)");
  assert_eq!(e.get("dock_separate_unpinned"), Some(0), "off by default (design)");
  assert_eq!(e.get("window_previews"), Some(1));
}

#[test]
fn dock_rows_accept_the_wide_ranges_and_hex_colours() {
  let s = Settings::parse(
    "dock_activation_delay=1500\npreview_delay=99999\ndock_icon_effect=select_light\n\
     dock_icon_background_color=#FF8000\ndock_hidden_effect=fade\ndock_show_labels=on\n",
  );
  assert_eq!(s.extra.dock_activation_delay, 1500);
  assert_eq!(s.extra.preview_delay, 1500, "clamped to the 1500 ms maximum");
  assert_eq!(s.extra.dock_icon_effect, 5);
  assert_eq!(s.extra.dock_icon_background_color, 0xff8000);
  assert_eq!(s.extra.dock_hidden_effect, 1);
  assert!(s.extra.dock_show_labels);
  let text = s.serialize();
  assert!(text.contains("dock_icon_background_color=#ff8000"));
  assert_eq!(Settings::parse(&text), s);
}

/// D-7.1: `min_effect` gains a third value (Suck); the 0.4.0 values 0 and 1 keep their meaning and garbage keeps the old one.
#[test]
fn min_effect_accepts_legacy_values_and_the_new_third_effect() {
  use nexora_core::dock::minimize::Effect;
  for (text, want) in [("0", 0), ("1", 1), ("2", 2), ("9", 2), ("-3", 0)] {
    let s = Settings::parse(&format!("min_effect={text}\n"));
    assert_eq!(s.min_effect, want, "min_effect={text}");
  }
  let kept = Settings::parse("min_effect=1\nmin_effect=oops\n");
  assert_eq!(kept.min_effect, 1, "garbage keeps the old value");
  assert_eq!(
    Effect::from_stored(Settings::parse("min_effect=2\n").min_effect),
    Effect::Suck
  );
  let s = Settings::parse("min_effect=2\n");
  assert_eq!(Settings::parse(&s.serialize()).min_effect, 2, "round trip");
}
