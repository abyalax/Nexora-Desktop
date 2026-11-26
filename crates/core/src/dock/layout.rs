#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
  Item,
  Separator,
}

#[derive(Clone, Copy, Debug)]
pub struct Metrics {
  pub icon: f32,
  pub gap: f32,
  pub max_scale: f32,
  pub reach: f32,
  pub separator: f32,
}

impl Metrics {
  pub fn custom(icon: f32, gap: f32, zoom: f32, scale: f32) -> Self {
    Metrics {
      icon: icon * scale,
      gap: gap * scale,
      max_scale: zoom.max(1.0),
      reach: 2.6 * icon * scale,
      separator: 10.0 * scale,
    }
  }

  pub fn new(icon: f32, scale: f32) -> Self {
    Metrics {
      icon: icon * scale,
      gap: 6.0 * scale,
      max_scale: 1.7,
      reach: 2.6 * icon * scale,
      separator: 10.0 * scale,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
  pub x: f32,
  pub w: f32,
  pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct Layout {
  pub cells: Vec<Placed>,
  pub total: f32,
}

fn base_width(slots: &[Slot], m: &Metrics) -> f32 {
  let mut w = 0.0;
  for (i, s) in slots.iter().enumerate() {
    if i > 0 {
      w += m.gap;
    }
    w += match s {
      Slot::Item => m.icon,
      Slot::Separator => m.separator,
    };
  }
  w
}

pub(crate) fn falloff(distance: f32, reach: f32) -> f32 {
  let t = (distance / reach).clamp(0.0, 1.0);
  0.5 * (1.0 + (std::f32::consts::PI * t).cos())
}

pub fn layout(slots: &[Slot], cursor_x: Option<f32>, area_w: f32, m: &Metrics) -> Layout {
  let base_total = base_width(slots, m);
  let base_left = (area_w - base_total) / 2.0;

  let mut scales = Vec::with_capacity(slots.len());
  let mut x = base_left;
  for s in slots {
    let w = match s {
      Slot::Item => m.icon,
      Slot::Separator => m.separator,
    };
    let scale = match (s, cursor_x) {
      (Slot::Item, Some(c)) => 1.0 + (m.max_scale - 1.0) * falloff((c - (x + w / 2.0)).abs(), m.reach),
      _ => 1.0,
    };
    scales.push(scale);
    x += w + m.gap;
  }

  let widths: Vec<f32> = slots
    .iter()
    .zip(&scales)
    .map(|(s, sc)| match s {
      Slot::Item => m.icon * sc,
      Slot::Separator => m.separator,
    })
    .collect();
  let total = widths.iter().sum::<f32>() + m.gap * (slots.len().saturating_sub(1)) as f32;
  let mut x = (area_w - total) / 2.0;
  let mut cells = Vec::with_capacity(slots.len());
  for (w, sc) in widths.iter().zip(&scales) {
    cells.push(Placed { x, w: *w, scale: *sc });
    x += w + m.gap;
  }
  Layout { cells, total }
}

/// Layout a row on a wider virtual canvas and expose the selected viewport.
/// `area_w` is the virtual canvas width; `scroll_x` is clamped to the available
/// horizontal range and hit testing remains in viewport coordinates.
pub fn layout_scrolled(
  slots: &[Slot],
  cursor_x: Option<f32>,
  area_w: f32,
  viewport_w: f32,
  scroll_x: f32,
  m: &Metrics,
) -> Layout {
  let scroll_max = (area_w - viewport_w).max(0.0);
  let scroll = if scroll_x.is_finite() {
    scroll_x.clamp(0.0, scroll_max)
  } else {
    0.0
  };
  let mut result = layout(slots, cursor_x.map(|x| x + scroll), area_w, m);
  for cell in &mut result.cells {
    cell.x -= scroll;
  }
  result
}

pub fn hit(slots: &[Slot], l: &Layout, x: f32, m: &Metrics) -> Option<usize> {
  let half = m.gap / 2.0;
  for (i, c) in l.cells.iter().enumerate() {
    if slots[i] == Slot::Item && x >= c.x - half && x <= c.x + c.w + half {
      return Some(i);
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  fn m() -> Metrics {
    Metrics::new(48.0, 1.0)
  }
  fn items(n: usize) -> Vec<Slot> {
    vec![Slot::Item; n]
  }

  #[test]
  fn without_cursor_everything_is_unscaled_and_centered() {
    let s = items(5);
    let l = layout(&s, None, 800.0, &m());
    assert!(l.cells.iter().all(|c| c.scale == 1.0));
    let expected = 5.0 * 48.0 + 4.0 * 6.0;
    assert!((l.total - expected).abs() < 1e-3);
    assert!((l.cells[0].x - (800.0 - expected) / 2.0).abs() < 1e-3);
  }

  #[test]
  fn cursor_on_middle_item_peaks_there_and_is_symmetric() {
    let s = items(7);
    let base = layout(&s, None, 900.0, &m());
    let mid = base.cells[3].x + base.cells[3].w / 2.0;
    let l = layout(&s, Some(mid), 900.0, &m());
    assert!((l.cells[3].scale - m().max_scale).abs() < 1e-3);
    assert!((l.cells[2].scale - l.cells[4].scale).abs() < 1e-3);
    assert!(l.cells[3].scale > l.cells[2].scale && l.cells[2].scale > l.cells[1].scale);
    assert!(l.total > base.total, "dock melebar saat magnifikasi");
  }

  #[test]
  fn items_beyond_reach_stay_at_scale_one() {
    let s = items(12);
    let base = layout(&s, None, 1200.0, &m());
    let c = base.cells[0].x + 24.0;
    let l = layout(&s, Some(c), 1200.0, &m());
    assert!((l.cells[11].scale - 1.0).abs() < 1e-6);
  }

  #[test]
  fn cells_never_overlap_and_stay_ordered() {
    let s = items(9);
    let base = layout(&s, None, 1000.0, &m());
    for step in 0..40 {
      let c = base.cells[0].x + step as f32 * 12.0;
      let l = layout(&s, Some(c), 1000.0, &m());
      for w in l.cells.windows(2) {
        assert!(w[1].x >= w[0].x + w[0].w - 1e-3, "tumpang tindih di c={c}");
      }
    }
  }

  #[test]
  fn separator_keeps_fixed_width_and_scale() {
    let s = [Slot::Item, Slot::Separator, Slot::Item];
    let base = layout(&s, None, 400.0, &m());
    let c = base.cells[1].x;
    let l = layout(&s, Some(c), 400.0, &m());
    assert_eq!(l.cells[1].scale, 1.0);
    assert_eq!(l.cells[1].w, m().separator);
  }

  #[test]
  fn hit_test_finds_items_and_ignores_separators() {
    let s = [Slot::Item, Slot::Separator, Slot::Item];
    let l = layout(&s, None, 400.0, &m());
    assert_eq!(hit(&s, &l, l.cells[0].x + 10.0, &m()), Some(0));
    assert_eq!(hit(&s, &l, l.cells[1].x + 2.0, &m()), None);
    assert_eq!(hit(&s, &l, l.cells[2].x + 10.0, &m()), Some(2));
    assert_eq!(hit(&s, &l, -50.0, &m()), None);
  }

  #[test]
  fn empty_dock_is_safe() {
    let l = layout(&[], Some(10.0), 300.0, &m());
    assert!(l.cells.is_empty());
    assert_eq!(l.total, 0.0);
  }

  #[test]
  fn scrolled_layout_is_viewport_relative_and_clamps_the_offset() {
    let s = items(8);
    let m = m();
    let area = 700.0;
    let viewport = 300.0;
    let start = layout_scrolled(&s, None, area, viewport, 0.0, &m);
    let end = layout_scrolled(&s, None, area, viewport, 900.0, &m);
    assert_eq!(end.cells[0].x, start.cells[0].x - (area - viewport));
    assert_eq!(
      layout_scrolled(&s, None, area, viewport, -10.0, &m).cells[0].x,
      start.cells[0].x
    );
  }

  #[test]
  fn hit_testing_tracks_items_after_scroll() {
    let s = items(8);
    let m = m();
    let l = layout_scrolled(&s, None, 700.0, 300.0, 200.0, &m);
    assert_eq!(hit(&s, &l, l.cells[3].x + l.cells[3].w / 2.0, &m), Some(3));
    assert_eq!(hit(&s, &l, 0.0, &m), Some(1));
  }

  #[test]
  fn magnification_tracks_the_pointer_in_scrolled_coordinates() {
    let s = items(8);
    let m = m();
    let baseline = layout_scrolled(&s, None, 700.0, 300.0, 200.0, &m);
    let pointer = baseline.cells[4].x + baseline.cells[4].w / 2.0;
    let moved = layout_scrolled(&s, Some(pointer), 700.0, 300.0, 200.0, &m);
    assert!((moved.cells[4].scale - m.max_scale).abs() < 1e-3);
  }
}
