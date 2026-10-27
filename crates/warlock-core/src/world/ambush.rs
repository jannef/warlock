//! Ambusher (spec §8 "Talents") and the thief's Sneak attack (spec §8 "Class abilities"): an
//! attack on a creature that has not seen its attacker does the talents' dice more damage, after
//! Protection (Sneak attack's and Improved sneak attack's dice add up, `Dice::and`), and with
//! Master thief threatens a critical sooner. "Has not seen" is the creature's own: it hunts no
//! one it has seen ([`super::Mind::last_seen`] unset) and its eyes, by creature sight
//! ([`crate::sight`]), do not see the attacker's cell now. It is asked before the strike, only
//! of an attacker with the dice (no draw); the dice are drawn from the gameplay stream only when
//! a hit lands on an unaware creature.
//!
//! The ranger's Favoured prey is here too: against the creature type the player has killed most
//! ([`super::Quests::prey`]), its attacks hit likelier and do more damage.

use super::{ActorId, ActorKind, World};
use crate::combat::{Blow, Profile};
use crate::content::Build;
use crate::fov::radius2;
use crate::sight::{self, LineOfSight};

impl World {
    /// Whether `attacker` falls on `target` unseen with its ambush dice: it has them, and the
    /// creature has not seen it.
    pub(super) fn sneaking(&mut self, attacker: ActorId, target: ActorId) -> bool {
        let has = (self.state.actors.get(&attacker)).is_some_and(|a| {
            !a.talents.is_empty() && a.talent_mods(&self.content).ambush.is_some()
        });
        has && self.unaware(target, attacker)
    }

    /// `profile` of an attack by `attacker`, `unseen` (it is [`World::sneaking`]), with Master
    /// thief's lower threat (never below 3).
    pub(super) fn sneak_threat(
        &self,
        attacker: ActorId,
        unseen: bool,
        mut profile: Profile,
    ) -> Profile {
        if !unseen {
            return profile;
        }
        if let Some(a) = self.state.actors.get(&attacker) {
            let lower = a.talent_mods(&self.content).ambush_threat;
            profile.threat = profile.threat.saturating_sub(lower).max(3);
        }
        profile
    }

    /// `blow` by `attacker`, `unseen` (it is [`World::sneaking`]), with the ambush dice added
    /// when it hit.
    pub(super) fn ambushed(&mut self, attacker: ActorId, unseen: bool, blow: Blow) -> Blow {
        let Some(dice) =
            (self.state.actors.get(&attacker)).and_then(|a| a.talent_mods(&self.content).ambush)
        else {
            return blow;
        };
        if !unseen || !blow.strike.is_hit() {
            return blow;
        }
        let extra = dice.roll(&mut self.state.rng).max(0);
        // A sneak attack that hits builds the attacker's resources (spec §8 "Resources").
        self.gain_resource(attacker, Build::SneakAttack);
        Blow {
            damage: blow.damage + extra,
            ..blow
        }
    }

    /// `profile` of an attack by `attacker` at `target`, with Favoured prey's to-hit and damage
    /// when the player makes it at a creature of the type they have killed most, and Sworn
    /// foe's at one whose type carries the tag it was taken for (each tag sworn adds).
    pub(super) fn favoured(
        &self,
        attacker: ActorId,
        target: ActorId,
        mut profile: Profile,
    ) -> Profile {
        if attacker != self.state.player {
            return profile;
        }
        let me = self.state.player();
        if me.talents.is_empty() {
            return profile;
        }
        let mods = me.talent_mods(&self.content);
        let creature = self.state.actors.get(&target).map(|a| a.creature);
        for (tag, to_hit, damage) in &mods.sworn_foe {
            if creature.is_some_and(|c| self.content.has_tag(c, tag)) {
                profile.to_hit += to_hit;
                profile.bonus += damage;
            }
        }
        let (to_hit, damage) = mods.favoured_prey;
        if (to_hit, damage) == (0, 0) {
            return profile;
        }
        let prey = self.state.quests.prey();
        if creature.is_some_and(|c| prey == Some(self.content.family_key(c))) {
            profile.to_hit += to_hit;
            profile.bonus += damage;
        }
        profile
    }

    /// Whether creature `id` has not seen actor `other`: it hunts no one it has seen, and it does
    /// not see `other`'s cell now. Never for the player.
    pub(super) fn unaware(&mut self, id: ActorId, other: ActorId) -> bool {
        let Some(me) = self.state.actors.get(&id) else {
            return false;
        };
        // Asleep, it has seen no one (`den.rs`).
        if me.kind == ActorKind::Creature && self.asleep(id) {
            return self.state.actors.contains_key(&other);
        }
        if me.kind != ActorKind::Creature || me.mind.last_seen.is_some() {
            return false;
        }
        self.state.actors.contains_key(&other) && !self.sees_now(id, other)
    }

    /// Whether creature `id`'s eyes see actor `other`'s cell now, by creature sight
    /// ([`crate::sight`]): within its sight radius, in its line of sight, light enough for its
    /// eyes (the Transfixing stare asks it). Never for the player.
    pub(super) fn sees_now(&mut self, id: ActorId, other: ActorId) -> bool {
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let (Some(me), Some(them)) = (state.actors.get(&id), state.actors.get(&other)) else {
            return false;
        };
        if me.kind != ActorKind::Creature {
            return false;
        }
        let at = them.pos;
        let def = content.creature(me.creature);
        let radius = sight::sight_radius(state, content, def.sight);
        let d2 = me.pos.dist2(at);
        if d2 > radius2(radius) {
            return false;
        }
        let level = state.current();
        let reach = LineOfSight::radius_for(d2).min(radius);
        let los = LineOfSight::compute(me.pos, reach, |p| level.is_opaque(content, p));
        let eye = me.eyes(content, &content.vision_profiles()[def.vision]);
        sight::sees(state, content, lights, &los, &eye, at)
    }
}
