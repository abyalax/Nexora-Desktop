//! Finder bar clock text, e.g. `Thu Nov 19  9:05 AM` (English) or `Kam 19 Nov  09.05` (Indonesian).

use crate::i18n::{self, Lang};
use crate::panels::calendar;

/// Calendar and clock fields needed to format the bar clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct When {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

const MONTHS_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MONTHS_ID: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];

/// Formats the bar clock in the given language, in 12-hour or 24-hour style.
pub fn format(n: When, lang: Lang, h24: bool) -> String {
    let wd = calendar::weekday(n.year, n.month, n.day) as usize;
    let days = i18n::weekday_labels(lang, false, calendar::weekday_labels(false));
    let month = (n.month - 1) as usize % 12;
    let time = if h24 {
        format!("{:02}.{:02}", n.hour, n.minute)
    } else {
        let h12 = if n.hour % 12 == 0 { 12 } else { n.hour % 12 };
        let ap = if n.hour < 12 { "AM" } else { "PM" };
        format!("{h12}:{:02} {ap}", n.minute)
    };
    match lang {
        Lang::En => format!("{} {} {}  {}", days[wd % 7], MONTHS_EN[month], n.day, time),
        Lang::Id => format!("{} {} {}  {}", days[wd % 7], n.day, MONTHS_ID[month], time),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THU: When = When { year: 2026, month: 11, day: 19, hour: 9, minute: 5 };

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
        let may = When { month: 5, day: 1, ..THU };
        assert!(format(may, Lang::Id, true).contains("Mei"));
    }
}
