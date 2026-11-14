//! Healing harms the undead (#1278; spec §8 "Spells"): a healing spell
//! ([`SpellData::heals_wounds`]: a `heal` that does not feed on a corpse, so Cure wounds, Mend,
//! and Benison, never Grave feast) that lands on a creature tagged [`UNDEAD`] does its roll as
//! harm instead of hit points back.
//!
//! - **The touch** (Cure wounds, Mend; `cast_heal`): aimed at an undead beside the caster, it is
//!   an attack: the touch's attack roll against its Defence, as Cause wounds', the heal's roll
//!   (its rung, rank, and hooks: [`Cast::roll`]) the harm, a critical rolling it again. Living
//!   creatures, friend or foe, are healed as before.
//! - **The burst about the caster** (Benison; `cast_heal_about`): every undead in it, friend or
//!   foe, takes the one roll as an area's harm (Evasion halving it); living allies are healed.
//! - **The harm** ([`harm_of`]): the spell as a damage spell of the `fire` type (there is no holy
//!   type, as Rebuke's; the owner's Q2), "the light of mending sears the ghoul": fire's
//!   resistances and its rule (one in four left burning) stand.
//!
//! Scrolls and wands of a healing spell cast through the same path; a miscast follows a cast
//! paid in mana as any other (`miscast.rs`). Creature healers never heal an undead, and an undead
//! creature never casts a heal (`creature_cast.rs`, [`World::heal_hurts`]). No player race is
//! undead. Determinism: the attack roll and the dice, as Cause wounds' draw them.

use crate::action::Event;
use crate::content::{Content, SpellData, SpellPurpose};
use crate::damage::Damage;
use crate::geom::Pos;
use crate::world::magic::cast::Cast;
use crate::world::magic::spell::Caster;
use crate::world::{Actor, ActorId, World};

/// The tag of the creatures a heal harms.
pub const UNDEAD: &str = "undead";

/// What a heal is as it harms the undead: `def` as a damage spell of the `fire` type, said as
/// "the light of mending sears".
pub(super) fn harm_of(def: &SpellData, content: &Content) -> SpellData {
    let mut harm = def.clone();
    harm.kind = SpellPurpose::Damage;
    harm.damage_type = content.damage_type_id("fire");
    harm.look.noun = "light of mending".into();
    harm.look.verb = "sears".into();
    harm
}

impl World {
    /// Whether actor `a` is undead: a heal harms it.
    pub(in crate::world) fn is_undead(&self, a: &Actor) -> bool {
        self.content.has_tag(a.creature, UNDEAD)
    }

    /// Whether a heal would harm actor `id` (gone: no).
    pub(in crate::world) fn heal_hurts(&self, id: ActorId) -> bool {
        (self.state.actors.get(&id)).is_some_and(|a| self.is_undead(a))
    }

    /// The player's healing touch `cast` on the undead at `at`, beside them: an attack with the
    /// heal's roll as fire (module docs). Whether it went off.
    pub(in crate::world) fn heal_sears(
        &mut self,
        cast: &Cast<'_>,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> bool {
        let Some(roll) = cast.roll else {
            return false;
        };
        if !self.can_loose_here() {
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        let content = self.content.clone();
        let harm = harm_of(cast.def, &content);
        let player = self.state.player;
        let here = self.player_pos();
        self.sound_at(harm.look.noise, here);
        let mut made = Vec::new();
        let shot = ("", roll, cast.reroll);
        let by = (&harm, cast.own_skills());
        let (turned, _) = self.cast_single(player, (here, at, 1), shot, by, &mut made);
        self.spell_lands(player, (&harm, &cast.riders), made, Some(cast), events);
        if let Some(turned) = turned {
            let mut back = Vec::new();
            self.turn_back(&turned, &mut back);
            self.spell_lands(turned.reflector, (&harm, &cast.riders), back, None, events);
        }
        true
    }

    /// Benison's `back` on the undead `undead` in its burst about the player at `here`: each
    /// takes it as an area's harm of fire (Evasion halving it), said as one cast.
    pub(in crate::world) fn heal_sears_about(
        &mut self,
        cast: &Cast<'_>,
        (undead, back, here): (&[ActorId], i32, Pos),
        events: &mut Vec<Event>,
    ) {
        if undead.is_empty() {
            return;
        }
        let content = self.content.clone();
        let harm = harm_of(cast.def, &content);
        let player = self.state.player;
        let mut made = Vec::new();
        for &id in undead {
            if self.state.actors.contains_key(&id) && self.state.death.is_none() {
                let damage = Damage::new(back, harm.damage_type);
                self.spell_hurts(Caster::Actor(player), id, (here, false), damage, &mut made);
            }
        }
        self.spell_lands(player, (&harm, &cast.riders), made, Some(cast), events);
    }
}
