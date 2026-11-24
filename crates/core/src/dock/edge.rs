//! Dock placement on any of the four screen edges (D-11.1 to D-11.3).
//!
//! The icon layout in `layout.rs` only knows one axis (the *main axis*, along the strip). This module maps between
//! screen coordinates and that axis, so left/right/top docks reuse the same layout and magnification code.

use super::layout::Placed;
use crate::shared::geometry::{Edge, Rect};

/// Window rectangle of a Dock attached to `edge` of `screen`.
///
/// `main` is the content length along the strip, `cross` the thickness across it (card plus magnification headroom),
/// `margin` the distance from the screen edge. The strip is centred on the main axis and clamped inside the screen.
pub fn window_rect(edge: Edge, screen: Rect, main: f32, cross: f32, margin: f32) -> Rect {
  let main = main.min(screen.along(edge)).max(0.0);
  let cross = cross.max(0.0);
  let margin = margin.max(0.0);
  match edge {
    Edge::Bottom => Rect::new(
      screen.x + (screen.w - main) / 2.0,
      screen.bottom() - margin - cross,
      main,
      cross,
    ),
    Edge::Top => Rect::new(screen.x + (screen.w - main) / 2.0, screen.y + margin, main, cross),
    Edge::Left => Rect::new(screen.x + margin, screen.y + (screen.h - main) / 2.0, cross, main),
    Edge::Right => Rect::new(
      screen.right() - margin - cross,
      screen.y + (screen.h - main) / 2.0,
      cross,
      main,
    ),
  }
}

/// Pointer position along the main axis, relative to the Dock window. Feed this to `layout()` as the cursor.
pub fn main_coord(edge: Edge, win: &Rect, px: f32, py: f32) -> f32 {
  if edge.is_horizontal() { px - win.x } else { py - win.y }
}

/// Pointer distance from the screen edge, across the strip (`0` at the edge).
pub fn depth_from_edge(edge: Edge, screen: &Rect, px: f32, py: f32) -> f32 {
  match edge {
    Edge::Bottom => screen.bottom() - py,
    Edge::Top => py - screen.y,
    Edge::Left => px - screen.x,
    Edge::Right => screen.right() - px,
  }
}

/// Rectangle of one laid-out cell. `cross_start` and `cross_len` locate the icon band across the strip, in window
/// coordinates. Magnified icons grow away from the edge, so the band is anchored on the edge side.
pub fn cell_rect(edge: Edge, cell: &Placed, icon: f32, cross_start: f32, cross_len: f32) -> Rect {
  let size = icon * cell.scale;
  let cross_pos = match edge {
    // Bottom and right docks sit against the far edge: growth goes up/left, so shift the start back.
    Edge::Bottom | Edge::Right => cross_start + cross_len - size,
    Edge::Top | Edge::Left => cross_start,
  };
  if edge.is_horizontal() {
    Rect::new(cell.x, cross_pos, cell.w.max(size), size)
  } else {
    Rect::new(cross_pos, cell.x, size, cell.w.max(size))
  }
}

/// Which side a popup (tooltip, preview, menu) opens towards: away from the edge the Dock is attached to.
pub fn popup_side(edge: Edge) -> Edge {
  match edge {
    Edge::Bottom => Edge::Top,
    Edge::Top => Edge::Bottom,
    Edge::Left => Edge::Right,
    Edge::Right => Edge::Left,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::dock::layout::{Metrics, Slot, layout};

  const SCREEN: Rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);

  #[test]
  fn bottom_dock_is_centred_and_flush_with_the_margin() {
    let r = window_rect(Edge::Bottom, SCREEN, 600.0, 100.0, 8.0);
    assert_eq!(r, Rect::new(660.0, 972.0, 600.0, 100.0));
  }

  #[test]
  fn top_dock_sits_below_the_top_margin() {
    let r = window_rect(Edge::Top, SCREEN, 600.0, 100.0, 8.0);
    assert_eq!(r, Rect::new(660.0, 8.0, 600.0, 100.0));
  }

  #[test]
  fn left_and_right_docks_swap_the_axes() {
    let l = window_rect(Edge::Left, SCREEN, 600.0, 100.0, 8.0);
    let r = window_rect(Edge::Right, SCREEN, 600.0, 100.0, 8.0);
    assert_eq!(l, Rect::new(8.0, 240.0, 100.0, 600.0));
    assert_eq!(r, Rect::new(1812.0, 240.0, 100.0, 600.0));
  }

  #[test]
  fn content_longer_than_the_screen_is_clamped() {
    let r = window_rect(Edge::Left, SCREEN, 5000.0, 100.0, 0.0);
    assert_eq!((r.y, r.h), (0.0, 1080.0));
  }

  #[test]
  fn main_coordinate_follows_the_edge_orientation() {
    let win = Rect::new(100.0, 200.0, 50.0, 400.0);
    assert_eq!(main_coord(Edge::Left, &win, 120.0, 260.0), 60.0);
    assert_eq!(main_coord(Edge::Bottom, &win, 120.0, 260.0), 20.0);
  }

  #[test]
  fn depth_is_zero_at_each_edge() {
    assert_eq!(depth_from_edge(Edge::Bottom, &SCREEN, 10.0, 1080.0), 0.0);
    assert_eq!(depth_from_edge(Edge::Top, &SCREEN, 10.0, 0.0), 0.0);
    assert_eq!(depth_from_edge(Edge::Left, &SCREEN, 0.0, 10.0), 0.0);
    assert_eq!(depth_from_edge(Edge::Right, &SCREEN, 1920.0, 10.0), 0.0);
  }

  #[test]
  fn same_order_on_every_edge_and_magnification_peaks_under_the_pointer() {
    let slots = [Slot::Item; 5];
    let m = Metrics::custom(48.0, 8.0, 1.8, 1.0);
    let l = layout(&slots, Some(400.0), 800.0, &m);
    // The layout is axis-agnostic, so the cell order is identical for every edge.
    for edge in Edge::ALL {
      let rects: Vec<Rect> = l.cells.iter().map(|c| cell_rect(edge, c, m.icon, 0.0, 100.0)).collect();
      let main = |r: &Rect| if edge.is_horizontal() { r.x } else { r.y };
      assert!(rects.windows(2).all(|w| main(&w[0]) < main(&w[1])));
    }
    let peak = l.cells.iter().map(|c| c.scale).fold(0.0, f32::max);
    assert!(peak > 1.5);
  }

  #[test]
  fn magnified_icons_grow_away_from_the_edge() {
    let cell = Placed {
      x: 10.0,
      w: 70.0,
      scale: 1.5,
    };
    let b = cell_rect(Edge::Bottom, &cell, 48.0, 0.0, 100.0);
    let t = cell_rect(Edge::Top, &cell, 48.0, 0.0, 100.0);
    assert_eq!(b.bottom(), 100.0);
    assert_eq!(t.y, 0.0);
    assert_eq!(b.h, 72.0);
  }

  #[test]
  fn popups_open_away_from_the_edge() {
    assert_eq!(popup_side(Edge::Bottom), Edge::Top);
    assert_eq!(popup_side(Edge::Left), Edge::Right);
    for e in Edge::ALL {
      assert_eq!(popup_side(popup_side(e)), e);
    }
  }
}
