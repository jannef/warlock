//! Hunger (`assets/data/hunger.toml`, spec §8 "Hunger"): how fast the player burns nutrition, the
//! least nutrition of each state, what the states do, overeating, and corpses (how they rot, and
//! what they give by their creature's size).
//!
//! The file is optional: without it nobody hungers, nothing can be eaten, and creatures leave no
//! corpses. The rules are in [`crate::hunger`] and `world/hunger.rs`.

use super::{ActionKind, ContentError};
use crate::dice::Dice;
use crate::hunger::Hunger;
use crate::stats::{PerStat, Stat};
use crate::time::{Duration, HOUR};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The most nutrition anything gives, or anything may be set to, either way.
pub const MAX_NUTRITION: i32 = 100_000;
/// The most turns one meal takes.
pub const MAX_MEAL: u32 = 100;
/// The longest a corpse lies before it is gone, in hours: a month.
pub const MAX_ROT_HOURS: u32 = 30 * 24;
/// The actions whose exertion burns extra nutrition: fighting.
pub const EXERTIONS: [ActionKind; 3] = [ActionKind::Melee, ActionKind::Shoot, ActionKind::Throw];
/// What a percent chance may be.
const PERCENT: RangeInclusive<u8> = 0..=100;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HungerFile {
    burn_seconds: u32,
    start: i32,
    max: i32,
    vomit_leaves: i32,
    states: States,
    bloated: Bloated,
    weak: Weak,
    fainting: Faint,
    starving: Starve,
    #[serde(default)]
    exertion: BTreeMap<String, u32>,
    overeating: Overeating,
    corpses: Option<CorpseSpec>,
}

/// The least nutrition of each state, from the most fed down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct States {
    pub bloated: i32,
    pub satiated: i32,
    pub not_hungry: i32,
    pub hungry: i32,
    pub weak: i32,
    pub fainting: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bloated {
    slow: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Weak {
    #[serde(default)]
    stats: BTreeMap<Stat, i8>,
}

/// Fainting (and Starving): the chance in percent each turn to faint, and the turns it costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faint {
    pub chance: u8,
    pub turns: u8,
}

/// Starving: `damage` hit points lost every `seconds` of game time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Starve {
    pub seconds: u32,
    pub damage: u16,
}

/// Overeating: the chances in percent to choke to death (0: never), else to vomit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overeating {
    pub choke: u8,
    pub vomit: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpseSpec {
    material: String,
    rotten_hours: u32,
    gone_hours: u32,
    rotten_vomit: u8,
    rotten_harm: Dice,
    size: BTreeMap<String, Food>,
}

/// What something edible gives: its nutrition, and how many turns of eating it takes (each the
/// `eat` action). Food objects have it (`food` in `data/objects.toml`), corpses by their size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Food {
    pub nutrition: u32,
    pub turns: u32,
}

impl Food {
    /// Checks the nutrition (1 to [`MAX_NUTRITION`]) and the turns (1 to [`MAX_MEAL`]).
    pub(super) fn check(&self) -> Result<(), String> {
        if self.nutrition == 0 || self.nutrition > MAX_NUTRITION as u32 {
            return Err(format!("`nutrition` must be 1 to {MAX_NUTRITION}"));
        }
        if !(1..=MAX_MEAL).contains(&self.turns) {
            return Err(format!("`turns` must be 1 to {MAX_MEAL}"));
        }
        Ok(())
    }
}

/// Corpses (spec §8 "Hunger"): what they are made of, how they rot, what eating a rotten one does,
/// and what each size of creature gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Corpses {
    /// A material id in `data/materials.toml`, resolved when objects are made.
    pub material: String,
    /// How long after the death a corpse turns rotten.
    pub rotten: Duration,
    /// How long after the death it is gone.
    pub gone: Duration,
    /// The chance in percent that a rotten corpse makes the eater vomit.
    pub rotten_vomit: u8,
    /// The hit points a rotten corpse that stays down costs.
    pub rotten_harm: Dice,
    /// What a corpse gives, by the index of its creature's size in [`super::Content::sizes`].
    pub sizes: Vec<Food>,
}

/// How the player hungers: `data/hunger.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HungerDef {
    /// Game seconds per nutrition point burned at 100 % hunger.
    pub burn_seconds: u32,
    /// Nutrition at the start of a run.
    pub start: i32,
    /// The most nutrition there can be.
    pub max: i32,
    /// The most nutrition left after vomiting.
    pub vomit_leaves: i32,
    pub states: States,
    /// Bloated: what an action's price becomes, in percent.
    pub bloated_slow: u16,
    /// Weak, Fainting, and Starving: what Strength and Dexterity lose (or gain).
    pub weak: PerStat<i32>,
    pub faint: Faint,
    pub starve: Starve,
    /// Extra points the player's exertions burn, by action (only [`EXERTIONS`]).
    pub exertion: BTreeMap<ActionKind, u32>,
    pub overeating: Overeating,
    /// `None`: creatures leave no corpses.
    pub corpses: Option<Corpses>,
}

impl HungerDef {
    /// The state `nutrition` puts the player in.
    pub fn state(&self, nutrition: i32) -> Hunger {
        let s = &self.states;
        match nutrition {
            n if n >= s.bloated => Hunger::Bloated,
            n if n >= s.satiated => Hunger::Satiated,
            n if n >= s.not_hungry => Hunger::NotHungry,
            n if n >= s.hungry => Hunger::Hungry,
            n if n >= s.weak => Hunger::Weak,
            n if n >= s.fainting => Hunger::Fainting,
            _ => Hunger::Starving,
        }
    }

    /// The least nutrition of `state`; `None` for Starving, which is everything below Fainting.
    pub fn least(&self, state: Hunger) -> Option<i32> {
        let s = &self.states;
        match state {
            Hunger::Bloated => Some(s.bloated),
            Hunger::Satiated => Some(s.satiated),
            Hunger::NotHungry => Some(s.not_hungry),
            Hunger::Hungry => Some(s.hungry),
            Hunger::Weak => Some(s.weak),
            Hunger::Fainting => Some(s.fainting),
            Hunger::Starving => None,
        }
    }

    /// The extra points exertion `kind` burns (0 for actions that do not exert).
    pub fn exertion(&self, kind: ActionKind) -> u32 {
        self.exertion.get(&kind).copied().unwrap_or(0)
    }

    /// Parses `data/hunger.toml`; `size_index` says where a size id is in
    /// [`super::Content::sizes`], of which there are `sizes`.
    pub(super) fn parse(
        file: &str,
        text: &str,
        sizes: usize,
        size_index: impl Fn(&str) -> Option<usize>,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: HungerFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !(1..=86_400).contains(&parsed.burn_seconds) {
            return Err(err("burn_seconds must be 1 to 86400".into()));
        }
        let s = parsed.states;
        let order = [
            ("max", parsed.max),
            ("bloated", s.bloated),
            ("satiated", s.satiated),
            ("not_hungry", s.not_hungry),
            ("hungry", s.hungry),
            ("weak", s.weak),
            ("fainting", s.fainting),
        ];
        if let Some((name, n)) = order.iter().find(|(_, n)| n.abs() > MAX_NUTRITION) {
            return Err(err(format!(
                "{name} {n} is out of range (±{MAX_NUTRITION})"
            )));
        }
        if let Some(w) = order.windows(2).find(|w| w[0].1 <= w[1].1) {
            return Err(err(format!(
                "states: {} ({}) must be more than {} ({}): max, bloated, satiated, not_hungry, \
                 hungry, weak, fainting, from the most down",
                w[0].0, w[0].1, w[1].0, w[1].1
            )));
        }
        for (name, n) in [
            ("start", parsed.start),
            ("vomit_leaves", parsed.vomit_leaves),
        ] {
            if !(s.fainting..=parsed.max).contains(&n) {
                return Err(err(format!(
                    "{name} {n} must be from fainting ({}) to max ({})",
                    s.fainting, parsed.max
                )));
            }
        }
        if !(100..=1000).contains(&parsed.bloated.slow) {
            return Err(err("bloated: slow must be 100 to 1000 (percent)".into()));
        }
        let mut weak = PerStat::splat(0);
        for (&stat, &by) in &parsed.weak.stats {
            if !matches!(stat, Stat::Strength | Stat::Dexterity) {
                return Err(err(format!(
                    "weak: only strength and dexterity count for less, not {}",
                    stat.id()
                )));
            }
            if !(-10..=10).contains(&by) {
                return Err(err(format!("weak: {} {by} must be -10 to 10", stat.id())));
            }
            weak[stat] = i32::from(by);
        }
        let f = parsed.fainting;
        if !PERCENT.contains(&f.chance) || !(1..=100).contains(&f.turns) {
            return Err(err(
                "fainting: chance must be 0 to 100 (percent) and turns 1 to 100".into(),
            ));
        }
        let st = parsed.starving;
        if !(1..=86_400).contains(&st.seconds) || !(1..=1000).contains(&st.damage) {
            return Err(err(
                "starving: seconds must be 1 to 86400 and damage 1 to 1000".into(),
            ));
        }
        let mut exertion = BTreeMap::new();
        for (id, points) in parsed.exertion {
            let kind = (EXERTIONS.into_iter())
                .find(|k| k.id() == id)
                .ok_or_else(|| {
                    let known: Vec<&str> = EXERTIONS.iter().map(|k| k.id()).collect();
                    err(format!(
                        "exertion: `{id}` is no exertion (expected {})",
                        known.join(", ")
                    ))
                })?;
            if points > 1000 {
                return Err(err(format!("exertion: {id} {points} must be 0 to 1000")));
            }
            exertion.insert(kind, points);
        }
        let o = parsed.overeating;
        if !PERCENT.contains(&o.choke) || !PERCENT.contains(&o.vomit) {
            return Err(err(
                "overeating: choke and vomit are chances in percent, 0 to 100".into(),
            ));
        }
        let corpses = (parsed.corpses)
            .map(|c| Corpses::resolve(c, sizes, &size_index))
            .transpose()
            .map_err(|m| err(format!("corpses: {m}")))?;
        Ok(Self {
            burn_seconds: parsed.burn_seconds,
            start: parsed.start,
            max: parsed.max,
            vomit_leaves: parsed.vomit_leaves,
            states: s,
            bloated_slow: parsed.bloated.slow,
            weak,
            faint: f,
            starve: st,
            exertion,
            overeating: o,
            corpses,
        })
    }
}

impl Corpses {
    fn resolve(
        spec: CorpseSpec,
        sizes: usize,
        size_index: impl Fn(&str) -> Option<usize>,
    ) -> Result<Self, String> {
        if spec.rotten_hours == 0
            || spec.gone_hours <= spec.rotten_hours
            || spec.gone_hours > MAX_ROT_HOURS
        {
            return Err(format!(
                "rotten_hours must be at least 1 and gone_hours more than it, at most \
                 {MAX_ROT_HOURS}"
            ));
        }
        if !PERCENT.contains(&spec.rotten_vomit) {
            return Err("rotten_vomit is a chance in percent, 0 to 100".into());
        }
        if spec.rotten_harm.min() < 0 || spec.rotten_harm.max() > 1000 {
            return Err(format!(
                "rotten_harm {} must roll 0 to 1000",
                spec.rotten_harm
            ));
        }
        let mut by_size: Vec<Option<Food>> = vec![None; sizes];
        for (id, food) in spec.size {
            let i = size_index(&id).ok_or_else(|| format!("size: unknown size `{id}`"))?;
            food.check().map_err(|m| format!("size {id}: {m}"))?;
            by_size[i] = Some(food);
        }
        let sizes = by_size
            .into_iter()
            .collect::<Option<Vec<Food>>>()
            .ok_or("size: every size in data/sizes.toml needs its nutrition and turns")?;
        let hours = |h: u32| Duration::seconds(u64::from(h) * HOUR);
        Ok(Self {
            material: spec.material,
            rotten: hours(spec.rotten_hours),
            gone: hours(spec.gone_hours),
            rotten_vomit: spec.rotten_vomit,
            rotten_harm: spec.rotten_harm,
            sizes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<HungerDef, ContentError> {
        let sizes = ["small", "medium"];
        HungerDef::parse("hunger.toml", text, 2, |s| {
            sizes.iter().position(|id| *id == s)
        })
    }

    fn builtin() -> String {
        crate::content::ContentFiles::builtin()
            .get("data/hunger.toml")
            .expect("the built-in hunger file")
            .to_string()
    }

    #[test]
    fn the_builtin_file_reads_states_from_the_most_fed_down() {
        let def = parse(&builtin()).expect("valid");
        assert_eq!((def.burn_seconds, def.start, def.max), (60, 3600, 6500));
        for (n, state) in [
            (6500, Hunger::Bloated),
            (5500, Hunger::Bloated),
            (5499, Hunger::Satiated),
            (3600, Hunger::NotHungry),
            (1439, Hunger::Hungry),
            (719, Hunger::Weak),
            (0, Hunger::Fainting),
            (-1, Hunger::Starving),
        ] {
            assert_eq!(def.state(n), state, "{n}");
        }
        assert_eq!(def.least(Hunger::Hungry), Some(720));
        assert_eq!(def.least(Hunger::Starving), None);
        assert_eq!(def.weak[Stat::Strength], -1);
        assert_eq!(def.weak[Stat::Dexterity], -1);
        assert_eq!(def.weak[Stat::Toughness], 0);
        assert_eq!(def.exertion(ActionKind::Melee), 2);
        assert_eq!(def.exertion(ActionKind::Wait), 0);
        assert_eq!(def.overeating.choke, 0, "choking to death is off");
        let corpses = def.corpses.expect("corpses");
        assert_eq!(corpses.rotten, Duration::hours(8));
        assert_eq!(corpses.gone, Duration::hours(36));
        assert_eq!(corpses.sizes[0].nutrition, 400);
        assert_eq!(corpses.sizes[1].turns, 6);
    }

    #[test]
    fn bad_files_are_refused() {
        let good = builtin();
        for (from, to, why) in [
            (
                "satiated = 4000",
                "satiated = 5500",
                "bloated (5500) must be more",
            ),
            ("max = 6500", "max = 4000", "max (4000) must be more"),
            (
                "start = 3600",
                "start = 7000",
                "start 7000 must be from fainting",
            ),
            ("slow = 125", "slow = 99", "slow must be 100 to 1000"),
            (
                "dexterity = -1 }",
                "dexterity = -1, toughness = -1 }",
                "not toughness",
            ),
            ("chance = 5", "chance = 101", "fainting: chance"),
            ("damage = 1", "damage = 0", "starving: seconds"),
            ("throw = 1", "wait = 1", "`wait` is no exertion"),
            ("choke = 0", "choke = 200", "overeating"),
            (
                "gone_hours = 36",
                "gone_hours = 8",
                "gone_hours more than it",
            ),
            (
                "rotten_harm = \"1d4\"",
                "rotten_harm = \"-1d4\"",
                "must roll 0",
            ),
            (
                "medium = { nutrition = 1000, turns = 6 }",
                "",
                "every size in data/sizes.toml",
            ),
            (
                "medium = { nutrition = 1000, turns = 6 }",
                "huge = { nutrition = 1000, turns = 6 }",
                "unknown size `huge`",
            ),
            (
                "small = { nutrition = 400, turns = 3 }",
                "small = { nutrition = 400, turns = 0 }",
                "`turns` must be 1",
            ),
            ("burn_seconds = 60", "burn_seconds = 0", "burn_seconds"),
        ] {
            assert!(good.contains(from), "{from}");
            let e = parse(&good.replace(from, to)).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        // Without corpses, creatures leave none.
        let at = good.find("[corpses]").expect("corpses");
        let def = parse(&good[..at]).expect("valid without corpses");
        assert!(def.corpses.is_none());
    }
}
