//! The on-disk `settings.cfg` stays readable while the code regroups it into per-feature sections.

use nexora_core::settings::{Behavior, DockSettings, IslandSettings, MenubarSettings, Settings};

const LEGACY: &str = include_str!("fixtures/settings_v0_4.cfg");

#[test]
fn legacy_file_maps_into_sections() {
  let s = Settings::parse(LEGACY);

  let dock = s.dock_settings();
  assert_eq!(dock.behavior, Behavior::Smart);
  assert_eq!((dock.size, dock.gap, dock.zoom, dock.margin), (56, 10, 180, 12));
  assert!(dock.enabled);
  assert!(!dock.recent_apps);
  assert_eq!(dock.min_effect, 1);

  let island = s.island_settings();
  assert_eq!(island.behavior, Behavior::Overlay);
  assert!(!island.enabled && !island.hover_open && !island.send && !island.lyrics_online);
  assert!(island.capsule);
  assert_eq!(island.idle, 2);
  assert_eq!(island.greet_name.as_str(), "Aby");

  let menubar = s.menubar_settings();
  assert!(menubar.enabled && !menubar.clock_24h);
  assert_eq!(menubar.battery_pct, 2);
}

#[test]
fn sections_round_trip_through_the_flat_file() {
  let s = Settings::parse(LEGACY);
  let again = Settings::parse(&s.serialize());
  assert_eq!(s, again);
  assert_eq!(s.dock_settings(), again.dock_settings());
  assert_eq!(s.menubar_settings(), again.menubar_settings());
  assert_eq!(s.island_settings(), again.island_settings());
}

#[test]
fn applying_one_section_leaves_the_others_untouched() {
  let base = Settings::parse(LEGACY);

  let mut dock: DockSettings = base.dock_settings();
  dock.size = 64;
  dock.behavior = Behavior::Fixed;
  let mut s = base;
  dock.apply(&mut s);
  assert_eq!(s.dock_size, 64);
  assert_eq!(s.menubar_settings(), base.menubar_settings());
  assert_eq!(s.island_settings(), base.island_settings());

  let mut menubar: MenubarSettings = base.menubar_settings();
  menubar.clock_24h = true;
  let mut s = base;
  menubar.apply(&mut s);
  assert!(s.clock_24h);
  assert_eq!(s.dock_settings(), base.dock_settings());
  assert_eq!(s.island_settings(), base.island_settings());

  let mut island: IslandSettings = base.island_settings();
  island.enabled = true;
  let mut s = base;
  island.apply(&mut s);
  assert!(s.notch_enabled);
  assert_eq!(s.dock_settings(), base.dock_settings());
  assert_eq!(s.menubar_settings(), base.menubar_settings());
}

#[test]
fn unmodified_sections_apply_as_identity() {
  let base = Settings::parse(LEGACY);
  let mut s = base;
  base.dock_settings().apply(&mut s);
  base.menubar_settings().apply(&mut s);
  base.island_settings().apply(&mut s);
  assert_eq!(s, base);
}
