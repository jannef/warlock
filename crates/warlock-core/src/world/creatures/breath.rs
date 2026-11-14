//! A dragon's breath (spec §8 "Creature behavior": "Dragons", #1099): a `combat = "dragon"`
//! creature with `[creature.breath]` in its type (`content/breath.rs`) breathes a cone of its
//! element. Enemy-only: no player ability.
//!
//! **When** ([`World::breathes_at`], asked in `ai.rs::fight` after a summoning and a throw):
//! its cooldown status is off and the hostile it sees stands in the cone it would breathe at
//! it (within its `length`, in its line of sight), with a clear line between them (no opaque
//! cell and nobody in the way, as a shot's). Deciding draws nothing. Otherwise it fights as a
//! melee creature: it closes in and strikes with its natural attacks (`natural.rs`), so it
//! never stands off doing nothing.
//!
//! **What** ([`World::creature_breathes`]): it takes the cooldown status, roars (a noise heard
//! as far as a howl), and the cone strikes through the spells' shared code
//! ([`World::cast_cone`]): the effect drawn over its cells, one roll of its damage for everyone
//! in it, in ascending actor id order, with no roll to hit, Evasion halving it and resistances
//! taking their share (the dragon's own kind, immune to its element, takes nothing); then
//! its element's spell type falls on those it harmed (fire's burning, one in four;
//! `spell_type.rs`), and its `riders`, each by its own chance, on everyone it harmed who still
//! lives (`World::rider`: a status's degrees, a drain's check; the gorgon's stiffening, #1138).
//! Its blows are the dragon's: one that kills the player is what killed them. The action costs a
//! `cast`.
//!
//! **Words**: the player who sees the dragon is told it breathes ("The fire drake breathes
//! fire!"); each blow on someone they see is said as a spell's ("The fire burns you."). Unseen,
//! the roar is heard.

use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::{ActionKind, Breath, the_start};
use crate::geom::Pos;
use crate::noise::Act;
use crate::world::magic::spell::{Caster, cone_cells};
use crate::world::{ActorId, World};

impl World {
    /// Whether creature `id` breathes now at `target`, a hostile it sees at `at` (module docs).
    pub(super) fn breathes_at(&self, id: ActorId, target: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(breath) = self.content.creature(me.creature).breath.as_ref() else {
            return false;
        };
        !me.effects.has(breath.status)
            && self.in_breath(me.pos, at, breath)
            && self.clear_shot(me.pos, at, id, target)
    }

    /// Whether `breath`, breathed from `from` at `at`, reaches `at`.
    fn in_breath(&self, from: Pos, at: Pos, breath: &Breath) -> bool {
        let level = self.state.current();
        let opaque = |p: Pos| level.is_opaque(&self.content, p);
        cone_cells(from, at, breath.length, opaque).contains(&at)
    }

    /// Creature `id` breathes at the cell `at` (its AI saw to it that it can). Returns the
    /// price: a `cast`.
    pub(super) fn creature_breathes(
        &mut self,
        id: ActorId,
        at: Pos,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let price = self.price_of(id, ActionKind::Cast);
        let me = &self.state.actors[&id];
        let def = content.creature(me.creature);
        let Some(breath) = def.breath.as_ref() else {
            return price;
        };
        let here = me.pos;
        self.afflict(id, breath.status, breath.degrees, None, events);
        self.make_noise(id, Act::Roar);
        if self.perception.sees_actors_at(here) {
            self.say(format!("{} {}!", the_start(&def.name), breath.verb));
        }
        let mut made = Vec::new();
        let cone = (
            &breath.effect[..],
            breath.damage.into(),
            0,
            Some(breath.kind),
        );
        self.cast_cone(
            Caster::Actor(id),
            (here, at, breath.length),
            cone,
            &mut made,
        );
        // Its element's spell type falls on those it harmed (`spell_type.rs`).
        let harmed: Vec<ActorId> = (made.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker,
                    target,
                    kind: AttackKind::Spell,
                    strike: Some(Strike::Hit { .. }),
                    ..
                } if attacker == id => Some(target),
                _ => None,
            })
            .collect();
        let verb = &content.damage_type(breath.kind).verb;
        self.say_spell((&breath.noun, verb), made, events);
        self.spell_type_lands(id, Some(breath.kind), &harmed, events);
        for target in harmed {
            for rider in breath.riders.clone() {
                if !self.state.actors.contains_key(&target) || self.state.death.is_some() {
                    break;
                }
                self.rider(id, target, rider, events);
            }
        }
        price
    }
}
