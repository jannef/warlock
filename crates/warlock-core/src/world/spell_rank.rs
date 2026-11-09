//! The rank a caster knows a spell at (spec §8 "Spells": "Ranks"; #1264), and what it gives
//! their casts: rank 1 as it is learned (or as a class gives it) to
//! [`MAX_SPELL_RANK`], raised by reading its grimoire again (`learn.rs`). The rank is kept in
//! [`Actor::spells`] beside the spell and saved; a spell a class grants that was never deepened
//! is not there and is at rank 1.
//!
//! **Rank 0 is forgotten** (a miscast's `forget`, #1289): [`Actor::forget_spell`] removes a
//! learned spell's entry, and writes rank 0 for one the class grants, which no other value
//! could say; [`Actor::knows_spell`] and [`Actor::spell_rank`] read it as not known, so it is
//! neither listed nor cast. It is learned again only from its grimoire, as any spell is
//! (`learn.rs`), at rank 1.
//!
//! What a rank gives is the spell's own `[spell.ranks]` table ([`crate::content::Ranks`]):
//! [`Actor::rank_mods`] is its sum up to the caster's rank, one more source of [`SpellMods`]
//! joined in [`Actor::spell_mods`] for an own cast only. A scroll read or a wand used casts at
//! rank 1 ([`super::CastBy`]), and casting trains no rank: nothing but the grimoire raises it.

use super::Actor;
use crate::content::{Content, MAX_SPELL_RANK, SpellData, SpellId, SpellMods};

impl Actor {
    /// The rank it knows `spell` at, 1 to [`MAX_SPELL_RANK`]; `None` for a spell it does not
    /// know.
    pub fn spell_rank(&self, content: &Content, spell: SpellId) -> Option<u8> {
        match self.spells.get(&spell) {
            Some(0) => None,
            Some(&rank) => Some(rank.min(MAX_SPELL_RANK)),
            None => self.knows_spell(content, spell).then_some(1),
        }
    }

    /// What its rank in `spell` gives a cast of its own: the spell's rank table summed up to
    /// its rank; nothing at rank 1, for a spell with no table, or for a spell it does not know.
    pub fn rank_mods(&self, content: &Content, spell: &SpellData) -> SpellMods {
        if spell.ranks.is_empty() {
            return SpellMods::default();
        }
        let Some(rank) = content
            .spell_id(&spell.id)
            .and_then(|id| self.spell_rank(content, id))
        else {
            return SpellMods::default();
        };
        spell.ranks.at(rank)
    }

    /// It knows `spell` at `rank` from now on (a learning at 1, a deepening above).
    pub(super) fn set_spell_rank(&mut self, spell: SpellId, rank: u8) {
        self.spells.insert(spell, rank.clamp(1, MAX_SPELL_RANK));
    }

    /// It forgets `spell` (module docs): a learned spell's entry goes, a class's spell is
    /// marked at rank 0. Nothing for a spell it does not know.
    pub(super) fn forget_spell(&mut self, content: &Content, spell: SpellId) {
        if !self.knows_spell(content, spell) {
            return;
        }
        if self.talent_mods(content).has_spell(spell) {
            self.spells.insert(spell, 0);
        } else {
            self.spells.remove(&spell);
        }
    }
}
