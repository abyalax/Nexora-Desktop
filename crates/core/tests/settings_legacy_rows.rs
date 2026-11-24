//! BASE-12 audit for M2 rows whose setting already existed in `Settings` before the registry (ADR-0004).
//!
//! These rows have no entry in `Extra::SPECS`, so the generic registry test does not cover them. Each test pins one
//! catalogue row: default, clamp or option set, garbage keeps the old value, round-trip through the 0.4.0 file.
//! Ranges and defaults are the ones in `docs/settings_catalog.md` (for an existing setting, core wins).

use nexora_core::settings::{Behavior, Material, Settings};

fn round_trip(s: &Settings) -> Settings {
  Settings::parse(&s.serialize())
}

#[test]
fn gen_03_stay_above_fullscreen_defaults_on_and_keeps_old_value_on_garbage() {
  assert!(Settings::default().above_fullscreen);
  let off = Settings::parse("above_fullscreen=off\n");
  assert!(!off.above_fullscreen);
  assert!(!round_trip(&off).above_fullscreen);
  assert!(!Settings::parse("above_fullscreen=off\nabove_fullscreen=banana\n").above_fullscreen);
}

#[test]
fn gen_04_ui_scale_is_75_to_200_percent_default_100() {
  assert_eq!(Settings::default().ui_scale, 100);
  assert_eq!(Settings::parse("ui_scale=1\n").ui_scale, 75, "clamp low");
  assert_eq!(Settings::parse("ui_scale=999\n").ui_scale, 200, "clamp high");
  assert_eq!(
    Settings::parse("ui_scale=125%\n").ui_scale,
    125,
    "percent sign is accepted"
  );
  assert_eq!(
    Settings::parse("ui_scale=150\nui_scale=abc\n").ui_scale,
    150,
    "garbage keeps old"
  );
  assert_eq!(round_trip(&Settings::parse("ui_scale=175\n")).ui_scale, 175);
}

#[test]
fn gen_05_blur_is_0_to_100_default_60() {
  assert_eq!(Settings::default().blur, 60);
  assert_eq!(Settings::parse("blur=-5\n").blur, 0, "clamp low");
  assert_eq!(Settings::parse("blur=500\n").blur, 100, "clamp high");
  assert_eq!(Settings::parse("blur=30\nblur=abc\n").blur, 30, "garbage keeps old");
  assert_eq!(round_trip(&Settings::parse("blur=33\n")).blur, 33);
}

#[test]
fn gen_07_accent_is_a_palette_index_with_a_multicolour_value() {
  assert_eq!(Settings::default().accent, 0);
  assert_eq!(Settings::parse("accent=9\n").accent, 7, "clamp high to the last swatch");
  assert_eq!(Settings::parse("accent=3\naccent=abc\n").accent, 3, "garbage keeps old");
  assert_eq!(round_trip(&Settings::parse("accent=5\n")).accent, 5);
  let multi = Settings::parse("accent=255\n").accent;
  assert_eq!(
    round_trip(&Settings::parse("accent=255\n")).accent,
    multi,
    "multicolour survives a round-trip"
  );
}

#[test]
fn ftd_03_time_format_is_a_bool_defaulting_to_24_hour() {
  assert!(Settings::default().clock_24h);
  let h12 = Settings::parse("clock_24h=off\n");
  assert!(!h12.clock_24h);
  assert!(!round_trip(&h12).clock_24h);
  assert!(!Settings::parse("clock_24h=off\nclock_24h=banana\n").clock_24h);
}

#[test]
fn ilb_03_island_display_mode_has_four_options_and_defaults_to_fixed() {
  assert_eq!(Settings::default().notch, Behavior::Fixed);
  for (text, want) in [
    ("fixed", Behavior::Fixed),
    ("overlay", Behavior::Overlay),
    ("smart", Behavior::Smart),
    ("ondemand", Behavior::OnDemand),
  ] {
    let s = Settings::parse(&format!("notch={text}\n"));
    assert_eq!(s.notch, want, "{text}");
    assert_eq!(round_trip(&s).notch, want, "{text} round-trip");
  }
  assert_eq!(
    Settings::parse("notch=smart\nnotch=sparkly\n").notch,
    Behavior::Smart,
    "garbage keeps old"
  );
}

#[test]
fn ilb_05_island_idle_style_is_0_to_2_default_0() {
  assert_eq!(Settings::default().notch_idle, 0);
  assert_eq!(Settings::parse("notch_idle=-3\n").notch_idle, 0, "clamp low");
  assert_eq!(Settings::parse("notch_idle=9\n").notch_idle, 2, "clamp high");
  assert_eq!(
    Settings::parse("notch_idle=1\nnotch_idle=abc\n").notch_idle,
    1,
    "garbage keeps old"
  );
  assert_eq!(round_trip(&Settings::parse("notch_idle=2\n")).notch_idle, 2);
}

#[test]
fn ilb_08_island_material_uses_core_names_and_defaults_to_black() {
  assert_eq!(Settings::default().notch_material, Material::Black);
  for m in Material::ALL {
    let s = Settings::parse(&format!("notch_material={}\n", m.key()));
    assert_eq!(s.notch_material, m);
    assert_eq!(round_trip(&s).notch_material, m);
  }
  assert_eq!(
    Settings::parse("notch_material=Opaque\n").notch_material,
    Material::Black,
    "design name Opaque"
  );
  assert_eq!(
    Settings::parse("notch_material=Translucent\n").notch_material,
    Material::Blur,
    "design name Translucent"
  );
  assert_eq!(
    Settings::parse("notch_material=glass\nnotch_material=sparkly\n").notch_material,
    Material::Glass
  );
}
