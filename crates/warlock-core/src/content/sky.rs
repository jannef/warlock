//! The moons (`assets/data/sky.toml`, spec §5 "Moonlight", §7): how many there are, how they
//! look, and how fast they wax and wane. Where they stand and the light they give at a moment is
//! [`crate::light::Sky`]. Without the file there are no moons: nights are dark.

use super::ContentError;
use crate::light::Light;
use crate::time::{DAY, GameTime};
use serde::Deserialize;

/// Most moons a sky may have.
pub const MAX_MOONS: usize = 8;
/// Longest cycle a moon may have, in days (ten years).
pub const MAX_PERIOD_DAYS: u32 = 3600;
/// Apparent size of a standard moon, in percent: one of this size gives its `color` and `sky`
/// at full moon, high in the night sky.
pub const STANDARD_SIZE: u16 = 100;
/// Largest apparent size, in percent of a standard moon.
pub const MAX_SIZE: u16 = 400;
/// [`MoonDef::phase_at`] of a whole cycle (16-bit fixed point).
pub const PHASE_ONE: u32 = 1 << 16;

/// One moon of the sky.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoonDef {
    pub name: String,
    /// Direct light of this moon when full and high in the night sky, `[r, g, b, uv]` (it falls
    /// on every cell under the open sky that no taller terrain shades from it).
    pub color: Light,
    /// Light from the sky around it at the same moment, on every cell under the open sky, shaded
    /// or not (default none).
    #[serde(default)]
    pub sky: Light,
    /// Apparent diameter in percent of a standard moon (default 100); its light scales with its
    /// area, the square of this.
    #[serde(default = "standard_size")]
    pub size: u16,
    /// Days from new moon to new moon.
    pub period_days: u32,
    /// The moon's age (days since its last new moon) at the calendar epoch (default 0: new on
    /// the first day of the first year).
    #[serde(default)]
    pub phase_offset_days: u32,
}

fn standard_size() -> u16 {
    STANDARD_SIZE
}

impl MoonDef {
    /// How far through its cycle the moon is at `time`, in 1/[`PHASE_ONE`]: 0 at new moon, a
    /// quarter at first quarter, half at full moon, three quarters at last quarter.
    pub fn phase_at(&self, time: GameTime) -> u32 {
        let period = u64::from(self.period_days) * DAY;
        let age = (time.seconds() + u64::from(self.phase_offset_days) * DAY) % period;
        (age * u64::from(PHASE_ONE) / period) as u32
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkyFile {
    #[serde(default)]
    moon: Vec<MoonDef>,
}

/// Parses and checks `data/sky.toml`: the moons in file order.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<MoonDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: SkyFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.moon.len() > MAX_MOONS {
        return Err(err(format!("at most {MAX_MOONS} moons")));
    }
    for (i, moon) in parsed.moon.iter().enumerate() {
        let err = |m: &str| err(format!("moon {} (`{}`): {m}", i + 1, moon.name));
        if moon.name.trim().is_empty() {
            return Err(err("empty name"));
        }
        if !(1..=MAX_PERIOD_DAYS).contains(&moon.period_days) {
            return Err(err(&format!(
                "`period_days` must be 1 to {MAX_PERIOD_DAYS}"
            )));
        }
        if moon.phase_offset_days >= moon.period_days {
            return Err(err("`phase_offset_days` must be less than `period_days`"));
        }
        if !(1..=MAX_SIZE).contains(&moon.size) {
            return Err(err(&format!("`size` must be 1 to {MAX_SIZE}")));
        }
    }
    Ok(parsed.moon)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = r#"
[[moon]]
name = "Moon"
color = [48, 28, 66, 0]
sky = [11, 7, 15, 0]
period_days = 30
"#;

    #[test]
    fn parses_moons_with_defaults() {
        let moons = parse("sky", ONE).expect("valid");
        assert_eq!(moons.len(), 1);
        let moon = &moons[0];
        assert_eq!((moon.size, moon.phase_offset_days), (STANDARD_SIZE, 0));
        assert_eq!(moon.color, Light::new(48, 28, 66, 0));
        assert!(parse("sky", "").expect("no moons").is_empty());
    }

    #[test]
    fn rejects_bad_moons() {
        for (bad, why) in [
            (ONE.replace("30", "0"), "period_days"),
            (ONE.replace("30", "3601"), "period_days"),
            (
                format!("{ONE}phase_offset_days = 30\n"),
                "phase_offset_days",
            ),
            (format!("{ONE}size = 0\n"), "size"),
            (ONE.replace("\"Moon\"", "\" \""), "empty name"),
            (format!("{ONE}tides = 2\n"), "unknown field"),
            (ONE.repeat(MAX_MOONS + 1), "at most"),
        ] {
            let e = parse("sky", &bad).expect_err(why);
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn the_phase_runs_through_one_cycle_per_period() {
        let moons = parse("sky", ONE).expect("valid");
        let moon = &moons[0];
        let at = |month, day, hour| GameTime::from_calendar(1, month, day, hour, 0).expect("valid");
        assert_eq!(moon.phase_at(at(1, 1, 0)), 0, "new at the epoch");
        assert_eq!(
            moon.phase_at(at(1, 16, 0)),
            PHASE_ONE / 2,
            "full on the 16th"
        );
        assert_eq!(moon.phase_at(at(1, 8, 12)), PHASE_ONE / 4, "first quarter");
        assert_eq!(
            moon.phase_at(at(4, 1, 0)),
            0,
            "a 30-day moon follows the month"
        );
        let mut last = 0;
        for day in 1..=30 {
            let phase = moon.phase_at(at(2, day, 6));
            assert!(phase > last || day == 1, "waxes through the month");
            last = phase;
        }
        let offset = MoonDef {
            phase_offset_days: 15,
            ..moon.clone()
        };
        assert_eq!(
            offset.phase_at(at(1, 1, 0)),
            PHASE_ONE / 2,
            "full at the epoch"
        );
        let week = MoonDef {
            period_days: 7,
            ..moon.clone()
        };
        assert_eq!(week.phase_at(at(1, 8, 0)), 0);
        assert_eq!(week.phase_at(at(1, 4, 12)), PHASE_ONE / 2);
    }
}
