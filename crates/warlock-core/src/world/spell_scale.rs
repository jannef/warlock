//! What a caster's stats do to a spell (`[spell.scales]`, `content/spell_scales.rs`; the combat
//! progression design §7, #556): the lift of its cap, its to-hit, its reach's cells and its
//! status's degrees more. Derived at cast time from [`Actor::effective_stats`]; nothing saved.
//!
//! The lift and the to-hit scales are a caster's own spells' (a cast paid in mana); a scroll read
//! or a wand used casts at the item's own power, with the unlifted cap and Dexterity's aim
//! (`world/cast.rs`). Reach and duration are the reader's or user's as for an own spell.

use super::Actor;
use crate::content::{Content, SpellData};

impl Actor {
    /// The percent its stats lift `spell`'s cap by: its `damage` scales' sum, never below 0,
    /// never past the tier's `max_lift` plus Keen mind's for the spell's school or element.
    pub fn spell_lift(&self, content: &Content, spell: &SpellData) -> i32 {
        let Some(rules) = content.spell_rules() else {
            return 0;
        };
        if spell.scales.damage.is_empty() {
            return 0;
        }
        let raised = spell.scales.damage.sum(&self.effective_stats(content));
        let more = self.talent_mods(content).focus_lift(&spell.tags(content));
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
    /// `to_hit` scales (Dexterity's modifier unless the spell names others).
    pub fn spell_to_hit(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.to_hit.signed(&self.effective_stats(content))
    }

    /// The cells more its stats give `spell`'s reach (range, length, a burst's radius): its
    /// `reach` scales (Willpower's unless the spell names others), never past the spell's caps.
    pub fn spell_reach_more(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.reach.steps(&self.effective_stats(content))
    }

    /// The degrees more its stats give `spell`'s buff or hex: its `duration` scales
    /// (Presence's unless the spell names others).
    pub fn spell_degrees_more(&self, content: &Content, spell: &SpellData) -> i32 {
        spell.scales.duration.steps(&self.effective_stats(content))
    }
}
