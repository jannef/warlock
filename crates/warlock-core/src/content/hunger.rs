//! Hunger (`assets/data/hunger.toml`, spec §8 "Hunger"): how fast the player burns nutrition, the
//! least nutrition of each state, what the states do, overeating, what rotten food does, and
//! corpses (what they give by their creature's size, and how long they keep).
//!
//! The file is optional: without it nobody hungers, nothing can be eaten, food does not rot, and
//! creatures leave no corpses. The rules are in [`crate::hunger`], [`crate::rot`], and
//! `world/hunger.rs`.

use super::{ActionKind, ContentError};
use crate::dice::Dice;
use crate::hunger::Hunger;
use crate::rot::Stage;
use crate::stats::{PerStat, Stat};
use crate::time::{DAY, Duration, HOUR};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The most nutrition anything gives, or anything may be set to, either way.
pub const MAX_NUTRITION: i32 = 100_000;
/// The most turns one meal takes.
pub const MAX_MEAL: u32 = 100;
/// The longest anything edible lasts before it has rotted away: ten years.
pub const MAX_SHELF_LIFE: Duration = Duration::seconds(3600 * DAY);
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
    #[serde(default)]
    sickness: Option<String>,
    rotten: Rotten,
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
    size: BTreeMap<String, Food>,
}

/// What eating rotten food does (spec §8 "Hunger"), whatever the food: the chance in percent that
/// it comes back up, else the degrees of sickness it gives (and it feeds all the same).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rotten {
    pub vomit: u8,
    pub harm: Dice,
}

/// What something edible gives: its nutrition, and how many turns of eating it takes (each the
/// `eat` action); and its shelf life: how long after it was made it keeps (rotten from then on)
/// and how long it lasts (rotted away by then). Food objects have it (`food` in
/// `data/objects/`), corpses by their size (`data/hunger.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Food {
    pub nutrition: u32,
    pub turns: u32,
    /// How long it stays good to eat: rotten from then on (old past half of it).
    #[serde(deserialize_with = "shelf_life")]
    pub keeps: Duration,
    /// How long it lasts before it has rotted away altogether.
    #[serde(deserialize_with = "shelf_life")]
    pub lasts: Duration,
}

impl Food {
    /// Checks the nutrition (1 to [`MAX_NUTRITION`]), the turns (1 to [`MAX_MEAL`]), and the
    /// shelf life (it keeps an hour or more, and lasts longer than it keeps, up to
    /// [`MAX_SHELF_LIFE`]).
    pub(super) fn check(&self) -> Result<(), String> {
        if self.nutrition == 0 || self.nutrition > MAX_NUTRITION as u32 {
            return Err(format!("`nutrition` must be 1 to {MAX_NUTRITION}"));
        }
        if !(1..=MAX_MEAL).contains(&self.turns) {
            return Err(format!("`turns` must be 1 to {MAX_MEAL}"));
        }
        if self.keeps < Duration::hours(1)
            || self.lasts <= self.keeps
            || self.lasts > MAX_SHELF_LIFE
        {
            return Err(format!(
                "`keeps` must be 1h or more and `lasts` longer than it, at most {}d",
                MAX_SHELF_LIFE.as_seconds() / DAY
            ));
        }
        Ok(())
    }

    /// How food of this shelf life is at `age` (spec §8 "Hunger"): old past half of what it
    /// keeps, rotten past it, gone once it has lasted.
    pub fn stage(&self, age: Duration) -> Stage {
        if age >= self.lasts {
            Stage::Gone
        } else if age >= self.keeps {
            Stage::Rotten
        } else if age.as_seconds().saturating_mul(2) >= self.keeps.as_seconds() {
            Stage::Old
        } else {
            Stage::Fresh
        }
    }
}

/// A shelf life in data: whole hours or days, `"8h"`, `"30d"`.
fn shelf_life<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let text = String::deserialize(d)?;
    let bad =
        || serde::de::Error::custom(format!("bad shelf life `{text}` (expected e.g. 8h, 30d)"));
    let (number, unit) = text.split_at(text.len().saturating_sub(1));
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad());
    }
    let n: u64 = number.parse().map_err(|_| bad())?;
    let scale = match unit {
        "h" => HOUR,
        "d" => DAY,
        _ => return Err(bad()),
    };
    n.checked_mul(scale).map(Duration::seconds).ok_or_else(bad)
}

/// Corpses (spec §8 "Hunger"): what they are made of, and what each size of creature gives and
/// how long it keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Corpses {
    /// A material id in `data/materials.toml`, resolved when objects are made.
    pub material: String,
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
    /// What harm from eating is (spec §8 "Status effects"): a status id in `data/status.toml`
    /// (checked when content loads), `poisoned`; rotten food's `harm` and a creature's `on_eat`
    /// `harm` are degrees of it. `None`: nothing eaten harms.
    pub sickness: Option<String>,
    /// What eating rotten food does.
    pub rotten: Rotten,
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
        let r = &parsed.rotten;
        if !PERCENT.contains(&r.vomit) {
            return Err(err("rotten: vomit is a chance in percent, 0 to 100".into()));
        }
        if r.harm.min() < 0 || r.harm.max() > 1000 {
            return Err(err(format!("rotten: harm {} must roll 0 to 1000", r.harm)));
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
            sickness: parsed.sickness,
            rotten: parsed.rotten,
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
        let mut by_size: Vec<Option<Food>> = vec![None; sizes];
        for (id, food) in spec.size {
            let i = size_index(&id).ok_or_else(|| format!("size: unknown size `{id}`"))?;
            food.check().map_err(|m| format!("size {id}: {m}"))?;
            by_size[i] = Some(food);
        }
        let sizes = by_size
            .into_iter()
            .collect::<Option<Vec<Food>>>()
            .ok_or("size: every size in data/sizes.toml needs what its corpses give")?;
        Ok(Self {
            material: spec.material,
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
        assert_eq!(def.sickness.as_deref(), Some("poisoned"));
        assert_eq!(
            (def.rotten.vomit, def.rotten.harm.to_string()),
            (50, "1d4".into())
        );
        let corpses = def.corpses.expect("corpses");
        assert_eq!(corpses.sizes[0].nutrition, 400);
        assert_eq!(corpses.sizes[1].turns, 6);
        // A corpse keeps 8 hours and is gone after 36.
        for food in &corpses.sizes {
            assert_eq!(
                (food.keeps, food.lasts),
                (Duration::hours(8), Duration::hours(36))
            );
        }
    }

    #[test]
    fn food_is_old_past_half_its_keeping_then_rotten_then_gone() {
        let food = Food {
            nutrition: 800,
            turns: 3,
            keeps: Duration::hours(5 * 24),
            lasts: Duration::hours(10 * 24),
        };
        for (hours, stage) in [
            (0, Stage::Fresh),
            (59, Stage::Fresh),
            (60, Stage::Old),
            (119, Stage::Old),
            (120, Stage::Rotten),
            (239, Stage::Rotten),
            (240, Stage::Gone),
            (10_000, Stage::Gone),
        ] {
            assert_eq!(food.stage(Duration::hours(hours)), stage, "{hours}");
        }
        // Old begins at half the keeping to the second (rounded up).
        let odd = Food {
            keeps: Duration::seconds(3601),
            ..food
        };
        assert_eq!(odd.stage(Duration::seconds(1800)), Stage::Fresh);
        assert_eq!(odd.stage(Duration::seconds(1801)), Stage::Old);
    }

    #[test]
    fn shelf_lives_are_hours_or_days() {
        let food = |keeps: &str, lasts: &str| {
            let text =
                format!("nutrition = 1\nturns = 1\nkeeps = \"{keeps}\"\nlasts = \"{lasts}\"\n");
            toml::from_str::<Food>(&text).map_err(|e| e.to_string())
        };
        let f = food("8h", "30d").expect("valid");
        assert_eq!(
            (f.keeps, f.lasts),
            (Duration::hours(8), Duration::hours(720))
        );
        for bad in ["8", "h", "8m", "-8h", "8 h", "1.5d"] {
            let e = food(bad, "30d").unwrap_err();
            assert!(e.contains("bad shelf life"), "{bad}: {e}");
        }
        let check = |keeps, lasts| food(keeps, lasts).expect("parses").check();
        assert!(check("1h", "2h").is_ok());
        assert!(
            check("3600d", "3600d").is_err(),
            "lasts longer than it keeps"
        );
        assert!(check("10d", "3601d").is_err(), "at most ten years");
        assert!(check("0h", "2h").is_err(), "keeps an hour or more");
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
                "turns = 6, keeps = \"8h\", lasts = \"36h\"",
                "turns = 6, keeps = \"8h\", lasts = \"8h\"",
                "size medium: `keeps` must be 1h or more and `lasts` longer than it",
            ),
            ("harm = \"1d4\"", "harm = \"-1d4\"", "must roll 0"),
            (
                "[rotten]\nvomit = 50",
                "[rotten]\nvomit = 150",
                "rotten: vomit",
            ),
            (
                "medium = { nutrition = 1000, turns = 6, keeps = \"8h\", lasts = \"36h\" }",
                "",
                "every size in data/sizes.toml",
            ),
            (
                "medium = { nutrition = 1000,",
                "huge = { nutrition = 1000,",
                "unknown size `huge`",
            ),
            (
                "small = { nutrition = 400, turns = 3,",
                "small = { nutrition = 400, turns = 0,",
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
