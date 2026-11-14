//! Hunger (`assets/data/hunger.toml`, spec §8 "Hunger"): how fast the player burns nutrition, the
//! least nutrition of each state, what the states do, overeating, what rotten food does, and
//! corpses (what they give by their creature's size, and how long they keep).
//!
//! The file is optional: without it nobody hungers, nothing can be eaten, food does not rot, and
//! creatures leave no corpses. The rules are in [`crate::hunger`], [`crate::rot`], and
//! `world/hunger.rs`.

use super::{ActionKind, ContentError, CorpseGifts};
use crate::dice::Dice;
use crate::hunger::Hunger;
use crate::rot::Stage;
use crate::stats::{PerStat, Stat};
use crate::time::{DAY, Duration, HOUR};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// The most nutrition anything gives, or anything may be set to, either way.
pub const MAX_NUTRITION: i32 = 100_000;
/// The actions whose exertion burns extra nutrition: fighting.
pub const EXERTIONS: [ActionKind; 3] = [ActionKind::Melee, ActionKind::Shoot, ActionKind::Throw];

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
    freshness: Freshness,
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
    #[serde(default)]
    by_tag: BTreeMap<String, CorpseGifts>,
    #[serde(default)]
    skills: BTreeMap<String, SkillNews>,
}

/// What the eater is told when a corpse teaches a skill (`[corpses.skills]`): `learn` the first
/// time (they were untrained), `rise` when it adds to it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillNews {
    pub learn: String,
    pub rise: String,
}

/// What eating rotten food does (spec §8 "Hunger"), whatever the food: the chance in percent that
/// it comes back up, else the degrees of sickness it gives (and it feeds all the same).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rotten {
    pub vomit: u8,
    pub harm: Dice,
}

/// Telling food's freshness by looking (`world/freshness.rs`): what rotted food adds to the
/// chance, and the most the chance is, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Freshness {
    pub rotten_ease: i32,
    pub most: i32,
}

/// What something edible gives: its nutrition, and how many turns of eating it takes (each the
/// `eat` action); and its shelf life: how long after it was made it keeps (rotten from then on)
/// and how long it lasts (rotted away by then). Food objects have it (`food` in
/// `data/objects/`), corpses by their size (`data/hunger.toml`).
///
/// `None` is never: the corpse of an `everlasting` creature (`Content::load_hunger`), which
/// stays fresh for ever. Data cannot say never: a shelf life read from a file is always some.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Food {
    pub nutrition: u32,
    pub turns: u32,
    /// How long it stays good to eat: rotten from then on (old past half of it); `None`: for ever.
    #[serde(deserialize_with = "shelf_life")]
    pub keeps: Option<Duration>,
    /// How long it lasts before it has rotted away altogether; `None`: for ever.
    #[serde(deserialize_with = "shelf_life")]
    pub lasts: Option<Duration>,
}

impl Food {
    /// Checks the nutrition (at most [`MAX_NUTRITION`]: the rules count it in an `i32`) and the
    /// turns (at least 1: a meal is eaten turn by turn).
    pub(super) fn check(&self) -> Result<(), String> {
        if self.nutrition > MAX_NUTRITION as u32 {
            return Err(format!("`nutrition` must be at most {MAX_NUTRITION}"));
        }
        if self.turns == 0 {
            return Err("`turns` must be at least 1".into());
        }
        Ok(())
    }

    /// How food of this shelf life is at `age` (spec §8 "Hunger"): old past half of what it
    /// keeps, rotten past it, gone once it has lasted.
    /// Food that keeps for ever is always fresh.
    pub fn stage(&self, age: Duration) -> Stage {
        if self.lasts.is_some_and(|lasts| age >= lasts) {
            Stage::Gone
        } else if let Some(keeps) = self.keeps {
            if age >= keeps {
                Stage::Rotten
            } else if age.as_seconds().saturating_mul(2) >= keeps.as_seconds() {
                Stage::Old
            } else {
                Stage::Fresh
            }
        } else {
            Stage::Fresh
        }
    }
}

/// A shelf life in data: whole hours or days, `"8h"`, `"30d"`.
fn shelf_life<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Duration>, D::Error> {
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
    n.checked_mul(scale)
        .map(|s| Some(Duration::seconds(s)))
        .ok_or_else(bad)
}

/// Corpses (spec §8 "Hunger"): what they are made of, and what each size of creature gives and
/// how long it keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Corpses {
    /// A material id in `data/materials.toml`, resolved when objects are made.
    pub material: String,
    /// What a corpse gives, by the index of its creature's size in [`super::Content::sizes`].
    pub sizes: Vec<Food>,
    /// What a corpse gives the stats by default, by creature tag (`[corpses.by_tag]`): laid
    /// under each creature's `on_eat` at load (`OnEat::resolve`).
    pub by_tag: BTreeMap<String, CorpseGifts>,
    /// The words of the skills a corpse may teach, by skill id (`[corpses.skills]`): a
    /// creature's `on_eat` `skills` may name only these.
    pub skills: BTreeMap<String, SkillNews>,
}

impl Corpses {
    /// What a creature of `tags` gives the stats by its tags alone: the largest per stat.
    pub fn gifts_of(&self, tags: &[String]) -> CorpseGifts {
        let mut gifts = CorpseGifts::default();
        for gift in tags.iter().filter_map(|t| self.by_tag.get(t)) {
            gifts.merge(gift);
        }
        gifts
    }
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
    /// What harm from eating is (spec §8 "Status effects"): a status id in `data/status/`
    /// (checked when content loads), `poisoned`; rotten food's `harm` and a creature's `on_eat`
    /// `harm` are degrees of it. `None`: nothing eaten harms.
    pub sickness: Option<String>,
    /// What eating rotten food does.
    pub rotten: Rotten,
    /// Telling freshness by looking.
    pub freshness: Freshness,
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
        // A point burned every 0 seconds would never let time pass.
        if parsed.burn_seconds == 0 {
            return Err(err("burn_seconds must be at least 1".into()));
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
        if let Some(w) = order.windows(2).find(|w| w[0].1 <= w[1].1) {
            return Err(err(format!(
                "states: {} ({}) must be more than {} ({}): max, bloated, satiated, not_hungry, \
                 hungry, weak, fainting, from the most down",
                w[0].0, w[0].1, w[1].0, w[1].1
            )));
        }
        // A price of 0 percent would let the bloated act without end in no time.
        if parsed.bloated.slow == 0 {
            return Err(err("bloated: slow must be above 0 (percent)".into()));
        }
        let mut weak = PerStat::splat(0);
        for (&stat, &by) in &parsed.weak.stats {
            weak[stat] = i32::from(by);
        }
        let f = parsed.fainting;
        let st = parsed.starving;
        if st.seconds == 0 {
            return Err(err("starving: seconds must be at least 1".into()));
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
            exertion.insert(kind, points);
        }
        let o = parsed.overeating;
        if !(0..=100).contains(&parsed.freshness.most) {
            return Err(err("[freshness]: `most` must be 0 to 100".into()));
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
            freshness: parsed.freshness,
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
            by_tag: spec.by_tag,
            skills: spec.skills,
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
                (Some(Duration::hours(8)), Some(Duration::hours(36)))
            );
        }
    }

    #[test]
    fn food_is_old_past_half_its_keeping_then_rotten_then_gone() {
        let food = Food {
            nutrition: 800,
            turns: 3,
            keeps: Some(Duration::hours(5 * 24)),
            lasts: Some(Duration::hours(10 * 24)),
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
            keeps: Some(Duration::seconds(3601)),
            ..food
        };
        assert_eq!(odd.stage(Duration::seconds(1800)), Stage::Fresh);
        assert_eq!(odd.stage(Duration::seconds(1801)), Stage::Old);
        // Food that keeps for ever (an everlasting corpse) is fresh at any age.
        let never = Food {
            keeps: None,
            lasts: None,
            ..food
        };
        for hours in [0, 60, 240, 30 * 24, u32::MAX as u64] {
            assert_eq!(never.stage(Duration::hours(hours)), Stage::Fresh, "{hours}");
        }
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
            (Some(Duration::hours(8)), Some(Duration::hours(720)))
        );
        for bad in ["8", "h", "8m", "-8h", "8 h", "1.5d", "never"] {
            let e = food(bad, "30d").unwrap_err();
            assert!(e.contains("bad shelf life"), "{bad}: {e}");
        }
        assert!(food("1h", "2h").expect("parses").check().is_ok());
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
            ("slow = 125", "slow = 0", "slow must be above 0"),
            ("seconds = 600", "seconds = 0", "starving: seconds"),
            ("throw = 1", "wait = 1", "`wait` is no exertion"),
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
                "`turns` must be at least 1",
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
