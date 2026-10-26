//! Rolled quests in play (spec §8 "Quests", docs/design/adventurers-guild.md §5, #422): the draw
//! as a quest is given, what the player can reach, the dev action `quest_roll:ID:TARGET`, and
//! the bounties system's Quarry sense. The rules of giving are `world/quest.rs`'s; the data is
//! `content/quest_roll.rs`'s.
//!
//! **The draw**: one keyed stream per quest, [`Rng::quest`] (`quest/<id>`), made as the quest
//! is given, used once, and never kept: nothing about it is saved but the [`Roll`] it drew. The
//! pool is in content order and weighted by the variants' `weight` (1 for a pick); its inputs
//! (the content and the hidden locations revealed) are all in the state, so a replay draws the
//! same. There are no rerolls: the roll stands until the quest is done.
//!
//! **Reach**: a pick of creatures or tags draws only from the branches the player can enter
//! when it is given ([`World::branch_open`]): a branch is closed while a hidden location of the
//! run's overworld that leads into it is not revealed. A roll that is never drawn again can so
//! never point at a place the player cannot go; a rolled quest whose pool is empty is not given.

use super::{ActorKind, Spent, World};
use crate::content::{Candidate, QuestDef, Roll, Rolled};
use crate::rng::Rng;

impl World {
    /// Whether the player can enter dungeon branch `branch`: no hidden location of the run's
    /// overworld leads into it, or each that does has been revealed.
    pub(super) fn branch_open(&self, branch: &str) -> bool {
        let Some(overworld) = self.content.overworld_of(&self.state.campaign) else {
            return true;
        };
        (overworld.hidden.values()).all(|h| {
            let into = (h.to.as_deref()).is_some_and(|to| self.content.is_in(to, branch));
            !into || self.state.revealed.contains(&h.id)
        })
    }

    /// What rolled quest `def` may draw now, in content order.
    pub(super) fn quest_pool(&self, def: &QuestDef) -> Vec<Candidate> {
        def.candidates(&self.content, &|b| self.branch_open(b))
    }

    /// Draws rolled quest `def`'s target from its pool, as it is given; `None` when the pool is
    /// empty (it is then not offered) or the quest is not rolled.
    pub(super) fn roll_quest(&self, def: &QuestDef) -> Option<Roll> {
        let pool = self.quest_pool(def);
        let total: u32 = pool.iter().map(|c| c.weight).sum();
        if total == 0 {
            return None;
        }
        let mut at = Rng::quest(self.state.seed, &def.id).below(total);
        let drawn = pool.iter().find(|c| {
            let here = at < c.weight;
            at = at.saturating_sub(c.weight);
            here
        })?;
        Some(roll_of(def, drawn))
    }

    /// The dev action `quest_roll:ID:TARGET` ([`crate::action::Action::QuestRoll`]): active
    /// rolled quest `id`, with no progress yet, draws `target` instead (a target its pool may
    /// hold, wherever the player can go, or a variant's id). Its kills count from now, and a
    /// variant's reveal fires (one revealed before stays: dev only). Takes no time; refused
    /// otherwise.
    pub(super) fn quest_roll_dev(&mut self, id: &str, target: &str) -> Spent {
        let content = self.content.clone();
        let Some(def) = content.quest_id(id).map(|q| content.quest(q)) else {
            self.say(format!("There is no quest `{id}`."));
            return Spent::Nothing;
        };
        if !def.is_rolled() || !self.state.quests.is_active(def) {
            self.say(format!("{} is no rolled quest you were given.", def.name));
            return Spent::Nothing;
        }
        if self.progress_of(def).0 > 0 {
            self.say(format!("{} has got somewhere already.", def.name));
            return Spent::Nothing;
        }
        let pool = def.candidates(&content, &|_| true);
        let variant_id = |c: &Candidate| match (&def.roll, c.variant) {
            (Some(Rolled::Variants(v)), Some(i)) => Some(v[i].id.as_str()),
            _ => None,
        };
        let Some(drawn) =
            (pool.iter()).find(|c| c.target == target || variant_id(c) == Some(target))
        else {
            self.say(format!("{} cannot draw `{target}`.", def.name));
            return Spent::Nothing;
        };
        let roll = roll_of(def, drawn);
        let quests = &mut self.state.quests;
        if let Some(record) = quests.records.get_mut(&def.id) {
            record.roll = Some(roll);
        }
        let given = quests.given(&content, def).into_owned();
        let kills = quests.kills_for(&content, &given);
        if let Some(record) = quests.records.get_mut(&def.id) {
            record.counted = kills;
        }
        if let Some(reveal) = &given.reveals
            && reveal.when == crate::content::RevealWhen::Given
        {
            self.reveal_location(&reveal.location);
        }
        self.say(format!("{} now asks for {target} (dev).", def.name));
        Spent::NoTime
    }

    /// Each rolled quest the player was given, by id in content order, with its roll and the
    /// size of the pool it could draw from now (the `progress` dump; dev only).
    pub fn quest_rolls(&self) -> Vec<(String, Roll, usize)> {
        (self.content.quests().iter())
            .filter_map(|def| {
                let roll = self.state.quests.record(&def.id)?.roll.clone()?;
                Some((def.id.clone(), roll, self.quest_pool(def).len()))
            })
            .collect()
    }

    /// Quarry sense (the bounties system, design §11): arriving on level `level`, a note when a
    /// living creature there is one an active kill quest of the player's still asks for (a
    /// type, a tag's member, a named monster).
    pub(super) fn sense_quarry(&mut self, level: &str) {
        if !self.state.player().talent_mods(&self.content).quarry_sense {
            return;
        }
        let content = self.content.clone();
        let wanted: Vec<QuestDef> = (content.quests().iter())
            .filter(|def| self.state.quests.is_active(def) && !self.ready(def))
            .map(|def| self.state.quests.given(&content, def).into_owned())
            .filter(|def| def.kill_target().is_some())
            .collect();
        let here = (self.state.actors.values()).any(|a| {
            a.kind == ActorKind::Creature
                && a.level == level
                && wanted
                    .iter()
                    .any(|def| def.counts_kill(&content, a.creature, level))
        });
        if here {
            self.note("Your quarry is here.");
        }
    }
}

/// The roll of quest `def` that draws `drawn`.
fn roll_of(def: &QuestDef, drawn: &Candidate) -> Roll {
    let variant = match (&def.roll, drawn.variant) {
        (Some(Rolled::Variants(v)), Some(i)) => Some(v[i].id.clone()),
        _ => None,
    };
    Roll {
        target: drawn.target.clone(),
        variant,
        count: def.count(),
    }
}
