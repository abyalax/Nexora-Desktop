//! Media playback item in the bar (F-17): width budget, the text shown, and plain versus coloured text.
//!
//! Lyrics come from the Island's parser (`dynamic_island::lyrics::Lyrics::current`) and the album colour from
//! `dynamic_island::tint::from_art`; this module only decides what the bar does with them.

use super::layout::ellipsize;

/// The media item never takes more than this share of the screen width.
pub const MAX_SHARE: f32 = 0.25;
/// Below this width (physical pixels) the item is not worth drawing.
pub const MIN_WIDTH: f32 = 60.0;

/// Width the media item may use: what is left after the other items and the margins, capped at [`MAX_SHARE`] of the
/// screen. `None` when less than [`MIN_WIDTH`] remains, so the item is dropped instead of drawing a stub.
pub fn budget(screen_w: f32, used_w: f32, margin: f32) -> Option<f32> {
  let free = screen_w - used_w - 2.0 * margin.max(0.0);
  let w = free.min(screen_w * MAX_SHARE);
  (w >= MIN_WIDTH).then_some(w)
}

/// The text of the item. With the lyrics menu on (FLB-13) and a current line, the line wins; otherwise the title,
/// followed by the artist when there is one.
pub fn label(title: &str, artist: &str, lyric: Option<&str>, show_lyrics: bool) -> String {
  if show_lyrics {
    if let Some(l) = lyric.map(str::trim).filter(|l| !l.is_empty()) {
      return l.to_string();
    }
  }
  match (title.trim(), artist.trim()) {
    ("", "") => String::new(),
    (t, "") => t.to_string(),
    ("", a) => a.to_string(),
    (t, a) => format!("{t} \u{2013} {a}"),
  }
}

/// The label cut to `width` with an ellipsis.
pub fn fit(text: &str, width: f32, measure: &dyn Fn(&str) -> f32) -> String {
  ellipsize(text, width, measure)
}

/// Text colour: plain uses the bar foreground; coloured (FLB-14) uses the album colour when there is one and falls back
/// to the bar foreground otherwise.
pub fn fg(colored: bool, art: Option<[u8; 3]>, bar_fg: [u8; 3]) -> [u8; 3] {
  if colored { art.unwrap_or(bar_fg) } else { bar_fg }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::dynamic_island::lyrics;

  fn w(s: &str) -> f32 {
    s.chars().count() as f32 * 8.0
  }

  #[test]
  fn budget_is_capped_at_a_quarter_of_the_screen() {
    assert_eq!(budget(2000.0, 300.0, 10.0), Some(500.0));
  }

  #[test]
  fn budget_shrinks_with_the_other_items() {
    assert_eq!(budget(1000.0, 800.0, 10.0), Some(180.0));
  }

  #[test]
  fn budget_is_none_when_nothing_useful_is_left() {
    assert_eq!(budget(1000.0, 940.0, 10.0), None);
    assert_eq!(budget(1000.0, 2000.0, 10.0), None);
  }

  #[test]
  fn lyrics_replace_the_title_only_when_enabled_and_present() {
    assert_eq!(label("Song", "Band", Some("la la"), true), "la la");
    assert_eq!(label("Song", "Band", Some("la la"), false), "Song \u{2013} Band");
    assert_eq!(label("Song", "Band", None, true), "Song \u{2013} Band");
    assert_eq!(label("Song", "Band", Some("   "), true), "Song \u{2013} Band");
  }

  #[test]
  fn title_and_artist_degrade_gracefully() {
    assert_eq!(label("Song", "", None, false), "Song");
    assert_eq!(label("", "Band", None, false), "Band");
    assert_eq!(label(" ", " ", None, false), "");
  }

  #[test]
  fn long_lines_are_cut_to_the_budget() {
    let out = fit("A very long lyric line that cannot fit", 80.0, &w);
    assert!(w(&out) <= 80.0);
    assert!(out.ends_with('\u{2026}'));
    assert_eq!(fit("short", 80.0, &w), "short");
  }

  #[test]
  fn the_island_lyrics_parser_feeds_the_label() {
    let l = lyrics::parse("[00:01.00]first line\n[00:05.00]second line\n");
    assert_eq!(label("Song", "Band", l.current(1_500), true), "first line");
    assert_eq!(label("Song", "Band", l.current(6_000), true), "second line");
  }

  #[test]
  fn colour_follows_the_setting_and_falls_back_to_the_bar() {
    let bar = [255, 255, 255];
    let art = Some([10, 120, 200]);
    assert_eq!(fg(true, art, bar), [10, 120, 200]);
    assert_eq!(fg(false, art, bar), bar);
    assert_eq!(fg(true, None, bar), bar);
  }
}
