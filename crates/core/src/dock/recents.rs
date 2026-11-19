//! Recently used apps that stay visible in the Dock after they quit.

use super::items::{Item, Pin, key_of};

/// Maximum number of remembered recents.
pub const KEEP: usize = 8;
/// Maximum number of recents shown after the running apps.
pub const SHOW: usize = 3;

/// Remembers running, unpinned apps (most recent first). Returns `true` when the list changed.
pub fn track(recents: &mut Vec<Pin>, items: &[Item]) -> bool {
  let mut changed = false;
  for it in items
    .iter()
    .filter(|i| i.running() && !i.pinned && !i.path.is_empty() && !i.path.contains('?'))
    .rev()
  {
    if !recents.iter().any(|r| key_of(&r.path) == it.key) {
      recents.insert(
        0,
        Pin {
          path: it.path.clone(),
          name: it.name.clone(),
        },
      );
      changed = true;
    }
  }
  recents.truncate(KEEP);
  changed
}

/// Appends up to [`SHOW`] recents that are not already present as items.
pub fn append(items: &mut Vec<Item>, recents: &[Pin]) {
  let mut added = 0;
  for r in recents {
    if added >= SHOW {
      break;
    }
    let k = key_of(&r.path);
    if items.iter().any(|i| i.key == k) {
      continue;
    }
    items.push(Item {
      key: k,
      name: r.name.clone(),
      path: r.path.clone(),
      pinned: false,
      hwnds: vec![],
    });
    added += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn running(path: &str, name: &str) -> Item {
    Item {
      key: key_of(path),
      name: name.into(),
      path: path.into(),
      pinned: false,
      hwnds: vec![1],
    }
  }

  #[test]
  fn tracks_new_running_apps_once() {
    let mut r = Vec::new();
    let items = [running("C:\\a\\a.exe", "A")];
    assert!(track(&mut r, &items));
    assert!(!track(&mut r, &items));
    assert_eq!(r.len(), 1);
  }

  #[test]
  fn ignores_pinned_and_empty_paths() {
    let mut r = Vec::new();
    let mut pinned = running("C:\\p\\p.exe", "P");
    pinned.pinned = true;
    let mut nopath = running("C:\\n\\n.exe", "N");
    nopath.path.clear();
    assert!(!track(&mut r, &[pinned, nopath]));
    assert!(r.is_empty());
  }

  #[test]
  fn keeps_at_most_keep_entries() {
    let mut r: Vec<Pin> = (0..KEEP + 3)
      .map(|i| Pin {
        path: format!("C:\\x\\{i}.exe"),
        name: i.to_string(),
      })
      .collect();
    track(&mut r, &[]);
    assert_eq!(r.len(), KEEP);
  }

  #[test]
  fn append_skips_existing_and_caps_at_show() {
    let recents: Vec<Pin> = (0..6)
      .map(|i| Pin {
        path: format!("C:\\x\\{i}.exe"),
        name: i.to_string(),
      })
      .collect();
    let mut items = vec![running("C:\\x\\0.exe", "0")];
    append(&mut items, &recents);
    assert_eq!(items.len(), 1 + SHOW);
    assert!(items.iter().skip(1).all(|i| !i.running() && !i.pinned));
  }
}
