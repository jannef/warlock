//! The climate (`assets/data/weather.toml`, spec §5 "Weather", "Temperature", §7): how likely each
//! weather is in each month, how warm each month is, how each weather changes the light from the
//! sky, the temperature, and sight on the overworld, and how rain looks. What the weather is at a
//! moment is [`crate::weather`], the temperature [`crate::temperature`]. Without the file the sky
//! is always clear and every month mild ([`MILD`]).

use super::{ContentError, ParticleEmitter, ParticleTrigger};
use crate::time::MONTHS_PER_YEAR;
use crate::weather::Weather;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The most overworld sight a weather may take away, in cells.
pub const MAX_SIGHT_LOSS: u8 = 8;
/// Without a weather file, every month's mean temperature, in °C.
pub const MILD: i8 = 10;
/// Without a weather file, the day's swing of temperature under a clear sky, in °C.
pub const MILD_SWING: u8 = 8;
/// The coldest and warmest a month's mean temperature may be, in °C.
pub const MONTH_TEMPERATURES: RangeInclusive<i8> = -40..=40;
/// The largest daily swing of temperature under a clear sky, in °C.
pub const MAX_SWING: u8 = 30;
/// The most a weather may cool the air, in °C.
pub const MAX_CHILL: i8 = 15;

/// One weather's effects. Clear weather is the sky as it is: all of its light, no particles.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherDef {
    /// Percent of the direct light of the sun and the moons that gets through the clouds (1 to
    /// 100).
    pub direct: u8,
    /// Percent of the sky's own light that gets through (1 to 100).
    pub sky: u8,
    /// Percent of the direct light the clouds spread over the whole sky, as sky light: soft light
    /// that reaches the shade too (0 to 100).
    #[serde(default)]
    pub scatter: u8,
    /// Cells of overworld sight lost (0 to [`MAX_SIGHT_LOSS`]).
    #[serde(default)]
    pub sight: u8,
    /// How much of a clear sky's daily swing of temperature is left under it, in percent (0 to
    /// 100): clouds keep the day cooler and the night milder.
    #[serde(default = "full_swing")]
    pub swing: u8,
    /// How much cooler the air is under it, in °C (0 to [`MAX_CHILL`]): rain cools it.
    #[serde(default)]
    pub chill: i8,
    /// Presentation only: drops falling on every visible cell under the open sky (an ambient
    /// emitter; `rate` per cell and second).
    #[serde(default)]
    pub drops: Option<ParticleEmitter>,
    /// Presentation only: splashes on visible open ground under the sky.
    #[serde(default)]
    pub splash: Option<ParticleEmitter>,
    /// Presentation only: rings on visible still water under the sky.
    #[serde(default)]
    pub ripples: Option<ParticleEmitter>,
}

impl WeatherDef {
    /// The clear sky: everything gets through, nothing falls.
    pub const CLEAR: WeatherDef = WeatherDef {
        direct: 100,
        sky: 100,
        scatter: 0,
        sight: 0,
        swing: 100,
        chill: 0,
        drops: None,
        splash: None,
        ripples: None,
    };

    /// Its particle emitters: drops, splashes, and ripples, where given.
    pub fn emitters(&self) -> impl Iterator<Item = &ParticleEmitter> {
        [&self.drops, &self.splash, &self.ripples]
            .into_iter()
            .flatten()
    }

    fn check(&self) -> Result<(), String> {
        if !(1..=100).contains(&self.direct) || !(1..=100).contains(&self.sky) {
            return Err("`direct` and `sky` must be 1 to 100 (percent)".into());
        }
        if self.scatter > 100 {
            return Err("`scatter` must be 0 to 100 (percent)".into());
        }
        if self.sight > MAX_SIGHT_LOSS {
            return Err(format!("`sight` must be 0 to {MAX_SIGHT_LOSS} (cells)"));
        }
        if self.swing > 100 {
            return Err("`swing` must be 0 to 100 (percent)".into());
        }
        if !(0..=MAX_CHILL).contains(&self.chill) {
            return Err(format!("`chill` must be 0 to {MAX_CHILL} (°C)"));
        }
        for (name, emitter) in [
            ("drops", &self.drops),
            ("splash", &self.splash),
            ("ripples", &self.ripples),
        ] {
            if let Some(e) = emitter {
                e.check().map_err(|m| format!("{name}: {m}"))?;
                if e.on != ParticleTrigger::Ambient {
                    return Err(format!("{name}: weather particles are `ambient`"));
                }
            }
        }
        Ok(())
    }
}

fn full_swing() -> u8 {
    100
}

/// One month: the chance of each weather in percent (they add up to 100), and its mean
/// temperature in °C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct MonthSpec {
    clear: u8,
    overcast: u8,
    rain: u8,
    heavy_rain: u8,
    temperature: i8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeatherFile {
    /// The day's swing of temperature under a clear sky, in °C: from the coldest hour, before
    /// dawn, to the warmest, in the afternoon.
    swing: u8,
    month: Vec<MonthSpec>,
    overcast: WeatherDef,
    rain: WeatherDef,
    heavy_rain: WeatherDef,
}

/// How likely each weather is in each month, how warm each month is, and what each weather
/// does.
#[derive(Clone, Debug, PartialEq)]
pub struct Climate {
    /// Per month (calendar order), the chance of each weather in percent, in [`Weather::ALL`]
    /// order.
    months: [[u8; 4]; MONTHS_PER_YEAR as usize],
    /// Per month (calendar order), its mean temperature in °C.
    temperatures: [i8; MONTHS_PER_YEAR as usize],
    /// The day's swing of temperature under a clear sky, in °C.
    swing: u8,
    /// In [`Weather::ALL`] order.
    defs: [WeatherDef; 4],
}

impl Default for Climate {
    /// Without a weather file the sky is always clear.
    fn default() -> Self {
        Self {
            months: [[100, 0, 0, 0]; MONTHS_PER_YEAR as usize],
            temperatures: [MILD; MONTHS_PER_YEAR as usize],
            swing: MILD_SWING,
            defs: [
                WeatherDef::CLEAR,
                WeatherDef::CLEAR,
                WeatherDef::CLEAR,
                WeatherDef::CLEAR,
            ],
        }
    }
}

impl Climate {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: WeatherFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.month.len() as u64 != MONTHS_PER_YEAR {
            return Err(err(format!(
                "expected {MONTHS_PER_YEAR} [[month]] entries, found {}",
                parsed.month.len()
            )));
        }
        if parsed.swing > MAX_SWING {
            return Err(err(format!("`swing` must be 0 to {MAX_SWING} (°C)")));
        }
        let mut months = [[0; 4]; MONTHS_PER_YEAR as usize];
        let mut temperatures = [0; MONTHS_PER_YEAR as usize];
        for (i, m) in parsed.month.iter().enumerate() {
            if !MONTH_TEMPERATURES.contains(&m.temperature) {
                return Err(err(format!(
                    "month {}: `temperature` must be {} to {} (°C)",
                    i + 1,
                    MONTH_TEMPERATURES.start(),
                    MONTH_TEMPERATURES.end()
                )));
            }
            temperatures[i] = m.temperature;
            let chances = [m.clear, m.overcast, m.rain, m.heavy_rain];
            let sum: u32 = chances.iter().map(|&c| u32::from(c)).sum();
            if sum != 100 {
                return Err(err(format!(
                    "month {}: the chances add up to {sum}, not 100",
                    i + 1
                )));
            }
            months[i] = chances;
        }
        for (name, def) in [
            ("overcast", &parsed.overcast),
            ("rain", &parsed.rain),
            ("heavy_rain", &parsed.heavy_rain),
        ] {
            def.check().map_err(|m| err(format!("[{name}]: {m}")))?;
        }
        Ok(Self {
            months,
            temperatures,
            swing: parsed.swing,
            defs: [
                WeatherDef::CLEAR,
                parsed.overcast,
                parsed.rain,
                parsed.heavy_rain,
            ],
        })
    }

    /// The chance of each weather in percent in the month with index `month` (0 to 11), in
    /// [`Weather::ALL`] order.
    pub fn chances(&self, month: usize) -> [u8; 4] {
        self.months[month % self.months.len()]
    }

    /// What `weather` does.
    pub fn def(&self, weather: Weather) -> &WeatherDef {
        &self.defs[weather.index()]
    }

    /// The mean temperature of the month with index `month` (0 to 11), in °C.
    pub fn temperature(&self, month: usize) -> i8 {
        self.temperatures[month % self.temperatures.len()]
    }

    /// The day's swing of temperature under a clear sky, in °C: from the coldest hour, before
    /// dawn, to the warmest, in the afternoon.
    pub fn swing(&self) -> u8 {
        self.swing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(months: &str, extra: &str) -> String {
        format!(
            "swing = 10\n{months}\n[overcast]\ndirect = 40\nsky = 90\nscatter = 20\nswing = 50\n\
             [rain]\ndirect = 25\nsky = 70\nsight = 2\n{extra}\n\
             [heavy_rain]\ndirect = 10\nsky = 50\nsight = 4\nchill = 3\n"
        )
    }

    fn months(chances: [u8; 4]) -> String {
        some_months(chances, 12)
    }

    fn some_months(chances: [u8; 4], n: usize) -> String {
        let [c, o, r, h] = chances;
        let month = format!(
            "[[month]]\nclear = {c}\novercast = {o}\nrain = {r}\nheavy_rain = {h}\ntemperature = -4\n"
        );
        month.repeat(n)
    }

    #[test]
    fn parses_the_months_and_the_weathers() {
        let text = file(
            &months([50, 25, 20, 5]),
            "[rain.drops]\nglyphs = \"′\"\ncolors = [\"#8090a0\"]\nrate = 0.5\nlife = 0.2\n",
        );
        let climate = Climate::parse("weather", &text).expect("valid");
        assert_eq!(climate.chances(3), [50, 25, 20, 5]);
        assert_eq!(*climate.def(Weather::Clear), WeatherDef::CLEAR);
        let rain = climate.def(Weather::Rain);
        assert_eq!(
            (rain.direct, rain.sky, rain.scatter, rain.sight),
            (25, 70, 0, 2)
        );
        assert_eq!(rain.emitters().count(), 1);
        assert_eq!(climate.def(Weather::Overcast).scatter, 20);
        assert_eq!(Climate::default().chances(5), [100, 0, 0, 0]);
        // Temperature: the months' means, the clear sky's swing, and what each weather keeps of
        // it and how much it cools the air.
        assert_eq!((climate.temperature(11), climate.swing()), (-4, 10));
        assert_eq!((rain.swing, rain.chill), (100, 0));
        assert_eq!(climate.def(Weather::Overcast).swing, 50);
        assert_eq!(climate.def(Weather::HeavyRain).chill, 3);
        assert_eq!(Climate::default().temperature(0), MILD);
    }

    #[test]
    fn rejects_bad_climates() {
        let good = months([50, 25, 20, 5]);
        for (bad, why) in [
            (file(&months([50, 25, 20, 6]), ""), "add up to 101"),
            (file(&some_months([50, 25, 20, 5], 11), ""), "expected 12"),
            (file(&good, "scatter = 101"), "scatter"),
            (file(&good, "sight = 9"), "sight"),
            (file(&good, "swing = 101"), "swing"),
            (file(&good, "chill = -1"), "chill"),
            (
                file(&good, "").replace("swing = 10\n", "swing = 31\n"),
                "swing",
            ),
            (
                file(&good, "").replace("temperature = -4", "temperature = 41"),
                "temperature",
            ),
            (
                file(&good, "").replace("temperature = -4\n", ""),
                "missing field `temperature`",
            ),
            (
                file(&good, "").replace("direct = 10", "direct = 0"),
                "direct",
            ),
            (file(&good, "fog = 1"), "unknown field"),
            (
                file(
                    &good,
                    "[rain.drops]\nglyphs = \"′\"\ncolors = [\"#8090a0\"]\nlife = 0.2\non = \"step\"\ncount = 2\n",
                ),
                "ambient",
            ),
            (good.clone(), "missing field"),
        ] {
            let e = Climate::parse("weather", &bad).expect_err(why);
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
