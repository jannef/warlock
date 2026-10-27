//! Skills (spec §8 "Skills"; #132): what an actor has trained in, ADOM-like.
//!
//! The skills themselves are content (`data/skills.toml`, [`crate::content::SkillDef`]): an id,
//! a name, a one-line description, what kind of skill it is, when it earns training marks, and
//! what it gives. This module holds what an actor has of them and the rules that read them.
//!
//! - **Trained or not**: every actor has a set of trained skills ([`Skills`]), each with a value
//!   ([`Trained`]). A skill that is not in the set is **untrained**: it does not exist for that
//!   actor, is never shown, and gives nothing. That is not the same as a value of 0, which is
//!   trained (and shown) but gives little.
//! - **Values** start from what the race and the class grant (dice, rolled when the character is
//!   made, [`roll_grants`]), or what a creature type lists, and grow only by **level-up rolls**
//!   (spec §8 "Skills"; `world/skill_roll.rs`): past the soft cap (100) a roll adds half
//!   ([`RollRules`]). At most [`MAX_VALUE`].
//! - **Training marks**: a meaningful use of a skill ([`Use`]) may earn it a mark, with the
//!   chance its rule in data gives ([`MarkRule`]), drawn from the run's own training stream
//!   (never the gameplay stream, so training never changes a fight's rolls). Marks are a
//!   reservoir ([`Skills::mark`]): they raise nothing by themselves but set the die of the
//!   skill's next level-up roll on the [`Ladder`], and the roll consumes half of them
//!   ([`Skills::roll`]). Only the player trains, for now.
//! - **Benefits**: a skill may give passive effects ([`Effect`]) that grow with its value
//!   ([`Benefit`]). The rules ask an actor's skills for an effect ([`Skills::effect`]); its
//!   untrained skills add nothing.
//!
//! What the player sees: their own trained skills with their values, and a coarse sense of how
//! practised each is (its ladder step, [`progress`]). Never a benefit's number, a mark, or a chance.
//!
//! **Pending hooks** (extension points for rules not on main yet): athletics will shorten the
//! turns stuck in a pit (#128), and escape artist will take a turn off being stuck per 33 points
//! (#128). Each becomes one more [`Effect`] (and a [`Use`] for its marks: a climb) read in that
//! one place, with its numbers in `data/skills.toml`; until then escape artist is trained and
//! shown but gives nothing. Athletics already adds to the Strength a kick at a door is rolled
//! against ([`Effect::Kick`], [`Use::Kick`]), and Open lock to picking a lock ([`Effect::Pick`],
//! [`Use::Pick`]).

use crate::content::Content;
use crate::dice::Dice;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::RangeInclusive;

/// The highest value a skill can reach.
pub const MAX_VALUE: u16 = 999;

/// What a grant (a race's or a class's dice, `data/races.toml`, `data/classes/`) may give:
/// every result its dice can have lies within.
pub const GRANTS: RangeInclusive<i32> = 0..=100;

/// How finely the player senses how practised a skill is (its ladder step): in quarters.
pub const PROGRESS_STEPS: u8 = 4;

/// The most a skill adds to searching or to noticing in passing (spec §8 "Searching").
pub const MAX_SEARCH_BONUS: i32 = 20;

/// The most a skill adds to disarming a trap (spec §8 "Disarming traps").
pub const MAX_DISARM_BONUS: i32 = 20;

/// The most a skill takes off the difficulty of studying a spell (spec §8 "Spells").
pub const MAX_STUDY_BONUS: i32 = 20;

/// The most cells farther a search may reach than the cells around the searcher.
pub const MAX_SEARCH_REACH: i32 = 2;

/// The most a skill adds to stealing from someone (spec §8 "Stealing").
pub const MAX_STEAL_BONUS: i32 = 20;
/// The most a skill adds to treating bleeding, poison, or a disease (spec §8 "Skills": Heal).
pub const MAX_HEAL_BONUS: i32 = 20;
/// The most a skill adds to picking a lock (spec §8 "Doors").
pub const MAX_PICK_BONUS: i32 = 20;
/// The most percent a skill adds to the chance of a forage finding food (spec §8 "Foraging").
pub const MAX_FORAGE_BONUS: i32 = 30;

/// Index of a skill in [`Content::skills`]: in file order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SkillId(pub u8);

/// What a skill is for: the wiki groups them so. Rules never read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillKind {
    /// It works by itself, all the time (stealth, listen, tumble).
    Passive,
    /// It is used as an action (search, open lock, heal), once actions for it exist.
    Action,
    /// Rules read it when something calls for it (athletics when kicking a door in).
    Rules,
}

impl SkillKind {
    /// How the wiki says it: `passive`.
    pub const fn name(self) -> &'static str {
        match self {
            SkillKind::Passive => "passive",
            SkillKind::Action => "action",
            SkillKind::Rules => "read by rules",
        }
    }
}

/// A passive effect a skill can give, by its value ([`Benefit`]). Each is read in one place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// A bonus to Defence in the Dodge category, whose bonuses add up (spec §8 "Combat").
    Dodge,
    /// Percent less noise from the actor's steps and doors (spec §5 "Hearing"; `noise::noise`).
    Quiet,
    /// How much fainter a sound the actor hears: its hearing threshold, that much lower (never
    /// below 1).
    Hearing,
    /// Points added to an active search for hidden features (spec §8 "Searching";
    /// `world/search.rs`).
    Search,
    /// Cells farther a search reaches than the cells around the searcher.
    SearchReach,
    /// Points added to noticing hidden features in passing (spec §8 "Searching"); a skill with
    /// it, trained, lets the actor notice them at all.
    Spot,
    /// Points taken off the difficulty of studying a spell (spec §8 "Spells";
    /// `world/learn.rs`).
    Study,
    /// Points added to disarming a trap (spec §8 "Disarming traps"; `world/disarm.rs`); a skill
    /// with it, trained, lets the actor disarm traps at all (the Disarm ability).
    Disarm,
    /// Percent less of the burden's slowdown (spec §8 "Burden"): Burdened ×1.25 becomes ×1.10
    /// at 60.
    Burden,
    /// Percent less time a journey on the overworld takes (spec §8 "Time").
    Travel,
    /// Percent added to the chance of a forage finding food (spec §8 "Foraging";
    /// `world/forage.rs`); a skill with it, trained, lets the actor forage at all (the abilities
    /// menu's Forage).
    Forage,
    /// Percent better prices in shops (spec §7 "Shops"; `world/shop.rs`): less to pay and more
    /// offered, together with Presence at most half.
    Bargain,
    /// Points added to stealing from someone (spec §8 "Stealing"; `world/steal.rs`); a skill
    /// with it, trained, lets the actor steal at all (the abilities menu's Steal).
    Steal,
    /// Points added to treating oneself (spec §8 "Skills": Heal; `world/heal.rs`): bleeding,
    /// poison, a disease; a skill with it, trained, lets the actor treat at all (the Heal
    /// ability).
    Heal,
    /// Points added to the Strength score a kick at a closed door is rolled against (spec §8
    /// "Doors"; `world/door.rs`).
    Kick,
    /// Points added to picking a lock (spec §8 "Doors"; `world/lockpick.rs`); a skill with it,
    /// trained, lets the actor pick locks at all (the Pick lock ability).
    Pick,
}

impl Effect {
    /// The largest value a benefit of this effect may reach: the percent effects at most 100.
    pub const fn max(self) -> i32 {
        match self {
            Effect::Dodge | Effect::Hearing => MAX_VALUE as i32,
            Effect::Quiet | Effect::Burden | Effect::Travel | Effect::Bargain => 100,
            Effect::Kick => *crate::stats::SCORES.end() as i32,
            Effect::Search | Effect::Spot => MAX_SEARCH_BONUS,
            Effect::Study => MAX_STUDY_BONUS,
            Effect::Steal => MAX_STEAL_BONUS,
            Effect::Heal => MAX_HEAL_BONUS,
            Effect::Disarm => MAX_DISARM_BONUS,
            Effect::Pick => MAX_PICK_BONUS,
            Effect::Forage => MAX_FORAGE_BONUS,
            Effect::SearchReach => MAX_SEARCH_REACH,
        }
    }
}

/// A meaningful use of a skill, which may earn it a training mark ([`MarkRule`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Use {
    /// A step within earshot of a creature that could hear it, before stealth made it quieter.
    Sneak,
    /// Hearing something new: a sound that is news (spec §5 "Hearing").
    Hear,
    /// A turn spent searching (spec §8 "Searching"), whatever it finds.
    Search,
    /// Noticing something hidden in passing (spec §8 "Searching").
    Spot,
    /// A spell studied (spec §8 "Spells"), whatever comes of it.
    Study,
    /// An attempt to disarm a trap (spec §8 "Disarming traps"), whatever comes of it.
    Disarm,
    /// An attack that missed because the actor dodged it (the Dexterity or Dodge category), or a
    /// known trap the player stepped past (spec §8 "Traps").
    Dodge,
    /// An action taken (not waiting) while the burden slows the actor.
    Burdened,
    /// A step on the overworld: a journey.
    Travel,
    /// A forage that ran its time (spec §8 "Foraging"), whatever it found.
    Forage,
    /// An attempt to steal from someone (spec §8 "Stealing"), whatever comes of it.
    Steal,
    /// A try at treating oneself (spec §8 "Skills": Heal), whatever comes of it.
    Heal,
    /// A kick at a closed door (spec §8 "Doors").
    Kick,
    /// A try at picking a lock (spec §8 "Doors"), whatever comes of it.
    Pick,
}

impl Use {
    /// How the wiki says it.
    pub const fn describe(self) -> &'static str {
        match self {
            Use::Sneak => "a step within earshot of a creature that could hear it",
            Use::Hear => "hearing something new",
            Use::Search => "a turn spent searching",
            Use::Spot => "noticing something hidden in passing",
            Use::Study => "studying a spell from a scroll",
            Use::Disarm => "trying to disarm a trap",
            Use::Dodge => "dodging an attack or a known trap",
            Use::Burdened => "acting (not waiting) while your load slows you",
            Use::Travel => "a journey on the overworld",
            Use::Forage => "foraging in the wild",
            Use::Steal => "trying to steal from someone",
            Use::Heal => "trying to treat bleeding, poison, or a disease",
            Use::Kick => "kicking at a closed door",
            Use::Pick => "a try at picking a lock",
        }
    }
}

/// When a skill earns a training mark: on each `on`, with `chance` percent (1 to 100), so skills
/// used often do not race ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkRule {
    pub on: Use,
    pub chance: u8,
}

/// How a benefit grows with the skill's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Growth {
    /// One for each of `at` the value has reached, then one more for every `every` points past
    /// the last of them (or past 0 without any).
    Steps { at: Vec<u16>, every: Option<u16> },
    /// A line through `points` (value, effect), ascending in value from 0, rounded down between
    /// them and level after the last.
    Curve(Vec<(u16, u16)>),
}

/// A passive effect of a skill and how it grows with the skill's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Benefit {
    pub effect: Effect,
    pub growth: Growth,
}

/// A benefit as data writes it: `{ effect = "dodge", steps = [20, 40], every = 25 }` or
/// `{ effect = "quiet", curve = [[0, 0], [100, 60]] }`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenefitSpec {
    effect: Effect,
    #[serde(default)]
    steps: Vec<u16>,
    every: Option<u16>,
    curve: Option<Vec<[u16; 2]>>,
}

impl BenefitSpec {
    /// Checks the growth and resolves it: steps strictly rising, `every` at least 1, a curve of
    /// at least two points from value 0, rising in value, within the effect's range.
    pub(crate) fn resolve(self) -> Result<Benefit, String> {
        let effect = self.effect;
        let growth = match self.curve {
            Some(points) => {
                if !self.steps.is_empty() || self.every.is_some() {
                    return Err("a benefit has `steps` and `every`, or a `curve`, not both".into());
                }
                if points.len() < 2 || points[0][0] != 0 {
                    return Err("a `curve` needs at least two points, the first at value 0".into());
                }
                if points.windows(2).any(|w| w[0][0] >= w[1][0]) {
                    return Err("a `curve`'s values must rise from point to point".into());
                }
                Growth::Curve(points.iter().map(|p| (p[0], p[1])).collect())
            }
            None => {
                if self.steps.is_empty() && self.every.is_none() {
                    return Err("a benefit needs `steps`, `every`, or a `curve`".into());
                }
                if self.steps.first() == Some(&0) || self.steps.windows(2).any(|w| w[0] >= w[1]) {
                    return Err("`steps` must rise from above 0".into());
                }
                if self.every == Some(0) {
                    return Err("`every` must be at least 1".into());
                }
                Growth::Steps {
                    at: self.steps,
                    every: self.every,
                }
            }
        };
        let benefit = Benefit { effect, growth };
        if (0..=MAX_VALUE).any(|v| !(0..=effect.max()).contains(&benefit.at(v))) {
            return Err(format!(
                "a `{}` benefit must stay within 0 to {}",
                effect.id(),
                effect.max()
            ));
        }
        Ok(benefit)
    }
}

impl Use {
    /// Its id in data: `sneak`.
    pub const fn id(self) -> &'static str {
        match self {
            Use::Sneak => "sneak",
            Use::Hear => "hear",
            Use::Search => "search",
            Use::Spot => "spot",
            Use::Study => "study",
            Use::Disarm => "disarm",
            Use::Dodge => "dodge",
            Use::Burdened => "burdened",
            Use::Travel => "travel",
            Use::Forage => "forage",
            Use::Steal => "steal",
            Use::Heal => "heal",
            Use::Kick => "kick",
            Use::Pick => "pick",
        }
    }
}

impl Effect {
    /// Its id in data: `dodge`.
    pub const fn id(self) -> &'static str {
        match self {
            Effect::Dodge => "dodge",
            Effect::Quiet => "quiet",
            Effect::Hearing => "hearing",
            Effect::Search => "search",
            Effect::SearchReach => "search_reach",
            Effect::Spot => "spot",
            Effect::Study => "study",
            Effect::Disarm => "disarm",
            Effect::Burden => "burden",
            Effect::Travel => "travel",
            Effect::Forage => "forage",
            Effect::Bargain => "bargain",
            Effect::Steal => "steal",
            Effect::Heal => "heal",
            Effect::Kick => "kick",
            Effect::Pick => "pick",
        }
    }
}

impl Benefit {
    /// What it gives at skill value `value`.
    pub fn at(&self, value: u16) -> i32 {
        match &self.growth {
            Growth::Steps { at, every } => {
                let reached = at.iter().filter(|&&s| s <= value).count() as i32;
                let last = at.last().copied().unwrap_or(0);
                let more = match every {
                    Some(every) if value > last => i32::from((value - last) / every),
                    _ => 0,
                };
                reached + more
            }
            Growth::Curve(points) => {
                let after = points.iter().position(|&(x, _)| x > value);
                match after {
                    // The first point is at 0, so a later one follows a reached one.
                    Some(i) => {
                        let ((x0, y0), (x1, y1)) = (points[i - 1], points[i]);
                        let (x0, y0, x1, y1) =
                            (i32::from(x0), i32::from(y0), i32::from(x1), i32::from(y1));
                        y0 + (y1 - y0) * (i32::from(value) - x0) / (x1 - x0)
                    }
                    None => points.last().map_or(0, |&(_, y)| i32::from(y)),
                }
            }
        }
    }
}

/// The skill ladder (spec §8 "Skills", ADOM's): the die a level-up roll on a skill throws, set by
/// the training marks it has gathered since its last roll. `dice[i]` is thrown from `marks[i]`
/// marks on (`[ladder]` in `data/skills.toml`: `+1` at 0 marks, `1d3` at 5, … `4d5` at 800).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ladder {
    /// The dice, one per step, lowest first.
    pub dice: Vec<Dice>,
    /// The marks each step starts at: 0 first, rising.
    pub marks: Vec<u32>,
}

impl Default for Ladder {
    /// ADOM's ladder: `+1`, `1d3`, `1d5`, `2d4`, `3d3`, `3d4`, `3d5`, `4d4`, `4d5`, the marks
    /// roughly doubling from step to step.
    fn default() -> Self {
        let dice = [
            (0, 1),
            (1, 3),
            (1, 5),
            (2, 4),
            (3, 3),
            (3, 4),
            (3, 5),
            (4, 4),
            (4, 5),
        ];
        Self {
            dice: (dice.iter())
                .map(|&(n, s)| {
                    if n == 0 {
                        Dice::fixed(s)
                    } else {
                        Dice::of(n as u32, s as u32)
                    }
                })
                .collect(),
            marks: vec![0, 5, 12, 25, 50, 100, 200, 400, 800],
        }
    }
}

impl Ladder {
    /// Checks that it has a step, a mark threshold per die, rising from 0, and dice that always
    /// add at least 1.
    pub(crate) fn check(&self) -> Result<(), String> {
        if self.dice.is_empty() || self.dice.len() != self.marks.len() {
            return Err("ladder: `dice` and `marks` need one entry per step, at least one".into());
        }
        if self.marks[0] != 0 || self.marks.windows(2).any(|w| w[0] >= w[1]) {
            return Err("ladder: `marks` start at 0 and rise from step to step".into());
        }
        if let Some(d) = self.dice.iter().find(|d| d.min() < 1) {
            return Err(format!("ladder: every die adds at least 1, not {d}"));
        }
        Ok(())
    }

    /// The step a skill with `marks` stands on (0 the lowest).
    pub fn step(&self, marks: u32) -> usize {
        self.marks.iter().rposition(|&m| m <= marks).unwrap_or(0)
    }

    /// The die of step `step`, the top one for any step above it.
    pub fn die(&self, step: usize) -> Dice {
        self.dice[step.min(self.top())]
    }

    /// The highest step.
    pub fn top(&self) -> usize {
        self.dice.len().saturating_sub(1)
    }
}

/// How many skill rolls a level gives beyond the class's and the race's, and what a roll costs
/// (`[rolls]` in `data/skills.toml`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RollRules {
    /// One more roll for an Intelligence score of this or more…
    pub intelligence_from: u8,
    /// …and one more for every this many points above it.
    pub intelligence_every: u8,
    /// The percent of a skill's marks a roll on it consumes, rounded up.
    pub spend: u8,
    /// From this value on a roll adds half its result, rounded up.
    pub soft_cap: u16,
}

impl Default for RollRules {
    /// +1 at Intelligence 14, +2 at 18, …; a roll spends half the marks; halved from 100.
    fn default() -> Self {
        Self {
            intelligence_from: 14,
            intelligence_every: 4,
            spend: 50,
            soft_cap: 100,
        }
    }
}

impl RollRules {
    /// Checks that the steps are at least 1 and the spend a percent.
    pub(crate) fn check(&self) -> Result<(), String> {
        if self.intelligence_from == 0 || self.intelligence_every == 0 || self.soft_cap == 0 {
            return Err(
                "rolls: `intelligence_from`, `intelligence_every`, and `soft_cap` must be \
                        at least 1"
                    .into(),
            );
        }
        if self.spend > 100 {
            return Err(format!("rolls: `spend` is a percent, not {}", self.spend));
        }
        Ok(())
    }

    /// The rolls an Intelligence score of `score` adds to a level's.
    pub fn for_intelligence(&self, score: u8) -> u8 {
        match score.checked_sub(self.intelligence_from) {
            Some(above) => 1 + above / self.intelligence_every.max(1),
            None => 0,
        }
    }

    /// The marks a roll consumes of `marks`: [`RollRules::spend`] percent, rounded up.
    pub fn spent(&self, marks: u32) -> u32 {
        let spent = (u64::from(marks) * u64::from(self.spend)).div_ceil(100);
        u32::try_from(spent).unwrap_or(marks).min(marks)
    }
}

/// A trained skill: its value and the training marks gathered since its last level-up roll
/// (a reservoir: they set the roll's die, [`Ladder`], and the roll consumes some).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trained {
    pub value: u16,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub marks: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// How practised a skill of `trained` is, in [`PROGRESS_STEPS`]: its marks' step on the ladder,
/// scaled to the meter (0 at `+1`, 2 at `2d4`, full near the top). What the player senses of
/// how big their next roll on it is, never the marks or the die.
pub fn progress(trained: Trained, ladder: &Ladder) -> u8 {
    let top = ladder.top().max(1);
    let step = ladder.step(trained.marks);
    let cells = (step * usize::from(PROGRESS_STEPS)).div_ceil(top);
    cells.min(usize::from(PROGRESS_STEPS)) as u8
}

/// An actor's trained skills, by skill. A skill not here is untrained: it gives nothing and is
/// never shown. Saved and hashed as a map from skill index to [`Trained`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Skills(BTreeMap<SkillId, Trained>);

impl Skills {
    /// Whether the actor is trained in nothing.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The skill `id`, if trained.
    pub fn get(&self, id: SkillId) -> Option<Trained> {
        self.0.get(&id).copied()
    }

    /// The value of skill `id`; `None` if untrained (not the same as 0).
    pub fn value(&self, id: SkillId) -> Option<u16> {
        self.get(id).map(|t| t.value)
    }

    /// Every trained skill, in skill order.
    pub fn iter(&self) -> impl Iterator<Item = (SkillId, Trained)> + '_ {
        self.0.iter().map(|(&id, &t)| (id, t))
    }

    /// Trains skill `id` (at 0 if it was untrained) and adds `by` to its value, within 0 to
    /// [`MAX_VALUE`].
    pub fn add(&mut self, id: SkillId, by: i32) {
        let trained = self.0.entry(id).or_default();
        let value = (i32::from(trained.value) + by).clamp(0, i32::from(MAX_VALUE));
        trained.value = value as u16;
    }

    /// One training mark for skill `id`, if trained (they gather until a level-up roll spends
    /// them, [`Skills::roll`]). Returns whether it was trained.
    pub fn mark(&mut self, id: SkillId) -> bool {
        let Some(trained) = self.0.get_mut(&id) else {
            return false;
        };
        trained.marks = trained.marks.saturating_add(1);
        true
    }

    /// A level-up roll of `rolled` on skill `id`, if trained: its value grows by it (by half,
    /// rounded up, from the soft cap on; within [`MAX_VALUE`]), and the roll consumes its share
    /// of the marks ([`RollRules::spent`]). Returns the points gained, `None` if untrained.
    pub fn roll(&mut self, id: SkillId, rolled: i32, rules: &RollRules) -> Option<u16> {
        let trained = self.0.get_mut(&id)?;
        let rolled = rolled.max(0) as u32;
        let gain = if trained.value >= rules.soft_cap {
            rolled.div_ceil(2)
        } else {
            rolled
        };
        let before = trained.value;
        trained.value = (u32::from(before) + gain).min(u32::from(MAX_VALUE)) as u16;
        trained.marks -= rules.spent(trained.marks);
        Some(trained.value - before)
    }

    /// What the trained skills give of `effect`, added up: each trained skill's benefits of
    /// that effect at its value ([`Benefit::at`]). Untrained skills give nothing.
    pub fn effect(&self, content: &Content, effect: Effect) -> i32 {
        self.iter()
            .map(|(id, t)| {
                (content.skill(id).benefits.iter())
                    .filter(|b| b.effect == effect)
                    .map(|b| b.at(t.value))
                    .sum::<i32>()
            })
            .sum()
    }

    /// Whether a skill with a benefit of `effect` is trained, whatever it gives at its value
    /// (the Spot skill lets the actor notice hidden features in passing at all, spec §8
    /// "Searching").
    pub fn trains_effect(&self, content: &Content, effect: Effect) -> bool {
        self.iter()
            .any(|(id, _)| (content.skill(id).benefits.iter()).any(|b| b.effect == effect))
    }

    /// What its skills give of `effect` with `bonus` added to some skills' values (what magic
    /// items add, spec §8 "Magic items"; `Actor::skill_effect`): a skill with a bonus gives as if
    /// its value were that much higher, and so does an untrained one, from 0. Values stay
    /// within 0 to [`MAX_VALUE`].
    pub fn effect_with(&self, content: &Content, effect: Effect, bonus: &[(SkillId, i32)]) -> i32 {
        if bonus.is_empty() {
            return self.effect(content, effect);
        }
        let mut ids: Vec<SkillId> = self.0.keys().copied().collect();
        ids.extend(bonus.iter().map(|&(id, _)| id));
        ids.sort();
        ids.dedup();
        (ids.into_iter())
            .map(|id| {
                let base = self.value(id).map_or(0, i32::from);
                let more: i32 = (bonus.iter())
                    .filter(|(s, _)| *s == id)
                    .map(|(_, v)| v)
                    .sum();
                let value = (base + more).clamp(0, i32::from(MAX_VALUE)) as u16;
                (content.skill(id).benefits.iter())
                    .filter(|b| b.effect == effect)
                    .map(|b| b.at(value))
                    .sum::<i32>()
            })
            .sum()
    }
}

impl FromIterator<(SkillId, Trained)> for Skills {
    fn from_iter<I: IntoIterator<Item = (SkillId, Trained)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "skill {}", self.0)
    }
}

/// Grants (a race's or a class's `skills`) as data gives them, by skill id: every result each
/// can have within [`GRANTS`]; resolved to skill indices by `skill_id`, in skill order. The
/// error names the skill.
pub fn grants(
    data: &BTreeMap<String, Dice>,
    skill_id: impl Fn(&str) -> Option<SkillId>,
) -> Result<Vec<(SkillId, Dice)>, String> {
    let mut out = Vec::with_capacity(data.len());
    for (name, &dice) in data {
        let id = skill_id(name).ok_or_else(|| format!("unknown skill `{name}`"))?;
        if !GRANTS.contains(&dice.min()) || !GRANTS.contains(&dice.max()) {
            return Err(format!(
                "{name} {dice} is out of range ({} to {})",
                GRANTS.start(),
                GRANTS.end()
            ));
        }
        out.push((id, dice));
    }
    out.sort_by_key(|&(id, _)| id);
    Ok(out)
}

/// Rolls `grants` from `rng` in their order (fixed numbers draw nothing) and adds each to
/// `skills`, training those it names.
pub fn roll_grants(grants: &[(SkillId, Dice)], skills: &mut Skills, rng: &mut Rng) {
    for &(id, dice) in grants {
        skills.add(id, dice.roll(rng));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps(at: &[u16], every: Option<u16>) -> Benefit {
        Benefit {
            effect: Effect::Dodge,
            growth: Growth::Steps {
                at: at.to_vec(),
                every,
            },
        }
    }

    fn spec(text: &str) -> Result<Benefit, String> {
        #[derive(Deserialize)]
        struct One {
            benefit: BenefitSpec,
        }
        let one: One = toml::from_str(&format!("benefit = {text}")).map_err(|e| e.to_string())?;
        one.benefit.resolve()
    }

    #[test]
    fn steps_count_the_thresholds_reached_then_every_so_many_points() {
        // Tumble: +1 at 20, 40, 60, 80, 100, then +1 per 25 past 100.
        let tumble = steps(&[20, 40, 60, 80, 100], Some(25));
        let at = |v| tumble.at(v);
        assert_eq!(
            [0, 19, 20, 39, 40, 99, 100, 124, 125, 150, 200].map(at),
            [0, 0, 1, 1, 2, 4, 5, 5, 6, 7, 9]
        );
        // Listen: one per 10.
        let listen = steps(&[], Some(10));
        assert_eq!([0, 9, 10, 55, 100].map(|v| listen.at(v)), [0, 0, 1, 5, 10]);
        // Steps alone stop at the last.
        assert_eq!(steps(&[10, 20], None).at(500), 2);
    }

    #[test]
    fn curves_run_straight_between_points_and_level_off_after_the_last() {
        let quiet = Benefit {
            effect: Effect::Quiet,
            growth: Growth::Curve(vec![(0, 0), (100, 60), (150, 75)]),
        };
        let at = |v| quiet.at(v);
        assert_eq!(
            [0, 1, 2, 50, 99, 100, 101, 125, 150, 999].map(at),
            [0, 0, 1, 30, 59, 60, 60, 67, 75, 75]
        );
    }

    #[test]
    fn benefits_parse_and_refuse_bad_growth() {
        assert_eq!(
            spec("{ effect = \"dodge\", steps = [20, 40], every = 25 }"),
            Ok(steps(&[20, 40], Some(25)))
        );
        assert_eq!(
            spec("{ effect = \"travel\", curve = [[0, 0], [100, 20]] }").map(|b| b.at(50)),
            Ok(10)
        );
        for (text, why) in [
            (
                "{ effect = \"dodge\" }",
                "needs `steps`, `every`, or a `curve`",
            ),
            ("{ effect = \"dodge\", steps = [40, 20] }", "must rise"),
            (
                "{ effect = \"dodge\", steps = [0, 20] }",
                "must rise from above 0",
            ),
            ("{ effect = \"dodge\", every = 0 }", "at least 1"),
            (
                "{ effect = \"quiet\", curve = [[5, 0], [100, 60]] }",
                "at value 0",
            ),
            (
                "{ effect = \"quiet\", curve = [[0, 0]] }",
                "at least two points",
            ),
            (
                "{ effect = \"quiet\", curve = [[0, 0], [0, 5]] }",
                "must rise",
            ),
            (
                "{ effect = \"quiet\", curve = [[0, 0], [100, 120]] }",
                "within 0 to 100",
            ),
            (
                "{ effect = \"quiet\", curve = [[0, 0], [9, 9]], steps = [3] }",
                "not both",
            ),
            ("{ effect = \"luck\", every = 3 }", "unknown variant `luck`"),
        ] {
            let err = spec(text).unwrap_err();
            assert!(err.contains(why), "{text}: {err}");
        }
    }

    #[test]
    fn the_ladder_steps_up_with_the_marks() {
        let ladder = Ladder::default();
        assert_eq!(ladder.check(), Ok(()));
        let die = |marks| ladder.die(ladder.step(marks)).to_string();
        assert_eq!(
            [0, 4, 5, 11, 12, 25, 50, 99, 100, 200, 400, 799, 800, 9000].map(die),
            [
                "+1", "+1", "1d3", "1d3", "1d5", "2d4", "3d3", "3d3", "3d4", "3d5", "4d4", "4d4",
                "4d5", "4d5"
            ]
        );
        assert_eq!(ladder.die(99).to_string(), "4d5", "above the top, the top");
        for bad in [
            Ladder {
                marks: vec![1],
                dice: vec![Dice::fixed(1)],
            },
            Ladder {
                marks: vec![0, 5, 5],
                dice: vec![Dice::fixed(1); 3],
            },
            Ladder {
                marks: vec![0],
                dice: vec![],
            },
            Ladder {
                marks: vec![0],
                dice: vec![Dice::fixed(0)],
            },
        ] {
            assert!(bad.check().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn rolls_grow_a_trained_skill_and_spend_half_its_marks() {
        let rules = RollRules::default();
        assert_eq!(rules.check(), Ok(()));
        let (stealth, listen) = (SkillId(3), SkillId(5));
        let mut skills = Skills::default();
        assert!(skills.is_empty());
        assert!(!skills.mark(stealth), "no marks for an untrained skill");
        assert_eq!(skills.roll(stealth, 3, &rules), None);
        assert_eq!(skills.get(stealth), None);
        skills.add(stealth, 0);
        assert_eq!(
            skills.value(stealth),
            Some(0),
            "trained at 0 is not untrained"
        );
        for _ in 0..50 {
            assert!(skills.mark(stealth));
        }
        assert_eq!(skills.value(stealth), Some(0), "marks alone raise nothing");
        // 50 → 25 → 12 → 6 → 3: each roll spends half the marks, rounded up.
        assert_eq!(skills.roll(stealth, 7, &rules), Some(7));
        assert_eq!(
            skills.get(stealth),
            Some(Trained {
                value: 7,
                marks: 25
            })
        );
        skills.roll(stealth, 1, &rules);
        assert_eq!(skills.get(stealth).map(|t| t.marks), Some(12));
        skills.roll(stealth, 1, &rules);
        skills.roll(stealth, 1, &rules);
        assert_eq!(skills.get(stealth).map(|t| t.marks), Some(3));
        // From the soft cap on a roll adds half, rounded up.
        skills.add(listen, 99);
        assert_eq!(
            skills.roll(listen, 5, &rules),
            Some(5),
            "below the cap, whole"
        );
        assert_eq!(skills.roll(listen, 5, &rules), Some(3));
        // Values stay within 0 to 999.
        skills.add(stealth, -50);
        assert_eq!(skills.value(stealth), Some(0));
        skills.add(stealth, 5000);
        assert_eq!(skills.roll(stealth, 9, &rules), Some(0));
        assert_eq!(skills.value(stealth), Some(MAX_VALUE));
        assert_eq!(
            skills.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            [stealth, listen]
        );
    }

    #[test]
    fn intelligence_adds_a_roll_from_14_and_every_4_more() {
        let rules = RollRules::default();
        assert_eq!(
            [3, 13, 14, 17, 18, 21, 22, 40].map(|s| rules.for_intelligence(s)),
            [0, 0, 1, 1, 2, 2, 3, 7]
        );
        assert_eq!(
            [0, 1, 2, 3, 49, 50].map(|m| rules.spent(m)),
            [0, 1, 1, 2, 25, 25]
        );
        assert!(
            RollRules {
                spend: 101,
                ..rules
            }
            .check()
            .is_err()
        );
        assert!(
            RollRules {
                intelligence_every: 0,
                ..rules
            }
            .check()
            .is_err()
        );
    }

    #[test]
    fn the_meter_shows_the_ladder_step_coarsely() {
        let ladder = Ladder::default();
        let at = |marks| progress(Trained { value: 0, marks }, &ladder);
        assert_eq!(
            [0, 5, 12, 25, 50, 100, 200, 400, 800].map(at),
            [0, 1, 1, 2, 2, 3, 3, 4, 4]
        );
    }

    #[test]
    fn skills_serialize_by_index_and_round_trip() {
        let mut skills = Skills::default();
        skills.add(SkillId(2), 17);
        skills.add(SkillId(0), 0);
        skills.mark(SkillId(2));
        let json = serde_json::to_string(&skills).expect("serializes");
        assert_eq!(json, r#"{"0":{"value":0},"2":{"value":17,"marks":1}}"#);
        let back: Skills = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, skills);
    }

    #[test]
    fn grants_check_their_range_and_roll_in_skill_order() {
        let ids = ["appraise", "stealth", "tumble"];
        let id = |name: &str| (ids.iter().position(|n| *n == name)).map(|i| SkillId(i as u8));
        let dice = |t: &str| t.parse::<Dice>().expect("dice");
        let data = BTreeMap::from([
            ("tumble".to_string(), dice("2d6+10")),
            ("appraise".to_string(), Dice::fixed(5)),
        ]);
        let resolved = grants(&data, id).expect("valid");
        assert_eq!(
            resolved,
            [(SkillId(0), Dice::fixed(5)), (SkillId(2), dice("2d6+10"))]
        );
        let mut rng = Rng::stream(1, "skills-test");
        let mut skills = Skills::default();
        roll_grants(&resolved, &mut skills, &mut rng);
        assert_eq!(skills.value(SkillId(0)), Some(5));
        assert!((12..=22).contains(&skills.value(SkillId(2)).expect("rolled")));
        assert_eq!(skills.value(SkillId(1)), None);
        let mut again = Rng::stream(1, "skills-test");
        assert_eq!(
            skills.value(SkillId(2)).map(i32::from),
            Some(dice("2d6+10").roll(&mut again))
        );
        for (name, text, why) in [
            ("luck", "1d6", "unknown skill `luck`"),
            ("stealth", "1d6-2", "out of range"),
            ("stealth", "2d50+5", "out of range"),
        ] {
            let data = BTreeMap::from([(name.to_string(), dice(text))]);
            let err = grants(&data, id).unwrap_err();
            assert!(err.contains(why), "{err}");
        }
    }
}
