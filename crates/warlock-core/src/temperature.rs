//! Temperature (spec §5 "Temperature"): how warm the air is on a level.
//!
//! **Underground** (dungeon levels, hand-made or generated) it is fixed: the level file's
//! `temperature`, else its dungeon branch's for that depth (`data/dungeons.toml`), else
//! [`UNDERGROUND`] (10 °C). It is kept with the level ([`crate::world::LevelState`]).
//!
//! **Under the open sky** (outdoor levels and the overworld) it follows the month, the time of
//! day, and the weather ([`outdoors`]), a pure function of the run seed and the time like the
//! weather itself, so long rests and journeys need no history:
//! - the month's mean (`temperature` per `[[month]]` in `data/weather.toml`): the middle of each
//!   month has its mean, and the days between two middles go linearly from one to the next;
//! - the day's swing around it, a cosine coldest at [`COLDEST`] (04:00) and warmest twelve hours
//!   later: a clear day's `swing` from the coldest to the warmest, less under clouds (each
//!   weather's `swing` percent): clear nights are cold, overcast ones milder;
//! - less the weather's `chill`: rain cools the air.
//!
//! Where the weather changes, its swing and chill change with the clouds, over an hour
//! ([`crate::weather::Blend`]). Integer math only (tenths of a degree, the sun's fixed-point
//! cosine), so it is the same on every platform.
//!
//! Nothing feels it yet (no hypothermia, warmth, or clothing): water does, which freezes below
//! zero ([`crate::ice`]). The player knows it only in words, under the open sky, and only when
//! it is worth saying ([`Feel`]).

use crate::content::Climate;
use crate::light::{ONE, TURN, cos};
use crate::time::{DAY, GameTime, HOUR, MONTH, MONTHS_PER_YEAR, YEAR};
use crate::weather::{self, Override};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::RangeInclusive;

/// The temperature underground where neither the level nor its dungeon branch gives one, in °C.
pub const UNDERGROUND: i8 = 10;
/// The coldest and warmest a level or a dungeon branch may be, in °C.
pub const LEVEL_TEMPERATURES: RangeInclusive<i8> = -40..=40;
/// The coldest moment of the day under the open sky, in seconds since midnight; the warmest is
/// twelve hours later (16:00).
pub const COLDEST: u64 = 4 * HOUR;

/// A temperature, in tenths of a degree Celsius.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Temperature(i32);

impl Temperature {
    /// Freezing: 0 °C.
    pub const ZERO: Temperature = Temperature(0);

    /// `c` whole degrees.
    pub const fn degrees(c: i32) -> Self {
        Self(c * 10)
    }

    pub const fn from_tenths(tenths: i32) -> Self {
        Self(tenths)
    }

    /// In tenths of a degree.
    pub const fn tenths(self) -> i32 {
        self.0
    }

    /// In whole degrees Celsius, rounded half away from zero (-0.5 °C is -1 °C, -0.4 °C is 0).
    pub const fn celsius(self) -> i32 {
        round_tenths(self.0)
    }

    /// In whole degrees Fahrenheit (32 + 9/5 of the Celsius), rounded half away from zero.
    pub const fn fahrenheit(self) -> i32 {
        // Tenths of a degree Fahrenheit, rounded the same way before the last rounding.
        let tenths = self.0 * 9;
        let tenths = if tenths >= 0 {
            (tenths + 2) / 5
        } else {
            (tenths - 2) / 5
        };
        round_tenths(tenths + 320)
    }

    /// Below freezing.
    pub const fn is_freezing(self) -> bool {
        self.0 < 0
    }
}

/// Tenths to whole units, halves away from zero.
const fn round_tenths(tenths: i32) -> i32 {
    if tenths >= 0 {
        (tenths + 5) / 10
    } else {
        (tenths - 5) / 10
    }
}

/// Dev text: `-3.5 °C`.
impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let a = self.0.unsigned_abs();
        write!(f, "{sign}{}.{} °C", a / 10, a % 10)
    }
}

/// The temperature under the open sky at `time` in the run with `seed`, under the weather of the
/// moment (the dev override `over` while it lasts).
pub fn outdoors(
    seed: u64,
    time: GameTime,
    climate: &Climate,
    over: Option<&Override>,
) -> Temperature {
    let blend = weather::blend(seed, time, climate, over);
    // In thousandths of a tenth of a degree, rounded to tenths at the end, so it changes
    // smoothly: the swing from the coldest moment to the warmest, and the cooling.
    let swing = i64::from(climate.swing()) * blend.mix(|w| i64::from(climate.def(w).swing) * 1000);
    let swing = swing / 10;
    let chill = blend.mix(|w| i64::from(climate.def(w).chill) * 10_000);
    let day = (swing * daily(time)).div_euclid(2 * ONE);
    let total = mean_fine(time, climate) + day - chill;
    Temperature((total + 500).div_euclid(1000) as i32)
}

/// The months' means at `time`, in tenths of a degree (rounded down): each month's mean at its
/// middle, and linearly from one middle to the next.
pub fn mean(time: GameTime, climate: &Climate) -> i64 {
    mean_fine(time, climate).div_euclid(1000)
}

/// [`mean`] in thousandths of a tenth of a degree.
fn mean_fine(time: GameTime, climate: &Climate) -> i64 {
    let half = MONTH / 2;
    // Time since the middle of the first month, within the year.
    let since = (time.seconds() % YEAR + YEAR - half) % YEAR;
    let month = (since / MONTH) as usize;
    let into = (since % MONTH) as i64;
    let from = i64::from(climate.temperature(month)) * 10_000;
    let to = i64::from(climate.temperature((month + 1) % MONTHS_PER_YEAR as usize)) * 10_000;
    from + ((to - from) * into).div_euclid(MONTH as i64)
}

/// Where the day's swing stands at `time`, in 1/[`ONE`]: -1 at [`COLDEST`], +1 twelve hours
/// later, a cosine between.
pub fn daily(time: GameTime) -> i64 {
    let since = (u64::from(time.second_of_day()) + DAY - COLDEST) % DAY;
    -cos(since as i64 * TURN / DAY as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Content, ContentFiles};
    use crate::weather::Weather;

    fn climate() -> Climate {
        Content::load(&ContentFiles::builtin())
            .expect("content")
            .climate()
            .clone()
    }

    fn at(month: u32, day: u32, hour: u32, minute: u32) -> GameTime {
        GameTime::from_calendar(1444, month, day, hour, minute).expect("a date")
    }

    /// Clear skies all day (the dev override, for three hours from `time`).
    fn clear(time: GameTime) -> Override {
        Override::new(Weather::Clear, time)
    }

    #[test]
    fn whole_degrees_round_half_away_from_zero_in_either_unit() {
        let c = |t: i32| Temperature::from_tenths(t).celsius();
        assert_eq!(
            [c(0), c(4), c(5), c(14), c(15), c(-4), c(-5), c(-15)],
            [0, 0, 1, 1, 2, 0, -1, -2]
        );
        assert_eq!(c(-35), -4);
        assert_eq!(c(375), 38);
        let f = |t: i32| Temperature::from_tenths(t).fahrenheit();
        // Freezing and boiling, body warmth, and where the scales meet.
        assert_eq!(f(0), 32);
        assert_eq!(f(1000), 212);
        assert_eq!(f(370), 99);
        assert_eq!(f(-400), -40);
        // -17.8 °C is 0 °F; -3.5 °C is 25.7 °F; 21.4 °C is 70.5 °F, rounded up.
        assert_eq!([f(-178), f(-35), f(214)], [0, 26, 71]);
        assert!(Temperature::from_tenths(-1).is_freezing());
        assert!(!Temperature::ZERO.is_freezing());
        assert_eq!(Temperature::from_tenths(-35).to_string(), "-3.5 °C");
        assert_eq!(Temperature::degrees(12).to_string(), "12.0 °C");
    }

    #[test]
    fn the_middle_of_each_month_has_its_mean_and_the_days_between_go_linearly() {
        let climate = climate();
        for month in 1..=12u32 {
            let mid = at(month, 16, 0, 0);
            let want = i64::from(climate.temperature(month as usize - 1)) * 10;
            assert_eq!(mean(mid, &climate), want, "month {month}");
        }
        // Halfway from the middle of Darkening (1 °C) to the middle of Deepwinter (-4 °C): the
        // first of Deepwinter, -1.5 °C. The year wraps from Deepwinter to Frostmere.
        assert_eq!(mean(at(12, 1, 0, 0), &climate), -15);
        let new_year = mean(at(1, 1, 0, 0), &climate);
        let (deepwinter, frostmere) = (climate.temperature(11), climate.temperature(0));
        assert_eq!(new_year, (i64::from(deepwinter) + i64::from(frostmere)) * 5);
    }

    #[test]
    fn the_day_is_coldest_before_dawn_and_warmest_in_the_afternoon() {
        assert_eq!(daily(at(4, 3, 4, 0)), -ONE);
        assert_eq!(daily(at(4, 3, 16, 0)), ONE);
        assert_eq!(daily(at(4, 3, 10, 0)), 0);
        assert_eq!(daily(at(4, 3, 22, 0)), 0);
        // A cosine: as warm two hours before the coldest moment as two hours after it.
        assert!((daily(at(4, 3, 2, 0)) - daily(at(4, 3, 6, 0))).abs() < 8);
        // Under a clear sky the day swings the whole swing around the month's mean.
        let climate = climate();
        let t = |h| {
            let time = at(6, 16, h, 0);
            outdoors(1, time, &climate, Some(&clear(time))).tenths()
        };
        let swing = i32::from(climate.swing()) * 10;
        let mean = i32::from(climate.temperature(5)) * 10;
        let (low, high) = (t(4), t(16));
        // The month's mean drifts a little over the day.
        assert!((high - low - swing).abs() <= 1, "{low} {high}");
        assert!(((high + low) / 2 - mean).abs() <= 1);
    }

    #[test]
    fn clouds_keep_nights_milder_and_days_cooler_and_rain_cools_the_air() {
        let climate = climate();
        let morning = at(11, 11, 4, 0);
        let afternoon = at(11, 11, 16, 0);
        let under = |w, time| {
            let over = Override::new(w, time);
            outdoors(7, time, &climate, Some(&over)).tenths()
        };
        let night = |w| under(w, morning);
        let day = |w| under(w, afternoon);
        assert!(night(Weather::Clear) < night(Weather::Overcast));
        assert!(day(Weather::Clear) > day(Weather::Overcast));
        assert!(day(Weather::Overcast) > day(Weather::Rain));
        assert!(day(Weather::Rain) > day(Weather::HeavyRain));
        // A clear Darkening night is freezing; a clear Darkening afternoon is not.
        assert!(night(Weather::Clear) < 0, "{}", night(Weather::Clear));
        assert!(day(Weather::Clear) > 0);
    }

    #[test]
    fn deep_winter_freezes_and_high_summer_does_not() {
        let climate = climate();
        let mean_at = |month: u32| {
            // The day's mean temperature at mid-month over many days of weather.
            let samples: Vec<i64> = (0..30 * 24)
                .map(|h| {
                    let t = at(month, 1, 0, 0) + crate::time::Duration::hours(h);
                    i64::from(outdoors(3, t, &climate, None).tenths())
                })
                .collect();
            samples.iter().sum::<i64>() / samples.len() as i64
        };
        assert!(mean_at(1) < -40, "Frostmere {}", mean_at(1));
        assert!(mean_at(7) > 180, "Emberheat {}", mean_at(7));
    }

    #[test]
    fn the_temperature_is_a_pure_function_of_seed_and_time_and_changes_smoothly() {
        let climate = climate();
        let start = at(11, 1, 0, 0);
        let mut last: Option<i32> = None;
        for m in 0..10 * 24 * 60 {
            let time = start + crate::time::Duration::minutes(m);
            let t = outdoors(5, time, &climate, None).tenths();
            assert_eq!(t, outdoors(5, time, &climate, None).tenths());
            // A minute never changes it by more than a few tenths: the weather's swing and chill
            // change with the clouds, over an hour.
            if let Some(last) = last {
                assert!((t - last).abs() <= 2, "{time}: {last} -> {t}");
            }
            last = Some(t);
        }
    }
}
