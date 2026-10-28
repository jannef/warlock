//! Mimics (spec §8 "Mimics"; docs/dev/creatures/mimics.md; the design:
//! docs/design/chests-and-treasure.md §5): `data/mimics.toml`, a generated family of creature
//! types, one a level, `mimic_<L>` (`mimic_1` … `mimic_60`), each an ordinary creature type every
//! rule reads as it reads a kobold. A mimic's level is the floor's item level, as a chest's; its
//! band (the last whose `from` it reaches) names it and matches the chest bands
//! (`data/chests.toml`), whose `hold` gives its slam's chance to hold. Its numbers are the
//! roster's formulas for a brute ([`Numbers::of`]). Generation draws nothing, so the ids, their
//! order, and the fingerprint are the same on every load.

use super::animated::Numbers;
use super::creature::CreatureSpec;
use super::{Content, ContentError, ContentFiles, CreatureId};
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The family file.
pub(super) const RULES: &str = "data/mimics.toml";

/// The prefix of a member's id (`mimic_7`).
pub const ID_PREFIX: &str = "mimic_";

/// The mimics as content defines them; empty (no levels) without `data/mimics.toml`.
#[derive(Clone, Debug, Default)]
pub struct Mimics {
    /// The family's id (`mimic`): what quests and first kills know every member by.
    pub family: String,
    /// The levels of its members, least and most; `[0, 0]`: no family.
    pub levels: [u8; 2],
    /// What it adds to the base stats.
    pub stats: BTreeMap<Stat, i32>,
    /// The factor on `base_hp` (2.5 × L × it).
    pub hp_factor: f64,
    /// Steps along the dice ladder from the curve's nearest.
    pub dice_steps: i32,
    /// Its hide's name, given from the level its Protection comes (the body rule's hide).
    pub hide: String,
    /// The fields every member shares, as written (`[creature]`).
    pub creature: toml::Table,
    /// The name bands, least first.
    pub bands: Vec<MimicBand>,
}

/// What a band's members are called and like.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MimicBand {
    pub id: String,
    pub name: String,
    /// The least level of the band: the matching chest band's.
    pub from: u8,
    /// Its colour (`"#rrggbb"`): its chest band's, so the disguise holds once revealed.
    pub fg: String,
    /// What a member weighs, in grams.
    pub weight: u32,
}

/// What makes a creature type a mimic (`CreatureDef::mimic`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MimicDef {
    /// The family's first type (`mimic_1`).
    pub family: CreatureId,
    /// Its place in the family, 0 for the first (its level less the family's least).
    pub member: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyFile {
    family: FamilySpec,
    creature: toml::Table,
    band: Vec<MimicBand>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilySpec {
    id: String,
    levels: [u8; 2],
    #[serde(default)]
    stats: BTreeMap<Stat, i32>,
    hp_factor: f64,
    #[serde(default)]
    dice_steps: i32,
    hide: String,
}

/// The fields a member's own numbers write: `[creature]` may not set them.
const OWN: &[&str] = &[
    "id", "name", "fg", "weight", "stats", "base_hp", "level", "xp", "natural", "plural",
];

impl Mimics {
    /// Reads and checks `data/mimics.toml` against the chest bands (loaded before); none
    /// without it.
    pub(super) fn parse(files: &ContentFiles, content: &Content) -> Result<Self, ContentError> {
        let Some(text) = files.get_optional(RULES) else {
            return Ok(Self::default());
        };
        let err = |m: String| ContentError::new(RULES, m);
        let file: FamilyFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let [from, to] = file.family.levels;
        if from == 0 || from > to || to > super::MAX_LEVEL {
            return Err(err(format!(
                "`levels` must be 1 to {} and least first",
                super::MAX_LEVEL
            )));
        }
        if !(0.0..=10.0).contains(&file.family.hp_factor) {
            return Err(err("`hp_factor` is 0 to 10".into()));
        }
        if let Some(key) = OWN.iter().find(|k| file.creature.contains_key(**k)) {
            return Err(err(format!(
                "[creature]: `{key}` is each member's own, not the family's"
            )));
        }
        let chests = &content.chests().bands;
        if file.band.len() != chests.len() {
            return Err(err(format!(
                "{} bands, but data/chests.toml has {}: a mimic's band is its chest's",
                file.band.len(),
                chests.len()
            )));
        }
        for (band, chest) in file.band.iter().zip(chests) {
            if band.from != chest.from {
                return Err(err(format!(
                    "band `{}`: `from` {} is not chest band `{}`'s {}",
                    band.id, band.from, chest.id, chest.from
                )));
            }
            if band.name.trim().is_empty() || band.weight == 0 {
                return Err(err(format!(
                    "band `{}`: a `name` and a `weight` are needed",
                    band.id
                )));
            }
        }
        Ok(Self {
            family: file.family.id,
            levels: file.family.levels,
            stats: file.family.stats,
            hp_factor: file.family.hp_factor,
            dice_steps: file.family.dice_steps,
            hide: file.family.hide,
            creature: file.creature,
            bands: file.band,
        })
    }

    /// Whether there is a family.
    pub fn is_empty(&self) -> bool {
        self.levels[0] == 0
    }

    /// The band of a member of `level`: the last whose `from` it reaches (the first below
    /// them all).
    pub fn band_index(&self, level: u8) -> Option<usize> {
        if self.bands.is_empty() {
            return None;
        }
        Some(
            self.bands
                .iter()
                .rposition(|b| b.from <= level)
                .unwrap_or(0),
        )
    }

    /// The numbers of the member of `level` (the roster's formulas).
    pub fn numbers(&self, level: u8) -> Numbers {
        Numbers::of(level, &self.stats, self.hp_factor, self.dice_steps)
    }

    /// The Protection of a member of `level`: the body rule's hide, (L − 10) ÷ 8, at least 0.
    pub fn protection(level: u8) -> i32 {
        ((i32::from(level) - 10) / 8).max(0)
    }

    /// The difficulty of a member's hold: a drain's, 12 + L ÷ 5.
    pub fn grip(level: u8) -> i32 {
        12 + i32::from(level) / 5
    }

    /// The family's id with its first member's (`mimic`, `mimic_1`), if there is a family.
    pub(super) fn first(&self) -> Option<(String, String)> {
        (!self.is_empty()).then(|| {
            (
                self.family.clone(),
                format!("{ID_PREFIX}{}", self.levels[0]),
            )
        })
    }

    /// The creature types the family generates, least first.
    pub(super) fn specs(&self, content: &Content) -> Result<Vec<CreatureSpec>, ContentError> {
        if self.is_empty() {
            return Ok(Vec::new());
        }
        (self.levels[0]..=self.levels[1])
            .map(|level| {
                self.member(level, content)
                    .map_err(|m| ContentError::new(RULES, format!("mimic at {level}: {m}")))
            })
            .collect()
    }

    /// The type of the member of `level`: `mimic_<level>`.
    fn member(&self, level: u8, content: &Content) -> Result<CreatureSpec, String> {
        let band = self.band_index(level).ok_or("no bands")?;
        let (named, chest) = (&self.bands[band], &content.chests().bands[band]);
        let numbers = self.numbers(level);
        let s = |v: &str| toml::Value::String(v.to_string());
        let int = |v: i64| toml::Value::Integer(v);
        let mut t = self.creature.clone();
        t.insert("id".into(), s(&format!("{ID_PREFIX}{level}")));
        t.insert("name".into(), s(&named.name));
        t.insert("fg".into(), s(&named.fg));
        t.insert("weight".into(), int(i64::from(named.weight)));
        let mut stats = toml::Table::new();
        for (stat, score) in &numbers.stats {
            stats.insert(stat.id().into(), int(i64::from(*score)));
        }
        t.insert("stats".into(), toml::Value::Table(stats));
        t.insert("base_hp".into(), int(i64::from(numbers.base_hp)));
        t.insert("level".into(), int(i64::from(level)));
        let xp = content.experience().guideline().xp(level);
        t.insert("xp".into(), int(i64::from(xp)));
        let protection = Self::protection(level);
        if protection > 0 {
            let mut hide = toml::Table::new();
            hide.insert("protection".into(), int(i64::from(protection)));
            hide.insert("name".into(), s(&self.hide));
            t.insert("natural".into(), toml::Value::Table(hide));
        }
        // The slam: the level's dice, sticky with the band's chance.
        let mut attack = match t.remove("attack") {
            Some(toml::Value::Table(a)) => a,
            _ => return Err("[creature] needs an `attack` table".into()),
        };
        attack.insert("damage".into(), s(numbers.dice));
        if chest.hold > 0 {
            let mut hold = toml::Table::new();
            hold.insert("status".into(), s("held"));
            hold.insert("chance".into(), int(i64::from(chest.hold)));
            hold.insert("difficulty".into(), int(i64::from(Self::grip(level))));
            attack.insert(
                "on_hit".into(),
                toml::Value::Array(vec![toml::Value::Table(hold)]),
            );
        }
        t.insert("attack".into(), toml::Value::Table(attack));
        let mut spec: CreatureSpec = toml::Value::Table(t)
            .try_into()
            .map_err(|e: toml::de::Error| e.to_string())?;
        spec.mimic = Some(MimicDef {
            family: CreatureId(0),
            member: level - self.levels[0],
        });
        Ok(spec)
    }
}

impl Content {
    /// The mimics (`data/mimics.toml`; spec §8 "Mimics").
    pub fn mimics(&self) -> &Mimics {
        &self.mimics
    }

    /// The mimic of `level`: the family's member of that level, the top member for a level
    /// beyond the family's, the least for one below; `None` without mimics.
    pub fn mimic_at(&self, level: u8) -> Option<CreatureId> {
        let m = &self.mimics;
        if m.is_empty() {
            return None;
        }
        let first = self.creature_id(&format!("{ID_PREFIX}{}", m.levels[0]))?;
        let member = level.clamp(m.levels[0], m.levels[1]) - m.levels[0];
        Some(CreatureId(first.0 + u16::from(member)))
    }

    /// Whether creature type `creature` is a mimic.
    pub fn is_mimic(&self, creature: CreatureId) -> bool {
        self.creature(creature).mimic.is_some()
    }
}

// Tests: `content/mimic_roster_tests.rs`.
