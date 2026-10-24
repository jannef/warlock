//! Diseases (`assets/data/diseases.toml`, spec §8 "Diseases"): the pathologies behind the one
//! status a sufferer shows, "Diseased" (the file's `status`, a status in `data/status.toml`).
//!
//! A disease is data, modelled on D&D 3.5's diseases: how hard it is to resist (its
//! `difficulty`, against a check of its `check` stat), how long it incubates (dice, in days),
//! how often it progresses (`interval` days) and what a failed check does then (stat damage,
//! hit points), how it is shaken off (`recovery`: checks won in a row; 0 never by itself),
//! whether it can kill, how it slows healing (`regen`), how it is caught besides creatures'
//! `on_hit` (named sources, [`SOURCES`]; terrain waded through, [`Wading`]), what cures it, and
//! its words. The rules that apply them live in `crate::disease` and `world/disease.rs`.
//!
//! Without the file there are no diseases.

use super::{ContentError, StatusId, TerrainId};
use crate::dice::Dice;
use crate::stats::{PerStat, SCORES, Stat};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The ways a disease may be caught that rules name (`caught`): eating a rotten corpse (hunger,
/// #137, exposes the eater).
pub const SOURCES: [&str; 1] = ["rotten_food"];

/// The difficulties a disease may have: what a check (2d12 plus a modifier) must reach.
pub const DIFFICULTIES: std::ops::RangeInclusive<i32> = 1..=50;

/// The longest incubation, and the longest interval between progressions, in days.
pub const MAX_DAYS: i32 = 365;

/// The most hit points one failed check may cost.
pub const MAX_HP: i32 = 1000;

/// Index of a disease in [`super::Content::disease_defs`], in file order.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, Deserialize,
)]
pub struct DiseaseId(pub u16);

/// The terrain a disease is caught in (`wading = { terrain = ["marsh"], per_hour = 4 }`): a
/// chance in percent for every hour spent crossing it, drawn as the player steps onto it by how
/// long the step takes; then a check against the disease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wading {
    pub terrain: Vec<TerrainId>,
    /// The chance in percent per hour (1 to 100).
    pub per_hour: u8,
}

/// A disease, in file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiseaseDef {
    pub id: String,
    /// As the character screen names it once the sufferer knows it: `filth fever`.
    pub name: String,
    /// What a check must reach to resist it (to not catch it, and to win a progression).
    pub difficulty: i32,
    /// The stat whose modifier the checks add: Toughness unless it says otherwise.
    pub check: Stat,
    /// Days from catching it to its first signs (at least 1).
    pub incubation: Dice,
    /// Days between its progressions.
    pub interval: u32,
    /// What a failed check does to each stat (points of stat damage; fixed 0 for none).
    pub damage: PerStat<Dice>,
    /// Hit points a failed check costs, if any.
    pub hp: Option<Dice>,
    /// Checks won in a row that shake it off; 0: never by itself, only a cure.
    pub recovery: u8,
    /// Whether it can kill: its hit point damage may take the last one, and its Toughness
    /// damage may take Toughness to 0. Otherwise it leaves at least 1 of each.
    pub kills: bool,
    /// How fast hit points come back while it shows, in percent of normal, if it slows them
    /// more than its status does (the slowest counts).
    pub regen: Option<u16>,
    /// How fast mana comes back while it shows, the same way (spec §8 "Mana").
    pub mana_regen: Option<u16>,
    /// The ways rules may expose someone to it ([`SOURCES`]): `rotten_food`.
    pub caught: Vec<String>,
    /// The terrain it is caught in, if any.
    pub wading: Option<Wading>,
    /// The means that cure it at once ([`crate::World`]'s cure takes one of these).
    pub cured_by: Vec<String>,
    /// Said when it shows: "You break out in a fever."
    pub onset: String,
    /// Said when a progression goes badly, in turn: the first progression's, the second's, ...
    pub symptoms: Vec<String>,
    /// Said when it is gone: "The fever breaks."
    pub recovered: String,
}

impl DiseaseDef {
    /// The incubation and the interval in seconds: the longest time between its turns.
    pub fn longest_wait(&self) -> u64 {
        let days = self.incubation.max().max(self.interval as i32);
        u64::try_from(days).unwrap_or(0) * crate::time::DAY
    }

    /// The interval between its progressions in seconds.
    pub fn interval_seconds(&self) -> u64 {
        u64::from(self.interval) * crate::time::DAY
    }

    /// What it says at its progression number `n` (1 for the first) when the check fails: its
    /// symptoms in turn, from the first again after the last.
    pub fn symptom(&self, n: u16) -> &str {
        let i = usize::from(n.max(1) - 1) % self.symptoms.len().max(1);
        self.symptoms.get(i).map_or("", String::as_str)
    }

    /// Whether it ever recovers by itself.
    pub fn recovers(&self) -> bool {
        self.recovery > 0
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiseaseFile {
    status: Option<String>,
    #[serde(default)]
    disease: Vec<DiseaseSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiseaseSpec {
    id: String,
    name: String,
    difficulty: i32,
    #[serde(default = "toughness")]
    check: Stat,
    incubation: Dice,
    #[serde(default = "one_day")]
    interval: u32,
    #[serde(default)]
    damage: BTreeMap<Stat, Dice>,
    hp: Option<Dice>,
    #[serde(default = "two_in_a_row")]
    recovery: u8,
    #[serde(default)]
    kills: bool,
    regen: Option<u16>,
    mana_regen: Option<u16>,
    #[serde(default)]
    caught: Vec<String>,
    wading: Option<WadingSpec>,
    #[serde(default)]
    cured_by: Vec<String>,
    onset: String,
    symptoms: Vec<String>,
    recovered: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WadingSpec {
    terrain: Vec<String>,
    per_hour: u8,
}

fn toughness() -> Stat {
    Stat::Toughness
}

fn one_day() -> u32 {
    1
}

fn two_in_a_row() -> u8 {
    2
}

/// Parses `data/diseases.toml`: the status diseases show as (`status`, resolved by
/// `status_id`; required with any disease) and the diseases in file order (their `wading`
/// terrain resolved by `terrain_id`).
pub(super) fn parse(
    file: &str,
    text: &str,
    status_id: impl Fn(&str) -> Option<StatusId>,
    terrain_id: impl Fn(&str) -> Option<TerrainId>,
) -> Result<(Option<StatusId>, Vec<DiseaseDef>), ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: DiseaseFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let status = match &parsed.status {
        Some(id) => {
            Some(status_id(id).ok_or_else(|| err(format!("status: unknown status `{id}`")))?)
        }
        None if parsed.disease.is_empty() => None,
        None => {
            return Err(err(
                "`status` (the status diseases show as, in status.toml) is required".into(),
            ));
        }
    };
    let mut list: Vec<DiseaseDef> = Vec::new();
    for spec in parsed.disease {
        let id = spec.id.clone();
        if list.iter().any(|d| d.id == id) {
            return Err(err(format!("duplicate disease id `{id}`")));
        }
        let def = resolve(spec, &terrain_id).map_err(|m| err(format!("disease `{id}`: {m}")))?;
        list.push(def);
    }
    Ok((status, list))
}

fn resolve(
    spec: DiseaseSpec,
    terrain_id: &impl Fn(&str) -> Option<TerrainId>,
) -> Result<DiseaseDef, String> {
    if spec.id.trim().is_empty() || spec.name.trim().is_empty() {
        return Err("`id` and `name` must not be empty".into());
    }
    if !DIFFICULTIES.contains(&spec.difficulty) {
        return Err(format!(
            "difficulty must be {} to {}",
            DIFFICULTIES.start(),
            DIFFICULTIES.end()
        ));
    }
    let incubation = spec.incubation;
    if incubation.min() < 1 || incubation.max() > MAX_DAYS {
        return Err(format!(
            "incubation must be 1 to {MAX_DAYS} days, whatever it rolls"
        ));
    }
    if !(1..=MAX_DAYS as u32).contains(&spec.interval) {
        return Err(format!("interval must be 1 to {MAX_DAYS} days"));
    }
    let mut damage = PerStat::splat(Dice::fixed(0));
    let most = i32::from(*SCORES.end());
    for (stat, dice) in spec.damage {
        if dice.min() < 1 || dice.max() > most {
            return Err(format!(
                "damage: {} {dice} must do 1 to {most} points, whatever it rolls",
                stat.id()
            ));
        }
        damage[stat] = dice;
    }
    if let Some(hp) = spec.hp
        && (hp.min() < 1 || hp.max() > MAX_HP)
    {
        return Err(format!(
            "hp: {hp} must cost 1 to {MAX_HP} hit points, whatever it rolls"
        ));
    }
    if damage.iter().all(|(_, d)| d.is_zero()) && spec.hp.is_none() {
        return Err("a failed check must do something: `damage` or `hp`".into());
    }
    if spec.regen.is_some_and(|r| r > super::status::MAX_REGEN) {
        return Err(format!(
            "regen must be 0 to {} (percent)",
            super::status::MAX_REGEN
        ));
    }
    if spec
        .mana_regen
        .is_some_and(|r| r > super::status::MAX_REGEN)
    {
        return Err(format!(
            "mana_regen must be 0 to {} (percent)",
            super::status::MAX_REGEN
        ));
    }
    if let Some(bad) = spec.caught.iter().find(|c| !SOURCES.contains(&c.as_str())) {
        return Err(format!(
            "caught: unknown source `{bad}` (known: {})",
            SOURCES.join(", ")
        ));
    }
    let wading = spec
        .wading
        .map(|w| {
            if !(1..=100).contains(&w.per_hour) {
                return Err("wading: per_hour must be 1 to 100 (percent)".to_string());
            }
            if w.terrain.is_empty() {
                return Err("wading: name at least one terrain".to_string());
            }
            let terrain = (w.terrain.iter())
                .map(|t| terrain_id(t).ok_or_else(|| format!("wading: unknown terrain `{t}`")))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Wading {
                terrain,
                per_hour: w.per_hour,
            })
        })
        .transpose()?;
    if spec.cured_by.iter().any(|c| c.trim().is_empty()) {
        return Err("cured_by: a cure must have a name".into());
    }
    let blank = |s: &String| s.trim().is_empty();
    if blank(&spec.onset)
        || blank(&spec.recovered)
        || spec.symptoms.is_empty()
        || spec.symptoms.iter().any(blank)
    {
        return Err("`onset`, `recovered`, and at least one symptom must have words".into());
    }
    Ok(DiseaseDef {
        id: spec.id,
        name: spec.name,
        difficulty: spec.difficulty,
        check: spec.check,
        incubation,
        interval: spec.interval,
        damage,
        hp: spec.hp,
        recovery: spec.recovery,
        kills: spec.kills,
        regen: spec.regen,
        mana_regen: spec.mana_regen,
        caught: spec.caught,
        wading,
        cured_by: spec.cured_by,
        onset: spec.onset,
        symptoms: spec.symptoms,
        recovered: spec.recovered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEVER: &str = "status = \"diseased\"\n[[disease]]\nid = \"fever\"\nname = \"fever\"\n\
                         difficulty = 12\nincubation = \"1d3\"\n\
                         damage = { dexterity = \"1d3\", toughness = \"1d3\" }\n\
                         onset = \"You feel hot.\"\nsymptoms = [\"You shiver.\"]\n\
                         recovered = \"The fever breaks.\"\n";

    fn statuses(id: &str) -> Option<StatusId> {
        (id == "diseased").then_some(StatusId(2))
    }

    fn terrain(id: &str) -> Option<TerrainId> {
        (id == "marsh").then_some(TerrainId(7))
    }

    fn parsed(text: &str) -> Result<(Option<StatusId>, Vec<DiseaseDef>), ContentError> {
        parse("diseases.toml", text, statuses, terrain)
    }

    #[test]
    fn diseases_parse_with_defaults() {
        let text = format!(
            "{FEVER}[[disease]]\nid = \"ague\"\nname = \"ague\"\ndifficulty = 13\n\
             check = \"willpower\"\nincubation = 2\ninterval = 3\nhp = \"1d4\"\nrecovery = 0\n\
             kills = true\nregen = 25\ncaught = [\"rotten_food\"]\n\
             wading = {{ terrain = [\"marsh\"], per_hour = 4 }}\ncured_by = [\"remove_disease\"]\n\
             onset = \"a\"\nsymptoms = [\"b\", \"c\"]\nrecovered = \"d\"\n"
        );
        let (status, list) = parsed(&text).expect("valid");
        assert_eq!(status, Some(StatusId(2)));
        let (fever, ague) = (&list[0], &list[1]);
        assert_eq!(
            (fever.check, fever.interval, fever.recovery, fever.kills),
            (Stat::Toughness, 1, 2, false),
            "the defaults"
        );
        assert_eq!(fever.damage[Stat::Dexterity].to_string(), "1d3");
        assert!(fever.damage[Stat::Strength].is_zero());
        assert_eq!(
            (fever.hp, fever.regen, fever.wading.as_ref()),
            (None, None, None)
        );
        assert_eq!(fever.longest_wait(), 3 * crate::time::DAY);
        assert_eq!(ague.check, Stat::Willpower);
        assert_eq!(ague.longest_wait(), 3 * crate::time::DAY, "the interval");
        assert!(!ague.recovers() && ague.kills);
        assert_eq!(
            ague.wading,
            Some(Wading {
                terrain: vec![TerrainId(7)],
                per_hour: 4
            })
        );
        // Symptoms in turn, from the first again.
        assert_eq!(
            [1, 2, 3].map(|n| ague.symptom(n)),
            ["b", "c", "b"],
            "in turn"
        );
        // Without diseases: none, and no status needed.
        assert_eq!(parsed("").expect("empty is valid"), (None, Vec::new()));
    }

    #[test]
    fn bad_diseases_are_refused() {
        for (text, why) in [
            (
                format!("{FEVER}{}", &FEVER[20..]),
                "duplicate disease id `fever`",
            ),
            (FEVER.replace("status = \"diseased\"\n", ""), "`status`"),
            (
                FEVER.replace("\"diseased\"", "\"sick\""),
                "unknown status `sick`",
            ),
            (
                FEVER.replace("difficulty = 12", "difficulty = 0"),
                "difficulty must be 1 to 50",
            ),
            (
                FEVER.replace("\"1d3\"\n", "\"1d3-1\"\n"),
                "incubation must be 1 to 365 days",
            ),
            (
                FEVER.replace("toughness = \"1d3\"", "toughness = \"1d3-1\""),
                "damage: toughness",
            ),
            (
                FEVER.replace("damage = {", "hp = 0\ndamage = {"),
                "hp: 0 must cost",
            ),
            (
                FEVER.replace(
                    "damage = { dexterity = \"1d3\", toughness = \"1d3\" }",
                    "damage = {}",
                ),
                "must do something",
            ),
            (format!("{FEVER}interval = 0\n"), "interval must be"),
            (format!("{FEVER}regen = 2000\n"), "regen must be 0 to 1000"),
            (
                format!("{FEVER}caught = [\"water\"]\n"),
                "unknown source `water`",
            ),
            (
                format!("{FEVER}wading = {{ terrain = [\"bog\"], per_hour = 2 }}\n"),
                "unknown terrain `bog`",
            ),
            (
                format!("{FEVER}wading = {{ terrain = [\"marsh\"], per_hour = 0 }}\n"),
                "per_hour must be 1 to 100",
            ),
            (
                format!("{FEVER}wading = {{ terrain = [], per_hour = 1 }}\n"),
                "at least one terrain",
            ),
            (
                format!("{FEVER}cured_by = [\" \"]\n"),
                "a cure must have a name",
            ),
            (
                FEVER.replace("[\"You shiver.\"]", "[]"),
                "at least one symptom",
            ),
            (format!("{FEVER}check = \"luck\"\n"), "unknown stat `luck`"),
            (format!("{FEVER}colour = 1\n"), "unknown field `colour`"),
        ] {
            let e = parsed(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn mana_paces_parse_and_are_checked() {
        let (_, list) = parsed(&format!("{FEVER}mana_regen = 25\n")).expect("valid");
        assert_eq!(list[0].mana_regen, Some(25));
        assert_eq!(
            parsed(FEVER).expect("valid").1[0].mana_regen,
            None,
            "as the status says"
        );
        let e = parsed(&format!("{FEVER}mana_regen = 2000\n")).unwrap_err();
        assert!(
            e.message.contains("mana_regen must be 0 to 1000"),
            "{}",
            e.message
        );
    }
}
