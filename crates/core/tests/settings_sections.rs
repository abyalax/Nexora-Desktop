//! The on-disk `settings.cfg` stays readable while the code regroups it into per-feature sections.

use nexora_core::settings::{
  AppearanceSettings, Behavior, DockSettings, FinderSettings, GeneralSettings, HotAction, InputSettings,
  IslandSettings, MenubarSettings, Settings, StageSettings,
};

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

const FULL: &str = include_str!("fixtures/settings_0_4_0_full.cfg");

#[test]
fn finder_is_the_code_name_and_menubar_stays_an_alias() {
  let s = Settings::parse(LEGACY);
  let finder: FinderSettings = s.finder_settings();
  let former: MenubarSettings = s.menubar_settings();
  assert_eq!(finder, former, "ADR-0003: same type, same value under both names");
  assert_eq!(finder, FinderSettings::from(&s));
}

#[test]
fn full_file_maps_into_the_four_new_sections() {
  let s = Settings::parse(FULL);

  let general: GeneralSettings = s.general_settings();
  assert!(!general.takeover && general.autostart && !general.above_fullscreen && !general.anim);
  assert_eq!(general.ui_scale, 125);

  let appearance: AppearanceSettings = s.appearance_settings();
  assert_eq!((appearance.blur, appearance.transparency), (12, 90));
  assert!(!appearance.refraction && !appearance.win_shadow && !appearance.icon_pack);
  assert_eq!((appearance.icon_tint, appearance.icon_variant), (3, 2));

  let stage: StageSettings = s.stage_settings();
  assert!(stage.enabled && !stage.switcher && stage.right && stage.media);
  assert_eq!(
    (stage.size, stage.gap, stage.count, stage.speed, stage.icons),
    (120, 30, 4, 150, 1)
  );
  assert!(!stage.desktop_click && !stage.titles && !stage.hide_desktop_icons);

  let input: InputSettings = s.input_settings();
  assert_eq!(
    input.hot,
    [
      HotAction::Mission,
      HotAction::Launchpad,
      HotAction::Lock,
      HotAction::Stage
    ]
  );
  assert!(!input.mac_shortcuts && !input.clipboard_history);
}

#[test]
fn applying_one_new_section_leaves_the_others_untouched() {
  let base = Settings::parse(LEGACY);

  let mut general = base.general_settings();
  general.ui_scale = 150;
  general.anim = !general.anim;
  let mut s = base;
  general.apply(&mut s);
  assert_eq!((s.ui_scale, s.anim), (150, general.anim));
  assert_eq!(s.appearance_settings(), base.appearance_settings());
  assert_eq!(s.stage_settings(), base.stage_settings());
  assert_eq!(s.input_settings(), base.input_settings());
  assert_eq!(s.dock_settings(), base.dock_settings());

  let mut appearance = base.appearance_settings();
  appearance.blur = 7;
  let mut s = base;
  appearance.apply(&mut s);
  assert_eq!(s.blur, 7);
  assert_eq!(s.general_settings(), base.general_settings());
  assert_eq!(s.stage_settings(), base.stage_settings());

  let mut stage = base.stage_settings();
  stage.count = 9;
  let mut s = base;
  stage.apply(&mut s);
  assert_eq!(s.stage_count, 9);
  assert_eq!(s.appearance_settings(), base.appearance_settings());
  assert_eq!(s.input_settings(), base.input_settings());

  let mut input = base.input_settings();
  input.hot[2] = HotAction::Lock;
  let mut s = base;
  input.apply(&mut s);
  assert_eq!(s.hot[2], HotAction::Lock);
  assert_eq!(s.general_settings(), base.general_settings());
  assert_eq!(s.stage_settings(), base.stage_settings());
}

#[test]
fn sections_cover_every_setting_except_widget_style() {
  // Starting from defaults and applying all seven sections of a fully customised file must rebuild that file's
  // settings. `widget_style` is the one legacy key no section owns yet (Widget page, ADR-0006). The table-driven
  // `extra` rows (ADR-0004) move into their section structs as each row's UI box lands, so they are copied whole.
  let base = Settings::parse(FULL);
  let mut s = Settings::default();
  base.general_settings().apply(&mut s);
  base.appearance_settings().apply(&mut s);
  base.stage_settings().apply(&mut s);
  base.input_settings().apply(&mut s);
  base.dock_settings().apply(&mut s);
  base.finder_settings().apply(&mut s);
  base.island_settings().apply(&mut s);
  s.widget_style = base.widget_style;
  s.extra = base.extra;
  assert_eq!(s, base);
  assert_ne!(
    Settings::default().widget_style,
    base.widget_style,
    "the fixture must exercise widget_style"
  );
}

#[test]
fn unmodified_new_sections_apply_as_identity() {
  let base = Settings::parse(FULL);
  let mut s = base;
  base.general_settings().apply(&mut s);
  base.appearance_settings().apply(&mut s);
  base.stage_settings().apply(&mut s);
  base.input_settings().apply(&mut s);
  assert_eq!(s, base);
}
