//! Ambusher (spec §8 "Talents"): an attack on a creature that has not seen its attacker does
//! the talent's dice more damage, after Protection. "Has not seen" is the creature's own: it
//! hunts no one it has seen ([`super::Mind::last_seen`] unset) and its eyes, by creature sight
//! ([`crate::sight`]), do not see the attacker's cell now. The dice are drawn from the gameplay
//! stream only when a hit lands with the talent on an unaware creature.

use super::{ActorId, ActorKind, World};
use crate::combat::Blow;
use crate::fov::radius2;
use crate::sight::{self, LineOfSight};

impl World {
    /// `blow` by `attacker` at `target` with the Ambusher's damage added when it hit a creature
    /// that has not seen the attacker.
    pub(super) fn ambushed(&mut self, attacker: ActorId, target: ActorId, blow: Blow) -> Blow {
        let Some(dice) =
            (self.state.actors.get(&attacker)).and_then(|a| a.talent_mods(&self.content).ambush)
        else {
            return blow;
        };
        if !blow.strike.is_hit() || !self.unaware(target, attacker) {
            return blow;
        }
        let extra = dice.roll(&mut self.state.rng).max(0);
        Blow {
            damage: blow.damage + extra,
            ..blow
        }
    }

    /// Whether creature `id` has not seen actor `other`: it hunts no one it has seen, and it does
    /// not see `other`'s cell now. Never for the player.
    pub(super) fn unaware(&mut self, id: ActorId, other: ActorId) -> bool {
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let (Some(me), Some(them)) = (state.actors.get(&id), state.actors.get(&other)) else {
            return false;
        };
        if me.kind != ActorKind::Creature || me.mind.last_seen.is_some() {
            return false;
        }
        let at = them.pos;
        let def = content.creature(me.creature);
        let radius = sight::sight_radius(state, content, def.sight);
        let d2 = me.pos.dist2(at);
        if d2 > radius2(radius) {
            return true;
        }
        let level = state.current();
        let reach = LineOfSight::radius_for(d2).min(radius);
        let los = LineOfSight::compute(me.pos, reach, |p| level.is_opaque(content, p));
        let eye = &content.vision_profiles()[def.vision];
        !sight::sees(state, content, lights, &los, eye, at)
    }
}
