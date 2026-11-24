//! Battery menu logic (F-16): remaining-time text (F-16.2) and automatic power plans (F-16.3). The platform reads the
//! battery and applies plans (F-16.4); nothing here touches the OS.

use crate::shared::i18n::Lang;

/// Where the machine gets its power.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
  Ac,
  Battery,
}

/// A Windows power mode (FBA-05, FBA-06).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
  Balanced,
  BestPerformance,
  BestPowerEfficiency,
}

impl Plan {
  /// From the registry choice index (`balanced`, `best_performance`, `best_power_efficiency`); unknown falls back to
  /// [`Plan::Balanced`].
  pub fn from_index(i: u8) -> Plan {
    match i {
      1 => Plan::BestPerformance,
      2 => Plan::BestPowerEfficiency,
      _ => Plan::Balanced,
    }
  }
}

/// The user's choices (FBA-04 to FBA-06).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prefs {
  pub auto_switch: bool,
  pub plugged: Plan,
  pub on_battery: Plan,
}

/// The plan wanted for `source`, or `None` when auto-switching is off or `current` already is that plan.
pub fn plan_for(source: Source, prefs: &Prefs, current: Plan) -> Option<Plan> {
  if !prefs.auto_switch {
    return None;
  }
  let want = match source {
    Source::Ac => prefs.plugged,
    Source::Battery => prefs.on_battery,
  };
  (want != current).then_some(want)
}

/// A power source must stay unchanged this long before the plan follows it, so a loose charger cannot make the plan
/// flap.
pub const SETTLE_MS: u64 = 3_000;

/// Hysteresis around [`plan_for`]. It acts on a *change of source* that has settled, not continuously, so a plan the
/// user picks by hand stays until the source changes again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutoPlan {
  candidate: Option<(Source, u64)>,
  applied: Option<Source>,
}

impl AutoPlan {
  /// Feeds the current source. Returns the plan to apply once, after the source has been stable for [`SETTLE_MS`] and
  /// differs from the last one acted on. The first settled source is acted on too, so the plan matches at startup.
  pub fn observe(&mut self, source: Source, now_ms: u64, prefs: &Prefs, current: Plan) -> Option<Plan> {
    let since = match self.candidate {
      Some((s, t)) if s == source && now_ms >= t => t,
      _ => {
        self.candidate = Some((source, now_ms));
        now_ms
      }
    };
    if now_ms - since < SETTLE_MS || self.applied == Some(source) {
      return None;
    }
    self.applied = Some(source);
    plan_for(source, prefs, current)
  }
}

/// Remaining-time text for the battery menu (FBA-03).
///
/// `minutes` is the estimate in minutes, `None` while Windows has none. While charging the estimate is the time until
/// full. A full battery on AC reads as charged.
pub fn remaining_text(lang: Lang, minutes: Option<u32>, charging: bool, full: bool) -> String {
  if full {
    return match lang {
      Lang::En => "Fully charged".to_string(),
      Lang::Id => "Terisi penuh".to_string(),
    };
  }
  let Some(min) = minutes else {
    return match (lang, charging) {
      (Lang::En, true) => "Charging".to_string(),
      (Lang::Id, true) => "Mengisi daya".to_string(),
      (Lang::En, false) => "Calculating\u{2026}".to_string(),
      (Lang::Id, false) => "Menghitung\u{2026}".to_string(),
    };
  };
  let (h, m) = (min / 60, min % 60);
  let span = match (lang, h, m) {
    (Lang::En, 0, m) => format!("{m} min"),
    (Lang::En, h, 0) => format!("{h} h"),
    (Lang::En, h, m) => format!("{h} h {m} min"),
    (Lang::Id, 0, m) => format!("{m} mnt"),
    (Lang::Id, h, 0) => format!("{h} j"),
    (Lang::Id, h, m) => format!("{h} j {m} mnt"),
  };
  match (lang, charging) {
    (Lang::En, true) => format!("{span} until full"),
    (Lang::Id, true) => format!("{span} hingga penuh"),
    (Lang::En, false) => format!("{span} remaining"),
    (Lang::Id, false) => format!("{span} tersisa"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const PREFS: Prefs = Prefs {
    auto_switch: true,
    plugged: Plan::BestPerformance,
    on_battery: Plan::BestPowerEfficiency,
  };

  #[test]
  fn plan_index_maps_the_registry_choices() {
    assert_eq!(Plan::from_index(0), Plan::Balanced);
    assert_eq!(Plan::from_index(1), Plan::BestPerformance);
    assert_eq!(Plan::from_index(2), Plan::BestPowerEfficiency);
    assert_eq!(Plan::from_index(9), Plan::Balanced);
  }

  #[test]
  fn plan_for_follows_the_source() {
    assert_eq!(
      plan_for(Source::Ac, &PREFS, Plan::Balanced),
      Some(Plan::BestPerformance)
    );
    assert_eq!(
      plan_for(Source::Battery, &PREFS, Plan::Balanced),
      Some(Plan::BestPowerEfficiency)
    );
  }

  #[test]
  fn plan_for_is_quiet_when_off_or_already_right() {
    let off = Prefs {
      auto_switch: false,
      ..PREFS
    };
    assert_eq!(plan_for(Source::Ac, &off, Plan::Balanced), None);
    assert_eq!(plan_for(Source::Ac, &PREFS, Plan::BestPerformance), None);
  }

  #[test]
  fn a_source_must_settle_before_the_plan_changes() {
    let mut a = AutoPlan::default();
    assert_eq!(a.observe(Source::Ac, 0, &PREFS, Plan::Balanced), None);
    assert_eq!(a.observe(Source::Ac, 2_999, &PREFS, Plan::Balanced), None);
    assert_eq!(
      a.observe(Source::Ac, 3_000, &PREFS, Plan::Balanced),
      Some(Plan::BestPerformance)
    );
  }

  #[test]
  fn flapping_never_switches_and_a_settled_source_switches_once() {
    let mut a = AutoPlan::default();
    let mut switched = Vec::new();
    let mut t = 0;
    for i in 0..40 {
      let src = if i % 2 == 0 { Source::Ac } else { Source::Battery };
      if let Some(p) = a.observe(src, t, &PREFS, Plan::Balanced) {
        switched.push(p);
      }
      t += 500;
    }
    assert!(switched.is_empty(), "a source that changes every 500 ms never settles");
    for _ in 0..10 {
      if let Some(p) = a.observe(Source::Battery, t, &PREFS, Plan::Balanced) {
        switched.push(p);
      }
      t += 1_000;
    }
    assert_eq!(switched, vec![Plan::BestPowerEfficiency]);
  }

  #[test]
  fn a_manual_plan_is_not_fought_until_the_source_changes() {
    let mut a = AutoPlan::default();
    a.observe(Source::Battery, 0, &PREFS, Plan::Balanced);
    assert_eq!(
      a.observe(Source::Battery, 3_000, &PREFS, Plan::Balanced),
      Some(Plan::BestPowerEfficiency)
    );
    assert_eq!(a.observe(Source::Battery, 60_000, &PREFS, Plan::BestPerformance), None);
    a.observe(Source::Ac, 61_000, &PREFS, Plan::BestPerformance);
    assert_eq!(
      a.observe(Source::Ac, 64_000, &PREFS, Plan::Balanced),
      Some(Plan::BestPerformance)
    );
  }

  #[test]
  fn switching_back_to_a_source_acts_again() {
    let mut a = AutoPlan::default();
    a.observe(Source::Ac, 0, &PREFS, Plan::Balanced);
    a.observe(Source::Ac, 3_000, &PREFS, Plan::Balanced);
    a.observe(Source::Battery, 10_000, &PREFS, Plan::BestPerformance);
    assert!(
      a.observe(Source::Battery, 13_000, &PREFS, Plan::BestPerformance)
        .is_some()
    );
    a.observe(Source::Ac, 20_000, &PREFS, Plan::BestPowerEfficiency);
    assert!(
      a.observe(Source::Ac, 23_000, &PREFS, Plan::BestPowerEfficiency)
        .is_some()
    );
  }

  #[test]
  fn auto_switch_off_still_consumes_the_change_without_a_plan() {
    let off = Prefs {
      auto_switch: false,
      ..PREFS
    };
    let mut a = AutoPlan::default();
    a.observe(Source::Ac, 0, &off, Plan::Balanced);
    assert_eq!(a.observe(Source::Ac, 3_000, &off, Plan::Balanced), None);
  }

  #[test]
  fn remaining_time_in_hours_and_minutes() {
    assert_eq!(remaining_text(Lang::En, Some(125), false, false), "2 h 5 min remaining");
    assert_eq!(remaining_text(Lang::En, Some(45), false, false), "45 min remaining");
    assert_eq!(remaining_text(Lang::En, Some(120), false, false), "2 h remaining");
    assert_eq!(remaining_text(Lang::Id, Some(125), false, false), "2 j 5 mnt tersisa");
    assert_eq!(remaining_text(Lang::Id, Some(45), false, false), "45 mnt tersisa");
    assert_eq!(remaining_text(Lang::Id, Some(60), false, false), "1 j tersisa");
  }

  #[test]
  fn charging_reads_as_time_until_full() {
    assert_eq!(remaining_text(Lang::En, Some(70), true, false), "1 h 10 min until full");
    assert_eq!(
      remaining_text(Lang::Id, Some(70), true, false),
      "1 j 10 mnt hingga penuh"
    );
  }

  #[test]
  fn unknown_estimates_and_full_batteries_have_their_own_text() {
    assert_eq!(remaining_text(Lang::En, None, false, false), "Calculating\u{2026}");
    assert_eq!(remaining_text(Lang::Id, None, false, false), "Menghitung\u{2026}");
    assert_eq!(remaining_text(Lang::En, None, true, false), "Charging");
    assert_eq!(remaining_text(Lang::Id, None, true, false), "Mengisi daya");
    assert_eq!(remaining_text(Lang::En, Some(10), true, true), "Fully charged");
    assert_eq!(remaining_text(Lang::Id, None, false, true), "Terisi penuh");
  }

  #[test]
  fn zero_minutes_is_reported_not_hidden() {
    assert_eq!(remaining_text(Lang::En, Some(0), false, false), "0 min remaining");
  }
}
