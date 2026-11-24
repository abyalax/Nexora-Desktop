//! Folder pin grid layout (D-17.1).

/// Icon size range of the folder grid (APP-21).
pub const ICON_MIN: u16 = 24;
pub const ICON_MAX: u16 = 96;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
  pub cols: usize,
  pub rows: usize,
  pub cell: f32,
}

/// Grid for `count` entries in a popup `width` pixels wide. Each cell is the icon plus `pad` on every side.
/// There is always at least one column, so a narrow popup never divides by zero.
pub fn grid(count: usize, width: f32, icon: u16, pad: f32) -> Grid {
  let cell = icon.clamp(ICON_MIN, ICON_MAX) as f32 + pad.max(0.0) * 2.0;
  let cols = ((width.max(0.0) / cell).floor() as usize).max(1);
  let rows = if count == 0 { 0 } else { count.div_ceil(cols) };
  Grid { cols, rows, cell }
}

/// Row and column of entry `i`.
pub fn cell_of(g: &Grid, i: usize) -> (usize, usize) {
  (i / g.cols, i % g.cols)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn columns_fit_the_width() {
    let g = grid(10, 400.0, 48, 8.0);
    assert_eq!((g.cell, g.cols, g.rows), (64.0, 6, 2));
  }

  #[test]
  fn narrow_popup_has_one_column() {
    assert_eq!(grid(5, 10.0, 48, 8.0).cols, 1);
    assert_eq!(grid(5, -3.0, 48, 8.0).rows, 5);
  }

  #[test]
  fn empty_folder_has_no_rows() {
    assert_eq!(grid(0, 400.0, 48, 8.0).rows, 0);
  }

  #[test]
  fn icon_size_is_clamped_to_the_range() {
    assert_eq!(grid(1, 400.0, 500, 0.0).cell, 96.0);
    assert_eq!(grid(1, 400.0, 1, 0.0).cell, 24.0);
  }

  #[test]
  fn cell_positions_fill_rows_left_to_right() {
    let g = grid(10, 400.0, 48, 8.0);
    assert_eq!(cell_of(&g, 0), (0, 0));
    assert_eq!(cell_of(&g, 5), (0, 5));
    assert_eq!(cell_of(&g, 6), (1, 0));
  }
}
