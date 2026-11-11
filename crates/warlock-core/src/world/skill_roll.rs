//! Skill rolls at level-up (spec §8 "Skills", "Talents"): every level gives the player a number
//! of rolls ([`roll_count`]: the class's, the race's, one more for every `intelligence_every`
//! points of Intelligence from `intelligence_from`, and the talents'), kept in the level's `Due`
//! until spent. Each is spent with `advance:SKILL` ([`crate::action::Action::Advance`]) on a
//! trained skill of the player's choice, any number on one skill: the dice are [`skill_roll`]'s,
//! the die the skill's training marks set on the ladder (`[ladder]` in `data/skills.toml`), one
//! step up for a skill the class is known for, and the roll adds 1 for any other skill the class
//! trains or a skill the race trains. Half the marks are spent by the roll, so the next roll on
//! the same skill is smaller: the shrinking dice are the only brake. Rolls draw from the
//! `levels` stream, never the gameplay stream, and only when the player chooses the skill.
//!
//! A skill also rolls by itself, mid-game, when its marks reach `[rolls] auto_at`
//! ([`World::mark_skill`], where every mark is added): the same roll, on the same stream, with
//! the same words, and in addition to the level's rolls (#1328).
//!
//! The level-up screen shows both (the owner, 2026-10-27: numbers on that screen): each skill's
//! dice and what set them, and where the level's rolls came from. [`roll_count`] and
//! [`skill_roll`] are the one place each is computed, for the rule and the screen alike. The
//! message is still words: "Your stealth improves.", never the number rolled.
//!
//! A level whose player is trained in nothing (a scenario without a character) gives no rolls:
//! nothing could be rolled on.

use super::{Actor, Spent, World};
use crate::character::Character;
use crate::content::Content;
use crate::dice::Dice;
use crate::rng::Rng;
use crate::skills::{SkillId, Trained};
use crate::stats::Stat;

/// Where a level's skill rolls come from ([`roll_count`]): what the level-up screen names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RollCount {
    /// The class's `rolls`.
    pub class: u8,
    /// The race's `rolls`.
    pub race: u8,
    /// Intelligence's: one from `intelligence_from`, one more every `intelligence_every` points.
    pub intelligence: u8,
    /// The talents' (Quick learner).
    pub talents: u8,
}

impl RollCount {
    /// The rolls a level gives.
    pub fn total(&self) -> u8 {
        (self.class)
            .saturating_add(self.race)
            .saturating_add(self.intelligence)
            .saturating_add(self.talents)
    }
}

/// The skill rolls total level `level` gives `player` (of `character`): the `rolls` of the
/// class the level falls in (spec §8 "Prestige classes"), their race's, Intelligence's (the
/// score as the character screen shows it without status effects), and their talents' (Quick
/// learner); none while they are trained in no skill.
pub fn roll_count(
    content: &Content,
    player: &Actor,
    character: Option<&Character>,
    level: u8,
) -> RollCount {
    if player.skills.is_empty() {
        return RollCount::default();
    }
    let (class, race) = match character {
        Some(c) => (
            (c.class_of(content, level)).map_or(0, |(table, _)| table.rolls()),
            c.spec.race(content).map_or(0, |race| race.rolls),
        ),
        None => (0, 0),
    };
    let score = player.raised_stats(content).score(Stat::Intelligence);
    RollCount {
        class,
        race,
        intelligence: content.skill_rolls().for_intelligence(score),
        talents: u8::try_from(player.talent_mods(content).skill_rolls.max(0)).unwrap_or(0),
    }
}

/// Why a roll on a skill adds 1 ([`SkillRoll::bonus`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollBonus {
    /// The class trains the skill (one it is not known for).
    Class,
    /// The race trains it.
    Race,
}

/// A level-up roll on a trained skill ([`skill_roll`]): the dice and what set them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillRoll {
    /// The skill's training marks.
    pub marks: u32,
    /// The die the marks set on the ladder.
    pub practice: Dice,
    /// Whether the class is known for the skill: one step up the ladder.
    pub known: bool,
    /// The die rolled: `practice`, a step up when `known`.
    pub die: Dice,
    /// Why the roll adds 1, if it does.
    pub bonus: Option<RollBonus>,
    /// Whether the skill is at the soft cap or above: it rises by half the roll, rounded up.
    pub halved: bool,
}

impl SkillRoll {
    /// The dice rolled, the bonus added (before any halving).
    pub fn dice(&self) -> Dice {
        self.die.plus(i32::from(self.bonus.is_some()))
    }
}

/// The level-up roll on `skill`, trained as `trained`, of a player of `character`: the die the
/// skill's marks set on the ladder, one step up for a skill the class is known for (or the
/// prestige class, once called: the union of both; spec §8 "Prestige classes"), +1 for any
/// other skill the class trains or a skill the race trains, halved from the soft cap on.
pub fn skill_roll(
    content: &Content,
    character: Option<&Character>,
    skill: SkillId,
    trained: Trained,
) -> SkillRoll {
    let class = character.and_then(|c| c.spec.class(content).ok());
    let prestige = character.and_then(|c| c.prestige(content));
    let race = character.and_then(|c| c.spec.race(content).ok());
    let known = class.is_some_and(|c| c.is_known_for(skill))
        || prestige.is_some_and(|p| p.known_for.contains(&skill));
    let trains = class.is_some_and(|c| c.trains(skill) && !known);
    let racial = race.is_some_and(|r| r.skills.iter().any(|&(s, _)| s == skill));
    let bonus = match (trains, racial) {
        (true, _) => Some(RollBonus::Class),
        (false, true) => Some(RollBonus::Race),
        (false, false) => None,
    };
    let ladder = content.ladder();
    let step = ladder.step(trained.marks);
    SkillRoll {
        marks: trained.marks,
        practice: ladder.die(step),
        known,
        die: ladder.die(step + usize::from(known)),
        bonus,
        halved: trained.value >= content.skill_rolls().soft_cap,
    }
}

impl World {
    /// The skill rolls total level `level` gives the player ([`roll_count`]).
    pub(super) fn level_rolls(&self, level: u8) -> u8 {
        let character = self.state.character.as_ref();
        roll_count(&self.content, self.state.player(), character, level).total()
    }

    /// `advance:SKILL` ([`crate::action::Action::Advance`]): one of the level's skill rolls on
    /// trained skill `skill`, if a roll is due; else why not, and nothing changes. Takes no
    /// time.
    pub(super) fn advance_skill(&mut self, skill: &str) -> Spent {
        let content = self.content.clone();
        let Some(id) = content.skill_id(skill) else {
            self.say(format!("There is no skill `{skill}`."));
            return Spent::Nothing;
        };
        let Some(at) = (self.state.progress.due.iter()).position(|d| d.rolls > 0) else {
            self.say("You have no skill rolls to spend.");
            return Spent::Nothing;
        };
        let def = content.skill(id);
        let Some(trained) = self.state.player().skills.get(id) else {
            self.say(format!(
                "You are not trained in {}.",
                def.name.to_lowercase()
            ));
            return Spent::Nothing;
        };
        self.roll_skill(id, trained);
        let due = &mut self.state.progress.due[at];
        due.rolls -= 1;
        if due.is_spent() {
            self.state.progress.due.remove(at);
        }
        self.settle_due();
        Spent::NoTime
    }

    /// One skill roll on the player's skill `id`, trained as `trained`: [`skill_roll`]'s dice
    /// on the `levels` stream, the value grown and the marks spent ([`crate::skills::Skills::roll`]),
    /// and the words, never the number. The one path of a level-up roll and an automatic one.
    fn roll_skill(&mut self, id: SkillId, trained: Trained) {
        let content = self.content.clone();
        let roll = skill_roll(&content, self.state.character.as_ref(), id, trained);
        let seed = self.state.seed;
        let rng = (self.state.levels_rng).get_or_insert_with(|| Rng::levels(seed));
        let rolled = roll.dice().roll(rng);
        self.player_mut()
            .skills
            .roll(id, rolled, content.skill_rolls());
        let name = content.skill(id).name.to_lowercase();
        self.say(format!("Your {name} improves."));
    }

    /// `by` training marks for the player's skill `id`, if trained: the one place marks are
    /// added (`World::train`, and any other gift of marks). A skill whose marks reach `[rolls]
    /// auto_at` rolls at once, by itself ([`crate::skills::RollRules::auto_at`]), the same
    /// roll a level-up's
    /// would be, and again while its spent marks are still at the threshold. Whatever level-up
    /// is due, its rolls are untouched. Returns whether the skill was trained.
    pub(super) fn mark_skill(&mut self, id: SkillId, by: u32) -> bool {
        if !self.player_mut().skills.add_marks(id, by) {
            return false;
        }
        let rules = *self.content.skill_rolls();
        while let Some(trained) = self.state.player().skills.get(id) {
            if rules.auto_at == 0
                || trained.marks < rules.auto_at
                || rules.spent(trained.marks) == 0
            {
                break;
            }
            self.roll_skill(id, trained);
        }
        true
    }

    /// The dev action `levelup:auto`'s part for skill rolls: every roll due on the player's
    /// first trained skill, in skill order.
    pub(super) fn advance_auto(&mut self) {
        let content = self.content.clone();
        while self.state.progress.due.iter().any(|d| d.rolls > 0) {
            let Some((first, _)) = self.state.player().skills.iter().next() else {
                break;
            };
            let id = content.skill(first).id.clone();
            if self.advance_skill(&id) == Spent::Nothing {
                break;
            }
        }
    }
}
