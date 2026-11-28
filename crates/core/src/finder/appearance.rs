//! Finder auto-hide and the look of the bar over a maximized window (F-8.1, F-8.2, F-8.4).
//!
//! Like the Dock, the bar reuses the shared policy in `crate::visibility::policy`: the window host gathers [`Signals`],
//! asks for a [`Transition`] and performs the side effects (animation, `ShowWindow`, work-area changes).

use crate::settings::{Look, Settings};
use crate::visibility::policy::{Inputs, Mode, should_hide, transition_to};

pub use crate::visibility::policy::Transition;

/// Everything the Finder needs to know to decide whether it is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Signals {
  /// A window sits under the bar's strip.
  pub overlapped: bool,
  /// A full-screen app is in the foreground.
  pub fullscreen: bool,
  /// The pointer dwelled on the top edge.
  pub edge_armed: bool,
  /// The pointer is on the bar.
  pub tracking: bool,
  /// A bar menu or popup is open.
  pub menu_open: bool,
  /// Short grace period after the pointer left.
  pub grace: bool,
  /// The bar is still starting up.
  pub starting: bool,
  /// The bar is being revealed by something else (for example a hot corner).
  pub revealing: bool,
  /// The Finder is switched off by the user.
  pub off: bool,
}

impl Signals {
  /// True while the user is interacting with the bar and it must not hide.
  pub fn interacting(&self) -> bool {
    self.tracking || self.menu_open || self.grace
  }

  fn inputs(&self) -> Inputs {
    Inputs {
      overlapped: self.overlapped,
      interacting: self.interacting(),
      edge_armed: self.edge_armed,
      starting: self.starting,
      fullscreen: self.fullscreen,
      revealing: self.revealing,
      desktop: false,
    }
  }
}

/// The visibility mode for the FLB-02 toggle: the bar either stays or hides when a window covers it.
pub fn mode(auto_hide: bool) -> Mode {
  if auto_hide { Mode::AutoHide } else { Mode::Always }
}

/// Whether the bar should currently be hidden.
pub fn wants_hidden(auto_hide: bool, signals: &Signals) -> bool {
  signals.off || should_hide(mode(auto_hide), &signals.inputs())
}

/// The change needed given whether the bar is `shown` right now.
pub fn transition(auto_hide: bool, signals: &Signals, shown: bool) -> Transition {
  transition_to(wants_hidden(auto_hide, signals), shown)
}

/// Whether the bar reserves its strip in the monitor work area. With auto-hide the bar overlays windows and never
/// reserves, so revealing it does not shove windows around; with a fixed bar the strip is released while it is hidden
/// (full screen, Finder off), which is what stops a gap from being left behind.
pub fn reserves_work_area(auto_hide: bool, shown: bool) -> bool {
  !auto_hide && shown
}

/// The bar's glass look. With FLB-04 on and a maximized window on this monitor the bar turns solid (no blur, no
/// refraction, the material's solid colour) so it stays readable against the window; otherwise it is the usual glass.
pub fn bar_look(s: &Settings, system_dark: bool, maximized: bool) -> Look {
  let mut l = crate::settings::bar_look(s, system_dark);
  if maximized && s.extra.finder_maximized_affects_appearance {
    l.tint = l.solid;
    l.blur_sigma = 0.0;
    l.highlight = 0.0;
    l.liquify = false;
  }
  l
}

#[cfg(test)]
mod tests {
  use super::*;

  fn covered() -> Signals {
    Signals {
      overlapped: true,
      ..Signals::default()
    }
  }

  #[test]
  fn the_toggle_selects_the_shared_mode() {
    assert_eq!(mode(true), Mode::AutoHide);
    assert_eq!(mode(false), Mode::Always);
  }

  #[test]
  fn auto_hide_hides_only_when_covered_and_idle() {
    assert!(!wants_hidden(true, &Signals::default()));
    assert!(wants_hidden(true, &covered()));
    assert_eq!(transition(true, &covered(), true), Transition::Hide);
    assert_eq!(transition(true, &covered(), false), Transition::Keep);
  }

  #[test]
  fn a_fixed_bar_ignores_overlap() {
    assert!(!wants_hidden(false, &covered()));
    assert_eq!(transition(false, &covered(), true), Transition::Keep);
  }

  #[test]
  fn top_edge_dwell_reveals_and_interaction_keeps_the_bar_shown() {
    let revealed = Signals {
      edge_armed: true,
      ..covered()
    };
    assert_eq!(transition(true, &revealed, false), Transition::Show);
    for busy in [
      Signals {
        tracking: true,
        ..covered()
      },
      Signals {
        menu_open: true,
        ..covered()
      },
      Signals {
        grace: true,
        ..covered()
      },
    ] {
      assert!(!wants_hidden(true, &busy));
    }
  }

  #[test]
  fn fullscreen_hides_a_fixed_bar_but_not_while_interacting() {
    let fs = Signals {
      fullscreen: true,
      ..Signals::default()
    };
    assert!(wants_hidden(false, &fs));
    assert!(!wants_hidden(false, &Signals { menu_open: true, ..fs }));
  }

  #[test]
  fn switched_off_always_hides() {
    let off = Signals {
      off: true,
      revealing: true,
      ..Signals::default()
    };
    assert!(wants_hidden(false, &off));
  }

  #[test]
  fn work_area_is_released_when_hidden_and_never_held_by_an_auto_hide_bar() {
    assert!(reserves_work_area(false, true));
    assert!(!reserves_work_area(false, false));
    assert!(!reserves_work_area(true, true));
    assert!(!reserves_work_area(true, false));
  }

  #[test]
  fn maximized_window_makes_the_bar_solid() {
    let s = Settings::default();
    let glass = bar_look(&s, false, false);
    let solid = bar_look(&s, false, true);
    assert_eq!(solid.tint, glass.solid);
    assert_eq!(solid.blur_sigma, 0.0);
    assert_eq!(solid.highlight, 0.0);
    assert!(!solid.liquify);
    assert_ne!(solid.tint, glass.tint);
    assert!(glass.blur_sigma > 0.0);
  }

  #[test]
  fn switching_the_row_off_keeps_the_glass_over_maximized_windows() {
    let mut s = Settings::default();
    s.extra.finder_maximized_affects_appearance = false;
    assert_eq!(bar_look(&s, true, true), bar_look(&s, true, false));
  }

  #[test]
  fn not_maximized_matches_the_settings_look() {
    let s = Settings::default();
    assert_eq!(bar_look(&s, false, false), crate::settings::bar_look(&s, false));
  }
}
