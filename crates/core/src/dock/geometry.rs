//! Dock window geometry derived from DPI and layout metrics.

use super::layout::{layout, Metrics, Slot};

/// Vertical frame of the Dock window, in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub margin: f32,
    pub card_h: f32,
    pub card_y: f32,
    pub icon_y: f32,
    pub dot_y: f32,
    /// Total window height (already rounded up to a whole pixel).
    pub win_h: f32,
}

/// Computes the frame. `margin_px` is the unscaled distance from the screen edge.
pub fn frame(dpi: f32, m: &Metrics, margin_px: f32) -> Frame {
    let d = dpi;
    let icon = m.icon;
    let (pad_top, pad_bottom) = (12.0 * d, 18.0 * d);
    let margin = margin_px * d;
    let card_h = pad_top + icon + pad_bottom;
    let headroom = icon * (m.max_scale - 1.0) + 6.0 * d;
    let win_h = (margin + card_h + headroom).ceil();
    let card_y = win_h - margin - card_h;
    let icon_y = card_y + pad_top;
    let dot_y = icon_y + icon + (pad_bottom - 4.0 * d) / 2.0 + 1.0 * d;
    Frame { margin, card_h, card_y, icon_y, dot_y, win_h }
}

/// Widest the content can become while the pointer sweeps across the strip (magnification).
pub fn max_content_width(slots: &[Slot], m: &Metrics) -> f32 {
    let area = 4000.0;
    let base = layout(slots, None, area, m);
    let (Some(first), Some(last)) = (base.cells.first(), base.cells.last()) else {
        return 0.0;
    };
    let (from, to) = (first.x, last.x + last.w);
    let mut best = base.total;
    let mut x = from;
    while x <= to {
        best = best.max(layout(slots, Some(x), area, m).total);
        x += m.icon / 4.0;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_is_consistent() {
        let m = Metrics::new(48.0, 1.0);
        let f = frame(1.0, &m, 8.0);
        assert_eq!(f.card_y + f.card_h + f.margin, f.win_h);
        assert!(f.icon_y > f.card_y && f.dot_y > f.icon_y);
    }

    #[test]
    fn frame_scales_with_dpi() {
        let m = Metrics::new(48.0, 1.0);
        assert!(frame(2.0, &m, 8.0).margin > frame(1.0, &m, 8.0).margin);
    }

    #[test]
    fn empty_strip_has_no_width() {
        assert_eq!(max_content_width(&[], &Metrics::new(48.0, 1.0)), 0.0);
    }

    #[test]
    fn magnification_never_shrinks_width() {
        let m = Metrics::new(48.0, 1.0);
        let slots = [Slot::Item, Slot::Separator, Slot::Item];
        let base = layout(&slots, None, 4000.0, &m).total;
        assert!(max_content_width(&slots, &m) >= base);
    }
}
