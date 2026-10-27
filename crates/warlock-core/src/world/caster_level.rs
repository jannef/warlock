//! Caster level (spec §8 "Spells", the magic design §3): a derived number, never stored. The
//! player's is their level times their class's casting percent (`casting` in `data/classes/`;
//! all of it without a class), a creature's its type's level; then what its talents (racial
//! and birth-sign gifts are talents too) and what it wears and wields add (`caster_level`, a
//! talent effect, flat or by character level, Arcane training's, and an affix effect), rounded
//! down, never below 0, and at least 1 for anyone with mana. It sets the learning check, the
//! cost of overreach, and a spell's dice.

use super::{Actor, ActorId, World};
use crate::character::Character;
use crate::content::{Content, Effect as ItemEffect};

/// The casting percent of a player with `character` (spec §8 "Spells"): their class's, 100
/// without a class (a scenario's own player, as a creature casts at its whole level).
pub fn casting_percent(content: &Content, character: Option<&Character>) -> u8 {
    (character.and_then(|c| c.spec.class(content).ok())).map_or(100, |class| class.casting)
}

impl Actor {
    /// Its caster level at character `level` with `casting` percent (spec §8 "Spells"): `level`
    /// × `casting` ÷ 100 rounded down, plus its talents' (at `level`: Arcane training's grows
    /// with it) and its magic items' `caster_level`;
    /// never below 0, and at least 1 while its maximum mana is above 0.
    pub fn caster_level(&self, content: &Content, level: u8, casting: u8) -> i32 {
        let base = i32::from(level) * i32::from(casting) / 100;
        let talents = self.talent_mods(content).caster_level_at(level);
        let items = self.mods(content).get(ItemEffect::CasterLevel);
        let level = (base + talents + items).max(0);
        if self.max_mana(content) > 0 {
            level.max(1)
        } else {
            level
        }
    }
}

impl World {
    /// Actor `id`'s caster level (spec §8 "Spells"; [`Actor::caster_level`]): the player's at
    /// their level and their class's casting percent, a creature's at its type's level and all
    /// of it. 0 for an actor that is not there.
    pub fn caster_level(&self, id: ActorId) -> i32 {
        let Some(actor) = self.state.actors.get(&id) else {
            return 0;
        };
        let (level, casting) = if id == self.state.player {
            let character = self.state.character.as_ref();
            let casting = casting_percent(&self.content, character);
            (self.player_level(), casting)
        } else {
            (self.content.creature(actor.creature).level, 100)
        };
        actor.caster_level(&self.content, level, casting)
    }
}
