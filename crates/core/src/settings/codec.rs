//! Text codec for `settings.cfg`: `key=value` lines, tolerant of unknown keys and legacy values.

use super::*;

impl Settings {
  pub fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    let mut legacy_glass = false;
    let mut seen = Seen::default();
    for line in text.trim_start_matches('\u{feff}').lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }
      let Some((k, v)) = line.split_once('=') else {
        continue;
      };
      let (k, v) = (k.trim(), v.trim());
      seen.note(k);
      match k {
        "takeover" => {
          if let Some(b) = parse_bool(v) {
            s.takeover = b;
          }
        }
        "dock" => s.dock = Behavior::parse(v).unwrap_or(s.dock),
        "notch" => s.notch = Behavior::parse(v).unwrap_or(s.notch),
        "blur" => s.blur = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.blur),
        "style" => {}
        "transparency" => {
          s.transparency = v
            .parse::<i32>()
            .map(|n| n.clamp(0, 100) as u8)
            .unwrap_or(s.transparency)
        }
        "theme" => s.theme = Theme::parse(v).unwrap_or(s.theme),
        "lang" => s.lang = Lang::parse(v).unwrap_or(s.lang),
        "dock_material" => s.dock_material = Material::parse(v).unwrap_or(s.dock_material),
        "notch_material" => s.notch_material = Material::parse(v).unwrap_or(s.notch_material),
        "dock_round" => s.dock_round = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.dock_round),
        "notch_round" => s.notch_round = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.notch_round),
        "dock_size" => s.dock_size = v.parse::<i32>().map(|n| n.clamp(32, 72) as u8).unwrap_or(s.dock_size),
        "dock_gap" => s.dock_gap = v.parse::<i32>().map(|n| n.clamp(0, 24) as u8).unwrap_or(s.dock_gap),
        "dock_zoom" => s.dock_zoom = v.parse::<i32>().map(|n| n.clamp(100, 250) as u8).unwrap_or(s.dock_zoom),
        "dock_margin" => s.dock_margin = v.parse::<i32>().map(|n| n.clamp(0, 60) as u8).unwrap_or(s.dock_margin),
        "dock_enabled" => {
          if let Some(b) = parse_bool(v) {
            s.dock_enabled = b;
          }
        }
        "menubar" => {
          if let Some(b) = parse_bool(v) {
            s.menubar = b;
          }
        }
        "hot_tl" => s.hot[0] = HotAction::parse(v).unwrap_or(s.hot[0]),
        "hot_tr" => s.hot[1] = HotAction::parse(v).unwrap_or(s.hot[1]),
        "hot_bl" => s.hot[2] = HotAction::parse(v).unwrap_or(s.hot[2]),
        "hot_br" => s.hot[3] = HotAction::parse(v).unwrap_or(s.hot[3]),
        "switcher" => {
          if let Some(b) = parse_bool(v) {
            s.switcher = b;
          }
        }
        "stage" => {
          if let Some(b) = parse_bool(v) {
            s.stage = b;
          }
        }
        "stage_right" => {
          if let Some(b) = parse_bool(v) {
            s.stage_right = b;
          }
        }
        "stage_desktop_click" => {
          if let Some(b) = parse_bool(v) {
            s.stage_desktop_click = b;
          }
        }
        "icon_style" => s.icon_style = IconStyle::parse(v).unwrap_or(s.icon_style),
        "downloads_stack" => {
          if let Some(b) = parse_bool(v) {
            s.downloads_stack = b;
          }
        }
        "icon_pack" => {
          if let Some(b) = parse_bool(v) {
            s.icon_pack = b;
          }
        }
        "recent_apps" => {
          if let Some(b) = parse_bool(v) {
            s.recent_apps = b;
          }
        }
        "mac_shortcuts" => {
          if let Some(b) = parse_bool(v) {
            s.mac_shortcuts = b;
          }
        }
        "accent" => {
          s.accent = v
            .parse::<i32>()
            .map(|n| {
              if n == ACCENT_MULTI as i32 {
                ACCENT_MULTI
              } else {
                n.clamp(0, 7) as u8
              }
            })
            .unwrap_or(s.accent)
        }
        "min_effect" => s.min_effect = v.parse::<i32>().map(|n| n.clamp(0, 1) as u8).unwrap_or(s.min_effect),
        "notch_idle" => s.notch_idle = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.notch_idle),
        "greet_name" => s.greet_name = ShortText::new(v),
        "greet_text" => s.greet_text = ShortText::new(v),
        "lyrics_compact" => {
          if let Some(b) = parse_bool(v) {
            s.lyrics_compact = b;
          }
        }
        "lyrics_online" => {
          if let Some(b) = parse_bool(v) {
            s.lyrics_online = b;
          }
        }
        "notch_downloads" => {
          if let Some(b) = parse_bool(v) {
            s.notch_downloads = b;
          }
        }
        "notch_send" => {
          if let Some(b) = parse_bool(v) {
            s.notch_send = b;
          }
        }
        "icon_variant" => s.icon_variant = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.icon_variant),
        "icon_tint" => s.icon_tint = v.parse::<i32>().map(|n| n.clamp(0, 4) as u8).unwrap_or(s.icon_tint),
        "clipboard_history" => {
          if let Some(b) = parse_bool(v) {
            s.clipboard_history = b;
          }
        }
        "above_fullscreen" => {
          if let Some(b) = parse_bool(v) {
            s.above_fullscreen = b;
          }
        }
        "dock_plates" => {
          if let Some(b) = parse_bool(v) {
            s.dock_plates = b;
          }
        }
        "hide_desktop_icons" => {
          if let Some(b) = parse_bool(v) {
            s.hide_desktop_icons = b;
          }
        }
        "stage_media" => {
          if let Some(b) = parse_bool(v) {
            s.stage_media = b;
          }
        }
        "stage_titles" => {
          if let Some(b) = parse_bool(v) {
            s.stage_titles = b;
          }
        }
        "stage_size" => s.stage_size = v.parse::<i32>().map(|n| n.clamp(70, 255) as u8).unwrap_or(s.stage_size),
        "stage_gap" => s.stage_gap = v.parse::<i32>().map(|n| n.clamp(0, 60) as u8).unwrap_or(s.stage_gap),
        "stage_count" => s.stage_count = v.parse::<i32>().map(|n| n.clamp(2, 10) as u8).unwrap_or(s.stage_count),
        "stage_speed" => {
          s.stage_speed = v
            .parse::<i32>()
            .map(|n| n.clamp(50, 200) as u8)
            .unwrap_or(s.stage_speed)
        }
        "stage_icons" => s.stage_icons = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.stage_icons),
        "menubar_style" => {
          if matches!(v.to_ascii_lowercase().as_str(), "glass" | "kaca") {
            legacy_glass = true;
          } else {
            s.menubar_style = BarStyle::parse(v).unwrap_or(s.menubar_style);
          }
        }
        "menubar_material" => {
          if let Some(b) = parse_bool(v) {
            s.menubar_material = b;
          }
        }
        "menubar_glass" => {
          if let Some(b) = parse_bool(v) {
            s.menubar_glass = b;
          }
        }
        "clock_24h" => {
          if let Some(b) = parse_bool(v) {
            s.clock_24h = b;
          }
        }
        "win_shadow" => {
          if let Some(b) = parse_bool(v) {
            s.win_shadow = b;
          }
        }
        "notch_hover_open" => {
          if let Some(b) = parse_bool(v) {
            s.notch_hover_open = b;
          }
        }
        "notch_enabled" => {
          if let Some(b) = parse_bool(v) {
            s.notch_enabled = b;
          }
        }
        "refraction" => {
          if let Some(b) = parse_bool(v) {
            s.refraction = b;
          }
        }
        "anim" => {
          if let Some(b) = parse_bool(v) {
            s.anim = b;
          }
        }
        "ui_scale" => {
          s.ui_scale = v
            .trim_end_matches('%')
            .parse::<i32>()
            .map(|n| n.clamp(75, 200) as u8)
            .unwrap_or(s.ui_scale)
        }
        "notch_island" => {
          if let Some(b) = parse_bool(v) {
            s.notch_island = b;
          }
        }
        "widget_style" => s.widget_style = v.parse::<u8>().map(|n| n.min(1)).unwrap_or(s.widget_style),
        "battery_pct" => s.battery_pct = v.parse::<u8>().map(|n| n.min(2)).unwrap_or(s.battery_pct),
        "autostart" => {
          if let Some(b) = parse_bool(v) {
            s.autostart = b;
          }
        }
        _ => {
          s.extra.apply(k, v);
        }
      }
    }
    if legacy_glass {
      s.menubar_glass = false;
    }
    s.reconcile(seen);
    s
  }

  pub fn serialize(&self) -> String {
    let mut out = format!(
      "# nexora settings\ntakeover={}\ndock={}\nnotch={}\nblur={}\nstyle={}\ntransparency={}\ntheme={}\nlang={}\ndock_material={}\ndock_round={}\nnotch_material={}\nnotch_round={}\nautostart={}\ndock_size={}\ndock_gap={}\ndock_zoom={}\ndock_margin={}\nanim={}\nrefraction={}\ndock_enabled={}\nnotch_enabled={}\nnotch_hover_open={}\nwin_shadow={}\nmenubar={}\nclock_24h={}\nmenubar_glass={}\nhot_tl={}\nhot_tr={}\nhot_bl={}\nhot_br={}\nstage={}\nswitcher={}\nstage_right={}\nstage_size={}\nstage_gap={}\nstage_count={}\nstage_speed={}\nstage_desktop_click={}\nstage_media={}\nstage_icons={}\nstage_titles={}\nhide_desktop_icons={}\nmenubar_style={}\ndock_plates={}\nabove_fullscreen={}\nclipboard_history={}\nicon_style={}\nicon_tint={}\naccent={}\nmac_shortcuts={}\ndownloads_stack={}\nrecent_apps={}\nicon_pack={}\nmenubar_material={}\nicon_variant={}\nmin_effect={}\nnotch_idle={}\ngreet_name={}\ngreet_text={}\nlyrics_compact={}\nlyrics_online={}\nnotch_downloads={}\nnotch_send={}\nui_scale={}\nnotch_island={}\nwidget_style={}\nbattery_pct={}\n",
      self.takeover,
      self.dock.key(),
      self.notch.key(),
      self.blur,
      self.style.key(),
      self.transparency,
      self.theme.key(),
      self.lang.key(),
      self.dock_material.key(),
      self.dock_round,
      self.notch_material.key(),
      self.notch_round,
      self.autostart,
      self.dock_size,
      self.dock_gap,
      self.dock_zoom,
      self.dock_margin,
      self.anim,
      self.refraction,
      self.dock_enabled,
      self.notch_enabled,
      self.notch_hover_open,
      self.win_shadow,
      self.menubar,
      self.clock_24h,
      self.menubar_glass,
      self.hot[0].key(),
      self.hot[1].key(),
      self.hot[2].key(),
      self.hot[3].key(),
      self.stage,
      self.switcher,
      self.stage_right,
      self.stage_size,
      self.stage_gap,
      self.stage_count,
      self.stage_speed,
      self.stage_desktop_click,
      self.stage_media,
      self.stage_icons,
      self.stage_titles,
      self.hide_desktop_icons,
      self.menubar_style.key(),
      self.dock_plates,
      self.above_fullscreen,
      self.clipboard_history,
      self.icon_style.key(),
      self.icon_tint,
      self.accent,
      self.mac_shortcuts,
      self.downloads_stack,
      self.recent_apps,
      self.icon_pack,
      self.menubar_material,
      self.icon_variant,
      self.min_effect,
      self.notch_idle,
      self.greet_name.as_str(),
      self.greet_text.as_str(),
      self.lyrics_compact,
      self.lyrics_online,
      self.notch_downloads,
      self.notch_send,
      self.ui_scale,
      self.notch_island,
      self.widget_style,
      self.battery_pct
    );
    let mut extra = String::new();
    self.extra.write_non_default(&mut extra);
    // Keys whose value the legacy bool already implies are not written (0.4.0 files stay byte-identical).
    let implied = [
      ("start_method", START_NAMES[usize::from(self.autostart)]),
      ("taskbar_scope", SCOPE_NAMES[usize::from(self.takeover)]),
      ("desktop_icons", ICON_NAMES[if self.hide_desktop_icons { 0 } else { 2 }]),
    ];
    for line in extra.lines() {
      let skip = implied.iter().any(|(k, v)| line == format!("{k}={v}"));
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

/// Which of the new keys that shadow a legacy bool were present in the file (spec reconciliation rows 8–10).
#[derive(Default, Clone, Copy)]
struct Seen {
  start: bool,
  scope: bool,
  icons: bool,
}

impl Seen {
  fn note(&mut self, key: &str) {
    match key {
      "start_method" => self.start = true,
      "taskbar_scope" => self.scope = true,
      "desktop_icons" => self.icons = true,
      _ => {}
    }
  }
}

impl Settings {
  /// New key wins when present (legacy bool derived from it); otherwise the new value is derived from the legacy bool.
  fn reconcile(&mut self, seen: Seen) {
    if seen.start {
      self.autostart = self.extra.start_method != 0;
    } else {
      self.extra.start_method = u8::from(self.autostart);
    }
    if seen.scope {
      self.takeover = self.extra.taskbar_scope != 0;
    } else {
      self.extra.taskbar_scope = u8::from(self.takeover);
    }
    if seen.icons {
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
