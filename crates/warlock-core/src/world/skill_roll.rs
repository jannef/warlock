//! Skill rolls at level-up (spec §8 "Skills", "Talents"): every level gives the player a number
//! of rolls ([`World::level_rolls`]: the class's, the race's, and one more for every
//! `intelligence_every` points of Intelligence from `intelligence_from`), kept in the level's
//! `Due` until spent. Each is spent with `advance:SKILL` ([`crate::action::Action::Advance`]) on
//! a trained skill of the player's choice, any number on one skill: the die is the one the
//! skill's training marks set on the ladder (`[ladder]` in `data/skills.toml`), one step up for
//! a skill the class is known for, and the roll adds 1 for any other skill the class trains or
//! a skill the race trains. Half the marks are spent by the roll, so the next roll on the same
//! skill is smaller: the shrinking dice are the only brake. Rolls draw from the `levels` stream,
//! never the gameplay stream. The player is told "Your stealth improves.", never the die.
//!
//! A level whose player is trained in nothing (a scenario without a character) gives no rolls:
//! nothing could be rolled on.

use super::{Spent, World};
use crate::rng::Rng;
use crate::skills::SkillId;
use crate::stats::Stat;

impl World {
    /// The skill rolls a level gives the player now: their class's `rolls`, their race's,
    /// Intelligence's (the score as the character screen shows it without status effects), and
    /// their talents' (Quick learner); none while they are trained in no skill.
    pub(super) fn level_rolls(&self) -> u8 {
        let content = &self.content;
        let player = self.state.player();
        if player.skills.is_empty() {
            return 0;
        }
        let (class, race) = match &self.state.character {
            Some(c) => (
                c.spec.class(content).map_or(0, |class| class.rolls),
                c.spec.race(content).map_or(0, |race| race.rolls),
            ),
            None => (0, 0),
        };
        let intelligence = player.raised_stats(content).score(Stat::Intelligence);
        let wits = content.skill_rolls().for_intelligence(intelligence);
        let talents = u8::try_from(player.talent_mods(content).skill_rolls.max(0)).unwrap_or(0);
        class
            .saturating_add(race)
            .saturating_add(wits)
            .saturating_add(talents)
    }

    /// The ladder step and the bonus a roll on `skill` gets from the player's class and race:
    /// one step up for a skill the class is known for, +1 for any other the class trains or one
    /// the race trains.
    fn affinity(&self, skill: SkillId) -> (usize, i32) {
        let content = &self.content;
        let Some(character) = &self.state.character else {
            return (0, 0);
        };
        let class = character.spec.class(content).ok();
        let race = character.spec.race(content).ok();
        let known = class.is_some_and(|c| c.is_known_for(skill));
        let trains = class.is_some_and(|c| c.trains(skill) && !known);
        let racial = race.is_some_and(|r| r.skills.iter().any(|&(s, _)| s == skill));
        (usize::from(known), i32::from(trains || racial))
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
        let (up, bonus) = self.affinity(id);
        let ladder = content.ladder();
        let die = ladder.die(ladder.step(trained.marks) + up);
        let seed = self.state.seed;
        let rng = (self.state.levels_rng).get_or_insert_with(|| Rng::levels(seed));
        let rolled = die.roll(rng) + bonus;
        self.player_mut()
            .skills
            .roll(id, rolled, content.skill_rolls());
        let due = &mut self.state.progress.due[at];
        due.rolls -= 1;
        if due.is_spent() {
            self.state.progress.due.remove(at);
        }
        self.say(format!("Your {} improves.", def.name.to_lowercase()));
        self.settle_due();
        Spent::NoTime
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
