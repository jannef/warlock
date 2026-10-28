//! Sneak attacks (spec §8 "Surprise and sneak attacks"; `docs/dev/surprise.md`): an attack on a
//! target that has not seen its attacker, or is Surprised, does the attacker's sneak dice more
//! damage, after Protection: the Sneak attack skill's d6s ([`Effect::SneakDice`]) and its
//! talents' ambush dice on top (Ambusher, Improved sneak attack; `Dice::and`), and with Master
//! thief threatens a critical sooner. A blow counts at any reach; a shot, a throw, or an attack
//! spell within `missile_reach` cells (Long reach's farther). Nothing lands on a target with no
//! vitals for the attacker ([`World::has_vitals_for`], `Content::has_vitals`: also immune to
//! criticals, `combat::Target::no_criticals`). "Has not seen" is [`World::unaware`]
//! (`surprise.rs`). It is asked before the strike (no draw); the dice are drawn from the gameplay
//! stream only when a hit lands, and the player's sneak attack marks Sneak attack and Dexterity
//! (training stream).
//!
//! The ranger's Favoured prey is here too: against the creature type the player has killed most
//! ([`super::Quests::prey`]), its attacks hit likelier and do more damage.

use super::{ActorId, World};
use crate::combat::{Blow, Profile, Target};
use crate::content::Build;
use crate::dice::Dice;
use crate::skills::{Effect, Use};

impl World {
    /// Whether `attacker` falls on `target` unseen with its sneak dice, a blow or (`missile`)
    /// not (tests ask it; `resolve` asks [`World::sneaks`], having asked `unaware` itself).
    #[cfg(test)]
    pub(super) fn sneaking(&mut self, attacker: ActorId, target: ActorId, missile: bool) -> bool {
        let unaware = self.unaware(target, attacker);
        self.sneaks(attacker, target, unaware, missile)
    }

    /// Whether `attacker`'s attack on `target`, a blow or (`missile`) a shot, a throw, or an
    /// attack spell, is a sneak attack, with whether `target` is `unaware` of it already asked:
    /// the attacker has sneak dice, the target is unaware or Surprised, has vitals for the
    /// attacker, and a missile flies no farther than the rules' `missile_reach` plus the
    /// attacker's Long reach (no limit, and vitals everywhere, without the surprise rules).
    pub(super) fn sneaks(
        &self,
        attacker: ActorId,
        target: ActorId,
        unaware: bool,
        missile: bool,
    ) -> bool {
        if !(unaware || self.surprised(target)) || self.sneak_dice(attacker).is_none() {
            return false;
        }
        if !self.has_vitals_for(attacker, target) {
            return false;
        }
        let (Some(a), Some(t), Some(rules)) = (
            self.state.actors.get(&attacker),
            self.state.actors.get(&target),
            self.content.surprise(),
        ) else {
            return true;
        };
        let reach = rules.missile_reach + a.talent_mods(&self.content).sneak_reach;
        !missile || a.pos.chebyshev(t.pos) <= reach
    }

    /// The dice actor `id`'s sneak attacks add: its Sneak attack skill's d6s and its talents'
    /// ambush dice, together; `None` with neither.
    pub(super) fn sneak_dice(&self, id: ActorId) -> Option<Dice> {
        let a = self.state.actors.get(&id)?;
        let count = a.skill_effect(&self.content, Effect::SneakDice);
        let skill = u32::try_from(count)
            .ok()
            .filter(|&n| n > 0)
            .map(|n| Dice::of(n, 6));
        let talents = a.talent_mods(&self.content).ambush;
        match (skill, talents) {
            (Some(skill), Some(talents)) => Some(skill.and(talents)),
            (skill, talents) => skill.or(talents),
        }
    }

    /// Whether actor `target` has vitals to find for `attacker`'s attacks
    /// (`Content::has_vitals`, with the attacker's Anatomist tags).
    pub(super) fn has_vitals_for(&self, attacker: ActorId, target: ActorId) -> bool {
        let Some(t) = self.state.actors.get(&target) else {
            return true;
        };
        let mods = (self.state.actors.get(&attacker)).map(|a| a.talent_mods(&self.content));
        let vitals_in = mods.as_ref().map_or(&[][..], |m| &m.vitals_in[..]);
        self.content.has_vitals(t.creature, vitals_in)
    }

    /// A blow of the player's that would be a critical whenever it hits (Vital strike, Master
    /// thief's finisher) hit a target that had no vitals (`vitalless`, asked before the blow):
    /// it was a plain blow, and the player is told so after its line.
    pub(super) fn no_vital_spot(&mut self, vitalless: bool, blow: Option<Blow>) {
        if vitalless && blow.is_some_and(|b| b.strike.is_hit()) {
            self.news.push("It has no vital spot.".into());
        }
    }

    /// `struck`, the side of `target` an attack by `attacker` strikes, immune to criticals when
    /// it has no vitals for the attacker.
    pub(super) fn vitals(&self, attacker: ActorId, target: ActorId, struck: Target) -> Target {
        Target {
            no_criticals: !self.has_vitals_for(attacker, target),
            ..struck
        }
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

    /// `blow` by `attacker`, `unseen` (it is [`World::sneaking`]), with the sneak dice added
    /// when it hit.
    pub(super) fn ambushed(&mut self, attacker: ActorId, unseen: bool, blow: Blow) -> Blow {
        if !unseen || !blow.strike.is_hit() {
            return blow;
        }
        let Some(dice) = self.sneak_dice(attacker) else {
            return blow;
        };
        let extra = dice.roll(&mut self.state.rng).max(0);
        // A sneak attack that hits builds the attacker's resources (spec §8 "Resources").
        self.gain_resource(attacker, Build::SneakAttack);
        // And marks the player's Sneak attack and Dexterity (training stream).
        if attacker == self.state.player && self.trains(Use::SneakAttack) {
            self.train(Use::SneakAttack);
        }
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
