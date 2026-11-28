#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
  Always,
  AutoHide,
  OnDemand,
  /// Never hides, not even for a full-screen app (Dock "Always force visibility", D-10.3).
  Forced,
  /// Visible only while the desktop is in front (Dock "Display only on the desktop", D-10.3).
  DesktopOnly,
}

impl Mode {
  pub fn parse(s: &str) -> Option<Mode> {
    match s {
      "always" | "fixed" => Some(Mode::Always),
      "auto" | "smart" | "auto-hide" => Some(Mode::AutoHide),
      "peek" | "demand" => Some(Mode::OnDemand),
      "forced" | "force" => Some(Mode::Forced),
      "desktop" | "desktop-only" => Some(Mode::DesktopOnly),
      _ => None,
    }
  }

  /// False for the modes that keep the surface up, so the host needs no reveal strip or dwell timer for them.
  pub fn can_hide(self) -> bool {
    !matches!(self, Mode::Always | Mode::Forced)
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs {
  pub overlapped: bool,
  pub interacting: bool,
  pub edge_armed: bool,
  pub starting: bool,
  pub fullscreen: bool,
  pub revealing: bool,
  /// The desktop is in front (no app window is the foreground). Only `Mode::DesktopOnly` reads it.
  pub desktop: bool,
}

pub fn should_hide(mode: Mode, i: &Inputs) -> bool {
  if mode == Mode::Forced {
    return false;
  }
  if i.revealing {
    return false;
  }
  if i.fullscreen {
    return !i.interacting;
  }
  if i.starting {
    return false;
  }
  match mode {
    Mode::Always => false,
    Mode::AutoHide => i.overlapped && !i.interacting && !i.edge_armed,
    Mode::OnDemand => !i.interacting && !i.edge_armed,
    Mode::DesktopOnly => !i.desktop && !i.interacting && !i.edge_armed,
    Mode::Forced => false,
  }
}

/// What a window host has to do to reach the desired visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
  Hide,
  Show,
  Keep,
}

/// The step needed to go from the `shown` state to the desired one (`hide == true` means hidden is desired).
pub fn transition_to(hide: bool, shown: bool) -> Transition {
  match (hide, shown) {
    (true, true) => Transition::Hide,
    (false, false) => Transition::Show,
    _ => Transition::Keep,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn i() -> Inputs {
    Inputs::default()
  }

  #[test]
  fn always_never_hides_except_fullscreen() {
    assert!(!should_hide(
      Mode::Always,
      &Inputs {
        overlapped: true,
        ..i()
      }
    ));
    assert!(should_hide(
      Mode::Always,
      &Inputs {
        fullscreen: true,
        ..i()
      }
    ));
  }

  #[test]
  fn autohide_hides_only_when_overlapped_and_idle() {
    assert!(!should_hide(Mode::AutoHide, &i()));
    assert!(should_hide(
      Mode::AutoHide,
      &Inputs {
        overlapped: true,
        ..i()
      }
    ));
    assert!(!should_hide(
      Mode::AutoHide,
      &Inputs {
        overlapped: true,
        interacting: true,
        ..i()
      }
    ));
    assert!(!should_hide(
      Mode::AutoHide,
      &Inputs {
        overlapped: true,
        edge_armed: true,
        ..i()
      }
    ));
  }

  #[test]
  fn ondemand_hides_unless_used_or_edge_touched() {
    assert!(should_hide(Mode::OnDemand, &i()));
    assert!(!should_hide(
      Mode::OnDemand,
      &Inputs {
        interacting: true,
        ..i()
      }
    ));
    assert!(!should_hide(
      Mode::OnDemand,
      &Inputs {
        edge_armed: true,
        ..i()
      }
    ));
  }

  #[test]
  fn startup_animation_blocks_hiding() {
    assert!(!should_hide(Mode::OnDemand, &Inputs { starting: true, ..i() }));
  }

  #[test]
  fn fullscreen_hides_even_when_edge_is_armed() {
    assert!(should_hide(
      Mode::AutoHide,
      &Inputs {
        fullscreen: true,
        edge_armed: true,
        ..i()
      }
    ));
    assert!(!should_hide(
      Mode::AutoHide,
      &Inputs {
        fullscreen: true,
        interacting: true,
        ..i()
      }
    ));
  }

  #[test]
  fn minimize_reveal_shows_surface_in_every_mode() {
    for mode in [Mode::Always, Mode::AutoHide, Mode::OnDemand] {
      assert!(!should_hide(
        mode,
        &Inputs {
          revealing: true,
          overlapped: true,
          fullscreen: true,
          ..i()
        }
      ));
    }
  }

  #[test]
  fn legacy_mode_names_map_to_one_policy() {
    assert_eq!(Mode::parse("fixed"), Some(Mode::Always));
    assert_eq!(Mode::parse("smart"), Some(Mode::AutoHide));
    assert_eq!(Mode::parse("peek"), Some(Mode::OnDemand));
    assert_eq!(Mode::parse("???"), None);
  }

  #[test]
  fn transition_only_changes_when_state_and_goal_differ() {
    assert_eq!(transition_to(true, true), Transition::Hide);
    assert_eq!(transition_to(false, false), Transition::Show);
    assert_eq!(transition_to(true, false), Transition::Keep);
    assert_eq!(transition_to(false, true), Transition::Keep);
  }

  /// D-10.3: table of mode x situation -> hidden. Rows are the situations, columns the five modes in this order:
  /// Always, AutoHide, OnDemand, Forced, DesktopOnly.
  #[test]
  fn mode_by_situation_table() {
    let modes = [
      Mode::Always,
      Mode::AutoHide,
      Mode::OnDemand,
      Mode::Forced,
      Mode::DesktopOnly,
    ];
    let rows: [(&str, Inputs, [bool; 5]); 8] = [
      ("idle, app in front", i(), [false, false, true, false, true]),
      (
        "idle, desktop in front",
        Inputs { desktop: true, ..i() },
        [false, false, true, false, false],
      ),
      (
        "window covers it",
        Inputs {
          overlapped: true,
          ..i()
        },
        [false, true, true, false, true],
      ),
      (
        "pointer on it",
        Inputs {
          interacting: true,
          overlapped: true,
          ..i()
        },
        [false, false, false, false, false],
      ),
      (
        "edge armed",
        Inputs {
          edge_armed: true,
          overlapped: true,
          ..i()
        },
        [false, false, false, false, false],
      ),
      (
        "full screen",
        Inputs {
          fullscreen: true,
          desktop: true,
          ..i()
        },
        [true, true, true, false, true],
      ),
      (
        "full screen, pointer on it",
        Inputs {
          fullscreen: true,
          interacting: true,
          ..i()
        },
        [false, false, false, false, false],
      ),
      (
        "starting up",
        Inputs { starting: true, ..i() },
        [false, false, false, false, false],
      ),
    ];
    for (name, inputs, expected) in rows {
      for (mode, want) in modes.into_iter().zip(expected) {
        assert_eq!(should_hide(mode, &inputs), want, "{mode:?} when {name}");
      }
    }
  }

  #[test]
  fn only_always_and_forced_never_need_a_reveal_strip() {
    assert!(!Mode::Always.can_hide() && !Mode::Forced.can_hide());
    assert!(Mode::AutoHide.can_hide() && Mode::OnDemand.can_hide() && Mode::DesktopOnly.can_hide());
  }

  #[test]
  fn revealing_wins_over_every_mode_but_never_hides_a_forced_one() {
    for mode in [
      Mode::Always,
      Mode::AutoHide,
      Mode::OnDemand,
      Mode::Forced,
      Mode::DesktopOnly,
    ] {
      assert!(
        !should_hide(
          mode,
          &Inputs {
            revealing: true,
            fullscreen: true,
            ..i()
          }
        ),
        "{mode:?}"
      );
    }
  }
}
