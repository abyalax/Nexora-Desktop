//! System tray model (F-11.1 to F-11.4). The platform enumerates icons and reports changes (F-11.5); this module owns
//! order, the overflow rule, the "new icon" timer and the colour mapping. Time comes in as milliseconds from the host,
//! so tests inject their own clock.

/// One tray icon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Icon {
  /// Stable id from the platform (for example a hash of the owner window and icon id).
  pub id: u64,
  /// The system promoted this icon to the visible area (it is not in the overflow list on the Windows taskbar).
  pub promoted: bool,
  /// When the icon first appeared, in host milliseconds.
  pub first_seen_ms: u64,
}

/// Ordered tray icons plus the drag lock (FLB-06).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tray {
  icons: Vec<Icon>,
  locked: bool,
}

impl Tray {
  pub fn new(locked: bool) -> Self {
    Tray {
      icons: Vec::new(),
      locked,
    }
  }

  pub fn icons(&self) -> &[Icon] {
    &self.icons
  }

  pub fn set_locked(&mut self, locked: bool) {
    self.locked = locked;
  }

  pub fn is_locked(&self) -> bool {
    self.locked
  }

  /// Adds an icon at the end, or updates `promoted` when the id is known (order and `first_seen_ms` are kept).
  /// Returns `true` when the icon is new.
  pub fn upsert(&mut self, id: u64, promoted: bool, now_ms: u64) -> bool {
    if let Some(i) = self.icons.iter_mut().find(|i| i.id == id) {
      i.promoted = promoted;
      return false;
    }
    self.icons.push(Icon {
      id,
      promoted,
      first_seen_ms: now_ms,
    });
    true
  }

  /// Removes an icon. Returns `true` when it existed.
  pub fn remove(&mut self, id: u64) -> bool {
    let before = self.icons.len();
    self.icons.retain(|i| i.id != id);
    self.icons.len() != before
  }

  /// Moves the icon `id` so it sits at index `to` (clamped). Refused while the order is locked or the id is unknown.
  pub fn move_to(&mut self, id: u64, to: usize) -> bool {
    if self.locked {
      return false;
    }
    let Some(from) = self.icons.iter().position(|i| i.id == id) else {
      return false;
    };
    let icon = self.icons.remove(from);
    let to = to.min(self.icons.len());
    self.icons.insert(to, icon);
    true
  }

  /// Splits the icons, in order, into the ones drawn in the bar and the ones in the overflow list.
  ///
  /// With `show_all` (TRY-01) nothing overflows. Otherwise promoted icons are drawn, the rest overflow, and an icon that
  /// appeared less than `new_secs` ago (TRY-04) is drawn for that long even when it is not promoted (F-11.3).
  pub fn split(&self, show_all: bool, now_ms: u64, new_secs: u16) -> (Vec<&Icon>, Vec<&Icon>) {
    let mut shown = Vec::new();
    let mut overflow = Vec::new();
    for i in &self.icons {
      if show_all || i.promoted || is_new(i, now_ms, new_secs) {
        shown.push(i);
      } else {
        overflow.push(i);
      }
    }
    (shown, overflow)
  }

  /// The next moment at which a "new icon" window ends, so the host can arm one timer instead of polling.
  pub fn next_expiry_ms(&self, now_ms: u64, new_secs: u16) -> Option<u64> {
    self
      .icons
      .iter()
      .filter(|i| !i.promoted && is_new(i, now_ms, new_secs))
      .map(|i| i.first_seen_ms + u64::from(new_secs) * 1000)
      .min()
  }
}

/// Whether the icon is inside its "newly started" window. `0` seconds switches the feature off.
pub fn is_new(i: &Icon, now_ms: u64, new_secs: u16) -> bool {
  new_secs > 0 && now_ms.saturating_sub(i.first_seen_ms) < u64::from(new_secs) * 1000
}

/// How an icon is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Render {
  /// The icon's own colours.
  Original,
  /// A single-colour silhouette in this foreground.
  Mono([u8; 3]),
}

/// Coloured or monochrome icons (FLB-08). Monochrome takes the bar foreground (`bar_fg`, from the bar look), so icons
/// stay readable on a light and on a dark bar.
pub fn render(colored: bool, bar_fg: [u8; 3]) -> Render {
  if colored {
    Render::Original
  } else {
    Render::Mono(bar_fg)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tray(ids: &[(u64, bool)]) -> Tray {
    let mut t = Tray::new(false);
    for (id, promoted) in ids {
      t.upsert(*id, *promoted, 0);
    }
    t
  }

  fn ids(v: &[&Icon]) -> Vec<u64> {
    v.iter().map(|i| i.id).collect()
  }

  #[test]
  fn upsert_appends_new_icons_and_updates_known_ones_in_place() {
    let mut t = Tray::new(false);
    assert!(t.upsert(1, false, 100));
    assert!(t.upsert(2, true, 200));
    assert!(!t.upsert(1, true, 900));
    assert_eq!(
      t.icons()[0],
      Icon {
        id: 1,
        promoted: true,
        first_seen_ms: 100
      }
    );
    assert_eq!(t.icons().len(), 2);
  }

  #[test]
  fn remove_reports_whether_the_icon_existed() {
    let mut t = tray(&[(1, true), (2, true)]);
    assert!(t.remove(1));
    assert!(!t.remove(1));
    assert_eq!(t.icons().len(), 1);
  }

  #[test]
  fn reorder_moves_an_icon_and_clamps_the_target() {
    let mut t = tray(&[(1, true), (2, true), (3, true)]);
    assert!(t.move_to(3, 0));
    assert_eq!(ids(&t.icons().iter().collect::<Vec<_>>()), vec![3, 1, 2]);
    assert!(t.move_to(3, 99));
    assert_eq!(ids(&t.icons().iter().collect::<Vec<_>>()), vec![1, 2, 3]);
    assert!(!t.move_to(42, 0));
  }

  #[test]
  fn lock_refuses_reordering_and_unlock_allows_it() {
    let mut t = tray(&[(1, true), (2, true)]);
    t.set_locked(true);
    assert!(t.is_locked());
    assert!(!t.move_to(2, 0));
    assert_eq!(t.icons()[0].id, 1);
    t.set_locked(false);
    assert!(t.move_to(2, 0));
    assert_eq!(t.icons()[0].id, 2);
  }

  #[test]
  fn overflow_keeps_unpromoted_icons_out_of_the_bar() {
    let t = tray(&[(1, true), (2, false), (3, true)]);
    let (shown, overflow) = t.split(false, 10_000, 5);
    assert_eq!(ids(&shown), vec![1, 3]);
    assert_eq!(ids(&overflow), vec![2]);
  }

  #[test]
  fn show_all_leaves_nothing_in_the_overflow() {
    let t = tray(&[(1, true), (2, false)]);
    let (shown, overflow) = t.split(true, 10_000, 5);
    assert_eq!(ids(&shown), vec![1, 2]);
    assert!(overflow.is_empty());
  }

  #[test]
  fn a_new_icon_is_shown_for_n_seconds_then_moves_to_the_overflow() {
    let mut t = Tray::new(false);
    t.upsert(7, false, 1_000);
    assert_eq!(ids(&t.split(false, 1_000, 5).0), vec![7]);
    assert_eq!(ids(&t.split(false, 5_999, 5).0), vec![7]);
    let (shown, overflow) = t.split(false, 6_000, 5);
    assert!(shown.is_empty());
    assert_eq!(ids(&overflow), vec![7]);
  }

  #[test]
  fn zero_seconds_turns_the_new_icon_window_off() {
    let mut t = Tray::new(false);
    t.upsert(7, false, 1_000);
    assert!(t.split(false, 1_000, 0).0.is_empty());
    assert_eq!(t.next_expiry_ms(1_000, 0), None);
  }

  #[test]
  fn next_expiry_is_the_earliest_open_window_and_ignores_promoted_icons() {
    let mut t = Tray::new(false);
    t.upsert(1, false, 2_000);
    t.upsert(2, false, 1_000);
    t.upsert(3, true, 500);
    assert_eq!(t.next_expiry_ms(2_500, 5), Some(6_000));
    assert_eq!(t.next_expiry_ms(6_000, 5), Some(7_000));
    assert_eq!(t.next_expiry_ms(7_000, 5), None);
  }

  #[test]
  fn clock_going_backwards_does_not_underflow() {
    let mut t = Tray::new(false);
    t.upsert(1, false, 5_000);
    assert_eq!(ids(&t.split(false, 1_000, 5).0), vec![1]);
  }

  #[test]
  fn colored_keeps_the_original_and_monochrome_takes_the_bar_foreground() {
    assert_eq!(render(true, [255, 255, 255]), Render::Original);
    assert_eq!(render(false, [255, 255, 255]), Render::Mono([255, 255, 255]));
    assert_eq!(render(false, [24, 24, 28]), Render::Mono([24, 24, 28]));
  }
}
