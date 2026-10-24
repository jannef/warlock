//! The element gifts that are more than one read at a hook (spec §8 "Birth sign"; the talents of
//! the `stars` tree): Fire's active, **Blaze**, used from the abilities menu's Birth sign group
//! ([`crate::ability::Ability::Sign`]) as [`Action::Gift`] (`use:blaze`). The passive gifts
//! (Kindled's lantern, Forge-born's marks, Tidal's mana) are one read each where their rule
//! lives: `WorldState::lantern_light`, `TalentMods::marks`, `World::mana_rate`.
//!
//! Blaze is a burst of flame around the player, cast with the scrolls' shared burst code
//! ([`super::spell::burst_cells`], [`World::spell_hurts`], [`World::shock`]): one gameplay roll
//! of its damage for every creature it reaches but the player (each takes it whole: a burst of
//! radius 1 is all edge), the ice it reaches melted, a fire on every walkable cell it reaches
//! but the player's (as a burning missile's where it lands: light for a while, then gone), then
//! its cooldown status. Refused, taking no time, without the gift, while the cooldown lasts, and
//! on the overworld.
//!
//! [`Action::Gift`]: crate::action::Action::Gift

use super::level::Fire;
use super::spell::{Caster, burst_cells};
use super::{ActorId, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::{Event, SpellShape};
use crate::content::SignActive;
use crate::damage::Damage;
use crate::geom::Pos;
use crate::ice::Shock;
use crate::time::Duration;
use std::collections::BTreeSet;

impl World {
    /// `use:ID[:X,Y]` ([`Action::Gift`](crate::action::Action::Gift)): the player uses the
    /// birth sign's active `active`, at cell `at` if it asks for one. Refused, taking no time,
    /// without it and while its cooldown lasts.
    pub(super) fn use_gift(
        &mut self,
        active: SignActive,
        _at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let ability = Ability::Sign(active);
        let me = self.state.player();
        if !ability.had_by(me, &content) {
            self.say("You have no such gift.");
            return Spent::Nothing;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            self.say(match why {
                Unavailable::While(status) => format!(
                    "You are still {}.",
                    content.status(status).name.to_lowercase()
                ),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return Spent::Nothing;
        }
        match active {
            SignActive::Blaze => self.blaze(events),
        }
    }

    /// Blaze (module docs): flames burst from the player, for the time of a `cast`.
    fn blaze(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(blaze) = self.state.player().talent_mods(&content).blaze else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let price = self.price_of(player, SignActive::Blaze.cost());
        let here = self.player_pos();
        let level = self.state.current();
        let opaque = |p: Pos| p != here && level.is_opaque(&content, p);
        let cells = burst_cells(here, blaze.radius, opaque);
        self.say("Flames burst from you!");
        let mut made = vec![Event::Spell {
            effect: blaze.spell.clone(),
            origin: here,
            shape: SpellShape::Burst {
                center: here,
                radius: blaze.radius,
            },
            cells: cells.clone(),
        }];
        let roll = blaze.damage.roll(&mut self.state.rng);
        let reached: BTreeSet<Pos> = cells.iter().copied().collect();
        let on = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.id != player && a.level == on && reached.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        for id in caught {
            let damage = Damage::new(roll, blaze.kind);
            self.spell_hurts(Caster::Actor(player), id, (here, false), damage, &mut made);
        }
        self.shock(here, blaze.radius, Shock::Heat, &mut made);
        self.kindle(&cells, here, blaze.fire, blaze.burns);
        let noun = content
            .sign_active(SignActive::Blaze)
            .map_or_else(|| "blaze".to_string(), |def| def.name.to_lowercase());
        let verb = blaze
            .kind
            .map_or("burns", |k| content.damage_type(k).verb.as_str());
        self.say_spell((&noun, verb), made, events);
        if self.state.death.is_none() {
            self.afflict(player, blaze.cooldown, 1, None, events);
        }
        Spent::Turn(price)
    }

    /// Fires on the walkable `cells` but `here` (Blaze's), each burning `seconds` with light
    /// `light`, or the fire there burning on until then (as a burning missile's, `come_down`).
    fn kindle(&mut self, cells: &[Pos], here: Pos, light: crate::content::LightId, seconds: u32) {
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let content = self.content.clone();
        let level = self.state.current_mut();
        for &p in cells {
            if p == here || !level.is_walkable(&content, p) {
                continue;
            }
            let fire = (level.fires.entry(p)).or_insert(Fire { light, until });
            fire.light = light;
            fire.until = fire.until.max(until);
        }
    }
}
