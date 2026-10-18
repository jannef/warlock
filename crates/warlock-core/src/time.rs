//! The game clock and calendar (spec §8 "Time").
//!
//! World time is whole seconds since the calendar epoch: year 1, day 1 of the first month, 00:00.
//! The calendar has 12 months of 30 days (a 360-day year) and 24-hour days; only the month names
//! are content ([`crate::content::Calendar`]). Everything is integer arithmetic, so time is
//! deterministic on every platform.
//!
//! [`Duration`] is how long something takes; [`GameTime`] is a moment on the clock. Both have a
//! text form for action scripts: durations are `6s`, `30m`, `1h`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

/// Seconds in a minute, an hour, a day, a month, and a year.
pub const MINUTE: u64 = 60;
pub const HOUR: u64 = 60 * MINUTE;
pub const DAY: u64 = 24 * HOUR;
pub const DAYS_PER_MONTH: u64 = 30;
pub const MONTHS_PER_YEAR: u64 = 12;
pub const MONTH: u64 = DAYS_PER_MONTH * DAY;
pub const YEAR: u64 = MONTHS_PER_YEAR * MONTH;
/// Minutes in a day: the length of per-minute-of-day tables.
pub const MINUTES_PER_DAY: u32 = 24 * 60;
/// [`GameTime::day_fraction`] of a whole day (16-bit fixed point).
pub const DAY_FRACTION_ONE: u32 = 1 << 16;
/// Dawn and dusk as seconds of the day (06:00 and 18:00), the targets of resting.
pub const DAWN: u64 = 6 * HOUR;
pub const DUSK: u64 = 18 * HOUR;

/// A span of game time in whole seconds.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Duration(u64);

impl Duration {
    pub const ZERO: Duration = Duration(0);

    pub const fn seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn minutes(minutes: u64) -> Self {
        Self(minutes * MINUTE)
    }

    pub const fn hours(hours: u64) -> Self {
        Self(hours * HOUR)
    }

    pub const fn as_seconds(self) -> u64 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// In words for messages, e.g. `1 hour`, `2 hours and 30 minutes`, `45 seconds` (whole
    /// minutes only once it is a minute or more).
    pub fn spoken(self) -> String {
        let unit = |n: u64, one: &str| {
            let s = if n == 1 { "" } else { "s" };
            format!("{n} {one}{s}")
        };
        let (hours, minutes) = (self.0 / HOUR, self.0 % HOUR / MINUTE);
        match (hours, minutes) {
            (0, 0) => unit(self.0, "second"),
            (0, m) => unit(m, "minute"),
            (h, 0) => unit(h, "hour"),
            (h, m) => format!("{} and {}", unit(h, "hour"), unit(m, "minute")),
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, other: Duration) -> Duration {
        Duration(self.0.saturating_add(other.0))
    }
}

impl Mul<u64> for Duration {
    type Output = Duration;

    fn mul(self, n: u64) -> Duration {
        Duration(self.0.saturating_mul(n))
    }
}

/// Text form: the largest unit that divides it exactly, e.g. `6s`, `30m`, `1h`, `90m`.
impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            s if s > 0 && s % HOUR == 0 => write!(f, "{}h", s / HOUR),
            s if s > 0 && s % MINUTE == 0 => write!(f, "{}m", s / MINUTE),
            s => write!(f, "{s}s"),
        }
    }
}

/// Parses a whole number with a unit: `s`, `m`, or `h` (case-insensitive), e.g. `90m`.
impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || format!("bad duration `{s}` (expected e.g. 30s, 10m, 1h)");
        let split = s.len().checked_sub(1).ok_or_else(bad)?;
        let (number, unit) = s.split_at(split);
        if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad());
        }
        let n: u64 = number.parse().map_err(|_| bad())?;
        let scale = match unit {
            "s" | "S" => 1,
            "m" | "M" => MINUTE,
            "h" | "H" => HOUR,
            _ => return Err(bad()),
        };
        n.checked_mul(scale).map(Duration).ok_or_else(bad)
    }
}

/// A moment on the game clock: whole seconds since the calendar epoch (year 1, month 1, day 1,
/// 00:00). Calendar fields are 1-based like dates are written (`day` 1 to 30, `month` 1 to 12);
/// hours, minutes, and seconds start at 0.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct GameTime(u64);

impl GameTime {
    pub const EPOCH: GameTime = GameTime(0);

    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// The moment of a calendar date and time of day; `None` if any field is out of range
    /// (`year` ≥ 1, `month` 1–12, `day` 1–30, `hour` 0–23, `minute` 0–59).
    pub fn from_calendar(year: u64, month: u32, day: u32, hour: u32, minute: u32) -> Option<Self> {
        let ok = year >= 1
            && (1..=MONTHS_PER_YEAR).contains(&u64::from(month))
            && (1..=DAYS_PER_MONTH).contains(&u64::from(day))
            && hour < 24
            && minute < 60;
        if !ok {
            return None;
        }
        (year - 1)
            .checked_mul(YEAR)?
            .checked_add(u64::from(month - 1) * MONTH + u64::from(day - 1) * DAY)?
            .checked_add(u64::from(hour) * HOUR + u64::from(minute) * MINUTE)
            .map(Self)
    }

    /// Seconds since the epoch.
    pub const fn seconds(self) -> u64 {
        self.0
    }

    /// The year, from 1.
    pub const fn year(self) -> u64 {
        self.0 / YEAR + 1
    }

    /// The month, 1 to 12.
    pub const fn month(self) -> u32 {
        (self.0 % YEAR / MONTH) as u32 + 1
    }

    /// The month as an index into the calendar's month names, 0 to 11.
    pub const fn month_index(self) -> usize {
        (self.0 % YEAR / MONTH) as usize
    }

    /// The day of the month, 1 to 30.
    pub const fn day(self) -> u32 {
        (self.0 % MONTH / DAY) as u32 + 1
    }

    pub const fn hour(self) -> u32 {
        (self.0 % DAY / HOUR) as u32
    }

    pub const fn minute(self) -> u32 {
        (self.0 % HOUR / MINUTE) as u32
    }

    pub const fn second(self) -> u32 {
        (self.0 % MINUTE) as u32
    }

    /// Seconds since midnight, 0 to 86 399.
    pub const fn second_of_day(self) -> u32 {
        (self.0 % DAY) as u32
    }

    /// Minutes since midnight, 0 to 1439: the index into per-minute tables such as the sun's path.
    pub const fn minute_of_day(self) -> u32 {
        (self.0 % DAY / MINUTE) as u32
    }

    /// How far through the day it is, in 1/65 536ths ([`DAY_FRACTION_ONE`] is a whole day):
    /// 0 at midnight, 16 384 at 06:00, 32 768 at noon.
    pub const fn day_fraction(self) -> u32 {
        (self.0 % DAY * DAY_FRACTION_ONE as u64 / DAY) as u32
    }

    /// The first moment strictly after this one at `second_of_day` (e.g. [`DAWN`]): later today,
    /// else tomorrow. At most a day ahead.
    pub fn next_at(self, second_of_day: u64) -> GameTime {
        let target = second_of_day % DAY;
        let now = self.0 % DAY;
        let wait = if target > now {
            target - now
        } else {
            DAY - now + target
        };
        self + Duration(wait)
    }

    /// Time elapsed since `earlier` (zero if `earlier` is later).
    pub const fn since(self, earlier: GameTime) -> Duration {
        Duration(self.0.saturating_sub(earlier.0))
    }
}

impl Add<Duration> for GameTime {
    type Output = GameTime;

    fn add(self, d: Duration) -> GameTime {
        GameTime(self.0.saturating_add(d.0))
    }
}

impl AddAssign<Duration> for GameTime {
    fn add_assign(&mut self, d: Duration) {
        *self = *self + d;
    }
}

/// `YYYY-MM-DD hh:mm:ss` with numeric months, e.g. `0001-04-12 08:00:00` (month names are
/// content; see [`crate::content::Calendar`]).
impl fmt::Display for GameTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second()
        )
    }
}

/// Parses a time of day `hh:mm` (24-hour clock) into seconds since midnight.
pub fn parse_time_of_day(text: &str) -> Result<u64, String> {
    let bad = || format!("bad time of day `{text}` (expected hh:mm, e.g. 08:00)");
    let (h, m) = text.split_once(':').ok_or_else(bad)?;
    let field = |s: &str, max: u64| -> Result<u64, String> {
        if s.len() != 2 || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad());
        }
        s.parse().ok().filter(|v| *v < max).ok_or_else(bad)
    };
    Ok(field(h, 24)? * HOUR + field(m, 60)? * MINUTE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn calendar_fields_of_known_moments() {
        let t = GameTime::from_calendar(1, 4, 12, 14, 32).expect("valid date");
        assert_eq!(
            (t.year(), t.month(), t.month_index(), t.day()),
            (1, 4, 3, 12)
        );
        assert_eq!((t.hour(), t.minute(), t.second()), (14, 32, 0));
        assert_eq!(t.to_string(), "0001-04-12 14:32:00");
        assert_eq!(GameTime::EPOCH.to_string(), "0001-01-01 00:00:00");
        // The last second of year 1 and the first of year 2.
        let end = GameTime::from_seconds(YEAR - 1);
        assert_eq!(end.to_string(), "0001-12-30 23:59:59");
        assert_eq!(
            (end + Duration::seconds(1)).to_string(),
            "0002-01-01 00:00:00"
        );
    }

    #[test]
    fn out_of_range_dates_are_rejected() {
        for (y, mo, d, h, mi) in [
            (0, 1, 1, 0, 0),
            (1, 0, 1, 0, 0),
            (1, 13, 1, 0, 0),
            (1, 1, 0, 0, 0),
            (1, 1, 31, 0, 0),
            (1, 1, 1, 24, 0),
            (1, 1, 1, 0, 60),
        ] {
            assert_eq!(
                GameTime::from_calendar(y, mo, d, h, mi),
                None,
                "{y} {mo} {d}"
            );
        }
        assert_eq!(GameTime::from_calendar(u64::MAX, 1, 1, 0, 0), None);
    }

    #[test]
    fn day_fraction_and_minute_of_day() {
        let at = |h, m| GameTime::from_calendar(3, 7, 9, h, m).expect("valid");
        assert_eq!(at(0, 0).day_fraction(), 0);
        assert_eq!(at(6, 0).day_fraction(), DAY_FRACTION_ONE / 4);
        assert_eq!(at(12, 0).day_fraction(), DAY_FRACTION_ONE / 2);
        assert!(at(23, 59).day_fraction() < DAY_FRACTION_ONE);
        assert_eq!(at(0, 0).minute_of_day(), 0);
        assert_eq!(at(14, 32).minute_of_day(), 14 * 60 + 32);
        assert_eq!(at(23, 59).minute_of_day(), MINUTES_PER_DAY - 1);
    }

    #[test]
    fn next_at_is_strictly_later_and_within_a_day() {
        let day = |d, h, m| GameTime::from_calendar(1, 1, d, h, m).expect("valid");
        assert_eq!(day(1, 18, 0).next_at(DAWN), day(2, 6, 0));
        assert_eq!(day(1, 5, 59).next_at(DAWN), day(1, 6, 0));
        assert_eq!(day(1, 6, 0).next_at(DAWN), day(2, 6, 0));
        assert_eq!(day(1, 8, 0).next_at(DUSK), day(1, 18, 0));
        // Month and year boundaries roll over.
        let last = GameTime::from_calendar(1, 12, 30, 20, 0).expect("valid");
        assert_eq!(
            last.next_at(DAWN),
            GameTime::from_calendar(2, 1, 1, 6, 0).expect("valid")
        );
    }

    #[test]
    fn durations_read_aloud() {
        assert_eq!(Duration::hours(1).spoken(), "1 hour");
        assert_eq!(Duration::minutes(150).spoken(), "2 hours and 30 minutes");
        assert_eq!(Duration::minutes(1).spoken(), "1 minute");
        assert_eq!(Duration::seconds(45).spoken(), "45 seconds");
        assert_eq!(Duration::seconds(61).spoken(), "1 minute");
    }

    #[test]
    fn duration_text_round_trips_and_rejects_garbage() {
        for d in [
            Duration::seconds(6),
            Duration::minutes(30),
            Duration::hours(1),
            Duration::minutes(90),
            Duration::ZERO,
        ] {
            assert_eq!(d.to_string().parse(), Ok(d), "{d}");
        }
        assert_eq!("1H".parse(), Ok(Duration::hours(1)));
        assert_eq!(Duration::minutes(90).to_string(), "90m");
        for bad in [
            "",
            "h",
            "1",
            "1d",
            "-1h",
            "1.5h",
            "+1h",
            "99999999999999999999h",
        ] {
            assert!(bad.parse::<Duration>().is_err(), "{bad}");
        }
    }

    #[test]
    fn times_of_day_parse() {
        assert_eq!(parse_time_of_day("08:00"), Ok(8 * HOUR));
        assert_eq!(parse_time_of_day("23:59"), Ok(23 * HOUR + 59 * MINUTE));
        for bad in ["8:00", "24:00", "12:60", "12-00", "", "ab:cd"] {
            assert!(parse_time_of_day(bad).is_err(), "{bad}");
        }
    }

    proptest! {
        #[test]
        fn calendar_round_trips(seconds in 0u64..(1000 * YEAR)) {
            let t = GameTime::from_seconds(seconds);
            let back = GameTime::from_calendar(t.year(), t.month(), t.day(), t.hour(), t.minute())
                .expect("fields of a real moment are valid");
            prop_assert_eq!(back + Duration::seconds(u64::from(t.second())), t);
            prop_assert_eq!(
                u64::from(t.second_of_day()),
                u64::from(t.hour()) * HOUR + u64::from(t.minute()) * MINUTE + u64::from(t.second())
            );
            prop_assert!(t.day_fraction() < DAY_FRACTION_ONE);
        }

        #[test]
        fn next_at_lands_on_the_time_of_day(seconds in 0u64..(10 * YEAR), at in 0u64..DAY) {
            let t = GameTime::from_seconds(seconds);
            let next = t.next_at(at);
            prop_assert!(next > t);
            prop_assert!(next.since(t) <= Duration::seconds(DAY));
            prop_assert_eq!(u64::from(next.second_of_day()), at);
        }
    }
}
