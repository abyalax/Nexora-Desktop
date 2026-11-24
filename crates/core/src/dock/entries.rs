//! Logical Dock entries: the ordered strip of Start, Launchpad, apps, separators and utilities.

use super::items::Item;
use super::layout::Slot;

/// Item key of the built-in Start button.
pub const START_KEY: &str = "nexora:start";
/// Item key of the built-in Launchpad button.
pub const LAUNCHPAD_KEY: &str = "nexora:launchpad";
/// Shell path opened by the Trash entry.
pub const TRASH_PATH: &str = "shell:RecycleBinFolder";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
  Start,
  Launchpad,
  /// Index into the Dock item list.
  App(usize),
  Sep,
  Trash,
  Downloads,
}

/// Options that change which entries the strip holds (DAD-08, DAD-09, DAD-10) plus the Downloads utility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
  pub show_downloads: bool,
  /// DAD-08: dot under running programs.
  pub show_indicators: bool,
  /// DAD-09: icons of running programs that are not pinned.
  pub show_unpinned: bool,
  /// DAD-10: separator between the pinned and the unpinned group.
  pub separate_unpinned: bool,
}

impl Default for Options {
  /// The catalogue defaults (`docs/settings_catalog.md`): indicators and unpinned icons on, separator off.
  fn default() -> Self {
    Options {
      show_downloads: false,
      show_indicators: true,
      show_unpinned: true,
      separate_unpinned: false,
    }
  }
}

/// Whether `item` gets an icon. Pinned items always do. An unpinned item is a running program (or a recent app that is
/// no longer running); only the running ones are governed by `show_unpinned`.
pub fn shows_item(item: &Item, o: &Options) -> bool {
  item.pinned || o.show_unpinned || !item.running()
}

/// Whether `item` gets the running-indicator dot.
pub fn shows_indicator(item: &Item, o: &Options) -> bool {
  o.show_indicators && item.running()
}

/// Builds the entry strip. `items` must be ordered with pinned items first; `Entry::App` keeps the index into `items`
/// even when earlier items are filtered out. The separator is drawn only when `separate_unpinned` is on and both
/// groups have at least one visible icon.
pub fn build_with(items: &[Item], o: &Options) -> Vec<Entry> {
  let shown: Vec<usize> = (0..items.len()).filter(|&i| shows_item(&items[i], o)).collect();
  let pinned = shown.iter().filter(|&&i| items[i].pinned).count();
  let mut entries = vec![Entry::Start, Entry::Launchpad];
  for (n, &i) in shown.iter().enumerate() {
    if o.separate_unpinned && n == pinned && pinned > 0 {
      entries.push(Entry::Sep);
    }
    entries.push(Entry::App(i));
  }
  entries.push(Entry::Sep);
  if o.show_downloads {
    entries.push(Entry::Downloads);
  }
  entries.push(Entry::Trash);
  entries
}

/// Builds the entry strip the way versions up to 0.4.0 did: every item, with a separator after the pinned group.
/// New code calls [`build_with`].
pub fn build(items: &[Item], show_downloads: bool) -> Vec<Entry> {
  let o = Options {
    show_downloads,
    show_indicators: true,
    show_unpinned: true,
    separate_unpinned: true,
  };
  build_with(items, &o)
}

/// Maps entries to layout slots (separators are narrow, everything else is an icon).
pub fn slots(entries: &[Entry]) -> Vec<Slot> {
  entries
    .iter()
    .map(|e| {
      if matches!(e, Entry::Sep) {
        Slot::Separator
      } else {
        Slot::Item
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn item(pinned: bool) -> Item {
    Item {
      key: String::new(),
      name: String::new(),
      path: String::new(),
      pinned,
      hwnds: vec![],
    }
  }

  #[test]
  fn separates_pinned_from_running() {
    let e = build(&[item(true), item(false)], false);
    assert_eq!(
      e,
      vec![
        Entry::Start,
        Entry::Launchpad,
        Entry::App(0),
        Entry::Sep,
        Entry::App(1),
        Entry::Sep,
        Entry::Trash
      ]
    );
  }

  #[test]
  fn no_separator_without_pins() {
    let e = build(&[item(false)], true);
    assert_eq!(
      e,
      vec![
        Entry::Start,
        Entry::Launchpad,
        Entry::App(0),
        Entry::Sep,
        Entry::Downloads,
        Entry::Trash
      ]
    );
  }

  #[test]
  fn slots_mark_separators() {
    let s = slots(&[Entry::Start, Entry::Sep]);
    assert_eq!(s, vec![Slot::Item, Slot::Separator]);
  }

  fn running(pinned: bool) -> Item {
    Item {
      hwnds: vec![1],
      ..item(pinned)
    }
  }

  fn opts(show_unpinned: bool, separate_unpinned: bool) -> Options {
    Options {
      show_unpinned,
      separate_unpinned,
      ..Options::default()
    }
  }

  #[test]
  fn indicator_follows_the_flag_and_running_state() {
    let on = Options::default();
    let off = Options {
      show_indicators: false,
      ..on
    };
    assert!(shows_indicator(&running(true), &on));
    assert!(!shows_indicator(&running(true), &off));
    assert!(
      !shows_indicator(&item(true), &on),
      "a pinned app that is not running has no dot"
    );
  }

  #[test]
  fn unpinned_matrix() {
    // (pinned, running, show_unpinned) -> visible
    let cases = [
      (true, true, true, true),
      (true, true, false, true),
      (true, false, false, true),
      (false, true, true, true),
      (false, true, false, false),
      // a recent app that is not running is not an "unpinned running program"
      (false, false, false, true),
      (false, false, true, true),
    ];
    for (pinned, run, show, want) in cases {
      let it = if run { running(pinned) } else { item(pinned) };
      assert_eq!(
        shows_item(&it, &opts(show, false)),
        want,
        "pinned={pinned} running={run} show={show}"
      );
    }
  }

  #[test]
  fn hidden_unpinned_keep_original_indices() {
    let items = [item(true), running(false), item(true)];
    let e = build_with(&items, &opts(false, true));
    assert_eq!(
      e,
      vec![
        Entry::Start,
        Entry::Launchpad,
        Entry::App(0),
        Entry::App(2),
        Entry::Sep,
        Entry::Trash
      ]
    );
  }

  #[test]
  fn separator_needs_both_groups_and_the_flag() {
    let sep = |items: &[Item], flag: bool| {
      build_with(items, &opts(true, flag))
        .iter()
        .filter(|e| **e == Entry::Sep)
        .count()
    };
    // trailing separator before the utilities is always there, so a group separator makes it 2
    assert_eq!(sep(&[item(true), running(false)], true), 2);
    assert_eq!(sep(&[item(true), running(false)], false), 1, "flag off");
    assert_eq!(sep(&[item(true), item(true)], true), 1, "pinned only");
    assert_eq!(sep(&[running(false), running(false)], true), 1, "unpinned only");
    assert_eq!(sep(&[], true), 1, "empty");
  }

  #[test]
  fn separator_disappears_when_the_unpinned_group_is_hidden() {
    let items = [item(true), running(false)];
    let e = build_with(&items, &opts(false, true));
    assert_eq!(e.iter().filter(|x| **x == Entry::Sep).count(), 1);
  }

  #[test]
  fn legacy_build_still_separates_after_the_pinned_group() {
    let e = build(&[item(true), running(false)], false);
    assert_eq!(e[3], Entry::Sep);
  }
}
