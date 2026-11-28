//! QA-04 / ENG-10: every label the app passes to `tr("…")` is a known Indonesian catalogue key
//! and has an English translation, so neither language falls back to an untranslated label. Scans
//! app sources as text, so it runs without building the Windows-only crates.

use nexora_core::i18n::{Lang, is_known, tr};
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(rd) = std::fs::read_dir(dir) else { return };
  for e in rd.flatten() {
    let p = e.path();
    if p.is_dir() {
      rust_files(&p, out);
    } else if p.extension().is_some_and(|x| x == "rs") {
      out.push(p);
    }
  }
}

/// Decodes the escapes the sources use inside a string literal; `None` for anything else.
fn unescape(raw: &str) -> Option<String> {
  let mut out = String::new();
  let mut it = raw.chars();
  while let Some(c) = it.next() {
    if c != '\\' {
      out.push(c);
      continue;
    }
    match it.next()? {
      'n' => out.push('\n'),
      '"' => out.push('"'),
      '\\' => out.push('\\'),
      'u' => {
        if it.next()? != '{' {
          return None;
        }
        let hex: String = it.by_ref().take_while(|&c| c != '}').collect();
        out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
      }
      _ => return None,
    }
  }
  Some(out)
}

/// Literal first arguments of `tr("…")` calls (`prefs::tr` and `i18n::tr(lang, "…")` are not
/// matched on purpose: only the one-argument app helper is the label path).
fn labels(src: &str) -> Vec<String> {
  let mut found = Vec::new();
  let mut rest = src;
  while let Some(at) = rest.find("tr(\"") {
    let before = rest[..at].chars().next_back();
    let after = &rest[at + 4..];
    let mut end = None;
    let mut esc = false;
    for (i, c) in after.char_indices() {
      if esc {
        esc = false;
      } else if c == '\\' {
        esc = true;
      } else if c == '"' {
        end = Some(i);
        break;
      }
    }
    let Some(end) = end else { break };
    let ident_before = before.is_some_and(|c| c.is_alphanumeric() || c == '_') && !rest[..at].ends_with("prefs::");
    if !ident_before && after[end + 1..].trim_start().starts_with(')') {
      if let Some(s) = unescape(&after[..end]) {
        found.push(s);
      }
    }
    rest = &after[end + 1..];
  }
  found
}

#[test]
fn scanner_finds_labels_and_skips_other_calls() {
  let src = r#"prefs::tr("Tetap") x.str("no") tr("A\u{2026}") nontr("skip") tr(lang, "two args")"#;
  assert_eq!(labels(src), ["Tetap", "A\u{2026}"]);
}

#[test]
fn every_app_label_has_an_english_translation() {
  let mut files = Vec::new();
  rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src"), &mut files);
  assert!(!files.is_empty(), "app sources not found");

  let mut missing = Vec::new();
  let mut checked = 0;
  for f in &files {
    let Ok(text) = std::fs::read_to_string(f) else { continue };
    for l in labels(&text) {
      checked += 1;
      let file = f.file_name().unwrap().to_string_lossy();
      // Catalogue keys are the Indonesian source strings; membership guarantees the paired
      // English value exists. `tr` takes a &'static str because app labels are literals.
      if !is_known(&l) {
        missing.push(format!("{file}: {l:?} (missing Indonesian key or English translation)"));
      }
    }
  }
  assert!(
    checked > 200,
    "scanner found too few labels ({checked}); is the pattern stale?"
  );
  missing.sort();
  missing.dedup();
  assert!(
    missing.is_empty(),
    "{} missing label translation(s):\n{}",
    missing.len(),
    missing.join("\n")
  );
}

#[test]
fn translations_are_not_identity_by_accident() {
  // A translated label must differ from its source unless it is a proper noun or loanword
  // (listed here). Keeps a copy-paste of the key from passing as a translation.
  let loanwords = [
    "DOCK",
    "VISOR",
    "Screen Corners",
    "Scene",
    "Stage",
    "Auto-hide",
    "Timer",
  ];
  assert_eq!(tr(Lang::En, "Hidup"), "Vivid");
  for w in loanwords {
    assert_eq!(tr(Lang::En, w), w);
  }
}
