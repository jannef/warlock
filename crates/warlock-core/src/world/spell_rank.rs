//! The rank a caster knows a spell at (spec §8 "Spells": "Ranks"; #1264), and what it gives
//! their casts: rank 1 as it is learned (or as a class gives it) to
//! [`MAX_SPELL_RANK`], raised by reading its grimoire again (`learn.rs`). The rank is kept in
//! [`Actor::spells`] beside the spell and saved; a spell a class grants that was never deepened
//! is not there and is at rank 1.
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
            Some(&rank) => Some(rank.clamp(1, MAX_SPELL_RANK)),
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
}
