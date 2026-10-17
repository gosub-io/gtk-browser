//! Form control values to and from the numbers a native date or time dialog works in.
//!
//! The engine hands a picker the control's value already sanitised - `YYYY-MM-DD`, `HH:MM`
//! (seconds allowed), `YYYY-MM-DDTHH:MM`, or empty - and takes the same shape back. Android's
//! dialogs count months from 0 and bound a date in epoch milliseconds, so the conversions live
//! here, apart from the JNI calls, where they can be tested on any host.

// Only the Android frontend opens native pickers; elsewhere this is exercised by its tests.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

/// A calendar date, month 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// A time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
}

/// `YYYY-MM-DD`, or `None` for anything else (an empty value included).
pub fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.splitn(3, '-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    ((1..=12).contains(&month) && (1..=31).contains(&day)).then_some(Date { year, month, day })
}

/// `HH:MM`, with any seconds after it ignored.
pub fn parse_time(value: &str) -> Option<Time> {
    let mut parts = value.splitn(3, ':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    (hour < 24 && minute < 60).then_some(Time { hour, minute })
}

/// `YYYY-MM-DDTHH:MM`, split into its date and time.
pub fn parse_date_time(value: &str) -> (Option<Date>, Option<Time>) {
    match value.split_once('T') {
        Some((date, time)) => (parse_date(date), parse_time(time)),
        None => (parse_date(value), None),
    }
}

pub fn format_date(date: Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

pub fn format_time(time: Time) -> String {
    format!("{:02}:{:02}", time.hour, time.minute)
}

/// A `min` or `max` attribute as the epoch milliseconds of that day's noon UTC. Noon, so the
/// dialog, which reads the bound in the phone's own time zone, lands on the same day in any
/// zone within 12 hours of UTC.
pub fn bound_millis(value: Option<&str>) -> Option<i64> {
    let date = parse_date(value?)?;
    Some((days_from_civil(date) * 86_400 + 12 * 3_600) * 1_000)
}

/// What a date dialog answers, `"y m d"` with a 0-based month, as a date.
pub fn date_from_dialog(answer: &str) -> Option<Date> {
    let mut parts = answer.split(' ').map(str::parse::<i64>);
    let year = parts.next()?.ok()?;
    let month = parts.next()?.ok()?;
    let day = parts.next()?.ok()?;
    Some(Date {
        year: i32::try_from(year).ok()?,
        month: u32::try_from(month + 1).ok()?,
        day: u32::try_from(day).ok()?,
    })
}

/// What a time dialog answers, `"h m"`, as a time.
pub fn time_from_dialog(answer: &str) -> Option<Time> {
    let mut parts = answer.split(' ').map(str::parse::<u32>);
    Some(Time {
        hour: parts.next()?.ok()?,
        minute: parts.next()?.ok()?,
    })
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
fn days_from_civil(date: Date) -> i64 {
    let month = i64::from(date.month);
    let year = i64::from(date.year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(date.day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_round_trips() {
        let date = parse_date("2026-03-07").unwrap();
        assert_eq!(
            date,
            Date {
                year: 2026,
                month: 3,
                day: 7
            }
        );
        assert_eq!(format_date(date), "2026-03-07");
    }

    #[test]
    fn an_empty_or_broken_value_is_no_date() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("yesterday"), None);
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("24:00"), None);
    }

    #[test]
    fn a_time_drops_its_seconds() {
        assert_eq!(parse_time("09:05:30"), Some(Time { hour: 9, minute: 5 }));
        assert_eq!(format_time(Time { hour: 9, minute: 5 }), "09:05");
    }

    #[test]
    fn a_date_time_splits_at_the_t() {
        assert_eq!(
            parse_date_time("2026-10-16T14:30"),
            (
                Some(Date {
                    year: 2026,
                    month: 10,
                    day: 16
                }),
                Some(Time { hour: 14, minute: 30 })
            )
        );
        assert_eq!(parse_date_time(""), (None, None));
    }

    #[test]
    fn the_dialog_counts_months_from_zero() {
        assert_eq!(
            date_from_dialog("2026 0 31"),
            Some(Date {
                year: 2026,
                month: 1,
                day: 31
            })
        );
        assert_eq!(time_from_dialog("7 45"), Some(Time { hour: 7, minute: 45 }));
        assert_eq!(date_from_dialog(""), None);
    }

    #[test]
    fn a_bound_is_noon_utc_of_its_day() {
        assert_eq!(bound_millis(Some("1970-01-01")), Some(12 * 3_600_000));
        // 2000-03-01 is day 11017: across a leap day and a century rule.
        assert_eq!(bound_millis(Some("2000-03-01")), Some((11_017 * 86_400 + 43_200) * 1_000));
        assert_eq!(bound_millis(Some("")), None);
        assert_eq!(bound_millis(None), None);
    }
}
