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
//!
//! **Beast-friend** (#1189): a player with the talent is never asked to kill what it calms
//! ([`beast_friend::calms`]: a creature type of an animal faction). The pool leaves out such a
//! type, a tag any of whose types (of the quest's campaign) is one, and a variant asking for
//! either; a `fallback` variant is then drawn only when no other is left (Leofric's rats). The
//! talent is read as the quest is given: the roll is saved, so a talent taken later changes
//! nothing about a quest given already.

use super::{Spent, World, beast_friend};
use crate::content::{Candidate, QuestDef, Roll, Rolled, TAG_TARGET};
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

    /// What rolled quest `def` may draw now, in content order: of the branches the player can
    /// enter, nothing their Beast-friend talent calms, and a fallback variant only when no
    /// other is left.
    pub(super) fn quest_pool(&self, def: &QuestDef) -> Vec<Candidate> {
        let mut pool = def.candidates(&self.content, &|b| self.branch_open(b));
        pool.retain(|c| !self.calmed(def, &c.target));
        let fallback = |c: &Candidate| match (&def.roll, c.variant) {
            (Some(Rolled::Variants(v)), Some(i)) => v[i].fallback,
            _ => false,
        };
        if pool.iter().any(|c| !fallback(c)) {
            pool.retain(|c| !fallback(c));
        }
        pool
    }

    /// Whether kill target `target` (a creature type's id, or `tag:` and a tag of quest `def`'s
    /// campaign) is one the player's Beast-friend talent calms, or any type of the tag is.
    fn calmed(&self, def: &QuestDef, target: &str) -> bool {
        let (content, player) = (&self.content, self.state.player());
        let calms = |c| beast_friend::calms(content, player, c);
        match target.strip_prefix(TAG_TARGET) {
            Some(tag) => content.tagged_in(tag, def.campaign.as_deref()).any(calms),
            None => content.creature_id(target).is_some_and(calms),
        }
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
