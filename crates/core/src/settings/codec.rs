//! Text codec for `settings.cfg`: `key=value` lines, tolerant of unknown keys and legacy values.

use super::*;
use std::fmt::Write as _;

impl Settings {
  pub fn parse(text: &str) -> Settings {
    let schema_version = text
      .trim_start_matches('\u{feff}')
      .lines()
      .filter_map(|line| line.trim().split_once('='))
      .filter(|(key, _)| key.trim() == "settings_version")
      .filter_map(|(_, value)| value.trim().parse::<u16>().ok())
      .filter(|version| *version > 0)
      .last()
      .unwrap_or(1);
    let mut settings = Settings {
      schema_version,
      ..Settings::default()
    };
    let mut state = super::registry::ParseState::default();
    for line in text.trim_start_matches('\u{feff}').lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }
      let Some((key, value)) = line.split_once('=') else {
        continue;
      };
      let (key, value) = (key.trim(), value.trim());
      state.note(key);
      if let Some(entry) = super::registry::LEGACY.iter().find(|entry| entry.key == key) {
        super::registry::apply(entry, &mut settings, value, &mut state);
      } else {
        settings.extra.apply(key, value);
      }
    }
    if state.legacy_glass {
      settings.menubar_glass = false;
    }
    settings.reconcile(state);
    settings
  }
  pub fn serialize(&self) -> String {
    let mut out = String::from("# nexora settings\n");
    if self.schema_version > 1 {
      let _ = writeln!(out, "settings_version={}", self.schema_version);
    }
    for entry in super::registry::LEGACY {
      super::registry::write(entry, self, &mut out);
    }
    let mut extra = String::new();
    self.extra.write_non_default(&mut extra);
    // Keys whose value the legacy bool already implies are omitted to keep 0.4.0 files byte-identical.
    let implied = [
      ("start_method", START_NAMES[usize::from(self.autostart)]),
      ("taskbar_scope", SCOPE_NAMES[usize::from(self.takeover)]),
      ("desktop_icons", ICON_NAMES[if self.hide_desktop_icons { 0 } else { 2 }]),
    ];
    for line in extra.lines() {
      let skip = implied.iter().any(|(key, value)| line == format!("{key}={value}"));
      if !skip {
        out.push_str(line);
        out.push('\n');
      }
    }
    out
  }
}
const START_NAMES: [&str; 2] = ["none", "registry"];
const SCOPE_NAMES: [&str; 2] = ["none", "all_displays"];
const ICON_NAMES: [&str; 3] = ["hide", "auto", "show"];

impl Settings {
  /// New key wins when present (legacy bool derived from it); otherwise the new value is derived from the legacy bool.
  fn reconcile(&mut self, seen: super::registry::ParseState) {
    if seen.seen_start {
      self.autostart = self.extra.start_method != 0;
    } else {
      self.extra.start_method = u8::from(self.autostart);
    }
    if seen.seen_scope {
      self.takeover = self.extra.taskbar_scope != 0;
    } else {
      self.extra.taskbar_scope = u8::from(self.takeover);
    }
    if seen.seen_icons {
      self.hide_desktop_icons = self.extra.desktop_icons != 2;
    } else {
      self.extra.desktop_icons = if self.hide_desktop_icons { 0 } else { 2 };
    }
  }

  /// Set the boot method and keep the legacy `autostart` bool in step (GEN-08).
  pub fn set_start_method(&mut self, method: u8) {
    self.extra.start_method = method.min(2);
    self.autostart = self.extra.start_method != 0;
  }

  /// Set the taskbar scope and keep the legacy `takeover` bool in step (GEN-09).
  pub fn set_taskbar_scope(&mut self, scope: u8) {
    self.extra.taskbar_scope = scope.min(2);
    self.takeover = self.extra.taskbar_scope != 0;
  }

  /// Set the desktop-icons mode and keep the legacy `hide_desktop_icons` bool in step (STG-07).
  pub fn set_desktop_icons(&mut self, mode: u8) {
    self.extra.desktop_icons = mode.min(2);
    self.hide_desktop_icons = self.extra.desktop_icons != 2;
  }
}

pub(crate) fn parse_bool(v: &str) -> Option<bool> {
  match v.to_ascii_lowercase().as_str() {
    "true" | "1" | "yes" | "on" => Some(true),
    "false" | "0" | "no" | "off" => Some(false),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn legacy_glass_spelling_turns_glass_off_even_when_the_glass_key_follows() {
    let s = Settings::parse("menubar_style=glass\nmenubar_glass=true\n");
    assert!(!s.menubar_glass);
  }

  #[test]
  fn widget_style_and_battery_pct_cap_but_reject_negative_numbers() {
    let s = Settings::parse("widget_style=9\nbattery_pct=-1\n");
    assert_eq!(s.widget_style, 1);
    assert_eq!(s.battery_pct, Settings::default().battery_pct);
  }

  #[test]
  fn dock_modes_round_trip_by_key_and_old_values_stay_stable() {
    for b in Behavior::DOCK_ALL {
      let s = Settings::parse(&format!("dock={}\n", b.key()));
      assert_eq!(s.dock, b);
      assert_eq!(Settings::parse(&s.serialize()).dock, b);
    }
    // D-10.3: the on-disk values of the 0.4.0 modes did not change.
    assert_eq!(Behavior::Fixed.key(), "fixed");
    assert_eq!(Behavior::Overlay.key(), "overlay");
    assert_eq!(Behavior::Smart.key(), "smart");
  }

  #[test]
  fn the_island_ignores_dock_only_modes() {
    let s = Settings::parse("notch=forcevisible\n");
    assert_eq!(s.notch, Settings::default().notch);
    assert_eq!(Settings::parse("notch=ondemand\n").notch, Behavior::OnDemand);
  }

  #[test]
  fn dock_only_modes_do_not_reserve_the_work_area() {
    for b in [Behavior::AlwaysShow, Behavior::ForceVisible, Behavior::DesktopOnly] {
      assert!(!b.reserves(), "{b:?}");
    }
    assert_eq!(Behavior::DOCK_ALL.len(), 7);
  }

  /// D-5.1: the Dock size, gap, zoom and margin clamp at both ends of their SPEC-03 ranges.
  #[test]
  fn dock_metrics_clamp_at_both_ends() {
    let low = Settings::parse("dock_size=-9\ndock_gap=-9\ndock_zoom=-9\ndock_margin=-9\n");
    assert_eq!(
      (low.dock_size, low.dock_gap, low.dock_zoom, low.dock_margin),
      (32, 0, 100, 0)
    );
    let high = Settings::parse("dock_size=999\ndock_gap=999\ndock_zoom=999\ndock_margin=999\n");
    assert_eq!(
      (high.dock_size, high.dock_gap, high.dock_zoom, high.dock_margin),
      (72, 24, 250, 60)
    );
    let garbage = Settings::parse("dock_size=big\ndock_gap=\n");
    assert_eq!((garbage.dock_size, garbage.dock_gap), (48, 14));
  }
}
