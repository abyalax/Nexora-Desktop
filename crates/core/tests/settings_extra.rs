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
  let legacy_keys: Vec<&str> = legacy.lines().filter_map(|l| l.split_once('=').map(|(k, _)| k)).collect();
  for spec in Extra::SPECS {
    assert!(!legacy_keys.contains(&spec.key), "{} collides with a legacy key", spec.key);
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
  let legacy_part: String = with.serialize().lines().take_while(|l| !l.starts_with("notch_media")).collect::<Vec<_>>().join("\n");
  assert!(legacy_part.contains("blur=33"));
}

#[test]
fn new_boot_taskbar_and_icon_keys_win_and_derive_the_legacy_bools() {
  let s = Settings::parse("autostart=false\ntakeover=true\nstart_method=task\ntaskbar_scope=none\ndesktop_icons=auto\n");
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
  assert_eq!((s.extra.start_method, s.extra.taskbar_scope, s.extra.desktop_icons), (1, 0, 2));
  let text = s.serialize();
  for k in ["start_method=", "taskbar_scope=", "desktop_icons="] {
    assert!(!text.lines().any(|l| l.starts_with(k)), "{k} is implied by the legacy bool");
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
