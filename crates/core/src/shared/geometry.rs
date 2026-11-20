//! Screen geometry shared by Dock, Finder and Dynamic Island placement logic. Pure math, no OS types.

/// Axis-aligned rectangle in physical pixels (`x`/`y` is the top-left corner).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
}

impl Rect {
  pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
  }

  pub fn right(&self) -> f32 {
    self.x + self.w
  }

  pub fn bottom(&self) -> f32 {
    self.y + self.h
  }

  /// `true` when the point is inside; the left/top edges are inclusive and the right/bottom edges exclusive.
  pub fn contains(&self, px: f32, py: f32) -> bool {
    px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
  }

  /// `true` when the two rectangles share any area (touching edges do not count).
  pub fn intersects(&self, other: &Rect) -> bool {
    self.x < other.right() && other.x < self.right() && self.y < other.bottom() && other.y < self.bottom()
  }

  /// Extent along the given edge: the width for top/bottom edges, the height for left/right edges.
  pub fn along(&self, edge: Edge) -> f32 {
    if edge.is_horizontal() { self.w } else { self.h }
  }
}

/// A screen edge that a surface (Dock, bar, activation strip) is attached to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
  Bottom,
  Left,
  Right,
  Top,
}

impl Edge {
  pub const ALL: [Edge; 4] = [Edge::Bottom, Edge::Left, Edge::Right, Edge::Top];

  pub fn parse(s: &str) -> Option<Edge> {
    match s.trim().to_ascii_lowercase().as_str() {
      "bottom" | "bawah" => Some(Edge::Bottom),
      "left" | "kiri" => Some(Edge::Left),
      "right" | "kanan" => Some(Edge::Right),
      "top" | "atas" => Some(Edge::Top),
      _ => None,
    }
  }

  pub fn key(self) -> &'static str {
    match self {
      Edge::Bottom => "bottom",
      Edge::Left => "left",
      Edge::Right => "right",
      Edge::Top => "top",
    }
  }

  /// `true` for the top and bottom edges, where a strip runs left to right.
  pub fn is_horizontal(self) -> bool {
    matches!(self, Edge::Top | Edge::Bottom)
  }
}

/// Distances to carve out of a monitor to reserve space for a surface.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Inset {
  pub left: f32,
  pub top: f32,
  pub right: f32,
  pub bottom: f32,
}

impl Inset {
  pub const NONE: Inset = Inset {
    left: 0.0,
    top: 0.0,
    right: 0.0,
    bottom: 0.0,
  };

  /// An inset of `amount` on one edge and nothing elsewhere.
  pub fn on(edge: Edge, amount: f32) -> Inset {
    let mut i = Inset::NONE;
    match edge {
      Edge::Left => i.left = amount,
      Edge::Top => i.top = amount,
      Edge::Right => i.right = amount,
      Edge::Bottom => i.bottom = amount,
    }
    i
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contains_is_half_open() {
    let r = Rect::new(10.0, 10.0, 20.0, 20.0);
    assert!(r.contains(10.0, 10.0) && r.contains(29.9, 29.9));
    assert!(!r.contains(30.0, 15.0) && !r.contains(15.0, 30.0) && !r.contains(9.9, 15.0));
  }

  #[test]
  fn touching_rectangles_do_not_intersect() {
    let a = Rect::new(0.0, 0.0, 10.0, 10.0);
    assert!(!a.intersects(&Rect::new(10.0, 0.0, 5.0, 5.0)));
    assert!(a.intersects(&Rect::new(9.0, 9.0, 5.0, 5.0)));
    assert!(!a.intersects(&Rect::new(0.0, 20.0, 5.0, 5.0)));
  }

  #[test]
  fn edges_round_trip_and_accept_indonesian_names() {
    for e in Edge::ALL {
      assert_eq!(Edge::parse(e.key()), Some(e));
    }
    assert_eq!(Edge::parse(" KIRI "), Some(Edge::Left));
    assert_eq!(Edge::parse("middle"), None);
    assert!(Edge::Top.is_horizontal() && !Edge::Right.is_horizontal());
  }

  #[test]
  fn along_follows_the_edge_direction() {
    let r = Rect::new(0.0, 0.0, 300.0, 100.0);
    assert_eq!((r.along(Edge::Bottom), r.along(Edge::Left)), (300.0, 100.0));
  }

  #[test]
  fn inset_touches_a_single_edge() {
    let i = Inset::on(Edge::Right, 40.0);
    assert_eq!((i.left, i.top, i.right, i.bottom), (0.0, 0.0, 40.0, 0.0));
  }
}
