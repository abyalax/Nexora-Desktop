//! Which items the Finder bar shows, and how wide each one is.
//!
//! The host in `app` measures text with the OS font stack through [`TextMetrics`]; deciding which items exist (logo,
//! app name, menus, clock, tray icons, battery) and their padding is domain logic and lives here.

use super::layout::Item;

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

/// Left (logo, app name, menus) and right (clock, controls, status) items with their widths.
pub fn build(inp: &BarInputs, m: &dyn TextMetrics) -> (Vec<Item>, Vec<Item>) {
  let d = inp.dpi;
  let pad = TEXT_PAD * d;
  let text = |s: &str, emphasized: bool| m.text(s, emphasized) + 2.0 * pad;

  let mut left = vec![
    Item {
      id: LOGO,
      width: inp.bar_height * LOGO_SCALE,
    },
    Item {
      id: APP,
      width: text(inp.app_label, true),
    },
  ];
  match inp.menus {
    Menus::Static(labels) => {
      for (id, label) in [M_FILE, M_EDIT, M_VIEW, M_WIN, M_HELP].into_iter().zip(labels.iter()) {
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
  let mut right = vec![
    Item {
      id: CLOCK,
      width: text(inp.clock, false),
    },
    Item { id: CTRL, width: icon },
    Item {
      id: SEARCH,
      width: icon,
    },
  ];
  if inp.show_stage {
    right.push(Item {
      id: STAGEI,
      width: icon,
    });
  }
  if inp.has_wifi {
    right.push(Item { id: WIFI, width: icon });
  }
  if let Some(pct) = inp.battery {
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
}
