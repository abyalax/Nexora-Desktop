//! Hardware status item (F-15): CPU and memory. The platform supplies raw counters; this module decides how often they
//! may be read and turns them into percentages. Acceptance: nothing is sampled faster than once per second while idle.

use crate::panels::widgets::cpu_percent;

/// Hard floor for any sampling interval.
pub const MIN_INTERVAL_MS: u64 = 1_000;
/// Interval while the bar item is only a glyph (menu closed).
pub const IDLE_INTERVAL_MS: u64 = 2_000;
/// Interval while the hardware menu is open and shows live numbers.
pub const OPEN_INTERVAL_MS: u64 = 1_000;

/// The sampling interval for the current state. Never below [`MIN_INTERVAL_MS`].
pub fn interval_ms(menu_open: bool) -> u64 {
  if menu_open { OPEN_INTERVAL_MS } else { IDLE_INTERVAL_MS }.max(MIN_INTERVAL_MS)
}

/// Rate limiter for the sampler. The host calls [`Sampler::due`] from whatever timer it has, so a timer that fires too
/// often cannot cause extra reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampler {
  last_ms: Option<u64>,
  interval_ms: u64,
}

impl Sampler {
  /// A sampler with `interval_ms`, raised to the floor when lower.
  pub fn new(interval_ms: u64) -> Self {
    Sampler {
      last_ms: None,
      interval_ms: interval_ms.max(MIN_INTERVAL_MS),
    }
  }

  pub fn set_interval_ms(&mut self, interval_ms: u64) {
    self.interval_ms = interval_ms.max(MIN_INTERVAL_MS);
  }

  /// Whether a sample may be taken now. Returns `true` at most once per interval and records the sample time. A clock
  /// that went backwards counts as due, so the sampler cannot get stuck.
  pub fn due(&mut self, now_ms: u64) -> bool {
    let ok = match self.last_ms {
      None => true,
      Some(last) => now_ms < last || now_ms - last >= self.interval_ms,
    };
    if ok {
      self.last_ms = Some(now_ms);
    }
    ok
  }

  /// The next time a sample may be taken, or `None` before the first one.
  pub fn next_ms(&self) -> Option<u64> {
    self.last_ms.map(|l| l + self.interval_ms)
  }
}

/// CPU and memory use in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
  pub cpu: u8,
  pub mem: u8,
}

/// Memory use in percent from used and total bytes. An unknown total (zero) reads as 0 %.
pub fn mem_percent(used: u64, total: u64) -> u8 {
  if total == 0 {
    return 0;
  }
  (u128::from(used.min(total)) * 100 / u128::from(total)) as u8
}

/// A reading from two CPU counter pairs `(idle, total)` and the memory counters.
pub fn reading(prev_cpu: (u64, u64), now_cpu: (u64, u64), mem_used: u64, mem_total: u64) -> Reading {
  Reading {
    cpu: cpu_percent(prev_cpu, now_cpu),
    mem: mem_percent(mem_used, mem_total),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn intervals_are_never_faster_than_once_per_second() {
    assert!(interval_ms(false) >= 1_000);
    assert!(interval_ms(true) >= 1_000);
    assert!(interval_ms(false) >= interval_ms(true));
  }

  #[test]
  fn a_too_small_interval_is_raised_to_the_floor() {
    let mut s = Sampler::new(10);
    assert!(s.due(0));
    assert!(!s.due(999));
    assert!(s.due(1_000));
    s.set_interval_ms(0);
    assert!(!s.due(1_500));
  }

  #[test]
  fn a_busy_timer_cannot_cause_extra_samples() {
    let mut s = Sampler::new(IDLE_INTERVAL_MS);
    let samples = (0..10_000u64).filter(|t| s.due(*t)).count();
    assert_eq!(samples, 5, "10 s at a 2 s interval");
  }

  #[test]
  fn the_first_call_is_always_due_and_next_is_scheduled() {
    let mut s = Sampler::new(2_000);
    assert_eq!(s.next_ms(), None);
    assert!(s.due(5_000));
    assert_eq!(s.next_ms(), Some(7_000));
  }

  #[test]
  fn a_clock_that_goes_backwards_does_not_stall_the_sampler() {
    let mut s = Sampler::new(1_000);
    assert!(s.due(10_000));
    assert!(s.due(2_000));
    assert!(!s.due(2_500));
  }

  #[test]
  fn memory_percent_handles_zero_and_overflow() {
    assert_eq!(mem_percent(0, 0), 0);
    assert_eq!(mem_percent(8, 16), 50);
    assert_eq!(mem_percent(99, 16), 100);
    assert_eq!(mem_percent(u64::MAX, u64::MAX), 100);
  }

  #[test]
  fn reading_combines_cpu_and_memory() {
    let r = reading((100, 200), (150, 300), 4, 16);
    assert_eq!(r, Reading { cpu: 50, mem: 25 });
    assert_eq!(reading((5, 5), (5, 5), 0, 0), Reading { cpu: 0, mem: 0 });
  }
}
