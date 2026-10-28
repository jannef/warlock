//! What a caster's stats do to a spell (`[spell.scales]`, `content/spell_scales.rs`; the combat
//! progression design §7, #556): the lift of its cap, its to-hit, its reach's cells and its
//! status's degrees more. Derived at cast time from [`Actor::effective_stats`]; nothing saved.
//!
//! The lift and the to-hit scales are a caster's own spells' (a cast paid in mana); a scroll read
//! or a wand used casts at the item's own power, with the unlifted cap and Dexterity's aim
//! (`world/cast.rs`). Reach and duration are the reader's or user's as for an own spell.
//!
//! What the caster wears joins each scale at its one read (the affixes design §4; global
//! effects, unidentified too): `spell_lift` (Potent) widens the lift with Keen mind's,
//! `spell_to_hit` (of the Magus) and `spell_threat` (Baleful) aim an own cast,
//! `spell_duration` (of Lingering) adds degrees; `spell_reach` (of Farcasting) is
//! [`Actor::spell_reach`]'s, beside School focus's.

use super::Actor;
use crate::combat;
use crate::content::{Content, Effect as ItemEffect, SpellData};

impl Actor {
    /// The percent its stats lift `spell`'s cap by: its `damage` scales' sum, never below 0,
    /// never past the tier's `max_lift` plus Keen mind's for the spell's school or element and
    /// its magic items' `spell_lift` (Potent), which widen it together.
    pub fn spell_lift(&self, content: &Content, spell: &SpellData) -> i32 {
        let Some(rules) = content.spell_rules() else {
            return 0;
        };
        if spell.scales.damage.is_empty() {
            return 0;
        }
        let raised = spell.scales.damage.sum(&self.effective_stats(content));
        let more = self.talent_mods(content).focus_lift(&spell.tags(content))
            + self.mods(content).get(ItemEffect::SpellLift).max(0);
        rules.lift(spell.tier, raised, more)
    }

    /// The most a cast of `spell` by it does to one target: the spell's cap lifted by
    /// [`Actor::spell_lift`].
    pub fn spell_cap(&self, content: &Content, spell: &SpellData) -> i32 {
        content.spell_rules().map_or(0, |rules| {
            rules.lifted_cap(spell, self.spell_lift(content, spell))
        })
    }

    /// What its stats add to the attack roll of an `attack` or `touch` `spell` of its own: its
    /// `to_hit` scales (Dexterity's modifier unless the spell names others), and its magic
    /// items' `spell_to_hit` (of the Magus).
    pub fn spell_to_hit(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.to_hit.signed(&self.effective_stats(content))
            + self.mods(content).get(ItemEffect::SpellToHit)
    }

    /// The natural roll from which an `attack` or `touch` spell of its own threatens a critical
    /// ([`combat::DEFAULT_THREAT`] for anyone): 22 less its magic items' `spell_threat`
    /// (Baleful), never below the blows' floor, [`combat::THREATS`]. A scroll read or a wand
    /// used keeps 22.
    pub fn spell_threat(&self, content: &Content) -> u8 {
        let wider = self.mods(content).get(ItemEffect::SpellThreat).max(0);
        let threat = i32::from(combat::DEFAULT_THREAT) - wider;
        u8::try_from(threat.max(i32::from(*combat::THREATS.start())))
            .unwrap_or(combat::DEFAULT_THREAT)
    }

    /// The cells more its stats give `spell`'s reach (range, length, a burst's radius): its
    /// `reach` scales (Willpower's unless the spell names others), never past the spell's caps.
    pub fn spell_reach_more(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.reach.steps(&self.effective_stats(content))
    }

    /// The degrees more its stats give `spell`'s buff or hex: its `duration` scales
    /// (Presence's unless the spell names others), and its magic items' `spell_duration` (of
    /// Lingering); the status's own `max` still holds them.
    pub fn spell_degrees_more(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.duration.steps(&self.effective_stats(content))
            + self.mods(content).get(ItemEffect::SpellDuration).max(0)
    }
}
