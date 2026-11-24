//! Finder bar clock text, e.g. `Thu Nov 19  9:05 AM` (English) or `Kam 19 Nov  09.05` (Indonesian).

use crate::panels::calendar;
use crate::shared::i18n::{self, Lang};

/// Calendar and clock fields needed to format the bar clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct When {
  pub year: i32,
  pub month: u32,
  pub day: u32,
  pub hour: u32,
  pub minute: u32,
}

const MONTHS_EN: [&str; 12] = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_ID: [&str; 12] = [
  "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
];

/// 12-hour or 24-hour time (FTD-03). The domain type behind the legacy `clock_24h` key: the key stays on disk so 0.4.0
/// files keep round-tripping byte for byte (ADR-0004), and [`TimeFormat::from_legacy`] reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeFormat {
  H12,
  H24,
}

impl TimeFormat {
  /// Reads the legacy `clock_24h` flag.
  pub fn from_legacy(clock_24h: bool) -> Self {
    if clock_24h { TimeFormat::H24 } else { TimeFormat::H12 }
  }

  /// The legacy flag for this format.
  pub fn is_h24(self) -> bool {
    self == TimeFormat::H24
  }

  /// `12`, `12h`, `12-hour`, `24`, `24h` or `24-hour` (case-insensitive). Anything else is garbage.
  pub fn parse(s: &str) -> Option<Self> {
    match s.trim().to_ascii_lowercase().as_str() {
      "12" | "12h" | "12-hour" => Some(TimeFormat::H12),
      "24" | "24h" | "24-hour" => Some(TimeFormat::H24),
      _ => None,
    }
  }
}

/// Date style of the bar clock (FTD-04). The index order matches the `finder_date_format` registry row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateFormat {
  /// `Thu Nov 19`
  Short,
  /// `Thursday, November 19`
  Long,
  /// `2026-11-19`
  Iso,
}

impl DateFormat {
  /// From the registry choice index; out-of-range values fall back to [`DateFormat::Short`].
  pub fn from_index(i: u8) -> Self {
    match i {
      1 => DateFormat::Long,
      2 => DateFormat::Iso,
      _ => DateFormat::Short,
    }
  }
}

/// What the bar clock shows (FTD-01 to FTD-04).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockStyle {
  pub show_time: bool,
  pub show_date: bool,
  pub time: TimeFormat,
  pub date: DateFormat,
}

impl Default for ClockStyle {
  fn default() -> Self {
    ClockStyle {
      show_time: true,
      show_date: true,
      time: TimeFormat::H24,
      date: DateFormat::Short,
    }
  }
}

const WEEKDAYS_EN_LONG: [&str; 7] = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];
const WEEKDAYS_ID_LONG: [&str; 7] = ["Minggu", "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu"];
const MONTHS_ID_LONG: [&str; 12] = [
  "Januari",
  "Februari",
  "Maret",
  "April",
  "Mei",
  "Juni",
  "Juli",
  "Agustus",
  "September",
  "Oktober",
  "November",
  "Desember",
];

fn time_text(n: When, f: TimeFormat) -> String {
  match f {
    TimeFormat::H24 => format!("{:02}.{:02}", n.hour, n.minute),
    TimeFormat::H12 => {
      let h12 = if n.hour % 12 == 0 { 12 } else { n.hour % 12 };
      let ap = if n.hour < 12 { "AM" } else { "PM" };
      format!("{h12}:{:02} {ap}", n.minute)
    }
  }
}

fn date_text(n: When, lang: Lang, f: DateFormat) -> String {
  let wd = calendar::weekday(n.year, n.month, n.day) as usize % 7;
  let month = (n.month.max(1) - 1) as usize % 12;
  match f {
    DateFormat::Short => {
      let days = i18n::weekday_labels(lang, false, calendar::weekday_labels(false));
      match lang {
        Lang::En => format!("{} {} {}", days[wd], MONTHS_EN[month], n.day),
        Lang::Id => format!("{} {} {}", days[wd], n.day, MONTHS_ID[month]),
      }
    }
    DateFormat::Long => {
      let name = i18n::month_name(lang, n.month, MONTHS_ID_LONG[month]);
      match lang {
        Lang::En => format!("{}, {} {}", WEEKDAYS_EN_LONG[wd], name, n.day),
        Lang::Id => format!("{}, {} {}", WEEKDAYS_ID_LONG[wd], n.day, name),
      }
    }
    DateFormat::Iso => format!("{:04}-{:02}-{:02}", n.year, n.month, n.day),
  }
}

/// Formats the bar clock for `style`. Date and time are separated by two spaces; an empty string means the clock item is
/// hidden (both toggles off).
pub fn format_clock(n: When, lang: Lang, style: ClockStyle) -> String {
  match (style.show_date, style.show_time) {
    (true, true) => format!("{}  {}", date_text(n, lang, style.date), time_text(n, style.time)),
    (true, false) => date_text(n, lang, style.date),
    (false, true) => time_text(n, style.time),
    (false, false) => String::new(),
  }
}

/// Formats the bar clock in the given language, in 12-hour or 24-hour style (short date, both parts shown).
pub fn format(n: When, lang: Lang, h24: bool) -> String {
  let style = ClockStyle {
    time: TimeFormat::from_legacy(h24),
    ..ClockStyle::default()
  };
  format_clock(n, lang, style)
}

#[cfg(test)]
mod tests {
  use super::*;

  const THU: When = When {
    year: 2026,
    month: 11,
    day: 19,
    hour: 9,
    minute: 5,
  };

  #[test]
  fn english_twelve_hour() {
    assert_eq!(format(THU, Lang::En, false), "Thu Nov 19  9:05 AM");
  }

  #[test]
  fn indonesian_twenty_four_hour() {
    assert_eq!(format(THU, Lang::Id, true), "Kam 19 Nov  09.05");
  }

  #[test]
  fn midnight_and_noon_use_twelve() {
    let midnight = When { hour: 0, ..THU };
    let noon = When { hour: 12, ..THU };
    assert!(format(midnight, Lang::En, false).ends_with("12:05 AM"));
    assert!(format(noon, Lang::En, false).ends_with("12:05 PM"));
  }

  #[test]
  fn indonesian_may_is_mei() {
    let may = When {
      month: 5,
      day: 1,
      ..THU
    };
    assert!(format(may, Lang::Id, true).contains("Mei"));
  }

  fn style(show_time: bool, show_date: bool, time: TimeFormat, date: DateFormat) -> ClockStyle {
    ClockStyle {
      show_time,
      show_date,
      time,
      date,
    }
  }

  #[test]
  fn legacy_flag_maps_to_the_enum_and_back() {
    assert_eq!(TimeFormat::from_legacy(true), TimeFormat::H24);
    assert_eq!(TimeFormat::from_legacy(false), TimeFormat::H12);
    assert!(TimeFormat::H24.is_h24() && !TimeFormat::H12.is_h24());
    assert_eq!(TimeFormat::parse(" 24-Hour "), Some(TimeFormat::H24));
    assert_eq!(TimeFormat::parse("12H"), Some(TimeFormat::H12));
    assert_eq!(TimeFormat::parse("soon"), None);
  }

  #[test]
  fn date_format_index_falls_back_to_short() {
    assert_eq!(DateFormat::from_index(0), DateFormat::Short);
    assert_eq!(DateFormat::from_index(1), DateFormat::Long);
    assert_eq!(DateFormat::from_index(2), DateFormat::Iso);
    assert_eq!(DateFormat::from_index(200), DateFormat::Short);
  }

  #[test]
  fn long_date_in_english_and_indonesian() {
    let s = style(false, true, TimeFormat::H24, DateFormat::Long);
    assert_eq!(format_clock(THU, Lang::En, s), "Thursday, November 19");
    assert_eq!(format_clock(THU, Lang::Id, s), "Kamis, 19 November");
  }

  #[test]
  fn iso_date_is_language_independent() {
    let s = style(false, true, TimeFormat::H24, DateFormat::Iso);
    assert_eq!(format_clock(THU, Lang::En, s), "2026-11-19");
    assert_eq!(format_clock(THU, Lang::Id, s), "2026-11-19");
    let single = When {
      month: 3,
      day: 4,
      ..THU
    };
    assert_eq!(format_clock(single, Lang::En, s), "2026-03-04");
  }

  #[test]
  fn short_date_matches_the_legacy_format() {
    let s = style(true, true, TimeFormat::H12, DateFormat::Short);
    assert_eq!(format_clock(THU, Lang::En, s), format(THU, Lang::En, false));
    assert_eq!(format_clock(THU, Lang::Id, s), "Kam 19 Nov  9:05 AM");
  }

  #[test]
  fn toggles_show_only_the_requested_parts() {
    let time_only = style(true, false, TimeFormat::H24, DateFormat::Short);
    let date_only = style(false, true, TimeFormat::H24, DateFormat::Short);
    let neither = style(false, false, TimeFormat::H24, DateFormat::Short);
    assert_eq!(format_clock(THU, Lang::En, time_only), "09.05");
    assert_eq!(format_clock(THU, Lang::En, date_only), "Thu Nov 19");
    assert_eq!(format_clock(THU, Lang::En, neither), "");
  }

  #[test]
  fn long_may_is_mei_in_indonesian() {
    let may = When {
      month: 5,
      day: 1,
      ..THU
    };
    let s = style(false, true, TimeFormat::H24, DateFormat::Long);
    assert!(format_clock(may, Lang::Id, s).ends_with("1 Mei"));
    assert!(format_clock(may, Lang::En, s).ends_with("May 1"));
  }
}
