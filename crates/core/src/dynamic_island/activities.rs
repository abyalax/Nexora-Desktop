//! Per-activity switches (I-14): which island activities and tabs the user has enabled.
//!
//! Pure functions only. The app builds [`Live`] from the OS, passes it through [`Activities::mask`], and the normal
//! priority function [`state::activity`] then never sees a disabled activity.

use super::state::{self, Activity, Live, Tab};
use crate::settings::Settings;

/// The five user-switchable activity groups (IAC-01, 04, 05, 06, 07).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activities {
  /// Media and lyrics (Play tab, lyrics, wide player).
  pub media: bool,
  /// Download progress in the compact island.
  pub downloads: bool,
  /// File shelf (Shelf tab and its drop zone).
  pub shelf: bool,
  /// Send to phone (transfer progress and its drop zone).
  pub send: bool,
  /// Timer and calendar strip (Timer tab, timer activity).
  pub timer: bool,
}

impl Default for Activities {
  fn default() -> Self {
    Activities::ALL_ON
  }
}

impl Activities {
  pub const ALL_ON: Activities = Activities {
    media: true,
    downloads: true,
    shelf: true,
    send: true,
    timer: true,
  };

  pub const ALL_OFF: Activities = Activities {
    media: false,
    downloads: false,
    shelf: false,
    send: false,
    timer: false,
  };

  pub fn from_settings(s: &Settings) -> Activities {
    Activities {
      media: s.extra.notch_media,
      downloads: s.notch_downloads,
      shelf: s.extra.notch_shelf,
      send: s.notch_send,
      timer: s.extra.notch_timer,
    }
  }

  /// Tabs the open island may show, in display order. A tab exists only while its activity is on.
  pub fn visible_tabs(&self) -> Vec<Tab> {
    Tab::ALL.into_iter().filter(|t| self.tab_enabled(*t)).collect()
  }

  pub fn tab_enabled(&self, tab: Tab) -> bool {
    match tab {
      Tab::Play => self.media,
      Tab::Shelf => self.shelf,
      Tab::Timer => self.timer,
    }
  }

  /// First tab to open on click/hover: Play when on, otherwise the first visible one. `None` when no tab is on.
  pub fn first_tab(&self) -> Option<Tab> {
    self.visible_tabs().first().copied()
  }

  /// Whether the drag-and-drop overlay has anything to offer (shelf drop or send-to-phone zone).
  pub fn accepts_drops(&self) -> bool {
    self.shelf || self.send
  }

  /// Hide every disabled activity from the live inputs. `download` is only masked here for the *download* part: the
  /// caller ORs the send-progress part in after masking with `send`.
  pub fn mask(&self, mut live: Live) -> Live {
    if !self.media {
      live.media = false;
      live.playing = false;
      live.lyric_line = false;
      live.wide_music = false;
    }
    if !self.downloads {
      live.download = false;
    }
    if !self.shelf {
      live.shelf = 0;
    }
    if !self.timer {
      live.timer_left_s = None;
    }
    live
  }

  /// Priority winner after masking: what the collapsed island shows right now.
  pub fn activity(&self, live: &Live) -> Option<Activity> {
    state::activity(&self.mask(*live))
  }
}

/// Redirect a mode that would open a disabled tab to the first enabled tab, or to `Rest` when no tab is on.
///
/// `state::step` stays untouched (and exhaustively tested); the host calls this on its result. Example: a click opens
/// `Tab::Play`; with media off it opens the shelf instead, and with every tab off the island stays collapsed.
pub fn retarget(mode: state::Mode, a: &Activities) -> state::Mode {
  match mode {
    state::Mode::Open(t) if !a.tab_enabled(t) => a.first_tab().map_or(state::Mode::Rest, state::Mode::Open),
    m => m,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn busy() -> Live {
    Live {
      media: true,
      playing: true,
      lyric_line: true,
      lyrics_on: true,
      timer_left_s: Some(30),
      charging_fresh: false,
      shelf: 3,
      glance: false,
      download: true,
      wide_music: true,
    }
  }

  #[test]
  fn defaults_are_all_on_and_match_settings_defaults() {
    assert_eq!(Activities::default(), Activities::ALL_ON);
    assert_eq!(Activities::from_settings(&Settings::default()), Activities::ALL_ON);
  }

  #[test]
  fn tabs_follow_their_switch_in_display_order() {
    assert_eq!(Activities::ALL_ON.visible_tabs(), vec![Tab::Play, Tab::Shelf, Tab::Timer]);
    let a = Activities {
      shelf: false,
      ..Activities::ALL_ON
    };
    assert_eq!(a.visible_tabs(), vec![Tab::Play, Tab::Timer]);
    assert_eq!(Activities::ALL_OFF.visible_tabs(), Vec::<Tab>::new());
    assert_eq!(Activities::ALL_OFF.first_tab(), None);
  }

  #[test]
  fn first_tab_skips_disabled_play() {
    let a = Activities {
      media: false,
      ..Activities::ALL_ON
    };
    assert_eq!(a.first_tab(), Some(Tab::Shelf));
  }

  #[test]
  fn a_disabled_activity_never_wins_priority() {
    let live = busy();
    let on = Activities::ALL_ON;
    assert_eq!(on.activity(&live), Some(Activity::TimerEnding));
    let no_timer = Activities { timer: false, ..on };
    assert_eq!(no_timer.activity(&live), Some(Activity::Download));
    let no_dl = Activities {
      timer: false,
      downloads: false,
      ..on
    };
    assert_eq!(no_dl.activity(&live), Some(Activity::Player));
    let no_media = Activities {
      timer: false,
      downloads: false,
      media: false,
      ..on
    };
    assert_eq!(no_media.activity(&live), Some(Activity::Shelf));
  }

  #[test]
  fn all_off_leaves_only_idle_content() {
    let live = busy();
    assert_eq!(Activities::ALL_OFF.activity(&live), None);
    let idle = Live {
      glance: true,
      ..live
    };
    assert_eq!(Activities::ALL_OFF.activity(&idle), Some(Activity::Glance));
    let charging = Live {
      charging_fresh: true,
      ..live
    };
    assert_eq!(Activities::ALL_OFF.activity(&charging), Some(Activity::Charging));
  }

  #[test]
  fn masking_never_changes_unrelated_inputs() {
    let live = busy();
    let m = Activities {
      media: false,
      ..Activities::ALL_ON
    }
    .mask(live);
    assert!(!m.media && !m.playing && !m.lyric_line && !m.wide_music);
    assert_eq!((m.shelf, m.download, m.timer_left_s), (3, true, Some(30)));
    assert_eq!(Activities::ALL_ON.mask(live), live);
  }

  #[test]
  fn opening_a_disabled_tab_is_redirected() {
    use state::Mode;
    assert_eq!(retarget(Mode::Open(Tab::Play), &Activities::ALL_ON), Mode::Open(Tab::Play));
    let no_media = Activities {
      media: false,
      ..Activities::ALL_ON
    };
    assert_eq!(retarget(Mode::Open(Tab::Play), &no_media), Mode::Open(Tab::Shelf));
    let no_shelf = Activities {
      shelf: false,
      ..Activities::ALL_ON
    };
    assert_eq!(retarget(Mode::Open(Tab::Shelf), &no_shelf), Mode::Open(Tab::Play), "a drop never opens a hidden shelf");
    assert_eq!(retarget(Mode::Open(Tab::Timer), &Activities::ALL_OFF), Mode::Rest);
    for m in [Mode::Rest, Mode::Peek, Mode::Drop] {
      assert_eq!(retarget(m, &Activities::ALL_OFF), m, "non-open modes pass through");
    }
  }

  #[test]
  fn drop_overlay_needs_shelf_or_send() {
    let a = Activities {
      shelf: false,
      ..Activities::ALL_ON
    };
    assert!(a.accepts_drops());
    assert!(!Activities { send: false, ..a }.accepts_drops());
  }

  #[test]
  fn settings_switches_flow_into_activities() {
    let s = Settings::parse("notch_media=false\nnotch_shelf=false\nnotch_timer=false\nnotch_send=false\n");
    let a = Activities::from_settings(&s);
    assert_eq!(a.media | a.shelf | a.timer | a.send, false);
    assert!(a.downloads);
    let back = Settings::parse(&s.serialize());
    assert_eq!(Activities::from_settings(&back), a);
  }
}
