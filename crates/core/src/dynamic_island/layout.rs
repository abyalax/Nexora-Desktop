//! Island geometry on the screen: where the resting pill sits, its activation strip and the reserved band.
//!
//! All values are physical pixels for a given `dpi` scale; the host in `app` only places windows with them.

use super::state::Size;

/// Gap kept under the pill so its shadow is not clipped, in logical pixels.
const SHADOW_GAP: f32 = 4.0;
/// Activation strip thickness at the top edge, in physical pixels.
const STRIP_HEIGHT: i32 = 2;
/// Activation strip overhang on each side of the pill, in logical pixels.
const STRIP_PAD: f32 = 60.0;
/// Height reserved under the Island when it reserves work area, in logical pixels.
const RESERVE_HEIGHT: f32 = 34.0;

/// Screen rectangle `(left, top, right, bottom)` of the resting pill, centered on the top edge.
pub fn pill_screen_rect(screen_w: i32, rest: Size, dpi: f32) -> (i32, i32, i32, i32) {
  let left = ((screen_w as f32 - rest.w) / 2.0) as i32;
  (left, 0, left + rest.w as i32, (rest.h + SHADOW_GAP * dpi) as i32)
}

/// Activation strip `(x, y, width, height)` over the pill: wider than the pill so approaching it arms the edge.
pub fn activation_strip(pill: (i32, i32, i32, i32), dpi: f32) -> (i32, i32, i32, i32) {
  let (l, _, r, _) = pill;
  let pad = (STRIP_PAD * dpi) as i32;
  (l - pad, 0, r - l + 2 * pad, STRIP_HEIGHT)
}

/// What other features see of the Island: `(left, right, height)` of the pill without the shadow gap.
pub fn published_extent(pill: (i32, i32, i32, i32), dpi: f32) -> (i32, i32, i32) {
  let (l, _, r, h) = pill;
  (l, r, h - (SHADOW_GAP * dpi) as i32)
}

/// Height of the band reserved at the top of the work area.
pub fn reserve_height(dpi: f32) -> i32 {
  (RESERVE_HEIGHT * dpi).ceil() as i32
}

#[cfg(test)]
mod tests {
  use super::*;

  const REST: Size = Size {
    w: 184.0,
    h: 30.0,
    radius: 15.0,
  };

  #[test]
  fn pill_is_centered_and_includes_the_shadow_gap() {
    let (l, t, r, b) = pill_screen_rect(1920, REST, 1.0);
    assert_eq!((l, t, r, b), (868, 0, 1052, 34));
    assert_eq!((l + r) / 2, 960);
  }

  #[test]
  fn geometry_scales_with_dpi() {
    let rest = Size {
      w: 276.0,
      h: 45.0,
      radius: 22.5,
    };
    let (l, _, r, b) = pill_screen_rect(2880, rest, 1.5);
    assert_eq!(r - l, 276);
    assert_eq!(b, 51);
    assert_eq!(reserve_height(1.5), 51);
  }

  #[test]
  fn strip_overhangs_the_pill_by_the_pad_on_both_sides() {
    let pill = pill_screen_rect(1920, REST, 1.0);
    let (x, y, w, h) = activation_strip(pill, 1.0);
    assert_eq!((x, y, w, h), (808, 0, 304, 2));
    assert_eq!(x + w - pill.2, pill.0 - x);
  }

  #[test]
  fn published_extent_drops_the_shadow_gap() {
    let pill = pill_screen_rect(1920, REST, 1.0);
    assert_eq!(published_extent(pill, 1.0), (868, 1052, 30));
  }

  #[test]
  fn reserve_height_rounds_up() {
    assert_eq!(reserve_height(1.0), 34);
    assert_eq!(reserve_height(1.25), 43);
  }
}
