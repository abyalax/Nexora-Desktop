//! Pin list text format: one `path<TAB>name` line per pin, `#` comments allowed.

use super::items::Pin;

/// Parses the pin file. `name_for` supplies a display name when a line has none.
pub fn parse(text: &str, name_for: impl Fn(&str) -> String) -> Vec<Pin> {
  text
    .trim_start_matches('\u{feff}')
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .map(|l| match l.split_once('\t') {
      Some((p, n)) if !n.trim().is_empty() => Pin {
        path: p.to_string(),
        name: n.trim().to_string(),
      },
      _ => Pin {
        path: l.to_string(),
        name: name_for(l),
      },
    })
    .collect()
}

/// Serializes pins in the format read by [`parse`].
pub fn serialize(pins: &[Pin]) -> String {
  pins.iter().map(|p| format!("{}\t{}\n", p.path, p.name)).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trips() {
    let pins = vec![
      Pin {
        path: "C:\\a.exe".into(),
        name: "A".into(),
      },
      Pin {
        path: "C:\\b.exe".into(),
        name: "B B".into(),
      },
    ];
    assert_eq!(parse(&serialize(&pins), |_| String::new()), pins);
  }

  #[test]
  fn skips_comments_blanks_and_bom() {
    let t = "\u{feff}# header\n\nC:\\a.exe\tA\n";
    assert_eq!(parse(t, |_| String::new()).len(), 1);
  }

  #[test]
  fn fills_missing_names() {
    let p = parse("C:\\a.exe\n", |s| format!("<{s}>"));
    assert_eq!(p[0].name, "<C:\\a.exe>");
  }
}
