//! The 0.4.0 settings registry. Its order is the on-disk legacy order; adding a row here keeps
//! parsing, defaults, ownership and byte-stable serialization together.

use super::{BarStyle, Behavior, HotAction, IconStyle, Material, Settings, ShortText, Theme};
use crate::i18n::Lang;
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
  General,
  Appearance,
  Dock,
  Finder,
  Island,
  Stage,
  Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
  Bool,
  Integer { min: i32, max: i32 },
  Enum,
  Color,
  Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
  pub key: &'static str,
  pub kind: Kind,
  pub default: String,
  pub owner: Owner,
  pub legacy: bool,
}

pub struct Registry {
  pub legacy: &'static [Entry],
  pub extensions: &'static [super::extra::Spec],
}

#[derive(Clone, Copy)]
pub struct Entry {
  pub key: &'static str,
  pub kind: Kind,
  pub default: &'static str,
  pub owner: Owner,
  pub legacy: bool,
  apply: fn(&mut Settings, &str, &mut ParseState),
  write: fn(&Settings, &mut String),
}

#[derive(Default)]
pub(super) struct ParseState {
  pub legacy_glass: bool,
  pub seen_start: bool,
  pub seen_scope: bool,
  pub seen_icons: bool,
}

impl ParseState {
  pub fn note(&mut self, key: &str) {
    match key {
      "start_method" => self.seen_start = true,
      "taskbar_scope" => self.seen_scope = true,
      "desktop_icons" => self.seen_icons = true,
      _ => {}
    }
  }
}

macro_rules! row {
  ($key:literal, $owner:ident, $kind:expr_2021, $default:literal, $apply:expr_2021, $write:expr_2021) => {
    Entry {
      key: $key,
      owner: Owner::$owner,
      kind: $kind,
      default: $default,
      legacy: true,
      apply: $apply,
      write: $write,
    }
  };
}

macro_rules! bool_row {
  ($key:literal, $field:ident, $owner:ident, $default:literal) => {
    row!(
      $key,
      $owner,
      Kind::Bool,
      $default,
      |s: &mut Settings, v: &str, _state: &mut ParseState| {
        if let Some(value) = super::codec::parse_bool(v) {
          s.$field = value;
        }
      },
      |s: &Settings, out: &mut String| {
        let _ = writeln!(out, "{}={}", $key, s.$field);
      }
    )
  };
}

macro_rules! int_row {
  ($key:literal, $field:ident, $owner:ident, $min:literal, $max:literal, $default:literal) => {
    row!(
      $key,
      $owner,
      Kind::Integer { min: $min, max: $max },
      $default,
      |s: &mut Settings, v: &str, _state: &mut ParseState| {
        if let Ok(value) = v.parse::<i32>() {
          s.$field = value.clamp($min, $max) as u8;
        }
      },
      |s: &Settings, out: &mut String| {
        let _ = writeln!(out, "{}={}", $key, s.$field);
      }
    )
  };
}

macro_rules! percent_row {
  ($key:literal, $field:ident, $owner:ident, $min:literal, $max:literal, $default:literal) => {
    row!(
      $key,
      $owner,
      Kind::Integer { min: $min, max: $max },
      $default,
      |s: &mut Settings, v: &str, _state: &mut ParseState| {
        if let Ok(value) = v.trim_end_matches('%').parse::<i32>() {
          s.$field = value.clamp($min, $max) as u8;
        }
      },
      |s: &Settings, out: &mut String| {
        let _ = writeln!(out, "{}={}", $key, s.$field);
      }
    )
  };
}

macro_rules! enum_row {
  ($key:literal, $field:ident, $ty:ty, $owner:ident, $default:literal) => {
    row!(
      $key,
      $owner,
      Kind::Enum,
      $default,
      |s: &mut Settings, v: &str, _state: &mut ParseState| {
        if let Some(value) = <$ty>::parse(v) {
          s.$field = value;
        }
      },
      |s: &Settings, out: &mut String| {
        let _ = writeln!(out, "{}={}", $key, s.$field.key());
      }
    )
  };
}

macro_rules! bool_hot_row {
  ($key:literal, $idx:literal) => {
    row!(
      $key,
      Dock,
      Kind::Enum,
      "none",
      |s: &mut Settings, v: &str, _state: &mut ParseState| {
        if let Some(value) = HotAction::parse(v) {
          s.hot[$idx] = value;
        }
      },
      |s: &Settings, out: &mut String| {
        let _ = writeln!(out, "{}={}", $key, s.hot[$idx].key());
      }
    )
  };
}

/// Ordered 0.4.0 keys. Legacy rows always serialize, even at their defaults.
pub const LEGACY: &[Entry] = &[
  bool_row!("takeover", takeover, General, "true"),
  enum_row!("dock", dock, Behavior, Dock, "fixed"),
  row!(
    "notch",
    Island,
    Kind::Enum,
    "fixed",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      // The Island offers fewer modes than the Dock; a Dock-only value in the Island key is garbage.
      if let Some(value) = Behavior::parse(v).filter(|b| b.island_ok()) {
        s.notch = value;
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "notch={}", s.notch.key());
    }
  ),
  int_row!("blur", blur, Appearance, 0, 100, "60"),
  row!(
    "style",
    Appearance,
    Kind::Enum,
    "clear",
    |_s: &mut Settings, _v: &str, _state: &mut ParseState| {},
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "style={}", s.style.key());
    }
  ),
  int_row!("transparency", transparency, Appearance, 0, 100, "50"),
  enum_row!("theme", theme, Theme, Appearance, "auto"),
  row!(
    "lang",
    General,
    Kind::Enum,
    "id",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      if let Some(value) = Lang::parse(v) {
        s.lang = value;
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "lang={}", s.lang.key());
    }
  ),
  enum_row!("dock_material", dock_material, Material, Dock, "glass"),
  int_row!("dock_round", dock_round, Dock, 0, 100, "50"),
  enum_row!("notch_material", notch_material, Material, Island, "black"),
  int_row!("notch_round", notch_round, Island, 0, 100, "50"),
  bool_row!("autostart", autostart, General, "false"),
  int_row!("dock_size", dock_size, Dock, 32, 72, "48"),
  int_row!("dock_gap", dock_gap, Dock, 0, 24, "14"),
  int_row!("dock_zoom", dock_zoom, Dock, 100, 250, "200"),
  int_row!("dock_margin", dock_margin, Dock, 0, 60, "8"),
  bool_row!("anim", anim, General, "true"),
  bool_row!("refraction", refraction, Appearance, "true"),
  bool_row!("dock_enabled", dock_enabled, Dock, "true"),
  bool_row!("notch_enabled", notch_enabled, Island, "true"),
  bool_row!("notch_hover_open", notch_hover_open, Island, "true"),
  bool_row!("win_shadow", win_shadow, Appearance, "true"),
  bool_row!("menubar", menubar, Finder, "false"),
  bool_row!("clock_24h", clock_24h, Finder, "true"),
  bool_row!("menubar_glass", menubar_glass, Finder, "false"),
  bool_hot_row!("hot_tl", 0),
  bool_hot_row!("hot_tr", 1),
  bool_hot_row!("hot_bl", 2),
  bool_hot_row!("hot_br", 3),
  bool_row!("stage", stage, Stage, "false"),
  bool_row!("switcher", switcher, Stage, "true"),
  bool_row!("stage_right", stage_right, Stage, "false"),
  int_row!("stage_size", stage_size, Stage, 70, 255, "170"),
  int_row!("stage_gap", stage_gap, Stage, 0, 60, "20"),
  int_row!("stage_count", stage_count, Stage, 2, 10, "6"),
  int_row!("stage_speed", stage_speed, Stage, 50, 200, "100"),
  bool_row!("stage_desktop_click", stage_desktop_click, Stage, "true"),
  bool_row!("stage_media", stage_media, Stage, "false"),
  int_row!("stage_icons", stage_icons, Stage, 0, 2, "2"),
  bool_row!("stage_titles", stage_titles, Stage, "true"),
  bool_row!("hide_desktop_icons", hide_desktop_icons, Stage, "true"),
  row!(
    "menubar_style",
    Finder,
    Kind::Enum,
    "light",
    |s: &mut Settings, v: &str, state: &mut ParseState| {
      if matches!(v.to_ascii_lowercase().as_str(), "glass" | "kaca") {
        state.legacy_glass = true;
      } else if let Some(value) = BarStyle::parse(v) {
        s.menubar_style = value;
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "menubar_style={}", s.menubar_style.key());
    }
  ),
  bool_row!("dock_plates", dock_plates, Dock, "true"),
  bool_row!("above_fullscreen", above_fullscreen, General, "true"),
  bool_row!("clipboard_history", clipboard_history, Input, "true"),
  enum_row!("icon_style", icon_style, IconStyle, Appearance, "default"),
  int_row!("icon_tint", icon_tint, Appearance, 0, 4, "0"),
  row!(
    "accent",
    Appearance,
    Kind::Integer { min: 0, max: 255 },
    "0",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      if let Ok(value) = v.parse::<i32>() {
        s.accent = if value == super::ACCENT_MULTI as i32 {
          super::ACCENT_MULTI
        } else {
          value.clamp(0, 7) as u8
        };
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "accent={}", s.accent);
    }
  ),
  bool_row!("mac_shortcuts", mac_shortcuts, Input, "true"),
  bool_row!("downloads_stack", downloads_stack, Dock, "true"),
  bool_row!("recent_apps", recent_apps, Dock, "true"),
  bool_row!("icon_pack", icon_pack, Appearance, "true"),
  bool_row!("menubar_material", menubar_material, Finder, "false"),
  int_row!("icon_variant", icon_variant, Appearance, 0, 2, "0"),
  int_row!("min_effect", min_effect, Dock, 0, 2, "0"),
  int_row!("notch_idle", notch_idle, Island, 0, 2, "0"),
  row!(
    "greet_name",
    Island,
    Kind::Text,
    "",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      s.greet_name = ShortText::new(v);
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "greet_name={}", s.greet_name.as_str());
    }
  ),
  row!(
    "greet_text",
    Island,
    Kind::Text,
    "",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      s.greet_text = ShortText::new(v);
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "greet_text={}", s.greet_text.as_str());
    }
  ),
  bool_row!("lyrics_compact", lyrics_compact, Island, "true"),
  bool_row!("lyrics_online", lyrics_online, Island, "true"),
  bool_row!("notch_downloads", notch_downloads, Island, "true"),
  bool_row!("notch_send", notch_send, Island, "true"),
  percent_row!("ui_scale", ui_scale, General, 75, 200, "100"),
  row!(
    "notch_island",
    Island,
    Kind::Bool,
    "false",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      if let Some(value) = super::codec::parse_bool(v) {
        s.notch_island = value;
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "notch_island={}", s.notch_island);
    }
  ),
  row!(
    "widget_style",
    Island,
    Kind::Integer { min: 0, max: 1 },
    "0",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      if let Ok(value) = v.parse::<u8>() {
        s.widget_style = value.min(1);
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "widget_style={}", s.widget_style);
    }
  ),
  row!(
    "battery_pct",
    Finder,
    Kind::Integer { min: 0, max: 2 },
    "1",
    |s: &mut Settings, v: &str, _state: &mut ParseState| {
      if let Ok(value) = v.parse::<u8>() {
        s.battery_pct = value.min(2);
      }
    },
    |s: &Settings, out: &mut String| {
      let _ = writeln!(out, "battery_pct={}", s.battery_pct);
    }
  ),
];

/// The complete settings registry. The two ordered row slices preserve the legacy file layout and extension rules.
pub const SETTINGS_REGISTRY: Registry = Registry {
  legacy: LEGACY,
  extensions: super::Extra::SPECS,
};

pub(super) fn apply(entry: &Entry, settings: &mut Settings, value: &str, state: &mut ParseState) {
  (entry.apply)(settings, value, state);
}

pub(super) fn write(entry: &Entry, settings: &Settings, out: &mut String) {
  (entry.write)(settings, out);
}

/// Unified view of both registry generations; legacy rows stay frozen while extension rows append.
pub fn all_metadata() -> impl Iterator<Item = Metadata> {
  let legacy = SETTINGS_REGISTRY.legacy.iter().map(|entry| Metadata {
    key: entry.key,
    kind: entry.kind,
    default: entry.default.to_string(),
    owner: entry.owner,
    legacy: true,
  });
  let extra = SETTINGS_REGISTRY.extensions.iter().map(|spec| {
    use super::extra::Kind as ExtraKind;
    let (kind, default) = match spec.kind {
      ExtraKind::Bool { default } => (Kind::Bool, default.to_string()),
      ExtraKind::Int { min, max, default } => (
        Kind::Integer {
          min: i32::from(min),
          max: i32::from(max),
        },
        default.to_string(),
      ),
      ExtraKind::Choice { default, .. } => (Kind::Enum, default.to_string()),
      ExtraKind::Color { default } => (Kind::Color, format!("{default:#08x}")),
    };
    Metadata {
      key: spec.key,
      kind,
      default,
      owner: spec.owner,
      legacy: false,
    }
  });
  legacy.chain(extra)
}

pub(super) const fn extra_owner(key: &str) -> Owner {
  if has_prefix(key, "dock_")
    || has_prefix(key, "launchpad_")
    || has_prefix(key, "minimize_")
    || has_prefix(key, "preview_")
    || key_is(key, "window_previews")
  {
    Owner::Dock
  } else if has_prefix(key, "finder_") || has_prefix(key, "tray_") || has_prefix(key, "battery_") {
    Owner::Finder
  } else if has_prefix(key, "notch_") || has_prefix(key, "greet_") || has_prefix(key, "lyrics_") {
    Owner::Island
  } else if has_prefix(key, "stage_") || key_is(key, "switcher") || key_is(key, "desktop_icons") {
    Owner::Stage
  } else if key_is(key, "clipboard_history") || key_is(key, "mac_shortcuts") {
    Owner::Input
  } else if key_is(key, "autostart")
    || key_is(key, "start_method")
    || key_is(key, "taskbar_scope")
    || key_is(key, "above_fullscreen")
    || key_is(key, "ui_scale")
    || key_is(key, "display_font")
  {
    Owner::General
  } else {
    Owner::Appearance
  }
}

const fn key_is(value: &str, expected: &str) -> bool {
  let value = value.as_bytes();
  let expected = expected.as_bytes();
  if value.len() != expected.len() {
    return false;
  }
  let mut i = 0;
  while i < value.len() {
    if value[i] != expected[i] {
      return false;
    }
    i += 1;
  }
  true
}

const fn has_prefix(value: &str, prefix: &str) -> bool {
  let value = value.as_bytes();
  let prefix = prefix.as_bytes();
  if value.len() < prefix.len() {
    return false;
  }
  let mut i = 0;
  while i < prefix.len() {
    if value[i] != prefix[i] {
      return false;
    }
    i += 1;
  }
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_legacy_registry_row_matches_the_golden_file_order_and_default() {
    let fixture = include_str!("../../tests/fixtures/settings_0_4_0_default.cfg");
    let keys: Vec<_> = fixture
      .lines()
      .filter_map(|line| line.split_once('=').map(|(key, _)| key))
      .collect();
    assert_eq!(
      LEGACY.iter().map(|entry| entry.key).collect::<Vec<_>>(),
      keys,
      "registry keys and order must preserve the 0.4.0 file"
    );

    let defaults = Settings::default();
    for entry in LEGACY {
      assert!(entry.legacy, "{} is a legacy key", entry.key);
      let mut line = String::new();
      write(entry, &defaults, &mut line);
      let value = line.trim_end().split_once('=').map(|(_, value)| value);
      assert_eq!(value, Some(entry.default), "{} default metadata", entry.key);
    }
  }

  #[test]
  fn registry_keys_are_unique_and_owned() {
    for (index, entry) in LEGACY.iter().enumerate() {
      assert!(
        LEGACY[..index].iter().all(|earlier| earlier.key != entry.key),
        "duplicate legacy key {}",
        entry.key
      );
      assert!(!entry.key.is_empty());
      match entry.kind {
        Kind::Integer { min, max } => assert!(min <= max, "{} range", entry.key),
        Kind::Bool | Kind::Enum | Kind::Color | Kind::Text => {}
      }
      let _owner = entry.owner;
    }
  }

  #[test]
  fn invalid_values_leave_every_non_text_legacy_setting_unchanged() {
    for entry in LEGACY {
      if entry.kind == Kind::Text {
        continue;
      }
      let mut settings = Settings::default();
      let before = settings.serialize();
      let mut state = ParseState::default();
      apply(entry, &mut settings, "not-a-setting-value", &mut state);
      assert_eq!(settings.serialize(), before, "invalid value for {}", entry.key);
    }
  }

  #[test]
  fn combined_registry_covers_legacy_and_extension_rows_once() {
    let metadata: Vec<_> = all_metadata().collect();
    assert_eq!(metadata.len(), LEGACY.len() + super::super::Extra::SPECS.len());
    for (index, entry) in metadata.iter().enumerate() {
      assert!(
        metadata[..index].iter().all(|earlier| earlier.key != entry.key),
        "duplicate setting key {}",
        entry.key
      );
      assert_eq!(entry.legacy, index < LEGACY.len(), "{} legacy marker", entry.key);
      assert!(
        !entry.default.is_empty() || entry.kind == Kind::Text,
        "{} has default metadata",
        entry.key
      );
    }
  }
}
