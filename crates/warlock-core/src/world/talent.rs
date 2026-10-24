//! Levels' choices and talents in the world (spec §8 "Talents"). A level that comes
//! ([`World::level_up`], from `grant_xp`) hands out what it brings: every `talent_every`th level
//! a talent pick, kept as a [`Due`] in `Progress` until it is made. **Nothing is banked**: while
//! a choice is due the rules refuse every other action ([`World::refuse_due`], "You must choose
//! a talent first."), so the level-up screen, the headless CLI, and replays all spend a level
//! on the spot, with [`crate::action::Action::Talent`] (`talent:hardy`,
//! `talent:weapon_focus:long_blades`) and, once skill rolls exist,
//! [`crate::action::Action::Advance`]. A pick nothing can be spent on (no talent the player may
//! take) is dropped, so a level never waits for the impossible. The dev action `levelup:auto`
//! spends every choice with the first the player may take.
//!
//! What a talent does is read through `Actor::talent_mods` where each rule lives; the one
//! hand-out here is hit points by level (Hardy): a rank's share at the pick, and each later
//! level's growth as the level comes, into the actor's base hit points, as levels' own hit
//! points are.

use super::{Spent, World};
use crate::action::Action;
use crate::talent::{self, Candidate, TalentMods};
use serde::{Deserialize, Serialize};

/// What a level still asks the player to choose (spec §8 "Talents").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Due {
    /// The level that brought the choices.
    pub level: u8,
    /// Talent picks left.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub talents: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

impl Due {
    /// Whether nothing is left to choose.
    pub fn is_spent(&self) -> bool {
        self.talents == 0
    }
}

impl World {
    /// The choices due now, the lowest level's first; `None` when the player may act freely.
    pub fn due(&self) -> Option<Due> {
        self.state.progress.due.first().copied()
    }

    /// Everything a prerequisite may ask of the player now ([`Candidate`]).
    pub(crate) fn candidate(&self) -> Candidate<'_> {
        Candidate::player(
            &self.content,
            self.state.player(),
            self.state.character.as_ref(),
            self.player_level(),
        )
    }

    /// Levels `from + 1..=to` came: each hands out what it brings (a talent pick every
    /// `talent_every` levels, `data/experience.toml`), and the hit points the talents held
    /// grow by at each of them.
    pub(super) fn level_up(&mut self, from: u8, to: u8) {
        let every = self.content.experience().talent_every();
        let mods = self.state.player().talent_mods(&self.content);
        let mut hp = 0;
        for level in from.saturating_add(1)..=to {
            if every > 0 && level % every == 0 {
                self.state.progress.due.push(Due { level, talents: 1 });
            }
            hp += mods.hp_at(level) - mods.hp_at(level - 1);
        }
        if hp != 0 {
            let player = self.player_mut();
            player.hp_base += hp;
            player.hp += hp;
        }
        self.settle_due();
    }

    /// Drops the talent picks nothing can be spent on: no talent the player may take now (a
    /// run without talents, or every talent they qualify for held). Asked again after each
    /// pick, so a pick that another pick unlocks is kept.
    fn settle_due(&mut self) {
        if self.state.progress.due.is_empty() {
            return;
        }
        if !talent::offers(&self.content, &self.candidate()).is_empty() {
            return;
        }
        self.state.progress.due.clear();
    }

    /// Refuses `action` while a level's choices are due, unless it is one of them.
    pub(super) fn refuse_due(&mut self, action: &Action) -> Option<Spent> {
        self.due()?;
        if matches!(
            action,
            Action::Talent { .. } | Action::Advance(_) | Action::LevelUpAuto
        ) {
            return None;
        }
        self.say("You must choose a talent first.");
        Some(Spent::Nothing)
    }

    /// `talent:ID[:CHOICE]` ([`Action::Talent`]): the player takes talent `id` for `choice`,
    /// one of the picks due, if its prerequisites hold; else why not, and nothing changes.
    /// Takes no time.
    pub(super) fn take_talent(&mut self, id: &str, choice: Option<&str>) -> Spent {
        let content = self.content.clone();
        if self.due().is_none_or(|d| d.talents == 0) {
            self.say("You have no talent to choose now.");
            return Spent::Nothing;
        }
        let Some(talent) = content.talents().id(id) else {
            self.say(format!("There is no talent `{id}`."));
            return Spent::Nothing;
        };
        let def = content.talents().get(talent);
        let name = def.name_for(&content, choice);
        let rank = match talent::may_take(&content, &self.candidate(), def, choice) {
            Ok(rank) => rank,
            Err(why) => {
                self.say(why.words(&content, &name));
                return Spent::Nothing;
            }
        };
        // The rank's hit points by level come now; each later level grows them.
        let hp = TalentMods::of(def.effects_at(rank)).hp_at(self.player_level());
        let player = self.player_mut();
        player.talents.take(id, choice);
        player.hp_base += hp;
        player.hp += hp;
        let due = &mut self.state.progress.due[0];
        due.talents -= 1;
        if due.is_spent() {
            self.state.progress.due.remove(0);
        }
        if def.rank_count() > 1 {
            self.say(format!("You learn {name} ({rank})."));
        } else {
            self.say(format!("You learn {name}."));
        }
        self.settle_due();
        Spent::NoTime
    }

    /// `advance:SKILL` ([`Action::Advance`]): a skill roll of the level, on a trained skill.
    /// The rolls come with the skill ladder (#230); until then no level has any, so it is
    /// refused. Takes no time.
    pub(super) fn advance_skill(&mut self, skill: &str) -> Spent {
        if self.content.skill_id(skill).is_none() {
            self.say(format!("There is no skill `{skill}`."));
            return Spent::Nothing;
        }
        self.say("You have no skill rolls to spend.");
        Spent::Nothing
    }

    /// The dev action `levelup:auto` ([`Action::LevelUpAuto`]): spends every choice due with
    /// the first the player may take (talents tree by tree, in file order, the first choice of
    /// one with a choice), for tests and benchmarks. Takes no time.
    pub(super) fn level_up_auto(&mut self) -> Spent {
        if self.due().is_none() {
            self.say("Nothing is due.");
            return Spent::Nothing;
        }
        let content = self.content.clone();
        while self.due().is_some_and(|d| d.talents > 0) {
            let offers = talent::offers(&content, &self.candidate());
            let Some((id, ways)) = offers.into_iter().next() else {
                break;
            };
            let talent = content.talents().get(id).id.clone();
            let choice = ways.into_iter().next().flatten();
            if self.take_talent(&talent, choice.as_deref()) == Spent::Nothing {
                break;
            }
        }
        Spent::NoTime
    }
}
