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
//! A level of the player's class's `bonus_talents` brings a **bonus pick** besides
//! ([`Due::bonus`]), of a talent from the class's trees (any for the commoner); a talent from
//! those trees spends a bonus pick before an ordinary one. The class abilities of a level are
//! given as it comes (`world/class.rs`).
//!
//! What a talent does is read through `Actor::talent_mods` where each rule lives; the hand-outs
//! are hit points by level (Hardy): a rank's share at the pick, and each later level's growth as
//! the level comes, into the actor's base hit points, as levels' own hit points are; and stat
//! gifts, once, at the pick (`world/class.rs`'s `hand_out`).

use super::{Spent, World, class};
use crate::action::Action;
use crate::content::{TalentEffect, TalentId, TreeId};
use crate::rng::Rng;
use crate::talent::{self, Candidate};
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
    /// Bonus talent picks left, from the class's trees (spec §8 "Class abilities").
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

    /// Levels `from + 1..=to` came: each hands out what it brings (its skill rolls, a talent
    /// pick every `talent_every` levels, `data/experience.toml`, a bonus pick at the class's
    /// levels for one, and the class abilities it gives), and the hit points and mana the
    /// talents held grow by at each of them.
    pub(super) fn level_up(&mut self, from: u8, to: u8) {
        let content = self.content.clone();
        let every = content.experience().talent_every();
        let rolls = self.level_rolls();
        let class = (self.state.character.as_ref()).and_then(|c| content.class(&c.spec.class));
        for level in from.saturating_add(1)..=to {
            // Versatile (the humans'): one more pick at its levels.
            let more = self
                .state
                .player()
                .talent_mods(&content)
                .extra_picks_at(level);
            let talents = u8::from(every > 0 && level % every == 0).saturating_add(more);
            let bonus = class.map_or(0, |c| c.bonus_at(level));
            if talents > 0 || rolls > 0 || bonus > 0 {
                (self.state.progress.due).push(Due {
                    level,
                    talents,
                    rolls,
                    bonus,
                });
            }
            // What the talents held grow by, then what this level's class abilities give.
            let mods = self.state.player().talent_mods(&content);
            let hp = mods.hp_at(level) - mods.hp_at(level - 1);
            let mana = mods.mana_at(level) - mods.mana_at(level - 1);
            let player = self.player_mut();
            player.hp_base += hp;
            player.hp += hp;
            player.mana_base += mana;
            player.mana += mana;
            self.grant_class(level, true);
            // Hardened: every level past the hit point levels adds a level's hit points again.
            let hardened = self.state.player().talent_mods(&content).hardened;
            if let Some(bonus) = hardened.filter(|_| level > content.experience().hp_level()) {
                let hp = self.level_hp(&content) + bonus;
                let player = self.player_mut();
                player.hp_base += hp;
                player.hp += hp;
            }
            self.check_stats();
        }
        self.settle_due();
    }

    /// Whether talent `id` may fill a bonus pick: it is of the class's bonus trees (`trees`;
    /// `None`: any).
    fn fits_bonus(&self, trees: &Option<Vec<TreeId>>, id: TalentId) -> bool {
        let tree = self.content.talents().get(id).tree;
        trees.as_ref().is_none_or(|t| t.contains(&tree))
    }

    /// The talents the player may take now for a bonus pick, with their choices.
    fn bonus_offers(&self) -> Vec<(TalentId, Vec<Option<String>>)> {
        let trees = self.bonus_trees();
        (talent::offers(&self.content, &self.candidate()).into_iter())
            .filter(|(id, _)| self.fits_bonus(&trees, *id))
            .collect()
    }

    /// Drops the choices nothing can be spent on: talent picks when no talent may be taken now
    /// (a run without talents, or every talent they qualify for held) and no skill roll is left
    /// that might open one, bonus picks likewise when no talent of the class's trees may be
    /// taken, and skill rolls when no skill is trained. Asked again after each pick, so a pick
    /// that another pick unlocks is kept.
    pub(super) fn settle_due(&mut self) {
        let due = &self.state.progress.due;
        if due.is_empty() {
            return;
        }
        let no_rolls = self.state.player().skills.is_empty();
        let rolls_left = !no_rolls && due.iter().any(|d| d.rolls > 0);
        let no_talents = due.iter().any(|d| d.talents > 0)
            && !rolls_left
            && talent::offers(&self.content, &self.candidate()).is_empty();
        let no_bonus =
            due.iter().any(|d| d.bonus > 0) && !rolls_left && self.bonus_offers().is_empty();
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
    /// one of the picks due (a bonus pick first, if it is of the class's bonus trees), if its
    /// prerequisites hold; else why not, and nothing changes. Takes no time.
    pub(super) fn take_talent(&mut self, id: &str, choice: Option<&str>) -> Spent {
        match self.learn(id, choice) {
            Ok(news) => {
                self.say(news);
                self.settle_due();
                Spent::NoTime
            }
            Err(why) => {
                self.say(why);
                Spent::Nothing
            }
        }
    }

    /// [`World::take_talent`] without its words: what is said of the talent learnt, or why it
    /// cannot be taken (and nothing changes).
    pub(super) fn learn(&mut self, id: &str, choice: Option<&str>) -> Result<String, String> {
        let content = self.content.clone();
        if !(self.state.progress.due.iter()).any(|d| d.talents > 0 || d.bonus > 0) {
            return Err("You have no talent to choose now.".into());
        }
        let Some(talent) = content.talents().id(id) else {
            return Err(format!("There is no talent `{id}`."));
        };
        let trees = self.bonus_trees();
        let bonus = self.fits_bonus(&trees, talent);
        let Some((at, as_bonus)) =
            (self.state.progress.due.iter())
                .enumerate()
                .find_map(|(i, d)| match () {
                    () if bonus && d.bonus > 0 => Some((i, true)),
                    () if d.talents > 0 => Some((i, false)),
                    () => None,
                })
        else {
            let names: Vec<String> = (trees.iter().flatten())
                .map(|&t| content.talents().tree(t).name.clone())
                .collect();
            return Err(format!(
                "You must choose a talent of {}.",
                talent::list_with(&names, "or")
            ));
        };
        let def = content.talents().get(talent);
        let name = def.name_for(&content, choice);
        let rank = match talent::may_take(&content, &self.candidate(), def, choice) {
            Ok(rank) => rank,
            Err(why) => return Err(why.words(&content, &name)),
        };
        // The rank's hit points by level come now (each later level grows them), and its gifts.
        let level = self.player_level();
        let player = self.player_mut();
        player.talents.take(id, choice);
        class::hand_out(&content, player, def, rank, level);
        self.train_chosen(def.effects_at(rank), choice);
        let due = &mut self.state.progress.due[at];
        if as_bonus {
            due.bonus -= 1;
        } else {
            due.talents -= 1;
        }
        if due.is_spent() {
            self.state.progress.due.remove(at);
        }
        Ok(match def.rank_count() > 1 {
            true => format!("You learn {name} ({rank})."),
            false => format!("You learn {name}."),
        })
    }

    /// Apprentice (spec §8 "Talents"): every `train_skill` of `effects`, a rank just taken for
    /// skill `choice`, trains it at its dice or adds them to it, rolled from the `levels`
    /// stream (never the gameplay stream: a pick moves no fight).
    fn train_chosen(&mut self, effects: impl Iterator<Item = TalentEffect>, choice: Option<&str>) {
        let Some(skill) = choice.and_then(|c| self.content.skill_id(c)) else {
            return;
        };
        for effect in effects {
            let TalentEffect::TrainSkill { dice } = effect else {
                continue;
            };
            let seed = self.state.seed;
            let rng = (self.state.levels_rng).get_or_insert_with(|| Rng::levels(seed));
            let rolled = dice.roll(rng);
            self.player_mut().skills.add(skill, rolled);
        }
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
        // Bonus picks first, from the class's trees, then the ordinary ones.
        while self
            .state
            .progress
            .due
            .iter()
            .any(|d| d.talents > 0 || d.bonus > 0)
        {
            let offers = match self.state.progress.due.iter().any(|d| d.bonus > 0) {
                true => self.bonus_offers(),
                false => talent::offers(&content, &self.candidate()),
            };
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
