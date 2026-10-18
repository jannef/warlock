//! Scenarios (`assets/scenarios/*.toml`): a named starting setup for a run.

use super::ContentError;
use crate::stats::{Stat, Stats};
use crate::time::{GameTime, HOUR, MINUTE, parse_time_of_day};
use serde::Deserialize;
use std::collections::BTreeMap;

/// A scenario: which level a run starts on, the player's eyes, lantern, and stats, plus
/// defaults.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDef {
    /// File stem; filled in by the loader.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub start_level: String,
    /// Seed used when none is given on the command line.
    #[serde(default)]
    pub seed: u64,
    /// The player's starting vision profile (an id in `data/vision.toml`).
    #[serde(default = "default_vision")]
    pub vision: String,
    /// Light preset of the lantern the player carries (unlit at the start), if any.
    pub lantern: Option<String>,
    /// Starting date `[year, month, day]` (all from 1); default `[1444, 11, 11]`.
    #[serde(default = "default_date")]
    start_date: [u32; 3],
    /// Starting time of day, `hh:mm`; default `08:00`.
    #[serde(default = "default_time")]
    start_time: String,
    /// The clock at the start of a run, from `start_date` and `start_time`; filled in by the
    /// loader.
    #[serde(skip)]
    pub start: GameTime,
    /// The player's scores by stat (spec §8 "Stats"); a missing one is average.
    #[serde(default)]
    stats: BTreeMap<Stat, u8>,
    /// The player's potentials by stat; a missing one equals the score.
    #[serde(default)]
    potential: BTreeMap<Stat, u8>,
    /// The player's stats at the start of a run, from `stats` and `potential`; filled in by the
    /// loader.
    #[serde(skip)]
    pub player_stats: Stats,
}

fn default_vision() -> String {
    "normal".into()
}

fn default_date() -> [u32; 3] {
    [1444, 11, 11]
}

fn default_time() -> String {
    "08:00".into()
}

impl ScenarioDef {
    pub(super) fn parse(file: &str, id: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let mut def: ScenarioDef = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        def.id = id.to_string();
        let of_day =
            parse_time_of_day(&def.start_time).map_err(|m| err(format!("start_time: {m}")))?;
        let [year, month, day] = def.start_date;
        def.start = GameTime::from_calendar(
            u64::from(year),
            month,
            day,
            (of_day / HOUR) as u32,
            (of_day % HOUR / MINUTE) as u32,
        )
        .ok_or_else(|| {
            err(format!(
                "start_date {:?} is not a date (year from 1, month 1-12, day 1-30)",
                def.start_date
            ))
        })?;
        def.player_stats = Stats::from_data(&def.stats, &def.potential).map_err(err)?;
        Ok(def)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_defaults_to_eight_on_11_darkening_1444() {
        let def = ScenarioDef::parse("s", "s", "name = \"S\"\nstart_level = \"hub\"\n").unwrap();
        assert_eq!(def.start.to_string(), "1444-11-11 08:00:00");
    }

    #[test]
    fn start_date_and_time_are_read_and_validated() {
        let parse = |extra: &str| {
            ScenarioDef::parse(
                "s",
                "s",
                &format!("name = \"S\"\nstart_level = \"hub\"\n{extra}"),
            )
        };
        let def = parse("start_date = [2, 4, 12]\nstart_time = \"21:45\"").unwrap();
        assert_eq!(def.start.to_string(), "0002-04-12 21:45:00");
        for (bad, why) in [
            ("start_date = [1, 13, 1]", "not a date"),
            ("start_date = [0, 1, 1]", "not a date"),
            ("start_date = [1, 1, 31]", "not a date"),
            ("start_time = \"25:00\"", "start_time"),
        ] {
            let err = parse(bad).unwrap_err();
            assert!(err.message.contains(why), "{bad}: {}", err.message);
        }
    }

    #[test]
    fn the_player_stats_are_read_and_validated() {
        let parse = |extra: &str| {
            ScenarioDef::parse(
                "s",
                "s",
                &format!("name = \"S\"\nstart_level = \"hub\"\n{extra}"),
            )
        };
        assert_eq!(parse("").unwrap().player_stats, Stats::default());
        let def =
            parse("stats = { dexterity = 14 }\npotential = { dexterity = 17, presence = 12 }")
                .unwrap();
        let stats = def.player_stats;
        assert_eq!(stats.score(Stat::Dexterity), 14);
        assert_eq!(stats.potential(Stat::Dexterity), 17);
        assert_eq!(stats.score(Stat::Presence), 10);
        assert_eq!(stats.potential(Stat::Presence), 12);
        assert_eq!(stats.potential(Stat::Strength), 10);
        for (bad, why) in [
            ("stats = { strength = 0 }", "strength 0 is out of range"),
            ("potential = { willpower = 9 }", "below its score"),
            ("stats = { luck = 12 }", "unknown stat `luck`"),
        ] {
            let err = parse(bad).unwrap_err();
            assert!(err.message.contains(why), "{bad}: {}", err.message);
        }
    }
}
