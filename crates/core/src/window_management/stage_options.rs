//! Stage Manager option logic that has no OS dependency (STG-01…STG-15 gaps, WAN-06).
//!
//! Window and drawing work stays in `app/stage.rs`; this module only answers "should it show / how long".

use crate::settings::{ListKind, ProgramLists};

/// STG-11 "Window List Show Hidden Mode" (stored: `default`, `always_show`, `always_hide`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HiddenMode {
  /// Shown while the strip holds at least one group.
  Default,
  /// Shown even when the strip is empty.
  AlwaysShow,
  /// Hidden until the pointer reaches the strip edge.
  AlwaysHide,
}

impl HiddenMode {
  /// From the stored index. Unknown values mean `Default`.
  pub fn from_index(i: u8) -> HiddenMode {
    match i {
      1 => HiddenMode::AlwaysShow,
      2 => HiddenMode::AlwaysHide,
      _ => HiddenMode::Default,
    }
  }
}

/// Whether the window strip is drawn.
pub fn strip_visible(mode: HiddenMode, groups: usize, pointer_on_edge: bool) -> bool {
  match mode {
    HiddenMode::Default => groups > 0,
    HiddenMode::AlwaysShow => true,
    HiddenMode::AlwaysHide => pointer_on_edge && groups > 0,
  }
}

/// STG-07 "Show Desktop Icons" (stored: `hide`, `auto`, `show`).
/// `desktop_showing` is true when no window covers the desktop.
pub fn desktop_icons_visible(mode: u8, desktop_showing: bool) -> bool {
  match mode {
    0 => false,
    2 => true,
    _ => desktop_showing,
  }
}

/// WAN-06: windows of programs in the *excluded programs* list stay out of the strip and the switcher.
pub fn left_out_of_stage(minimize_excluded: bool, lists: &ProgramLists, exe: &str) -> bool {
  minimize_excluded && lists.contains(ListKind::ExcludedPrograms, exe)
}

/// Strip icon size (STG-13, 16–80). Values outside the range are clamped.
pub fn icon_size(stored: u16) -> u16 {
  stored.clamp(16, 80)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hidden_mode_maps_and_unknown_is_default() {
    assert_eq!(HiddenMode::from_index(0), HiddenMode::Default);
    assert_eq!(HiddenMode::from_index(1), HiddenMode::AlwaysShow);
    assert_eq!(HiddenMode::from_index(2), HiddenMode::AlwaysHide);
    assert_eq!(HiddenMode::from_index(7), HiddenMode::Default);
  }

  #[test]
  fn strip_visibility_table() {
    use HiddenMode::*;
    // (mode, groups, pointer on edge) -> visible
    let rows = [
      (Default, 0, false, false),
      (Default, 2, false, true),
      (AlwaysShow, 0, false, true),
      (AlwaysShow, 2, false, true),
      (AlwaysHide, 2, false, false),
      (AlwaysHide, 2, true, true),
      (AlwaysHide, 0, true, false),
    ];
    for (m, g, p, want) in rows {
      assert_eq!(strip_visible(m, g, p), want, "{m:?} groups={g} edge={p}");
    }
  }

  #[test]
  fn desktop_icons_follow_the_three_states() {
    assert!(!desktop_icons_visible(0, true));
    assert!(desktop_icons_visible(2, false));
    assert!(desktop_icons_visible(1, true));
    assert!(!desktop_icons_visible(1, false));
    assert!(desktop_icons_visible(9, true), "unknown behaves like auto");
  }

  #[test]
  fn excluded_programs_are_left_out_only_when_enabled() {
    let mut l = ProgramLists::default();
    l.add(ListKind::ExcludedPrograms, "Chrome.exe");
    assert!(left_out_of_stage(true, &l, "chrome"));
    assert!(left_out_of_stage(true, &l, r"C:\Apps\CHROME.EXE"));
    assert!(!left_out_of_stage(false, &l, "chrome"));
    assert!(!left_out_of_stage(true, &l, "notepad"));
  }

  #[test]
  fn icon_size_clamps() {
    assert_eq!((icon_size(0), icon_size(36), icon_size(500)), (16, 36, 80));
  }
}
