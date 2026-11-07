//! Flasks (spec §8 "Potions": "Flasks"; the potions wave 2 design §3, §5): a potion thrown
//! shatters where it comes down, and a flask (a potion with `[object.throw]`,
//! [`crate::content::Flask`]) casts its spell there, as a scroll casts one where it is read.
//!
//! **The throw** ([`World::throw_flask`], from `throw:c:X,Y` through [`World::throw`]): any
//! potion may be thrown, a flask as far as its `range`, any other as far as an improvised
//! weapon, each plus the thrower's `throw_range` talents and items' range. Refused, with no time,
//! as every throw is: on the overworld, at oneself, out of range, unpaid goods, or while
//! Engulfed (`blocks = ["throw"]`). Priced as a throw (`attack_price(Throw)`, stance and burden
//! counted, no weapon skill's speed: glass, not a weapon), one taken from the stack (never kept:
//! it broke), "You throw the fizzy pink potion." by the name the player knows, the throw's noise
//! where they stand. It trains nothing and draws nothing of its own: no to-hit roll.
//!
//! **Where it breaks** ([`World::shatter_flask`]): by a burst spell's aim rule
//! ([`World::burst_center`]): the first creature on the line from the thrower through the cell
//! aimed at, else the last cell before something solid, else that cell. It flies there as an
//! [`Event::Missile`] (cut by perception to what the player sees, as any missile) and nothing
//! lands: its shattering is heard there (the flask's `noise`, a crash of glass by default; a
//! potion without a `throw` table as loud), and a flask's spell is cast there.
//!
//! **The splash** is the spell the flask names, cast by the thrower (its blows, kills, and zone
//! are theirs) from where it breaks, through the cast path's shared code at the flask's rung of
//! each ladder and nobody's stats or hooks ([`Cast::by_flask`], `Pay::Flask`: no school marks, no
//! triggers): a burst damage spell strikes as a scroll's burst does ([`World::cast_burst_at`],
//! its spell type's rules, riders, and zone), a burst hex lays its zone or its status on everyone
//! in its cells but the thrower, a ray's or a bolt's hex falls on whoever stands where it broke.
//! A death of the player by their own flask is [`Cause::Flask`].
//!
//! **What the player learns**: a flask whose breaking cell they see is said ("A sweet mist puffs
//! from the shards.", its `words`) and its type learned ("It was a sleeping draught."); one they
//! do not see is only heard ("You hear a crash of glass."; creatures hear it by the hearing
//! rules), and nothing is learned. A potion without a `throw` table spills where it lands, uselessly: "The
//! potion shatters; the draught soaks away." where seen, and nothing is learned.

use super::cast::Cast;
use super::spell::{Caster, burst_cells};
use super::talk::capitalized;
use super::{ActorId, AttackSkills, Cause, Spent, World};
use crate::action::{AttackKind, Event, MissileEnd, Positions, SpellShape};
use crate::content::{
    ActionKind, Burst, ObjectId, Shape, SpellPurpose, SpellZone, StatusId, TrapNoise,
};
use crate::geom::Pos;
use crate::noise::{Act, SHATTER_NOISE, SoundKind};

impl World {
    /// The player throws one of inventory stack `item`, a potion, at cell `at` (module docs).
    /// [`World::throw`] has checked the overworld and the stack.
    pub(super) fn throw_flask(&mut self, item: usize, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(range) = self.state.player().throw_range(item, &content) else {
            return Spent::Nothing;
        };
        if !self.may_loose(item) || !self.within_reach(at, range, "throw") {
            return Spent::Nothing;
        }
        // Priced as the throw begins, with the flask still in the pack.
        let price = self.attack_price(player, ActionKind::Throw, None, AttackSkills::default());
        let me = self.player_mut();
        let flask = me.inventory[item].single();
        me.inventory[item].count -= 1;
        if me.inventory[item].count == 0 {
            me.inventory.remove(item);
            me.equipment.after_removal(&Positions::of([item]));
        }
        let title = self.lore().title(&flask);
        self.say(format!("You throw the {title}."));
        self.make_noise(player, Act::Attack(AttackKind::Thrown));
        let from = self.player_pos();
        self.shatter_flask(player, flask.object, (from, at, range), events);
        self.say_news();
        self.exert(ActionKind::Throw);
        Spent::Turn(price)
    }

    /// A potion of kind `object`, thrown by `by` from `from` at `at` as far as `range` cells,
    /// flies and shatters (module docs): where it breaks it is heard, and a flask's splash is
    /// cast there. What the player sees is said and learned.
    pub(super) fn shatter_flask(
        &mut self,
        by: ActorId,
        object: ObjectId,
        (from, at, range): (Pos, Pos, i32),
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let center = self.burst_center(from, at, range);
        let steps = from.chebyshev(center).max(0) as usize;
        let path: Vec<Pos> = if center == from {
            Vec::new()
        } else {
            from.ray(at)
                .take(steps)
                .take_while(|&p| p != center)
                .chain([center])
                .collect()
        };
        let level = self.state.current().id.clone();
        let struck = (self.state.actor_at(&level, center)).is_some_and(|a| a.id != by);
        events.push(Event::Missile {
            object,
            from: Some(from),
            path,
            end: if struck {
                MissileEnd::Struck
            } else {
                MissileEnd::Fell
            },
        });
        let def = content.object(object);
        let noise = (def.flask.as_ref()).map_or(
            TrapNoise {
                sound: SoundKind::Shatter,
                loudness: SHATTER_NOISE,
            },
            |f| f.noise,
        );
        self.sound_at(Some(noise), center);
        let seen = self.perception.is_visible(center);
        // What the player makes in their own action is spent before they hear it (`run_turn`):
        // their own flask breaking unseen is told here.
        if !seen && by == self.state.player {
            self.say("You hear a crash of glass.");
        }
        let Some(flask) = &def.flask else {
            if seen {
                self.say("The potion shatters; the draught soaks away.");
            }
            return;
        };
        if seen {
            if let Some(words) = &flask.words {
                self.say(words.clone());
            }
            if self.learn_scroll(object) {
                self.say(format!("It was {}.", def.describe(1)));
            }
        }
        let cast = Cast::by_flask(by, &content, flask);
        self.splash(&cast, center, events);
        if by == self.state.player
            && let Some(death) = &mut self.state.death
        {
            death.by = None;
            death.cause = Some(Cause::Flask(object));
        }
    }

    /// `cast`, a flask's splash, goes off where it broke, at `center`, by its kind and shape
    /// (module docs). Its effect shows there: it flies nowhere (the flask flew).
    fn splash(&mut self, cast: &Cast<'_>, center: Pos, events: &mut Vec<Event>) {
        let def = cast.def;
        let radius = cast.reach.radius.unwrap_or(0);
        let level = self.state.current();
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        let cells = match def.shape {
            Shape::Burst => burst_cells(center, radius, opaque),
            _ => vec![center],
        };
        let effect = def.look.effect.clone().unwrap_or_default();
        match def.kind {
            SpellPurpose::Damage => {
                let burst = Burst {
                    spell: effect,
                    radius,
                    damage: cast.roll.unwrap_or_default(),
                    reroll: 0,
                    kind: def.damage_type,
                    blast: def.look.blast,
                    ice: def.look.ice,
                };
                let mut made = Vec::new();
                let only = &def.effect.only_tags;
                let caster = Caster::Actor(cast.by);
                let rolled =
                    self.cast_burst_at(caster, (center, center), (&burst, only), &mut made);
                self.spell_type_spreads(cast.by, (def, cast.reach.jumps), rolled, &mut made);
                self.spell_lands(cast.by, (def, &cast.riders), made, Some(cast), events);
                self.lay_spell_zone(cast, cells, events);
            }
            _ => {
                if !effect.is_empty() && def.shape == Shape::Burst {
                    events.push(Event::Spell {
                        effect,
                        origin: center,
                        shape: SpellShape::Burst { center, radius },
                        cells: cells.clone(),
                    });
                }
                match (def.effect.zone, def.effect.status, cast.degrees) {
                    (Some(SpellZone { .. }), _, _) => self.lay_spell_zone(cast, cells, events),
                    (None, Some((status, _)), Some(degrees)) => {
                        self.splash_hex(cast, &cells, (status, degrees), events);
                    }
                    _ => {}
                }
            }
        }
    }

    /// A flask's hex: its status, at `degrees`, on everyone on `cells` but the thrower (of the
    /// spell's tags and ground, if it names them), each the player sees named ("The kobold is
    /// drowsy.").
    fn splash_hex(
        &mut self,
        cast: &Cast<'_>,
        cells: &[Pos],
        (status, degrees): (StatusId, u16),
        events: &mut Vec<Event>,
    ) {
        let level = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != cast.by && cells.contains(&a.pos))
            .filter(|a| self.is_of_tags(a, &cast.def.effect.only_tags))
            .filter(|a| self.stands_on(a.pos, cast.def.effect.only_terrain))
            .filter(|a| self.sees_flash(a, cast.def.effect.only_sighted))
            .map(|a| a.id)
            .collect();
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "hexed".into());
        for id in caught {
            if !self.state.actors.contains_key(&id) || self.state.death.is_some() {
                continue;
            }
            self.afflict(id, status, degrees, Some(cast.by), events);
            let them = &self.state.actors[&id];
            if id != self.state.player && self.perception.sees_actors_at(them.pos) {
                let who = capitalized(&self.called(them));
                self.say(format!("{who} is {word}."));
            }
            self.spell_landed(cast, id, true, events);
        }
    }
}
