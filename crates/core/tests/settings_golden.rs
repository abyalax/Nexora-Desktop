//! Files written by the 0.4.0 build (before the regroup) must keep loading and round-trip byte for byte.

use nexora_core::settings::Settings;

fn round_trips(text: &str) {
  let parsed = Settings::parse(text);
  assert_eq!(
    parsed.serialize(),
    text,
    "serialize(parse(file)) must equal the original file"
  );
  assert_eq!(Settings::parse(&parsed.serialize()), parsed);
}

#[test]
fn default_file_from_0_4_0_round_trips() {
  round_trips(include_str!("fixtures/settings_0_4_0_default.cfg"));
}

#[test]
fn fully_customised_file_from_0_4_0_round_trips() {
  round_trips(include_str!("fixtures/settings_0_4_0_full.cfg"));
}
