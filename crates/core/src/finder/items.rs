//! Which items the Finder bar shows, and how wide each one is.
//!
//! The host in `app` measures text with the OS font stack through [`TextMetrics`]; deciding which items exist (logo,
//! app name, menus, clock, tray icons, battery) and their padding is domain logic and lives here.

use super::layout::Item;
use crate::settings::Settings;

pub const LOGO: u32 = 1;
pub const APP: u32 = 2;
pub const M_FILE: u32 = 3;
pub const M_EDIT: u32 = 4;
pub const M_VIEW: u32 = 5;
pub const M_WIN: u32 = 6;
pub const M_HELP: u32 = 7;
pub const CLOCK: u32 = 10;
pub const CTRL: u32 = 11;
pub const SEARCH: u32 = 12;
pub const WIFI: u32 = 13;
pub const BATT: u32 = 14;
pub const STAGEI: u32 = 15;
pub const BT: u32 = 16;
pub const NOTIF: u32 = 17;
pub const VOLUME: u32 = 18;
pub const BRIGHT: u32 = 19;
pub const HW: u32 = 20;
pub const IME: u32 = 21;
pub const ACCT: u32 = 22;
pub const MEDIA: u32 = 23;
/// Chevron that opens the tray overflow list (F-11.2).
pub const TRAY_MORE: u32 = 24;
/// First id of the tray icons (`TRAY0 + index`).
pub const TRAY0: u32 = 100;
/// How many tray icons the bar can show.
pub const TRAY_MAX: usize = 32;
/// First id of the active application's own menus (`NATIVE0 + index`).
pub const NATIVE0: u32 = 50;
/// How many of the active application's top-level menus the bar can show.
pub const NATIVE_MAX: usize = 20;

/// Left padding of the logo relative to the bar height.
const LOGO_SCALE: f32 = 1.05;
/// Horizontal padding on each side of a text item, in logical pixels.
const TEXT_PAD: f32 = 11.0;
/// Width of an icon item, in logical pixels.
const ICON_W: f32 = 30.0;
/// Width of the battery glyph, in logical pixels.
const BATTERY_W: f32 = 40.0;
/// Gap between the battery glyph and its percentage label, in logical pixels.
const BATTERY_LABEL_GAP: f32 = 5.0;
/// Gap between neighbouring bar items, in logical pixels.
pub const ITEM_MARGIN: f32 = 6.0;

/// Width of one tray icon, in logical pixels.
const TRAY_W: f32 = 24.0;

/// Index of a tray icon for a bar item id, if the id is one.
pub fn tray_index(id: u32) -> Option<usize> {
  let i = id.checked_sub(TRAY0)? as usize;
  (i < TRAY_MAX).then_some(i)
}

/// Index of an application menu for a bar item id, if the id is one.
pub fn native_index(id: u32) -> Option<usize> {
  let i = id.checked_sub(NATIVE0)? as usize;
  (i < NATIVE_MAX).then_some(i)
}

/// Whether the id opens a dropdown menu from the left side of the bar.
pub fn is_menu(id: u32) -> bool {
  matches!(id, LOGO | APP | M_FILE | M_EDIT | M_VIEW | M_WIN | M_HELP) || native_index(id).is_some()
}

/// Text widths as the OS font stack measures them, in physical pixels.
pub trait TextMetrics {
  /// Width of `text` in the bar font; `emphasized` selects the heavier weight used for the app name.
  fn text(&self, text: &str, emphasized: bool) -> f32;
  /// Width of a small label such as the battery percentage.
  fn small(&self, text: &str) -> f32;
}

/// Where the left-hand menus come from.
#[derive(Clone, Copy, Debug)]
pub enum Menus<'a> {
  /// The application exposes no menus: show the five static ones (File, Edit, View, Window, Help).
  Static(&'a [String; 5]),
  /// The active application's own top-level menu titles.
  Native(&'a [String]),
}

/// Inputs that decide the bar's item set.
#[derive(Clone, Copy, Debug)]
pub struct BarInputs<'a> {
  pub dpi: f32,
  pub bar_height: f32,
  pub app_label: &'a str,
  pub menus: Menus<'a>,
  pub clock: &'a str,
  pub show_stage: bool,
  pub has_wifi: bool,
  /// Battery percentage, `None` on machines without a battery.
  pub battery: Option<u8>,
  /// Percentage is drawn beside the battery glyph rather than inside it or hidden.
  pub battery_beside: bool,
}

/// Which optional bar items the user wants (F-1, F-3, F-4.4, F-18). Every flag is a Finder row; the default shows all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Show {
  /// FLB-01: the logo (menu icon).
  pub menu_icon: bool,
  /// FLB-07: the Window menu. Hiding it only affects the five static menus; an application's own menus stay intact.
  pub window_options: bool,
  /// FLB-10
  pub search: bool,
  /// FLB-09
  pub input_method: bool,
  /// FLB-11
  pub account: bool,
  /// FLB-12
  pub media: bool,
  /// FNO-01
  pub notifications: bool,
  /// FAU-01, FAU-02 and FAU-03: the audio item stays while any of its three parts is on.
  pub audio: bool,
  /// FDI-01
  pub brightness: bool,
  /// FHW-01
  pub wifi: bool,
  /// FHW-02
  pub bluetooth: bool,
  /// FHW-03
  pub hardware: bool,
  /// FBA-01
  pub battery: bool,
  /// TRY-03
  pub tray: bool,
  /// FTD-01 or FTD-02: the clock item exists while the time or the date is shown.
  pub clock: bool,
}

impl Default for Show {
  fn default() -> Self {
    Show {
      menu_icon: true,
      window_options: true,
      search: true,
      input_method: true,
      account: true,
      media: true,
      notifications: true,
      audio: true,
      brightness: true,
      wifi: true,
      bluetooth: true,
      hardware: true,
      battery: true,
      tray: true,
      clock: true,
    }
  }
}

impl Show {
  /// Projects the Finder rows of the settings registry.
  pub fn from_settings(s: &Settings) -> Show {
    let e = &s.extra;
    Show {
      menu_icon: e.finder_menu_icon,
      window_options: e.finder_window_options,
      search: e.finder_search_icon,
      input_method: e.finder_input_method_menu,
      account: e.finder_account_icon,
      media: e.finder_media_playback_icon,
      notifications: e.finder_show_notifications,
      audio: e.finder_show_volume || e.finder_show_audio_output || e.finder_show_media_controls,
      brightness: e.finder_show_brightness,
      wifi: e.finder_show_wifi,
      bluetooth: e.finder_show_bluetooth,
      hardware: e.finder_show_hardware_icons,
      battery: e.finder_show_battery_menu,
      tray: e.tray_system_finder,
      clock: e.finder_show_time || e.finder_show_date,
    }
  }
}

/// What the machine offers right now (the platform decides). An item needs both its [`Show`] flag and its availability.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Available {
  pub bluetooth: bool,
  pub notifications: bool,
  pub volume: bool,
  pub brightness: bool,
  pub hardware: bool,
  pub input_method: bool,
  pub account: bool,
  /// Width of the media item in physical pixels, `None` when nothing is playing (see `media::budget`).
  pub media: Option<f32>,
  /// Tray icons to draw, already filtered by the tray overflow rule (F-11.2).
  pub tray: usize,
  /// The tray has icons hidden in the overflow list.
  pub tray_overflow: bool,
}

/// Left (logo, app name, menus) and right (clock, controls, status) items with their widths.
pub fn build(inp: &BarInputs, m: &dyn TextMetrics) -> (Vec<Item>, Vec<Item>) {
  build_with(inp, &Show::default(), &Available::default(), m)
}

/// Like [`build`], honouring the user's visibility rows and the machine's availability. Right-hand items run from the
/// outer edge inwards: clock, control centre, search, account, stage, then the status items, with the battery last.
pub fn build_with(inp: &BarInputs, show: &Show, avail: &Available, m: &dyn TextMetrics) -> (Vec<Item>, Vec<Item>) {
  let d = inp.dpi;
  let pad = TEXT_PAD * d;
  let text = |s: &str, emphasized: bool| m.text(s, emphasized) + 2.0 * pad;

  let mut left = Vec::new();
  if show.menu_icon {
    left.push(Item {
      id: LOGO,
      width: inp.bar_height * LOGO_SCALE,
    });
  }
  left.push(Item {
    id: APP,
    width: text(inp.app_label, true),
  });
  match inp.menus {
    Menus::Static(labels) => {
      for (id, label) in [M_FILE, M_EDIT, M_VIEW, M_WIN, M_HELP].into_iter().zip(labels.iter()) {
        if id == M_WIN && !show.window_options {
          continue;
        }
        left.push(Item {
          id,
          width: text(label, false),
        });
      }
    }
    Menus::Native(titles) => {
      for (i, title) in titles.iter().enumerate().take(NATIVE_MAX) {
        left.push(Item {
          id: NATIVE0 + i as u32,
          width: text(title, false),
        });
      }
    }
  }

  let icon = ICON_W * d;
  let mut right = Vec::new();
  if show.clock && !inp.clock.is_empty() {
    right.push(Item {
      id: CLOCK,
      width: text(inp.clock, false),
    });
  }
  right.push(Item { id: CTRL, width: icon });
  if show.search {
    right.push(Item {
      id: SEARCH,
      width: icon,
    });
  }
  if show.account && avail.account {
    right.push(Item { id: ACCT, width: icon });
  }
  if inp.show_stage {
    right.push(Item {
      id: STAGEI,
      width: icon,
    });
  }
  if show.tray {
    for i in 0..avail.tray.min(TRAY_MAX) {
      right.push(Item {
        id: TRAY0 + i as u32,
        width: TRAY_W * d,
      });
    }
    if avail.tray_overflow {
      right.push(Item {
        id: TRAY_MORE,
        width: TRAY_W * d,
      });
    }
  }
  if show.wifi && inp.has_wifi {
    right.push(Item { id: WIFI, width: icon });
  }
  for (on, id) in [
    (show.bluetooth && avail.bluetooth, BT),
    (show.audio && avail.volume, VOLUME),
    (show.brightness && avail.brightness, BRIGHT),
    (show.hardware && avail.hardware, HW),
    (show.input_method && avail.input_method, IME),
    (show.notifications && avail.notifications, NOTIF),
  ] {
    if on {
      right.push(Item { id, width: icon });
    }
  }
  if let (true, Some(w)) = (show.media, avail.media) {
    right.push(Item { id: MEDIA, width: w });
  }
  if let (true, Some(pct)) = (show.battery, inp.battery) {
    right.push(Item {
      id: BATT,
      width: BATTERY_W * d + battery_label_extra(pct, inp.battery_beside, d, m),
    });
  }
  (left, right)
}

/// Extra width the battery item needs for its percentage label when it is drawn beside the glyph.
pub fn battery_label_extra(pct: u8, beside: bool, dpi: f32, m: &dyn TextMetrics) -> f32 {
  if beside {
    m.small(&format!("{pct}%")) + BATTERY_LABEL_GAP * dpi
  } else {
    0.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Fixed-pitch metrics: 8 px per character, 10 when emphasized, 6 for small labels.
  struct Mono;

  impl TextMetrics for Mono {
    fn text(&self, text: &str, emphasized: bool) -> f32 {
      text.chars().count() as f32 * if emphasized { 10.0 } else { 8.0 }
    }

    fn small(&self, text: &str) -> f32 {
      text.chars().count() as f32 * 6.0
    }
  }

  fn labels() -> [String; 5] {
    ["File", "Edit", "View", "Window", "Help"].map(String::from)
  }

  fn inputs<'a>(menus: Menus<'a>) -> BarInputs<'a> {
    BarInputs {
      dpi: 1.0,
      bar_height: 26.0,
      app_label: "Code",
      menus,
      clock: "10:42",
      show_stage: false,
      has_wifi: false,
      battery: None,
      battery_beside: false,
    }
  }

  fn ids(items: &[Item]) -> Vec<u32> {
    items.iter().map(|i| i.id).collect()
  }

  #[test]
  fn static_menus_follow_the_logo_and_app_name() {
    let l = labels();
    let (left, _) = build(&inputs(Menus::Static(&l)), &Mono);
    assert_eq!(ids(&left), vec![LOGO, APP, M_FILE, M_EDIT, M_VIEW, M_WIN, M_HELP]);
  }

  #[test]
  fn native_menus_replace_the_static_ones_and_are_capped() {
    let titles: Vec<String> = (0..30).map(|i| format!("M{i}")).collect();
    let (left, _) = build(&inputs(Menus::Native(&titles)), &Mono);
    assert_eq!(left.len(), 2 + NATIVE_MAX);
    assert_eq!(left[2].id, NATIVE0);
    assert_eq!(left.last().unwrap().id, NATIVE0 + NATIVE_MAX as u32 - 1);
  }

  #[test]
  fn text_items_get_padding_on_both_sides_and_the_logo_scales_with_the_bar() {
    let l = labels();
    let (left, _) = build(&inputs(Menus::Static(&l)), &Mono);
    assert_eq!(left[0].width, 26.0 * 1.05);
    assert_eq!(left[1].width, 4.0 * 10.0 + 22.0);
    assert_eq!(left[2].width, 4.0 * 8.0 + 22.0);
  }

  #[test]
  fn right_side_always_has_clock_controls_and_search() {
    let l = labels();
    let (_, right) = build(&inputs(Menus::Static(&l)), &Mono);
    assert_eq!(ids(&right), vec![CLOCK, CTRL, SEARCH]);
  }

  #[test]
  fn optional_status_items_appear_only_when_available() {
    let l = labels();
    let mut inp = inputs(Menus::Static(&l));
    inp.show_stage = true;
    inp.has_wifi = true;
    inp.battery = Some(80);
    let (_, right) = build(&inp, &Mono);
    assert_eq!(ids(&right), vec![CLOCK, CTRL, SEARCH, STAGEI, WIFI, BATT]);
  }

  #[test]
  fn battery_percentage_beside_the_glyph_widens_the_item() {
    let l = labels();
    let mut inp = inputs(Menus::Static(&l));
    inp.battery = Some(80);
    let width = |inp: &BarInputs| build(inp, &Mono).1.last().unwrap().width;
    let plain = width(&inp);
    inp.battery_beside = true;
    assert_eq!(plain, 40.0);
    assert_eq!(width(&inp), 40.0 + 3.0 * 6.0 + 5.0);
  }

  #[test]
  fn dpi_scales_paddings_but_not_measured_text() {
    let l = labels();
    let mut inp = inputs(Menus::Static(&l));
    inp.dpi = 2.0;
    let (left, right) = build(&inp, &Mono);
    assert_eq!(left[2].width, 4.0 * 8.0 + 44.0);
    assert_eq!(right[1].width, 60.0);
  }

  #[test]
  fn menu_ids_are_recognised() {
    assert!(is_menu(LOGO) && is_menu(M_HELP) && is_menu(NATIVE0) && is_menu(NATIVE0 + 19));
    assert!(!is_menu(NATIVE0 + 20) && !is_menu(CLOCK) && !is_menu(WIFI));
    assert_eq!(native_index(NATIVE0 + 3), Some(3));
    assert_eq!(native_index(NATIVE0 - 1), None);
  }

  fn show_ids(show: &Show, avail: &Available, tweak: impl FnOnce(&mut BarInputs)) -> (Vec<u32>, Vec<u32>) {
    let l = labels();
    let mut inp = inputs(Menus::Static(&l));
    tweak(&mut inp);
    let (left, right) = build_with(&inp, show, avail, &Mono);
    (ids(&left), ids(&right))
  }

  #[test]
  fn hiding_the_menu_icon_drops_only_the_logo() {
    let show = Show {
      menu_icon: false,
      ..Show::default()
    };
    let (left, _) = show_ids(&show, &Available::default(), |_| {});
    assert_eq!(left, vec![APP, M_FILE, M_EDIT, M_VIEW, M_WIN, M_HELP]);
  }

  #[test]
  fn hiding_window_options_drops_the_window_menu_but_not_native_menus() {
    let show = Show {
      window_options: false,
      ..Show::default()
    };
    let (left, _) = show_ids(&show, &Available::default(), |_| {});
    assert_eq!(left, vec![LOGO, APP, M_FILE, M_EDIT, M_VIEW, M_HELP]);

    let titles: Vec<String> = ["A", "Window", "B"].map(String::from).to_vec();
    let (left, _) = build_with(&inputs(Menus::Native(&titles)), &show, &Available::default(), &Mono);
    assert_eq!(left.len(), 2 + 3);
  }

  #[test]
  fn search_input_method_and_account_toggles_each_remove_their_item() {
    let avail = Available {
      input_method: true,
      account: true,
      ..Available::default()
    };
    let (_, all) = show_ids(&Show::default(), &avail, |_| {});
    assert_eq!(all, vec![CLOCK, CTRL, SEARCH, ACCT, IME]);
    for (show, gone) in [
      (
        Show {
          search: false,
          ..Show::default()
        },
        SEARCH,
      ),
      (
        Show {
          input_method: false,
          ..Show::default()
        },
        IME,
      ),
      (
        Show {
          account: false,
          ..Show::default()
        },
        ACCT,
      ),
    ] {
      let (_, right) = show_ids(&show, &avail, |_| {});
      assert!(!right.contains(&gone));
      assert_eq!(right.len(), all.len() - 1);
    }
  }

  #[test]
  fn status_toggles_hide_items_even_when_the_machine_has_them() {
    let avail = Available {
      bluetooth: true,
      volume: true,
      brightness: true,
      hardware: true,
      notifications: true,
      ..Available::default()
    };
    let (_, right) = show_ids(&Show::default(), &avail, |i| {
      i.has_wifi = true;
      i.battery = Some(50);
    });
    assert_eq!(
      right,
      vec![CLOCK, CTRL, SEARCH, WIFI, BT, VOLUME, BRIGHT, HW, NOTIF, BATT]
    );
    let off = Show {
      wifi: false,
      bluetooth: false,
      audio: false,
      brightness: false,
      hardware: false,
      notifications: false,
      battery: false,
      ..Show::default()
    };
    let (_, right) = show_ids(&off, &avail, |i| {
      i.has_wifi = true;
      i.battery = Some(50);
    });
    assert_eq!(right, vec![CLOCK, CTRL, SEARCH]);
  }

  #[test]
  fn unavailable_items_never_appear() {
    let (_, right) = show_ids(&Show::default(), &Available::default(), |_| {});
    assert_eq!(right, vec![CLOCK, CTRL, SEARCH]);
  }

  #[test]
  fn clock_item_is_dropped_when_empty_or_switched_off() {
    let (_, right) = show_ids(&Show::default(), &Available::default(), |i| i.clock = "");
    assert_eq!(right, vec![CTRL, SEARCH]);
    let off = Show {
      clock: false,
      ..Show::default()
    };
    let (_, right) = show_ids(&off, &Available::default(), |_| {});
    assert_eq!(right, vec![CTRL, SEARCH]);
  }

  #[test]
  fn tray_icons_sit_between_stage_and_wifi_and_obey_tray_off() {
    let avail = Available {
      tray: 2,
      tray_overflow: true,
      ..Available::default()
    };
    let (_, right) = show_ids(&Show::default(), &avail, |i| i.has_wifi = true);
    assert_eq!(right, vec![CLOCK, CTRL, SEARCH, TRAY0, TRAY0 + 1, TRAY_MORE, WIFI]);
    let off = Show {
      tray: false,
      ..Show::default()
    };
    let (_, right) = show_ids(&off, &avail, |_| {});
    assert_eq!(right, vec![CLOCK, CTRL, SEARCH]);
    assert_eq!(tray_index(TRAY0 + 5), Some(5));
    assert_eq!(tray_index(TRAY0 + TRAY_MAX as u32), None);
    assert_eq!(tray_index(TRAY0 - 1), None);
  }

  #[test]
  fn media_item_takes_the_budgeted_width() {
    let avail = Available {
      media: Some(180.0),
      ..Available::default()
    };
    let l = labels();
    let (_, right) = build_with(&inputs(Menus::Static(&l)), &Show::default(), &avail, &Mono);
    assert_eq!(right.last().map(|i| (i.id, i.width)), Some((MEDIA, 180.0)));
    let off = Show {
      media: false,
      ..Show::default()
    };
    let (_, right) = build_with(&inputs(Menus::Static(&l)), &off, &avail, &Mono);
    assert!(right.iter().all(|i| i.id != MEDIA));
  }

  #[test]
  fn show_follows_the_settings_rows() {
    let mut s = Settings::default();
    assert_eq!(Show::from_settings(&s), Show::default());
    s.extra.finder_menu_icon = false;
    s.extra.finder_search_icon = false;
    s.extra.finder_show_time = false;
    assert!(Show::from_settings(&s).clock, "the date is still on");
    s.extra.finder_show_date = false;
    s.extra.finder_show_volume = false;
    assert!(Show::from_settings(&s).audio, "output and media controls are still on");
    s.extra.finder_show_audio_output = false;
    s.extra.finder_show_media_controls = false;
    let show = Show::from_settings(&s);
    assert!(!show.menu_icon && !show.search && !show.clock && !show.audio);
  }
}
