//! Feature start-up isolation (REC-02, `docs/feature.md` cross-cutting rule 3): a feature that
//! fails to start is recorded and skipped; every other feature still starts.
//!
//! Pure logic only. The host calls [`Startup::run`] once per feature and decides how to log.
//! `NEXORA_FAIL_FEATURE` (comma separated names, or `all`) forces a feature to fail so the
//! rule can be tested by hand on Windows (`docs/qa/rec-02-feature-failure.md`).

use std::fmt::Display;

/// Name of every feature the app starts, in start order (`app/bootstrap.rs`).
pub const START_ORDER: [&str; 15] = [
  "dock",
  "notch",
  "mission",
  "launchpad",
  "macshortcuts",
  "clipboard",
  "spotlight",
  "stage",
  "controlcenter",
  "switcher",
  "menubar",
  "topmost",
  "hotcorner",
  "fx",
  "widgets",
];

/// True when `list` (the value of `NEXORA_FAIL_FEATURE`) names `feature`. Names are matched
/// case-insensitively and may be separated by `,` or `;`. `all` matches every feature.
pub fn forced(list: Option<&str>, feature: &str) -> bool {
  let Some(list) = list else { return false };
  list
    .split([',', ';'])
    .map(str::trim)
    .filter(|n| !n.is_empty())
    .any(|n| n.eq_ignore_ascii_case("all") || n.eq_ignore_ascii_case(feature))
}

/// What happened to one feature at start-up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
  Started,
  Failed(String),
}

/// Record of a start-up run, in the order the features were attempted.
#[derive(Debug, Default)]
pub struct Startup {
  forced: Option<String>,
  outcomes: Vec<(&'static str, Outcome)>,
}

impl Startup {
  /// `forced` is the raw `NEXORA_FAIL_FEATURE` value.
  pub fn new(forced: Option<String>) -> Startup {
    Startup {
      forced,
      outcomes: Vec::new(),
    }
  }

  /// Starts `name` with `init` unless it is forced to fail. A failure never stops the caller.
  pub fn run<E: Display>(&mut self, name: &'static str, init: impl FnOnce() -> Result<(), E>) -> &Outcome {
    let outcome = if forced(self.forced.as_deref(), name) {
      Outcome::Failed("dipaksa gagal (NEXORA_FAIL_FEATURE)".into())
    } else {
      match init() {
        Ok(()) => Outcome::Started,
        Err(e) => Outcome::Failed(e.to_string()),
      }
    };
    self.outcomes.push((name, outcome));
    &self.outcomes[self.outcomes.len() - 1].1
  }

  pub fn outcomes(&self) -> &[(&'static str, Outcome)] {
    &self.outcomes
  }

  pub fn failed(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
    self.outcomes.iter().filter_map(|(n, o)| match o {
      Outcome::Failed(m) => Some((*n, m.as_str())),
      Outcome::Started => None,
    })
  }

  pub fn all_started(&self) -> bool {
    self.failed().next().is_none()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn start_all(forced_list: Option<&str>) -> (Startup, Vec<&'static str>) {
    let mut s = Startup::new(forced_list.map(str::to_string));
    let mut ran = Vec::new();
    for name in START_ORDER {
      s.run(name, || -> Result<(), String> {
        ran.push(name);
        Ok(())
      });
    }
    (s, ran)
  }

  #[test]
  fn nothing_forced_starts_everything() {
    let (s, ran) = start_all(None);
    assert!(s.all_started());
    assert_eq!(ran, START_ORDER);
  }

  #[test]
  fn each_feature_forced_to_fail_leaves_the_others_running() {
    for victim in START_ORDER {
      let (s, ran) = start_all(Some(victim));
      let failed: Vec<_> = s.failed().map(|(n, _)| n).collect();
      assert_eq!(failed, [victim], "only {victim} may fail");
      assert!(!ran.contains(&victim), "{victim} must not run its init");
      assert_eq!(
        ran.len(),
        START_ORDER.len() - 1,
        "all others still start after {victim}"
      );
    }
  }

  #[test]
  fn a_real_error_is_recorded_and_start_up_continues() {
    let mut s = Startup::new(None);
    s.run("mission", || Err::<(), _>("no compositor"));
    s.run("launchpad", || Ok::<(), String>(()));
    assert_eq!(s.outcomes()[0].1, Outcome::Failed("no compositor".into()));
    assert_eq!(s.outcomes()[1].1, Outcome::Started);
    assert!(!s.all_started());
  }

  #[test]
  fn forced_list_parsing() {
    assert!(forced(Some("fx"), "fx"));
    assert!(forced(Some(" Stage , FX;widgets"), "fx"));
    assert!(forced(Some("all"), "dock"));
    assert!(!forced(Some("stage"), "fx"));
    assert!(!forced(Some(""), "fx"));
    assert!(!forced(Some(",;"), "fx"));
    assert!(!forced(None, "fx"));
    assert!(!forced(Some("fxx"), "fx"), "whole names only");
  }

  #[test]
  fn all_forces_every_feature_and_nothing_runs() {
    let (s, ran) = start_all(Some("all"));
    assert_eq!(s.failed().count(), START_ORDER.len());
    assert!(ran.is_empty());
  }

  #[test]
  fn start_order_has_no_duplicates() {
    for (i, n) in START_ORDER.iter().enumerate() {
      assert!(!START_ORDER[i + 1..].contains(n), "duplikat: {n}");
    }
  }
}
