//! Levels' choices and talents in the world (spec §8 "Talents"). A level that comes
//! ([`World::level_up`], from `grant_xp`) hands out what it brings: its skill rolls
//! (`world/skill_roll.rs`) and every `talent_every`th level a talent pick, kept as a [`Due`] in
//! `Progress` until they are made. **Nothing is banked**: while a choice is due the rules refuse
//! every other action ([`World::refuse_due`], "You have skill rolls to spend." / "You must
//! choose a talent first."), so the level-up screen, the headless CLI, and replays all spend a
//! level on the spot, with [`crate::action::Action::Advance`] (`advance:stealth`) and
//! [`crate::action::Action::Talent`] (`talent:hardy`, `talent:weapon_focus:long_blades`). A pick
//! nothing can be spent on (no talent the player may take, no trained skill to roll) is dropped,
//! so a level never waits for the impossible. The dev action `levelup:auto` spends every choice
//! with the first the player may take.
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
    /// Skill rolls left (`world/skill_roll.rs`).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rolls: u8,
    /// Bonus talent picks left, from the class's bonus trees (spec §8 "Class abilities").
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bonus: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

impl Due {
    /// Whether nothing is left to choose.
    pub fn is_spent(&self) -> bool {
        self.talents == 0 && self.rolls == 0 && self.bonus == 0
    }

    /// Whether a talent pick of any kind is left.
    pub fn picks(&self) -> bool {
        self.talents > 0 || self.bonus > 0
    }
}

impl World {
    /// The choices due now, the lowest level's first; `None` when the player may act freely.
    pub fn due(&self) -> Option<Due> {
        self.state.progress.due.first().copied()
    }

    /// The trees the player's class's bonus picks come from (none without a character).
    pub(crate) fn bonus_trees(&self) -> Vec<crate::content::TreeId> {
        (self.state.character.as_ref())
            .and_then(|c| c.spec.class(&self.content).ok())
            .map(|c| c.bonus_trees.clone())
            .unwrap_or_default()
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

    /// Levels `from + 1..=to` came: each hands out what it brings (its skill rolls, a talent
    /// pick every `talent_every` levels, `data/experience.toml`, the class's abilities and
    /// bonus picks at its table's levels, spec §8 "Class abilities"), and the hit points the
    /// talents held grow by at each of them.
    pub(super) fn level_up(&mut self, from: u8, to: u8) {
        let every = self.content.experience().talent_every();
        let rolls = self.level_rolls();
        let class = (self.state.character.as_ref()).and_then(|c| c.spec.class(&self.content).ok());
        let bonus_levels = class.map(|c| c.bonus_levels.clone()).unwrap_or_default();
        let mut hp = 0;
        for level in from.saturating_add(1)..=to {
            let talents = u8::from(every > 0 && level % every == 0);
            let bonus = u8::from(bonus_levels.contains(&level));
            if talents > 0 || rolls > 0 || bonus > 0 {
                (self.state.progress.due).push(Due {
                    level,
                    talents,
                    rolls,
                    bonus,
                });
            }
            self.check_stats();
            hp += self.class_level(level);
            let mods = self.state.player().talent_mods(&self.content);
            hp += mods.hp_at(level) - mods.hp_at(level - 1);
        }
        if hp != 0 {
            let player = self.player_mut();
            player.hp_base += hp;
            player.hp += hp;
        }
        self.settle_due();
    }

    /// Drops the choices nothing can be spent on: talent picks when no talent may be taken now
    /// (a run without talents, or every talent they qualify for held) and no skill roll is left
    /// that might open one, and skill rolls when no skill is trained. Asked again after each
    /// pick, so a pick that another pick unlocks is kept.
    pub(super) fn settle_due(&mut self) {
        let due = &self.state.progress.due;
        if due.is_empty() {
            return;
        }
        let no_rolls = self.state.player().skills.is_empty();
        let rolls_left = !no_rolls && due.iter().any(|d| d.rolls > 0);
        let offers = talent::offers(&self.content, &self.candidate());
        let no_talents = due.iter().any(|d| d.talents > 0) && !rolls_left && offers.is_empty();
        let trees = self.bonus_trees();
        let no_bonus = due.iter().any(|d| d.bonus > 0)
            && !rolls_left
            && !(offers.iter())
                .any(|(id, _)| trees.contains(&self.content.talents().get(*id).tree));
        if !no_rolls && !no_talents && !no_bonus {
            return;
        }
        for due in &mut self.state.progress.due {
            if no_rolls {
                due.rolls = 0;
            }
            if no_talents {
                due.talents = 0;
            }
            if no_bonus {
                due.bonus = 0;
            }
        }
        self.state.progress.due.retain(|d| !d.is_spent());
    }

    /// Refuses `action` while a level's choices are due, unless it is one of them.
    pub(super) fn refuse_due(&mut self, action: &Action) -> Option<Spent> {
        let due = self.due()?;
        if matches!(
            action,
            Action::Talent { .. } | Action::Advance(_) | Action::LevelUpAuto
        ) {
            return None;
        }
        if due.rolls > 0 {
            self.say("You have skill rolls to spend.");
        } else {
            self.say("You must choose a talent first.");
        }
        Some(Spent::Nothing)
    }

    /// `talent:ID[:CHOICE]` ([`Action::Talent`]): the player takes talent `id` for `choice`,
    /// one of the picks due, if its prerequisites hold; else why not, and nothing changes.
    /// Takes no time.
    pub(super) fn take_talent(&mut self, id: &str, choice: Option<&str>) -> Spent {
        let content = self.content.clone();
        let Some(at) = (self.state.progress.due.iter()).position(Due::picks) else {
            self.say("You have no talent to choose now.");
            return Spent::Nothing;
        };
        let Some(talent) = content.talents().id(id) else {
            self.say(format!("There is no talent `{id}`."));
            return Spent::Nothing;
        };
        let def = content.talents().get(talent);
        let name = def.name_for(&content, choice);
        // A bonus pick is spent on a talent of the class's bonus trees first; any other talent
        // takes the level's own pick.
        let trees = self.bonus_trees();
        let due = self.state.progress.due[at];
        let bonus = due.bonus > 0 && trees.contains(&def.tree);
        if !bonus && due.talents == 0 {
            let names: Vec<&str> = (trees.iter())
                .map(|&t| content.talents().tree(t).name.as_str())
                .collect();
            self.say(format!(
                "Your class's pick is from {}.",
                crate::talent::or_list(&names)
            ));
            return Spent::Nothing;
        }
        let rank = match talent::may_take(&content, &self.candidate(), def, choice) {
            Ok(rank) => rank,
            Err(why) => {
                self.say(why.words(&content, &name));
                return Spent::Nothing;
            }
        };
        // The rank's hit points by level come now; each later level grows them.
        let hp = TalentMods::of(&self.content, def.effects_at(rank)).hp_at(self.player_level());
        let player = self.player_mut();
        player.talents.take(id, choice);
        player.hp_base += hp;
        player.hp += hp;
        let due = &mut self.state.progress.due[at];
        if bonus {
            due.bonus -= 1;
        } else {
            due.talents -= 1;
        }
        if due.is_spent() {
            self.state.progress.due.remove(at);
        }
        if def.rank_count() > 1 {
            self.say(format!("You learn {name} ({rank})."));
        } else {
            self.say(format!("You learn {name}."));
        }
        self.settle_due();
        Spent::NoTime
    }

    /// The dev action `levelup:auto` ([`Action::LevelUpAuto`]): spends every choice due with
    /// the first the player may take (skill rolls on the first trained skill, then talents tree
    /// by tree, in file order, the first choice of one with a choice), for tests and
    /// benchmarks. Takes no time.
    pub(super) fn level_up_auto(&mut self) -> Spent {
        if self.due().is_none() {
            self.say("Nothing is due.");
            return Spent::Nothing;
        }
        self.advance_auto();
        let content = self.content.clone();
        while let Some(due) = self.state.progress.due.iter().find(|d| d.picks()).copied() {
            let trees = self.bonus_trees();
            let offers = talent::offers(&content, &self.candidate());
            // A bonus pick alone takes the first talent of the class's trees.
            let mut offers = (offers.into_iter()).filter(|(id, _)| {
                due.talents > 0 || trees.contains(&content.talents().get(*id).tree)
            });
            let Some((id, ways)) = offers.next() else {
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
