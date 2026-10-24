//! Light-blindness in the world (spec §8 "Character creation"; a race's `light_blind`, drow):
//! strong light on a light-blind player's own cell dazzles them. [`LightBlind::dazzled_by`] is
//! the one rule; the perception reads it for how far the player sees
//! ([`crate::perception`]), [`World::glared`] for their attacks, and [`World::glare_news`] says
//! when it begins and ends. Creatures are never light-blind.

use super::{ActorId, World};
use crate::combat::Profile;
use crate::content::LightBlind;
use crate::sight;

/// Said as strong light begins to dazzle the player.
const DAZZLED: &str = "The bright light dazzles you.";
/// Said as it stops.
const ADJUSTED: &str = "Your eyes adjust to the dimmer light.";

impl World {
    /// What dazzles actor `id`, if strong light on its cell does: the player's race's
    /// [`LightBlind`], when the light there, as their eyes perceive it (the same light their
    /// perception finds), reaches its glare. `None` for anyone else.
    fn dazzles(&mut self, id: ActorId) -> Option<LightBlind> {
        let actor = self.state.actors.get(&id)?;
        let blind = self.state.race_of(&self.content, actor)?.light_blind?;
        let pos = actor.pos;
        let vision = self.content.vision_index(&self.state.vision).unwrap_or(0);
        let eye = &self.content.vision_profiles()[vision];
        let light = sight::light(&self.state, &self.content, &mut self.lights, pos, eye, true);
        blind.dazzled_by(light).then_some(blind)
    }

    /// `profile`, an attack actor `attacker` makes, as being dazzled leaves it: its race's
    /// `to_hit` worse while strong light dazzles it.
    pub(super) fn glared(&mut self, attacker: ActorId, mut profile: Profile) -> Profile {
        if let Some(blind) = self.dazzles(attacker) {
            profile.to_hit += blind.to_hit;
        }
        profile
    }

    /// Tells the player that strong light began or stopped dazzling them, if it did since the
    /// perception before (`before`); a note that does not stop a rest.
    pub(super) fn glare_news(&mut self, before: bool) {
        match (before, self.perception.dazzled()) {
            (false, true) => self.note(DAZZLED),
            (true, false) => self.note(ADJUSTED),
            _ => {}
        }
    }
}
