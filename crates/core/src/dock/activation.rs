//! Dock activation bar and reveal delay (D-13.1, D-13.2).
//!
//! `hit_rect` says where the invisible trigger strip is; [`step`] is the pure dwell state machine: the pointer enters the
//! strip, the Dock is armed, and after the delay it is revealed. Leaving cancels. The host supplies the clock.

use crate::shared::geometry::{Edge, Rect};

/// Rectangle of the activation bar on `screen`. `width` runs along the edge, `height` across it, `distance` is the
/// offset from the screen edge. The bar is centred along the edge and clamped inside the screen.
pub fn hit_rect(edge: Edge, screen: Rect, width: f32, height: f32, distance: f32) -> Rect {
  let (len, depth) = (width.clamp(1.0, screen.along(edge)), height.max(1.0));
  let distance = distance
    .max(0.0)
    .min((if edge.is_horizontal() { screen.h } else { screen.w }) - depth)
    .max(0.0);
  match edge {
    Edge::Bottom => Rect::new(
      screen.x + (screen.w - len) / 2.0,
      screen.bottom() - distance - depth,
      len,
      depth,
    ),
    Edge::Top => Rect::new(screen.x + (screen.w - len) / 2.0, screen.y + distance, len, depth),
    Edge::Left => Rect::new(screen.x + distance, screen.y + (screen.h - len) / 2.0, depth, len),
    Edge::Right => Rect::new(
      screen.right() - distance - depth,
      screen.y + (screen.h - len) / 2.0,
      depth,
      len,
    ),
  }
}

/// What may trigger the reveal (DAD-15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
  Edge,
  Bar,
  EdgeAndBar,
}

impl Trigger {
  /// From the stored choice index (`edge`, `bar`, `edge_and_bar`). Unknown values fall back to the edge.
  pub fn from_index(i: u8) -> Trigger {
    match i {
      1 => Trigger::Bar,
      2 => Trigger::EdgeAndBar,
      _ => Trigger::Edge,
    }
  }

  pub fn uses_edge(self) -> bool {
    matches!(self, Trigger::Edge | Trigger::EdgeAndBar)
  }

  pub fn uses_bar(self) -> bool {
    matches!(self, Trigger::Bar | Trigger::EdgeAndBar)
  }
}

/// True when the pointer is in a zone that the trigger setting allows.
pub fn pointer_in_zone(trigger: Trigger, on_edge: bool, in_bar: bool) -> bool {
  (trigger.uses_edge() && on_edge) || (trigger.uses_bar() && in_bar)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
  Idle,
  /// Pointer is in the zone since `since_ms`.
  Armed {
    since_ms: u64,
  },
  Revealed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
  /// The pointer is in (`true`) or out of (`false`) an allowed zone. Sent on every pointer move or timer tick.
  Zone(bool),
  /// The Dock was hidden again by the host; start over.
  Hidden,
}

/// One transition. `now_ms` is a monotonic clock supplied by the host, `delay_ms` the user's setting.
pub fn step(state: State, ev: Event, now_ms: u64, delay_ms: u32) -> State {
  match (state, ev) {
    (_, Event::Hidden) => State::Idle,
    (State::Idle, Event::Zone(true)) => {
      if delay_ms == 0 {
        State::Revealed
      } else {
        State::Armed { since_ms: now_ms }
      }
    }
    (State::Armed { since_ms }, Event::Zone(true)) => {
      if now_ms.saturating_sub(since_ms) >= delay_ms as u64 {
        State::Revealed
      } else {
        state
      }
    }
    (State::Armed { .. }, Event::Zone(false)) => State::Idle,
    // Once revealed, leaving the zone does not hide the Dock: visibility policy owns that decision.
    (s, _) => s,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SCREEN: Rect = Rect::new(0.0, 0.0, 1000.0, 600.0);

  #[test]
  fn bar_sits_on_each_edge() {
    assert_eq!(
      hit_rect(Edge::Bottom, SCREEN, 100.0, 4.0, 0.0),
      Rect::new(450.0, 596.0, 100.0, 4.0)
    );
    assert_eq!(
      hit_rect(Edge::Top, SCREEN, 100.0, 4.0, 0.0),
      Rect::new(450.0, 0.0, 100.0, 4.0)
    );
    assert_eq!(
      hit_rect(Edge::Left, SCREEN, 100.0, 4.0, 0.0),
      Rect::new(0.0, 250.0, 4.0, 100.0)
    );
    assert_eq!(
      hit_rect(Edge::Right, SCREEN, 100.0, 4.0, 0.0),
      Rect::new(996.0, 250.0, 4.0, 100.0)
    );
  }

  #[test]
  fn distance_moves_the_bar_inwards() {
    assert_eq!(hit_rect(Edge::Bottom, SCREEN, 100.0, 4.0, 40.0).y, 556.0);
  }

  #[test]
  fn oversized_distance_and_width_stay_on_screen() {
    let r = hit_rect(Edge::Bottom, SCREEN, 5000.0, 30.0, 9999.0);
    assert!(r.x >= 0.0 && r.right() <= 1000.0 && r.y >= 0.0 && r.bottom() <= 600.0);
  }

  #[test]
  fn trigger_gates_the_zones() {
    assert!(pointer_in_zone(Trigger::Edge, true, false));
    assert!(!pointer_in_zone(Trigger::Edge, false, true));
    assert!(pointer_in_zone(Trigger::Bar, false, true));
    assert!(!pointer_in_zone(Trigger::Bar, true, false));
    assert!(pointer_in_zone(Trigger::EdgeAndBar, false, true));
    assert!(pointer_in_zone(Trigger::EdgeAndBar, true, false));
    assert_eq!(Trigger::from_index(9), Trigger::Edge);
  }

  #[test]
  fn dwell_reveals_after_the_delay() {
    let s = step(State::Idle, Event::Zone(true), 1000, 150);
    assert_eq!(s, State::Armed { since_ms: 1000 });
    assert_eq!(step(s, Event::Zone(true), 1100, 150), s);
    assert_eq!(step(s, Event::Zone(true), 1150, 150), State::Revealed);
  }

  #[test]
  fn leaving_cancels_the_dwell() {
    let s = step(State::Idle, Event::Zone(true), 0, 150);
    assert_eq!(step(s, Event::Zone(false), 50, 150), State::Idle);
  }

  #[test]
  fn zero_delay_reveals_immediately() {
    assert_eq!(step(State::Idle, Event::Zone(true), 5, 0), State::Revealed);
  }

  #[test]
  fn hidden_resets_from_every_state() {
    for s in [State::Idle, State::Armed { since_ms: 1 }, State::Revealed] {
      assert_eq!(step(s, Event::Hidden, 9, 150), State::Idle);
    }
  }

  #[test]
  fn revealed_ignores_zone_changes() {
    assert_eq!(step(State::Revealed, Event::Zone(false), 9, 150), State::Revealed);
  }

  #[test]
  fn clock_going_backwards_does_not_panic() {
    let s = State::Armed { since_ms: 500 };
    assert_eq!(step(s, Event::Zone(true), 100, 150), s);
  }
}
