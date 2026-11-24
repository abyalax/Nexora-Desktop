//! List-valued settings (ADR-0005): the program lists that cannot live in the `Copy` [`Settings`](super::Settings).
//!
//! This module is pure data: parsing, serialising, sanitising and matching. Reading and writing `lists.cfg`
//! (atomic temp-file + rename) and the shared `Arc<ProgramLists>` snapshot belong to `app` (`prefs::lists()`).
//!
//! File format: one list per `[name]` header, one name per line, `#` starts a comment line. Missing, empty or garbled
//! lines are skipped, entries before a known header and under an unknown header are ignored, and a missing file means
//! empty lists. Lists are **not** part of the `settings_version` / registry of ADR-0004.
//!
//! Matching is case-insensitive on the executable (or Start Menu entry) name: any directory part is dropped and a
//! trailing `.exe` is ignored, so `Chrome`, `chrome.exe` and `C:\Program Files\Google\Chrome\CHROME.EXE` are the
//! same entry.

use std::fmt::Write as _;

/// Upper bound per list, so a corrupt or hostile file cannot grow memory without limit.
pub const MAX_ENTRIES: usize = 1024;
/// Upper bound per entry, in characters (Windows `MAX_PATH`).
pub const MAX_NAME_LEN: usize = 260;

/// The four lists in the settings catalogue. (The two "remove windows" lists named in ADR-0005 have no catalogue row
/// yet, see WAN-06 "meaning unclear"; add a variant here when SPEC defines them.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ListKind {
  /// WAN-04, "Add excluded programs".
  ExcludedPrograms,
  /// WAN-05, "Manage running program blacklist".
  ProgramBlacklist,
  /// DMN-09, "Minimize animation blacklist".
  MinimizeBlacklist,
  /// DLP-05, "Edit deleted Start Menu programs".
  DeletedStartMenuPrograms,
}

impl ListKind {
  pub const ALL: [ListKind; 4] = [
    ListKind::ExcludedPrograms,
    ListKind::ProgramBlacklist,
    ListKind::MinimizeBlacklist,
    ListKind::DeletedStartMenuPrograms,
  ];

  /// Section header in `lists.cfg`.
  pub fn section(self) -> &'static str {
    match self {
      ListKind::ExcludedPrograms => "excluded_programs",
      ListKind::ProgramBlacklist => "program_blacklist",
      ListKind::MinimizeBlacklist => "minimize_blacklist",
      ListKind::DeletedStartMenuPrograms => "deleted_start_menu_programs",
    }
  }

  fn from_section(s: &str) -> Option<ListKind> {
    let s = s.trim();
    Self::ALL.into_iter().find(|k| k.section().eq_ignore_ascii_case(s))
  }

  fn index(self) -> usize {
    self as usize
  }
}

/// Comparison key: file name only, lower-case, without a trailing `.exe`.
fn key(name: &str) -> String {
  let base = name.rsplit(['\\', '/']).next().unwrap_or(name).trim().to_lowercase();
  match base.strip_suffix(".exe") {
    Some(stem) if !stem.is_empty() => stem.to_string(),
    _ => base,
  }
}

/// `Some(trimmed)` when `name` can be stored and read back unchanged, `None` for garbage.
fn sanitize(name: &str) -> Option<&str> {
  let n = name.trim();
  let bad = n.is_empty()
    || n.chars().count() > MAX_NAME_LEN
    || n.starts_with('#')
    || n.starts_with('[')
    || n.chars().any(|c| c.is_control() || c == ';');
  // An entry that reduces to nothing (for example just a path separator) matches nothing and is dropped too.
  if bad || key(n).is_empty() {
    None
  } else {
    Some(n)
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgramLists {
  lists: [Vec<String>; 4],
}

impl ProgramLists {
  /// Parse `lists.cfg`. Never fails: whatever cannot be read is skipped.
  pub fn parse(text: &str) -> ProgramLists {
    let mut out = ProgramLists::default();
    let mut current: Option<ListKind> = None;
    for raw in text.lines() {
      let line = raw.trim_start_matches('\u{feff}').trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }
      if let Some(inner) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
        current = ListKind::from_section(inner);
        continue;
      }
      if let Some(kind) = current {
        out.add(kind, line);
      }
    }
    out
  }

  /// Text for `lists.cfg`. Empty lists are omitted, so default lists write an empty-bodied file; output is
  /// deterministic, and `parse(serialize(x)) == x`.
  pub fn serialize(&self) -> String {
    let mut out = String::from("# nexora program lists\n");
    for kind in ListKind::ALL {
      let entries = self.get(kind);
      if entries.is_empty() {
        continue;
      }
      let _ = writeln!(out, "\n[{}]", kind.section());
      for e in entries {
        let _ = writeln!(out, "{e}");
      }
    }
    out
  }

  pub fn get(&self, kind: ListKind) -> &[String] {
    &self.lists[kind.index()]
  }

  pub fn is_empty(&self) -> bool {
    self.lists.iter().all(Vec::is_empty)
  }

  /// Case-insensitive match on the executable name (see the module docs). Garbage input matches nothing.
  pub fn contains(&self, kind: ListKind, name: &str) -> bool {
    let k = key(name);
    !k.is_empty() && self.get(kind).iter().any(|e| key(e) == k)
  }

  /// Add one entry. `false` when it is garbage, already present, or the list is full.
  pub fn add(&mut self, kind: ListKind, name: &str) -> bool {
    let Some(n) = sanitize(name) else { return false };
    if self.contains(kind, n) || self.get(kind).len() >= MAX_ENTRIES {
      return false;
    }
    self.lists[kind.index()].push(n.to_string());
    true
  }

  /// Remove every entry equal to `name` (same matching as [`contains`](Self::contains)). `true` if any went.
  pub fn remove(&mut self, kind: ListKind, name: &str) -> bool {
    let k = key(name);
    let list = &mut self.lists[kind.index()];
    let before = list.len();
    list.retain(|e| key(e) != k);
    !k.is_empty() && list.len() != before
  }

  /// Replace a whole list from the semicolon-separated text of the settings row (WAN-04 "semicolon separated").
  /// Garbage pieces and duplicates are dropped, order is kept.
  pub fn set_from_text(&mut self, kind: ListKind, text: &str) {
    self.lists[kind.index()].clear();
    for piece in text.split(';') {
      self.add(kind, piece);
    }
  }

  /// The text shown in the settings row: entries joined with `;`.
  pub fn to_text(&self, kind: ListKind) -> String {
    self.get(kind).join(";")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const K: ListKind = ListKind::MinimizeBlacklist;

  #[test]
  fn default_is_empty_and_missing_file_means_empty() {
    assert!(ProgramLists::default().is_empty());
    assert_eq!(ProgramLists::parse(""), ProgramLists::default());
    for kind in ListKind::ALL {
      assert!(!ProgramLists::default().contains(kind, "chrome.exe"));
    }
  }

  #[test]
  fn matching_is_case_insensitive_on_the_executable_name() {
    let mut l = ProgramLists::default();
    assert!(l.add(K, "Chrome.exe"));
    for hit in [
      "chrome",
      "CHROME.EXE",
      "chrome.exe",
      "C:\\Program Files\\Google\\Chrome\\chrome.exe",
      "/x/y/Chrome",
    ] {
      assert!(l.contains(K, hit), "{hit}");
    }
    for miss in ["chromium.exe", "chrome2", "", "   ", ".exe"] {
      assert!(!l.contains(K, miss), "{miss:?}");
    }
    assert!(
      !l.contains(ListKind::ExcludedPrograms, "chrome"),
      "lists are independent"
    );
  }

  #[test]
  fn add_rejects_garbage_duplicates_and_overflow() {
    let mut l = ProgramLists::default();
    assert!(l.add(K, "  notepad.exe  "));
    assert_eq!(l.get(K), ["notepad.exe"]);
    assert!(!l.add(K, "NOTEPAD"), "duplicate by matching rules");
    for bad in [
      "",
      "   ",
      "# note",
      "[section]",
      "a;b",
      "bad\nname",
      "bad\u{0}",
      "\\",
      "x".repeat(MAX_NAME_LEN + 1).as_str(),
    ] {
      assert!(!l.add(K, bad), "{bad:?}");
    }
    assert_eq!(l.get(K).len(), 1);
    let mut full = ProgramLists::default();
    for i in 0..MAX_ENTRIES {
      assert!(full.add(K, &format!("app{i}.exe")));
    }
    assert!(!full.add(K, "one-too-many.exe"));
    assert_eq!(full.get(K).len(), MAX_ENTRIES);
  }

  #[test]
  fn parse_skips_comments_garbage_and_unknown_sections() {
    let text = "\u{feff}# header\norphan.exe\n\n[minimize_blacklist]\n# a comment\nVLC.exe\n\n   \nVLC\n[unknown]\nignored.exe\n[Excluded_Programs]\ngame.exe\n[minimize_blacklist]\nbad;name\nmpv.exe\n";
    let l = ProgramLists::parse(text);
    assert_eq!(
      l.get(K),
      ["VLC.exe", "mpv.exe"],
      "orphan, comment, dup, garbage and unknown-section lines skipped"
    );
    assert_eq!(
      l.get(ListKind::ExcludedPrograms),
      ["game.exe"],
      "headers are case-insensitive"
    );
    assert!(l.get(ListKind::ProgramBlacklist).is_empty());
  }

  #[test]
  fn serialize_round_trips_and_is_deterministic() {
    let mut l = ProgramLists::default();
    l.add(ListKind::DeletedStartMenuPrograms, "Notepad");
    l.add(ListKind::ExcludedPrograms, "game.exe");
    l.add(ListKind::ExcludedPrograms, "tool.exe");
    l.add(K, "vlc.exe");
    let text = l.serialize();
    assert_eq!(ProgramLists::parse(&text), l);
    assert_eq!(l.serialize(), text);
    assert_eq!(ProgramLists::parse(&text).serialize(), text);
    let excluded = text.find("[excluded_programs]").unwrap();
    let minimize = text.find("[minimize_blacklist]").unwrap();
    let deleted = text.find("[deleted_start_menu_programs]").unwrap();
    assert!(excluded < minimize && minimize < deleted, "fixed section order");
    assert!(!text.contains("[program_blacklist]"), "empty lists are not written");
  }

  #[test]
  fn semicolon_text_round_trips_and_cleans_up() {
    let mut l = ProgramLists::default();
    l.set_from_text(ListKind::ExcludedPrograms, " a.exe ;; B.exe;a;  ;#c;[d];e.exe;");
    assert_eq!(l.get(ListKind::ExcludedPrograms), ["a.exe", "B.exe", "e.exe"]);
    assert_eq!(l.to_text(ListKind::ExcludedPrograms), "a.exe;B.exe;e.exe");
    let text = l.to_text(ListKind::ExcludedPrograms);
    let mut again = ProgramLists::default();
    again.set_from_text(ListKind::ExcludedPrograms, &text);
    assert_eq!(again, l);
    l.set_from_text(ListKind::ExcludedPrograms, "");
    assert!(l.is_empty(), "empty text clears the list");
  }

  #[test]
  fn remove_uses_the_matching_rules() {
    let mut l = ProgramLists::default();
    l.add(K, "Zoom.exe");
    assert!(!l.remove(K, "teams.exe"));
    assert!(!l.remove(K, ""));
    assert!(l.remove(K, "C:\\apps\\zoom"));
    assert!(l.get(K).is_empty());
  }
}
