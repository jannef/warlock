//! On a kill (the affixes design, `docs/design/affixes-systems.md` §4, §5, §8, decision 7;
//! #632): of Siphoning's mana (`mana_on_kill`) and of the Hunt's point of a pool
//! (`resource_on_kill`, keyed by the pool the affix names), read from what the slayer wears and
//! wields. **The limit**: only a kill by the wearer's own blow, shot, throw, or spell, the slain
//! within [`SIPHON_RANGE`] cells, counts. The one caller is the kill point of an attack, where
//! both actors' cells are known: `fight.rs::resolve` (a blow, a shot, a throw, a creature's
//! spell) and `spell.rs::spell_strikes` (a spell of the player's, a scroll's, a breath). A death
//! by a status tick (poison, bleeding), a trap, a fall, or hunger goes through `dies` alone and
//! never reaches it; a summoned creature's kill is its own (the summoner wears nothing of it).
//! Creatures' kills count under the same rule. The data's `builds` and the talents' (a kill
//! builds Inspiration, Second breath) stay in `dies`, for any kill. No draw, no saved state: a
//! slayer wearing none of these changes nothing.

use super::{ActorId, World};
use crate::content::{Effect as ItemEffect, ResourceId};
use crate::fov::radius2;
use crate::geom::Pos;

/// How far the slain may stand from the slayer for the on-kill affixes to count, in cells
/// (Euclidean, as every radius: [`radius2`]): melee always, a shot or a spell at close quarters.
/// The player is told "close by"; the wiki says three.
pub const SIPHON_RANGE: i32 = 3;

impl World {
    /// Actor `by`'s own blow, shot, or spell killed actor `slain`, which stood at `at`: when
    /// that was within [`SIPHON_RANGE`] of `by`, its worn `mana_on_kill` raises its mana, never
    /// past its maximum, and its worn `resource_on_kill` raises each pool it holds that the
    /// affix names, never past that pool's maximum (an affix never grants a pool).
    pub(super) fn siphon(&mut self, by: ActorId, (slain, at): (ActorId, Pos)) {
        if by == slain {
            return;
        }
        let level = self.holder_level(by);
        let Some(actor) = self.state.actors.get(&by) else {
            return;
        };
        if actor.pos.dist2(at) > radius2(SIPHON_RANGE) {
            return;
        }
        let mods = actor.mods(&self.content);
        let mana = mods.get(ItemEffect::ManaOnKill);
        let pools: Vec<(ResourceId, u16, u16)> = (actor.resources.iter())
            .filter_map(|pool| {
                let more = mods.resource(ItemEffect::ResourceOnKill, pool.resource);
                let more = u16::try_from(more).ok().filter(|&m| m > 0)?;
                let max = actor.resource_max(&self.content, pool.resource, level);
                Some((pool.resource, more, max))
            })
            .collect();
        if mana <= 0 && pools.is_empty() {
            return;
        }
        let max_mana = if mana > 0 {
            actor.max_mana(&self.content)
        } else {
            0
        };
        let Some(actor) = self.state.actors.get_mut(&by) else {
            return;
        };
        if mana > 0 {
            actor.mana = actor
                .mana
                .max(actor.mana.saturating_add(mana).min(max_mana));
        }
        for (resource, more, max) in pools {
            if let Some(pool) = actor.resources.get_mut(resource) {
                pool.value = pool.value.max(pool.value.saturating_add(more).min(max));
            }
        }
    }
}
