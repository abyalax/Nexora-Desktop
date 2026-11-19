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

/// Builds the entry strip. `items` must be ordered with pinned items first.
pub fn build(items: &[Item], show_downloads: bool) -> Vec<Entry> {
  let mut entries = vec![Entry::Start, Entry::Launchpad];
  let pinned = items.iter().filter(|i| i.pinned).count();
  for i in 0..items.len() {
    if i == pinned && pinned > 0 {
      entries.push(Entry::Sep);
    }
    entries.push(Entry::App(i));
  }
  entries.push(Entry::Sep);
  if show_downloads {
    entries.push(Entry::Downloads);
  }
  entries.push(Entry::Trash);
  entries
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
}
