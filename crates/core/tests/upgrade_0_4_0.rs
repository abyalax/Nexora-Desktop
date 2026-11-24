//! QA-02: a real 0.4.0 `settings.cfg` loads in the current build, keeps every value, and the
//! current build adds no key until a setting changes (ADR-0004). Files written by the current
//! build stay readable under 0.4.0 semantics (unknown keys ignored).

use nexora_core::settings::Settings;

const DEFAULT_0_4_0: &str = include_str!("fixtures/settings_0_4_0_default.cfg");
const FULL_0_4_0: &str = include_str!("fixtures/settings_0_4_0_full.cfg");

fn keys(text: &str) -> Vec<&str> {
  text
    .lines()
    .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim()))
    .collect()
}

fn lines(text: &str) -> Vec<&str> {
  text
    .lines()
    .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    .collect()
}

#[test]
fn loading_and_saving_adds_no_key() {
  for old in [DEFAULT_0_4_0, FULL_0_4_0] {
    let saved = Settings::parse(old).serialize();
    assert_eq!(keys(&saved), keys(old), "an upgrade must not introduce keys");
  }
}

#[test]
fn every_value_survives_the_upgrade() {
  for old in [DEFAULT_0_4_0, FULL_0_4_0] {
    let saved = Settings::parse(old).serialize();
    assert_eq!(lines(&saved), lines(old));
  }
}

#[test]
fn a_changed_setting_adds_only_its_own_key() {
  let mut s = Settings::parse(DEFAULT_0_4_0);
  let before = keys(DEFAULT_0_4_0).len();
  let new_text = "dock_position=left\n";
  let merged = format!("{}{}", s.serialize(), new_text);
  s = Settings::parse(&merged);
  let saved = s.serialize();
  assert_eq!(keys(&saved).len(), before + 1, "exactly one new key");
  assert!(keys(&saved).contains(&"dock_position"));
}

#[test]
fn unknown_keys_are_ignored_like_0_4_0_does() {
  let with_unknown = format!("{FULL_0_4_0}key_from_the_future=1\n");
  assert_eq!(Settings::parse(&with_unknown), Settings::parse(FULL_0_4_0));
}

#[test]
fn a_file_saved_by_the_new_build_reloads_unchanged() {
  let mut s = Settings::parse(FULL_0_4_0);
  s = Settings::parse(&format!("{}dock_position=left\nfont_scale=125\n", s.serialize()));
  let saved = s.serialize();
  assert_eq!(Settings::parse(&saved), s);
  assert_eq!(Settings::parse(&saved).serialize(), saved);
}
