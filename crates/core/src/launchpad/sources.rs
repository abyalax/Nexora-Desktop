//! Launchpad icon sources and grid (D-8.1, D-8.3, D-8.4).

use crate::settings::{ListKind, ProgramLists};

/// Where an app entry was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
  /// Added to the Launchpad by the user.
  Added,
  StartMenu,
  Desktop,
}

/// Which origins are listed (DLP-04).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sources {
  Added,
  AddedStartMenu,
  AddedDesktop,
  All,
}

impl Sources {
  /// From the stored choice index. Unknown values fall back to `Added`.
  pub fn from_index(i: u8) -> Sources {
    match i {
      1 => Sources::AddedStartMenu,
      2 => Sources::AddedDesktop,
      3 => Sources::All,
      _ => Sources::Added,
    }
  }

  pub fn includes(self, o: Origin) -> bool {
    matches!((self, o), (_, Origin::Added) | (Sources::All, _) | (Sources::AddedStartMenu, Origin::StartMenu) | (Sources::AddedDesktop, Origin::Desktop))
  }
}

/// An app as enumerated by the platform layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
  pub name: String,
  pub origin: Origin,
}

/// Entries allowed by `sources`. Start Menu entries the user deleted (DLP-05) are dropped; other origins are not
/// affected by that list.
pub fn visible<'a>(entries: &'a [Entry], sources: Sources, lists: &ProgramLists) -> Vec<&'a Entry> {
  entries
    .iter()
    .filter(|e| sources.includes(e.origin))
    .filter(|e| !(e.origin == Origin::StartMenu && lists.contains(ListKind::DeletedStartMenuPrograms, &e.name)))
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
  pub cols: usize,
  pub rows: usize,
  pub per_page: usize,
}

impl Grid {
  pub fn pages(&self, count: usize) -> usize {
    if self.per_page == 0 {
      0
    } else {
      count.div_ceil(self.per_page)
    }
  }
}

/// Grid from the area, icon size and spacings (DLP-01 to DLP-03). Always at least one cell per page.
pub fn grid(width: f32, height: f32, icon: u16, h_space: u16, v_space: u16) -> Grid {
  let (cw, ch) = (icon as f32 + h_space as f32, icon as f32 + v_space as f32);
  let cols = (((width.max(0.0) + h_space as f32) / cw.max(1.0)).floor() as usize).max(1);
  let rows = (((height.max(0.0) + v_space as f32) / ch.max(1.0)).floor() as usize).max(1);
  Grid {
    cols,
    rows,
    per_page: cols * rows,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn e(name: &str, origin: Origin) -> Entry {
    Entry {
      name: name.into(),
      origin,
    }
  }

  fn all() -> Vec<Entry> {
    vec![
      e("A", Origin::Added),
      e("S", Origin::StartMenu),
      e("D", Origin::Desktop),
    ]
  }

  fn names(v: Vec<&Entry>) -> Vec<&str> {
    v.into_iter().map(|e| e.name.as_str()).collect()
  }

  #[test]
  fn each_source_choice_lists_the_right_origins() {
    let l = ProgramLists::default();
    let a = all();
    assert_eq!(names(visible(&a, Sources::Added, &l)), ["A"]);
    assert_eq!(names(visible(&a, Sources::AddedStartMenu, &l)), ["A", "S"]);
    assert_eq!(names(visible(&a, Sources::AddedDesktop, &l)), ["A", "D"]);
    assert_eq!(names(visible(&a, Sources::All, &l)), ["A", "S", "D"]);
  }

  #[test]
  fn deleted_start_menu_programs_are_hidden_case_insensitively() {
    let mut l = ProgramLists::default();
    l.add(ListKind::DeletedStartMenuPrograms, "s");
    let a = all();
    assert_eq!(names(visible(&a, Sources::All, &l)), ["A", "D"]);
  }

  #[test]
  fn deleted_list_does_not_hide_added_or_desktop_entries() {
    let mut l = ProgramLists::default();
    l.add(ListKind::DeletedStartMenuPrograms, "A");
    l.add(ListKind::DeletedStartMenuPrograms, "D");
    assert_eq!(visible(&all(), Sources::All, &l).len(), 3);
  }

  #[test]
  fn unknown_index_is_added_only() {
    assert_eq!(Sources::from_index(42), Sources::Added);
  }

  #[test]
  fn grid_counts_columns_rows_and_pages() {
    let g = grid(1000.0, 600.0, 52, 22, 24);
    assert_eq!((g.cols, g.rows), (13, 8));
    assert_eq!(g.pages(g.per_page), 1);
    assert_eq!(g.pages(g.per_page + 1), 2);
    assert_eq!(g.pages(0), 0);
  }

  #[test]
  fn tiny_area_still_has_one_cell() {
    let g = grid(1.0, 1.0, 100, 60, 60);
    assert_eq!((g.cols, g.rows, g.per_page), (1, 1, 1));
  }
}
