//! Ambusher (spec §8 "Talents") and the thief's Sneak attack (spec §8 "Class abilities"): an
//! attack on a target that has not seen its attacker, or is Surprised, does the talents' dice
//! more damage, after Protection (Sneak attack's and Improved sneak attack's dice add up,
//! `Dice::and`), and with Master thief threatens a critical sooner. "Has not seen" is
//! [`World::unaware`] (`surprise.rs`, spec §8 "Surprise and sneak attacks"). It is asked before
//! the strike (no draw); the dice are drawn from the gameplay stream only when a hit lands.
//!
//! The ranger's Favoured prey is here too: against the creature type the player has killed most
//! ([`super::Quests::prey`]), its attacks hit likelier and do more damage.

use super::{ActorId, World};
use crate::combat::{Blow, Profile};
use crate::content::Build;

impl World {
    /// Whether `attacker` falls on `target` unseen with its ambush dice: it has them, and the
    /// target has not seen it ([`World::unaware`], `surprise.rs`) or is Surprised (tests ask it;
    /// `resolve` asks [`World::sneaks`], having asked `unaware` itself).
    #[cfg(test)]
    pub(super) fn sneaking(&mut self, attacker: ActorId, target: ActorId) -> bool {
        let unaware = self.unaware(target, attacker);
        self.sneaks(attacker, target, unaware)
    }

    /// [`World::sneaking`], with whether `target` is `unaware` of `attacker` already asked.
    pub(super) fn sneaks(&self, attacker: ActorId, target: ActorId, unaware: bool) -> bool {
        let has = (self.state.actors.get(&attacker)).is_some_and(|a| {
            !a.talents.is_empty() && a.talent_mods(&self.content).ambush.is_some()
        });
        has && (unaware || self.surprised(target))
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
}
