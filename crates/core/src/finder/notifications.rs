//! Notification queue of the Finder (F-12.1). The platform listener (F-12.2) pushes toasts; the menu and the Dynamic
//! Island (F-12.4) read what is still alive. Entries live for the duration setting (FNO-03) and identical toasts
//! collapse into one with a counter.

/// Largest number of entries kept; the oldest is dropped first.
pub const CAPACITY: usize = 50;
/// FNO-03 range, in seconds.
pub const TTL_MIN_SECS: u16 = 1;
pub const TTL_MAX_SECS: u16 = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
  pub id: u64,
  pub app: String,
  pub title: String,
  pub body: String,
  /// Last time this toast arrived, in host milliseconds.
  pub at_ms: u64,
  /// How many identical toasts collapsed into this entry (at least 1).
  pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queue {
  items: Vec<Note>,
  ttl_ms: u64,
  next_id: u64,
}

impl Queue {
  /// A queue whose entries live `ttl_secs` seconds (clamped to FNO-03's 1–15).
  pub fn new(ttl_secs: u16) -> Self {
    Queue {
      items: Vec::new(),
      ttl_ms: ttl_ms(ttl_secs),
      next_id: 1,
    }
  }

  /// Applies a changed FNO-03 value live.
  pub fn set_ttl_secs(&mut self, secs: u16) {
    self.ttl_ms = ttl_ms(secs);
  }

  /// Adds a toast. A toast identical to a live one (same app, title and body) refreshes that entry and raises its
  /// counter instead of adding a row. Returns the entry id.
  pub fn push(&mut self, app: &str, title: &str, body: &str, now_ms: u64) -> u64 {
    self.expire(now_ms);
    if let Some(n) = self
      .items
      .iter_mut()
      .find(|n| n.app == app && n.title == title && n.body == body)
    {
      n.at_ms = now_ms;
      n.count = n.count.saturating_add(1);
      return n.id;
    }
    let id = self.next_id;
    self.next_id += 1;
    self.items.push(Note {
      id,
      app: app.to_string(),
      title: title.to_string(),
      body: body.to_string(),
      at_ms: now_ms,
      count: 1,
    });
    if self.items.len() > CAPACITY {
      self.items.remove(0);
    }
    id
  }

  /// Drops entries older than the duration.
  pub fn expire(&mut self, now_ms: u64) {
    let ttl = self.ttl_ms;
    self.items.retain(|n| now_ms.saturating_sub(n.at_ms) < ttl);
  }

  /// Live entries, newest first.
  pub fn live(&self, now_ms: u64) -> Vec<&Note> {
    let mut v: Vec<&Note> = self
      .items
      .iter()
      .filter(|n| now_ms.saturating_sub(n.at_ms) < self.ttl_ms)
      .collect();
    v.reverse();
    v
  }

  /// Removes one entry (the user dismissed it). Returns `true` when it existed.
  pub fn dismiss(&mut self, id: u64) -> bool {
    let before = self.items.len();
    self.items.retain(|n| n.id != id);
    self.items.len() != before
  }

  pub fn clear(&mut self) {
    self.items.clear();
  }

  /// When the oldest live entry expires, so the host can arm one timer.
  pub fn next_expiry_ms(&self) -> Option<u64> {
    self.items.iter().map(|n| n.at_ms + self.ttl_ms).min()
  }
}

fn ttl_ms(secs: u16) -> u64 {
  u64::from(secs.clamp(TTL_MIN_SECS, TTL_MAX_SECS)) * 1000
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn entries_live_for_the_duration_and_then_expire() {
    let mut q = Queue::new(5);
    q.push("Mail", "Hi", "Body", 1_000);
    assert_eq!(q.live(5_999).len(), 1);
    assert!(q.live(6_000).is_empty());
    q.expire(6_000);
    assert_eq!(q.next_expiry_ms(), None);
  }

  #[test]
  fn identical_toasts_collapse_and_refresh_the_timer() {
    let mut q = Queue::new(5);
    let a = q.push("Mail", "Hi", "Body", 1_000);
    let b = q.push("Mail", "Hi", "Body", 4_000);
    assert_eq!(a, b);
    let live = q.live(8_000);
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].count, 2);
    assert!(q.live(9_000).is_empty());
  }

  #[test]
  fn different_toasts_stay_separate_and_the_newest_comes_first() {
    let mut q = Queue::new(10);
    q.push("Mail", "One", "", 0);
    q.push("Mail", "Two", "", 1);
    q.push("Chat", "One", "", 2);
    let titles: Vec<_> = q.live(3).iter().map(|n| (n.app.as_str(), n.title.as_str())).collect();
    assert_eq!(titles, vec![("Chat", "One"), ("Mail", "Two"), ("Mail", "One")]);
  }

  #[test]
  fn an_expired_duplicate_starts_a_fresh_entry() {
    let mut q = Queue::new(2);
    let a = q.push("Mail", "Hi", "", 0);
    let b = q.push("Mail", "Hi", "", 5_000);
    assert_ne!(a, b);
    assert_eq!(q.live(5_000)[0].count, 1);
  }

  #[test]
  fn duration_is_clamped_to_the_setting_range() {
    let mut q = Queue::new(0);
    q.push("A", "t", "", 0);
    assert_eq!(q.live(999).len(), 1);
    assert!(q.live(1_000).is_empty());
    let mut q = Queue::new(99);
    q.push("A", "u", "", 0);
    assert_eq!(q.live(14_999).len(), 1);
    assert!(q.live(15_000).is_empty());
  }

  #[test]
  fn a_shorter_duration_applies_to_entries_already_queued() {
    let mut q = Queue::new(10);
    q.push("A", "t", "", 0);
    q.set_ttl_secs(2);
    assert!(q.live(2_000).is_empty());
  }

  #[test]
  fn capacity_drops_the_oldest_entry() {
    let mut q = Queue::new(15);
    for i in 0..(CAPACITY + 3) {
      q.push("A", &format!("t{i}"), "", 0);
    }
    let live = q.live(1);
    assert_eq!(live.len(), CAPACITY);
    assert_eq!(live.last().unwrap().title, "t3");
  }

  #[test]
  fn dismiss_and_clear_remove_entries() {
    let mut q = Queue::new(10);
    let a = q.push("A", "1", "", 0);
    q.push("A", "2", "", 0);
    assert!(q.dismiss(a));
    assert!(!q.dismiss(a));
    assert_eq!(q.live(1).len(), 1);
    q.clear();
    assert!(q.live(1).is_empty());
  }

  #[test]
  fn next_expiry_is_the_oldest_entry() {
    let mut q = Queue::new(5);
    q.push("A", "1", "", 2_000);
    q.push("A", "2", "", 1_000);
    assert_eq!(q.next_expiry_ms(), Some(6_000));
  }
}
