//! The climate (`assets/data/weather.toml`, spec §5 "Weather", §7): how likely each weather is in
//! each month, how it changes the light from the sky and sight on the overworld, and how rain
//! looks. What the weather is at a moment is [`crate::weather`]. Without the file the sky is
//! always clear.

use super::{ContentError, ParticleEmitter, ParticleTrigger};
use crate::time::MONTHS_PER_YEAR;
use crate::weather::Weather;
use serde::Deserialize;

/// The most overworld sight a weather may take away, in cells.
pub const MAX_SIGHT_LOSS: u8 = 8;

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

/// The chances of each weather in one month, in percent; they add up to 100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct MonthSpec {
    clear: u8,
    overcast: u8,
    rain: u8,
    heavy_rain: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeatherFile {
    month: Vec<MonthSpec>,
    overcast: WeatherDef,
    rain: WeatherDef,
    heavy_rain: WeatherDef,
}

/// How likely each weather is in each month, and what each one does.
#[derive(Clone, Debug, PartialEq)]
pub struct Climate {
    /// Per month (calendar order), the chance of each weather in percent, in [`Weather::ALL`]
    /// order.
    months: [[u8; 4]; MONTHS_PER_YEAR as usize],
    /// In [`Weather::ALL`] order.
    defs: [WeatherDef; 4],
}

impl Default for Climate {
    /// Without a weather file the sky is always clear.
    fn default() -> Self {
        Self {
            months: [[100, 0, 0, 0]; MONTHS_PER_YEAR as usize],
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
        let mut months = [[0; 4]; MONTHS_PER_YEAR as usize];
        for (i, m) in parsed.month.iter().enumerate() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(months: &str, extra: &str) -> String {
        format!(
            "{months}\n[overcast]\ndirect = 40\nsky = 90\nscatter = 20\n\
             [rain]\ndirect = 25\nsky = 70\nsight = 2\n{extra}\n\
             [heavy_rain]\ndirect = 10\nsky = 50\nsight = 4\n"
        )
    }

    fn months(chances: [u8; 4]) -> String {
        some_months(chances, 12)
    }

    fn some_months(chances: [u8; 4], n: usize) -> String {
        let [c, o, r, h] = chances;
        format!("[[month]]\nclear = {c}\novercast = {o}\nrain = {r}\nheavy_rain = {h}\n").repeat(n)
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
    }

    #[test]
    fn rejects_bad_climates() {
        let good = months([50, 25, 20, 5]);
        for (bad, why) in [
            (file(&months([50, 25, 20, 6]), ""), "add up to 101"),
            (file(&some_months([50, 25, 20, 5], 11), ""), "expected 12"),
            (file(&good, "scatter = 101"), "scatter"),
            (file(&good, "sight = 9"), "sight"),
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
