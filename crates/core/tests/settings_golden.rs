//! Files written by the 0.4.0 build (before the regroup) must keep loading and round-trip byte for byte.

use nexora_core::settings::Settings;

fn round_trips(text: &str) {
  let parsed = Settings::parse(text);
  let serialized = parsed.serialize();
  // Normalize line endings for cross-platform comparison
  let normalized_text = text.replace("\r\n", "\n");
  assert_eq!(
    serialized, normalized_text,
    "serialize(parse(file)) must equal the original file"
  );
  assert_eq!(Settings::parse(&serialized), parsed);
}

#[test]
fn default_file_from_0_4_0_round_trips() {
  round_trips(include_str!("fixtures/settings_0_4_0_default.cfg"));
}

#[test]
fn fully_customised_file_from_0_4_0_round_trips() {
  round_trips(include_str!("fixtures/settings_0_4_0_full.cfg"));
}

#[test]
fn schema_version_is_read_before_settings_and_only_new_files_emit_it() {
  let newer = Settings::default();
  assert_eq!(newer.schema_version, 2);
  let serialized = newer.serialize();
  assert!(serialized.starts_with("# nexora settings\nsettings_version=2\n"));
  assert_eq!(Settings::parse(&serialized), newer);

  let parsed = Settings::parse("blur=33\nsettings_version=7\n");
  assert_eq!(parsed.schema_version, 7);
  assert_eq!(parsed.blur, 33);
  assert_eq!(Settings::parse(&parsed.serialize()), parsed);

  let legacy = Settings::parse("settings_version=invalid\nblur=42\n");
  assert_eq!(legacy.schema_version, 1);
  assert!(!legacy.serialize().contains("settings_version="));
}
