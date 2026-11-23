//! Persistent user preferences (the on-disk `settings.cfg` model).
//!
//! Layout: `enums` (option types), `text` (fixed-size text field), `codec` (parse/serialize),
//! `look` (settings -> glass look), and the per-feature sections `general`, `appearance`, `stage`, `input`, `dock`,
//! `finder` and `island`. Public paths are unchanged: everything is re-exported here.

use crate::i18n::Lang;

mod appearance;
mod codec;
mod dock;
mod enums;
mod finder;
mod general;
mod input;
mod island;
mod look;
pub mod preview;
mod stage;
mod text;

pub use appearance::AppearanceSettings;
pub(crate) use codec::parse_bool;
pub use dock::DockSettings;
pub use enums::*;
pub use finder::FinderSettings;
pub use general::GeneralSettings;
pub use input::InputSettings;
pub use island::IslandSettings;
pub use look::{Look, bar_look, look};
pub use stage::StageSettings;
pub use text::ShortText;

/// Former name of [`FinderSettings`] (ADR-0003). Migrate callers, then delete this alias.
pub type MenubarSettings = FinderSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
  pub takeover: bool,
  pub dock: Behavior,
  pub notch: Behavior,
  pub blur: u8,
  pub style: BlurStyle,
  pub transparency: u8,
  pub theme: Theme,
  pub lang: Lang,
  pub dock_material: Material,
  pub dock_round: u8,
  pub notch_material: Material,
  pub notch_round: u8,
  pub autostart: bool,
  pub dock_size: u8,
  pub dock_gap: u8,
  pub dock_zoom: u8,
  pub dock_margin: u8,
  pub anim: bool,
  pub refraction: bool,
  pub dock_enabled: bool,
  pub notch_enabled: bool,
  pub notch_hover_open: bool,
  pub win_shadow: bool,
  pub menubar: bool,
  pub clock_24h: bool,
  pub menubar_glass: bool,
  pub menubar_material: bool,
  pub menubar_style: BarStyle,
  pub hot: [HotAction; 4],
  pub stage: bool,
  pub switcher: bool,
  pub stage_right: bool,
  pub stage_size: u8,
  pub stage_gap: u8,
  pub stage_count: u8,
  pub stage_speed: u8,
  pub stage_desktop_click: bool,
  pub stage_media: bool,
  pub stage_icons: u8,
  pub stage_titles: bool,
  pub hide_desktop_icons: bool,
  pub dock_plates: bool,
  pub above_fullscreen: bool,
  pub clipboard_history: bool,
  pub icon_style: IconStyle,
  pub icon_tint: u8,
  pub accent: u8,
  pub icon_variant: u8,
  pub min_effect: u8,
  pub notch_idle: u8,
  pub greet_name: ShortText,
  pub greet_text: ShortText,
  pub lyrics_compact: bool,
  pub lyrics_online: bool,
  pub notch_downloads: bool,
  pub notch_send: bool,
  pub mac_shortcuts: bool,
  pub downloads_stack: bool,
  pub recent_apps: bool,
  pub icon_pack: bool,
  pub ui_scale: u8,
  pub notch_island: bool,
  pub widget_style: u8,
  pub battery_pct: u8,
}

impl Default for Settings {
  fn default() -> Self {
    Settings {
      takeover: true,
      dock: Behavior::Fixed,
      notch: Behavior::Fixed,
      blur: 60,
      style: BlurStyle::Clear,
      transparency: 50,
      theme: Theme::Auto,
      lang: Lang::Id,
      dock_material: Material::Glass,
      dock_round: 50,
      notch_material: Material::Black,
      notch_round: 50,
      autostart: false,
      dock_size: 48,
      dock_gap: 14,
      dock_zoom: 200,
      dock_margin: 8,
      anim: true,
      refraction: true,
      dock_enabled: true,
      notch_enabled: true,
      notch_hover_open: true,
      win_shadow: true,
      menubar: false,
      clock_24h: true,
      menubar_glass: false,
      menubar_material: false,
      menubar_style: BarStyle::Light,
      hot: [HotAction::None; 4],
      stage: false,
      switcher: true,
      stage_right: false,
      stage_size: 170,
      stage_gap: 20,
      stage_count: 6,
      stage_speed: 100,
      stage_desktop_click: true,
      stage_media: false,
      stage_icons: 2,
      stage_titles: true,
      hide_desktop_icons: true,
      dock_plates: true,
      above_fullscreen: true,
      clipboard_history: true,
      icon_style: IconStyle::Default,
      icon_tint: 0,
      accent: 0,
      icon_variant: 0,
      min_effect: 0,
      notch_idle: 0,
      greet_name: ShortText::EMPTY,
      greet_text: ShortText::EMPTY,
      lyrics_compact: true,
      lyrics_online: true,
      notch_downloads: true,
      notch_send: true,
      mac_shortcuts: true,
      downloads_stack: true,
      recent_apps: true,
      icon_pack: true,
      ui_scale: 100,
      notch_island: false,
      widget_style: 0,
      battery_pct: 1,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::visibility::policy::Mode;

  #[test]
  fn bar_styles_force_theme_and_are_denser_than_glass() {
    let mut s = Settings::default();
    s.menubar_style = BarStyle::Light;
    let light = bar_look(&s, true);
    s.menubar_style = BarStyle::Dark;
    let dark = bar_look(&s, false);
    assert!(!light.dark && dark.dark);
    assert!(light.tint[0] >= 230 && dark.tint[0] >= 230);
    assert_eq!(BarStyle::parse("terang"), Some(BarStyle::Light));
    let again = Settings::parse(&s.serialize());
    assert_eq!(again.menubar_style, BarStyle::Dark);
  }

  #[test]
  fn legacy_glass_bar_becomes_transparent_and_light_glass_is_brighter() {
    let s = Settings::parse(
      "menubar_glass=true
menubar_style=glass
",
    );
    assert!(!s.menubar_glass, "latar kaca lama menjadi transparan");
    let mut st = Settings::default();
    st.theme = Theme::Light;
    let light = look(&st, false, Material::Glass);
    st.theme = Theme::Dark;
    let dark = look(&st, false, Material::Glass);
    assert!(
      light.tint[0] >= 40 && light.tint[0] <= 60,
      "kaca terang cukup terang: {}",
      light.tint[0]
    );
    assert!(dark.tint[0] >= 60);
  }

  #[test]
  fn roundtrip() {
    let s = Settings {
      takeover: false,
      dock: Behavior::Smart,
      notch: Behavior::Overlay,
      blur: 12,
      style: BlurStyle::Clear,
      transparency: 90,
      theme: Theme::Light,
      lang: Lang::En,
      dock_material: Material::Black,
      dock_round: 80,
      notch_material: Material::Glass,
      notch_round: 10,
      autostart: true,
      dock_size: 60,
      dock_gap: 10,
      dock_zoom: 200,
      dock_margin: 30,
      anim: false,
      refraction: false,
      dock_enabled: false,
      notch_enabled: false,
      notch_hover_open: false,
      win_shadow: false,
      menubar: true,
      clock_24h: false,
      menubar_glass: true,
      hot: [
        HotAction::Mission,
        HotAction::Launchpad,
        HotAction::Lock,
        HotAction::Stage,
      ],
      stage: true,
      switcher: false,
      ..Default::default()
    };
    assert_eq!(Settings::parse(&s.serialize()), s);
  }

  #[test]
  fn defaults_are_full_takeover() {
    let d = Settings::default();
    assert!(d.takeover && d.dock.reserves() && d.notch.reserves());
  }

  #[test]
  fn dock_metrics_clamped() {
    let s = Settings::parse(
      "dock_size=500
dock_zoom=5
dock_margin=900",
    );
    assert_eq!((s.dock_size, s.dock_zoom, s.dock_margin), (72, 100, 60));
  }

  #[test]
  fn junk_is_ignored_and_clamped() {
    let s = Settings::parse("\u{feff}blur=999\ntransparency=-5\ndock=banana\nwat\ntheme=gelap\n#x=1");
    assert_eq!(s.blur, 100);
    assert_eq!(s.transparency, 0);
    assert_eq!(s.dock, Behavior::Fixed);
    assert_eq!(s.theme, Theme::Dark);
  }

  #[test]
  fn behavior_maps_to_policy() {
    assert_eq!(Behavior::Fixed.mode(), Mode::Always);
    assert!(Behavior::Fixed.reserves());
    assert!(!Behavior::Overlay.reserves());
    assert_eq!(Behavior::Overlay.mode(), Mode::Always);
    assert_eq!(Behavior::Smart.mode(), Mode::AutoHide);
    assert!(!Behavior::Smart.reserves());
    for old in ["autohide", "peek", "smart"] {
      assert_eq!(Behavior::parse(old), Some(Behavior::Smart));
    }
  }

  #[test]
  fn materials_differ() {
    let s = Settings::default();
    let g = look(&s, false, Material::Glass);
    let k = look(&s, false, Material::Black);
    assert!(!g.dark && k.dark, "hitam selalu gelap");
    assert!(g.tint[0] < k.tint[0], "alfa: glass < black");
    assert_eq!(Material::parse("clear"), Some(Material::Glass));
  }

  #[test]
  fn round_factor_scales() {
    assert!((round_factor(50) - 1.0).abs() < 1e-6);
    assert!(round_factor(0) < 0.5 && round_factor(100) > 1.5);
  }

  #[test]
  fn blur_material_is_foggier_than_glass_and_follows_theme() {
    let s = Settings::default();
    let glass = look(&s, true, Material::Glass);
    let blur = look(&s, true, Material::Blur);
    assert!(
      blur.blur_sigma > glass.blur_sigma * 1.5,
      "blur {} vs glass {}",
      blur.blur_sigma,
      glass.blur_sigma
    );
    assert!(glass.liquify && !blur.liquify, "Crystal = gaya Liquify");
    assert!(
      glass.blur_sigma <= 7.0 && glass.rim[0] == 140,
      "Liquify: blur tipis, kilau atas 55%"
    );
    let no_refr = Settings {
      refraction: false,
      ..Settings::default()
    };
    assert!(
      !look(&no_refr, true, Material::Glass).liquify,
      "refraksi dimatikan: kaca biasa"
    );
    assert!(blur.dark && !look(&s, false, Material::Blur).dark);
    assert!(
      blur.tint[0] >= 60 && blur.rim[0] > 0 && blur.rim[0] < 60,
      "rim Blur sangat tipis"
    );
    assert_eq!(Material::parse("blur"), Some(Material::Blur));
    assert_eq!(Material::Blur.key(), "blur");
    assert_eq!(Material::ALL.len(), 3);
  }

  #[test]
  fn notch_greeting_settings_round_trip() {
    let s = Settings::parse("notch_idle=1\ngreet_name=  Rani =)  \ngreet_text=Semangat, \u{1F680}!\n");
    assert_eq!(s.notch_idle, 1);
    assert_eq!(s.greet_name.as_str(), "Rani =)");
    let again = Settings::parse(&s.serialize());
    assert_eq!(
      (again.notch_idle, again.greet_name, again.greet_text),
      (1, s.greet_name, s.greet_text)
    );
    assert_eq!(Settings::default().notch_idle, 0);
    assert_eq!(
      Settings::parse(&Settings::parse("notch_idle=2\n").serialize()).notch_idle,
      2,
      "musik & lirik"
    );
    assert_eq!(Settings::parse("notch_idle=9\n").notch_idle, 2);
    let d = Settings::default();
    assert!(
      d.lyrics_compact && d.lyrics_online,
      "lirik online aktif bawaan (keputusan pengguna)"
    );
    let off =
      Settings::parse(&Settings::parse("lyrics_online=false\nlyrics_compact=0\nnotch_downloads=0\n").serialize());
    assert!(!off.lyrics_online && !off.lyrics_compact && !off.notch_downloads);
    assert!(d.notch_downloads, "unduhan di notch aktif bawaan");
    assert!(d.notch_send, "zona Kirim aktif bawaan");
    assert!(!Settings::parse(&Settings::parse("notch_send=0\n").serialize()).notch_send);
    assert!(Settings::default().greet_name.is_empty());
    assert_eq!(ShortText::new("a\nb").as_str(), "ab");
    let long = "é".repeat(80);
    let t = ShortText::new(&long);
    assert!(t.as_str().len() <= 96 && t.as_str().chars().all(|c| c == 'é'));
  }

  #[test]
  fn multicolor_accent_and_icon_variant_round_trip() {
    let s = Settings::parse("accent=255\nicon_variant=2\n");
    assert_eq!(s.accent, ACCENT_MULTI);
    assert_eq!(s.icon_variant, 2);
    let again = Settings::parse(&s.serialize());
    assert_eq!((again.accent, again.icon_variant), (ACCENT_MULTI, 2));
    assert_eq!(Settings::parse("accent=9\n").accent, 7);
  }

  #[test]
  fn blur_style_option_is_gone_and_always_clear() {
    let s = Settings::parse(
      "style=vivid
menubar_material=true
",
    );
    assert_eq!(s.style, BlurStyle::Clear);
    assert!(s.menubar_material);
    assert!(Settings::default().style == BlurStyle::Clear);
  }

  #[test]
  fn look_responds_to_controls() {
    let mut s = Settings::default();
    let base = look(&s, true, Material::Glass);
    s.blur = 100;
    assert!(look(&s, true, Material::Glass).blur_sigma > base.blur_sigma);
    s = Settings::default();
    s.transparency = 100;
    assert!(look(&s, true, Material::Glass).tint[0] < base.tint[0]);
    s = Settings::default();
    s.theme = Theme::Light;
    assert!(!look(&s, true, Material::Glass).dark);
    s.theme = Theme::Auto;
    assert!(look(&s, true, Material::Glass).dark && !look(&s, false, Material::Glass).dark);
  }

  #[test]
  fn scale_island_and_widget_style_round_trip() {
    let s = Settings::parse("ui_scale=125%\nnotch_island=1\nwidget_style=1\n");
    assert_eq!((s.ui_scale, s.notch_island, s.widget_style), (125, true, 1));
    let again = Settings::parse(&s.serialize());
    assert_eq!((again.ui_scale, again.notch_island, again.widget_style), (125, true, 1));
    assert_eq!(Settings::parse("ui_scale=999").ui_scale, 200);
    assert_eq!(Settings::default().ui_scale, 100);
    assert_eq!(
      Settings::parse(&Settings::parse("battery_pct=2").serialize()).battery_pct,
      2
    );
    assert_eq!(Settings::parse("battery_pct=9").battery_pct, 2);
    assert_eq!(Settings::default().battery_pct, 1);
  }
}
