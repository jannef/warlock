//! The pause menu's line on the world (#1679): the date, the moon, the weather, and the
//! temperature as one short sentence ending ", the world waits." in place of the status bar's
//! old segments. Drawn from what the player perceives only: the weather only under the open sky,
//! where they know it.

use crate::settings::TemperatureUnit;
use warlock_core::content::MoonDef;
use warlock_core::light::MoonPhase;
use warlock_core::perception::PerceptionView;
use warlock_core::temperature::Temperature;
use warlock_core::time::GameTime;
use warlock_core::weather::Weather;

/// What the pause menu tells of the world, read when it opens (no time passes while it is open).
#[derive(Debug, Clone, PartialEq)]
pub struct Almanac {
    /// `The 11th of Darkening, 1444`.
    date: String,
    /// `under a waxing gibbous moon`; `None` without moons.
    moon: Option<String>,
    /// `clear skies`; `None` where the player does not know the weather.
    sky: Option<&'static str>,
    temperature: Temperature,
}

impl Almanac {
    pub fn of(view: &PerceptionView<'_>) -> Self {
        let time = view.time();
        Self {
            date: date(view.content().calendar().month_name(time), time),
            moon: moon(view.content().moons(), time),
            sky: view.weather().map(sky),
            temperature: view.temperature(),
        }
    }

    /// The sentence in lines of at most `width` characters, at most two: the moon gives way,
    /// then the weather, before a third line would be needed.
    pub fn lines(&self, unit: TemperatureUnit, width: i32) -> Vec<String> {
        let mut moon = self.moon.as_deref();
        let mut sky = self.sky;
        loop {
            let lines = super::menu::wrap(&self.sentence(moon, sky, unit), width);
            if lines.len() <= 2 {
                return lines;
            }
            if moon.take().is_none() && sky.take().is_none() {
                return lines;
            }
        }
    }

    /// `The 11th of Darkening, 1444, under a waxing gibbous moon, clear skies at -2 °C, the
    /// world waits.`; out of the weather, `the air at 10 °C`.
    fn sentence(&self, moon: Option<&str>, sky: Option<&str>, unit: TemperatureUnit) -> String {
        let degrees = unit.show(self.temperature);
        let air = match sky {
            Some(sky) => format!("{sky} at {degrees}"),
            None => format!("the air at {degrees}"),
        };
        let parts = [Some(self.date.as_str()), moon, Some(&air)];
        let parts: Vec<&str> = parts.into_iter().flatten().collect();
        format!("{}, the world waits.", parts.join(", "))
    }
}

/// `The 11th of Darkening, 1444`.
fn date(month: &str, time: GameTime) -> String {
    let day = time.day();
    let suffix = match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("The {day}{suffix} of {month}, {}", time.year())
}

/// `under a waxing gibbous moon`; with several moons, each by name (`under the Moon full and
/// Ilse new`); `None` without moons.
fn moon(moons: &[MoonDef], time: GameTime) -> Option<String> {
    let phase = |m: &MoonDef| MoonPhase::of(m.phase_at(time));
    match moons {
        [] => None,
        [moon] => Some(format!("under a {} moon", words(phase(moon)))),
        _ => {
            let each: Vec<String> = (moons.iter())
                .map(|m| format!("{} {}", m.name, words(phase(m))))
                .collect();
            let (last, rest) = each.split_last()?;
            Some(format!("under {} and {last}", rest.join(", ")))
        }
    }
}

/// A phase in words, as it stands before "moon".
fn words(phase: MoonPhase) -> &'static str {
    match phase {
        MoonPhase::New => "new",
        MoonPhase::WaxingCrescent => "waxing crescent",
        MoonPhase::FirstQuarter => "waxing half",
        MoonPhase::WaxingGibbous => "waxing gibbous",
        MoonPhase::Full => "full",
        MoonPhase::WaningGibbous => "waning gibbous",
        MoonPhase::LastQuarter => "waning half",
        MoonPhase::WaningCrescent => "waning crescent",
    }
}

/// The weather as the sentence says it.
fn sky(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "clear skies",
        Weather::Overcast => "grey skies",
        Weather::Rain => "rain",
        Weather::HeavyRain => "heavy rain",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::{Action, World};

    #[test]
    fn days_take_their_ordinals() {
        let at = |day| GameTime::from_calendar(1444, 11, day, 0, 0).expect("valid");
        let said: Vec<String> = [1, 2, 3, 4, 11, 12, 13, 21, 22, 23, 30]
            .into_iter()
            .map(|d| date("Darkening", at(d)))
            .collect();
        let days: Vec<&str> = said.iter().map(|s| s.split(' ').nth(1).unwrap()).collect();
        assert_eq!(
            days,
            [
                "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd", "30th"
            ]
        );
        assert_eq!(date("Darkening", at(11)), "The 11th of Darkening, 1444");
    }

    #[test]
    fn the_moon_is_named_by_its_phase() {
        let content = warlock_assets::content().expect("builtin content");
        let at = |day| GameTime::from_calendar(1, 4, day, 0, 0).expect("valid");
        let moons = content.moons();
        assert_eq!(moon(moons, at(1)).as_deref(), Some("under a new moon"));
        assert_eq!(moon(moons, at(16)).as_deref(), Some("under a full moon"));
        assert_eq!(
            moon(moons, at(27)).as_deref(),
            Some("under a waning crescent moon")
        );
        assert_eq!(moon(&[], at(1)), None);
        let ilse = MoonDef {
            name: "Ilse".into(),
            period_days: 7,
            ..moons[0].clone()
        };
        let two = [moons[0].clone(), ilse];
        assert_eq!(
            moon(&two, at(16)).as_deref(),
            Some("under Moon full and Ilse new")
        );
    }

    #[test]
    fn tells_the_weather_only_under_the_open_sky() {
        let content = Arc::new(warlock_assets::content().expect("content"));
        // Underground: no weather, the air's temperature.
        let mut hub = World::new(content.clone(), "showcase", None).expect("starts");
        hub.apply(&Action::SetWeather(Weather::Rain));
        let below = Almanac::of(&hub.perception()).lines(TemperatureUnit::Celsius, 200);
        assert_eq!(
            below,
            [
                "The 11th of Darkening, 1444, under a waxing gibbous moon, the air at 10 °C, \
              the world waits."
            ]
        );
        let fahrenheit = Almanac::of(&hub.perception()).lines(TemperatureUnit::Fahrenheit, 200);
        assert!(fahrenheit[0].contains("the air at 50 °F"), "{fahrenheit:?}");
        // Outdoors, the weather with the temperature.
        let mut yard = World::new(content, "courtyard", None).expect("starts");
        yard.apply(&Action::SetWeather(Weather::HeavyRain));
        let line = Almanac::of(&yard.perception()).lines(TemperatureUnit::Celsius, 200);
        assert!(line[0].contains("moon, heavy rain at "), "{line:?}");
        assert!(line[0].ends_with(", the world waits."), "{line:?}");
    }

    #[test]
    fn fits_two_lines_giving_up_the_moon_then_the_weather() {
        let almanac = Almanac {
            date: "The 23rd of Deepwinter, 1444".into(),
            moon: Some("under a waxing crescent moon".into()),
            sky: Some("heavy rain"),
            temperature: Temperature::degrees(-12),
        };
        let unit = TemperatureUnit::Celsius;
        assert_eq!(
            almanac.lines(unit, 54),
            [
                "The 23rd of Deepwinter, 1444, under a waxing crescent",
                "moon, heavy rain at -12 °C, the world waits.",
            ]
        );
        assert_eq!(
            almanac.lines(unit, 40),
            [
                "The 23rd of Deepwinter, 1444, heavy rain",
                "at -12 °C, the world waits.",
            ]
        );
        assert_eq!(
            almanac.lines(unit, 30),
            [
                "The 23rd of Deepwinter, 1444,",
                "the air at -12 °C, the world",
                "waits."
            ]
        );
    }
}
