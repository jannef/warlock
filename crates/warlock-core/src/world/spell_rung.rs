//! The rung a caster casts a spell at, and what the spell is then (spec §8 "Spells": "Ladders";
//! the spell scaling design §4.3, §7.1): derived at cast time from
//! [`Actor::effective_stats`], its talents, and its worn items ([`Actor::spell_mods`]); nothing
//! is saved.
//!
//! - **The rung**: `by_stats` is how many rungs of a ladder the caster's effective score in the
//!   ladder's stat reaches; `rung = min(by_stats + rungs, ladder length)`, never below 0 (the
//!   base). A scroll read or a wand used counts no stats for its roll: its own rungs plus the
//!   reader's `scroll_rungs` or `wand_rungs` ([`CastBy`]). Reach and degrees are always the
//!   caster's own, by stats.
//! - **The roll** ([`Actor::spell_roll`]): the ladder's value at the rung (the base at 0), then
//!   the dice hook, `grow(sides).more(dice).plus(bonus)`. What the abilities menu will show.
//! - **The reach** ([`Actor::spell_reach`]): the range or length ladder's value (`reach_rungs`
//!   on top of the stats' rungs), then `cells`; the radius ladder's, then `radius_cells`. Held
//!   only by the engine's limits: [`MAX_REACH`] (a cone [`MAX_CONE`]), the burst table's length.
//! - **The degrees** ([`Actor::spell_degrees`]): the degrees ladder's value (`degree_rungs` on
//!   top), then `degrees`; the status's own `max` holds it as it is given (`Effects::add`).
//! - **The aim** ([`Actor::spell_to_hit`]): the spell's `aim` stat's modifier (Dexterity unless
//!   it names another) and the worn `spell_to_hit` (of the Magus).
//!
//! Integers only: a count, a sum, a minimum. Nothing holds what the hooks add (decision 16).

use super::Actor;
use super::cast::Reach;
use crate::combat;
use crate::content::{
    Content, Effect as ItemEffect, MAX_CONE, MAX_REACH, Shape, SpellData, SpellLadder as Ladder,
    SpellMods,
};
use crate::dice::Roll;
use crate::stats::{Stat, Stats};

/// Who casts a spell, for the rung of its roll: the caster's own cast (by its stats and rungs), a
/// spell scroll read at its own rungs, or a wand used at its own rungs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastBy {
    Own,
    Scroll(u8),
    Wand(u8),
}

/// The rung of `ladder` a caster with `stats` reaches with `more` rungs from outside: the
/// rungs its score reaches plus `more`, never below 0 nor past the last.
fn climb(ladder: &Ladder, stats: &Stats, more: i32) -> usize {
    let by_stats = ladder.rung_at(i32::from(stats.score(ladder.stat)));
    rung_of(ladder, by_stats as i32 + more)
}

/// Rung `n` of `ladder`, never below 0 nor past the last.
fn rung_of(ladder: &Ladder, n: i32) -> usize {
    usize::try_from(n.max(0)).unwrap_or(0).min(ladder.len())
}

/// `base`, or `ladder`'s value at `rung` if it has one there.
fn number_at(ladder: Option<&Ladder>, rung: usize, base: i32) -> i32 {
    (ladder.and_then(|l| l.value(rung)))
        .and_then(|v| v.number())
        .unwrap_or(base)
}

impl Actor {
    /// The rung of `spell`'s roll ladder it casts at as `by` (module docs) with `mods`; 0 for a
    /// spell with no such ladder.
    pub fn spell_rung(
        &self,
        content: &Content,
        spell: &SpellData,
        by: CastBy,
        mods: &SpellMods,
    ) -> usize {
        let Some(ladder) = spell.grows.roll() else {
            return 0;
        };
        match by {
            CastBy::Own => climb(ladder, &self.effective_stats(content), mods.rungs),
            CastBy::Scroll(own) => rung_of(ladder, i32::from(own) + mods.scroll_rungs),
            CastBy::Wand(own) => rung_of(ladder, i32::from(own) + mods.wand_rungs),
        }
    }

    /// The roll `spell` makes as it casts it as `by` (module docs), harm or healing; `None` for
    /// a spell that rolls nothing.
    pub fn spell_roll(&self, content: &Content, spell: &SpellData, by: CastBy) -> Option<Roll> {
        self.spell_roll_with(content, spell, by, &self.spell_mods(content, spell))
    }

    /// [`Actor::spell_roll`] with its `mods` already summed.
    pub(super) fn spell_roll_with(
        &self,
        content: &Content,
        spell: &SpellData,
        by: CastBy,
        mods: &SpellMods,
    ) -> Option<Roll> {
        let rung = self.spell_rung(content, spell, by, mods);
        let roll = spell.roll_at(rung)?;
        Some(roll.grow(mods.sides).more(mods.dice).plus(mods.bonus))
    }

    /// How far `spell` reaches as it casts it (module docs): the farthest cell it may be aimed
    /// at (a touch's 1), and a burst's radius; each `None` where the spell has no such reach.
    pub fn spell_reach(&self, content: &Content, spell: &SpellData) -> Reach {
        let Some(rules) = content.spell_rules() else {
            return Reach::default();
        };
        let mods = self.spell_mods(content, spell);
        let stats = self.effective_stats(content);
        let grown = |base: Option<u8>, ladder: Option<&Ladder>, more: i32, most: u8| {
            let base = i32::from(base?);
            let rung = ladder.map_or(0, |l| climb(l, &stats, mods.reach_rungs));
            let cells = number_at(ladder, rung, base) + more;
            Some(cells.clamp(1, i32::from(most)))
        };
        let g = &spell.grows;
        let range = match spell.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Cone => grown(spell.length, g.length.as_ref(), mods.cells, MAX_CONE),
            Shape::Line => grown(spell.length, g.length.as_ref(), mods.cells, MAX_REACH),
            Shape::Ray | Shape::Bolt | Shape::Burst => {
                grown(spell.range, g.range.as_ref(), mods.cells, MAX_REACH)
            }
        };
        Reach {
            range,
            radius: grown(
                spell.radius,
                g.radius.as_ref(),
                mods.radius_cells,
                rules.max_radius(),
            ),
        }
    }

    /// The degrees of `spell`'s buff or hex status as it casts it (module docs): its base, the
    /// degrees ladder's value, then `degrees`; at least 1. `None` for a spell with no such
    /// status.
    pub fn spell_degrees(&self, content: &Content, spell: &SpellData) -> Option<u16> {
        let (_, base) = spell.effect.status?;
        let mods = self.spell_mods(content, spell);
        let ladder = spell.grows.degrees.as_ref();
        let stats = self.effective_stats(content);
        let rung = ladder.map_or(0, |l| climb(l, &stats, mods.degree_rungs));
        let degrees = number_at(ladder, rung, i32::from(base)) + mods.degrees;
        Some(u16::try_from(degrees.max(1)).unwrap_or(u16::MAX))
    }

    /// What it adds to the attack roll of an `attack` or `touch` `spell` of its own: the
    /// spell's `aim` stat's modifier (Dexterity unless it names another), and its magic items'
    /// `spell_to_hit` (of the Magus). A scroll's or a wand's aim is Dexterity's alone.
    pub fn spell_to_hit(&self, content: &Content, spell: &SpellData) -> i32 {
        self.effective_stats(content).modifier(spell.aim)
            + self.mods(content).get(ItemEffect::SpellToHit)
    }

    /// The to-hit of a scroll read or a wand used: Dexterity's modifier.
    pub fn item_spell_to_hit(&self, content: &Content) -> i32 {
        self.effective_stats(content).modifier(Stat::Dexterity)
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
}
