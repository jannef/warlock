//! Spell reflection (spec §8 "Fighting styles"; the combat progression design §4.3, §4.4, §5;
//! #566): Sword and Board's capstone turns a spell its holder's shield blocked back at the
//! caster.
//!
//! - **Trigger**: a spell that rolls an attack against Defence (an `attack` or `touch`
//!   resolution: a ray, a bolt, a touch; a scroll's or a wand's too) whose miss the roll put down
//!   to the target's Shield ([`crate::combat::Defence::why_missed`], as a blocked blow's) raises
//!   [`Trigger::SpellBlocked`] for the target, with what it holds (`cast.rs::cast_single`, the
//!   one place a spell's attack roll is made). A miss by Parry or Dexterity, a fumble, and every
//!   spell with no attack roll (bursts, cones, lines, hexes asking a check) raise nothing.
//! - **Effect**: [`WeaponDoes::ReflectSpell`] in [`World::weapon_trigger`], the one place triggered
//!   abilities fire, only marks the reflector ([`World::reflect_ready`]) while its `per_turn` cap
//!   allows; the cast code then casts the spell again ([`World::turn_back`]): from the reflector's
//!   cell back along the way it came (cells it flew through empty) at its caster, with the
//!   caster's dice, lifted cap, to-hit, and riders, rolling its own attack against the caster's
//!   Defence. No mana is spent again, and the reflector pays nothing (no time, no stamina).
//! - **Caps**: `per_turn` reflections a turn of the reflector's, counted here and cleared as its
//!   action is spent ([`World::reflections_spent`], from `spend_actions`); and the reflected
//!   flight is the mark: it never raises the trigger, so it is never reflected again (two
//!   shield-bearing casters cannot bat a bolt back and forth).
//!
//! Creatures' spells are still intents (`ai.rs`), so today only the player's spells are blocked
//! and turned back, by a creature with the rank; the code takes any caster and reflector.
//! Nothing here is saved. Determinism: deciding draws nothing; the reflected spell's attack roll
//! and dice are drawn as the caster's cast draws them, after the blocked roll.
//!
//! [`Trigger::SpellBlocked`]: crate::content::Trigger::SpellBlocked
//! [`WeaponDoes::ReflectSpell`]: crate::content::WeaponDoes::ReflectSpell

use super::weapon_ability::Fired;
use super::{ActorId, AttackSkills, World};
use crate::action::{AttackKind, Event, SpellShape};
use crate::combat::{Blow, Category};
use crate::content::{Shape, SpellData, Trigger};
use crate::dice::Dice;
use crate::geom::Pos;

/// The reflections' transient state (never saved).
#[derive(Clone, Debug, Default)]
pub(super) struct Reflections {
    /// Reflections made in the turn of each actor's under way, by actor.
    made: Vec<(ActorId, u8)>,
    /// The actor whose ability has just chosen to turn the blocked spell back.
    turning: Option<ActorId>,
}

/// A single-target spell as its caster cast it: what a reflection casts again.
#[derive(Clone, Copy, Debug)]
pub(super) struct Shot<'s> {
    pub(super) def: &'s SpellData,
    /// Its effect's id (drawn along its way), or empty.
    pub(super) effect: &'s str,
    pub(super) dice: Dice,
    /// The caster's cap (lifted, for its own spell): the most it does.
    pub(super) cap: i32,
    /// The caster's to-hit with it.
    pub(super) to_hit: i32,
    /// The spell attack skill and caster style it is cast with (the caster's own cast of a
    /// spell that rolls to hit; none for a scroll, a wand, or a reflected flight).
    pub(super) skills: AttackSkills,
}

/// A spell turned back: by whom, at whom, the way back, and the spell.
#[derive(Clone, Debug)]
pub(super) struct Turned<'s> {
    pub(super) reflector: ActorId,
    pub(super) caster: ActorId,
    path: Vec<Pos>,
    pub(super) shot: Shot<'s>,
}

impl World {
    /// [`crate::content::WeaponDoes::ReflectSpell`] fired for `by`: it turns the spell back if
    /// it has made fewer than `per_turn` reflections this turn of its own.
    pub(super) fn reflect_ready(&mut self, by: ActorId, per_turn: u8) {
        let made = &mut self.reflection.made;
        let count = match made.iter_mut().find(|(a, _)| *a == by) {
            Some((_, count)) => count,
            None => {
                made.push((by, 0));
                &mut made.last_mut().expect("just pushed").1
            }
        };
        if *count < per_turn {
            *count += 1;
            self.reflection.turning = Some(by);
        }
    }

    /// Actor `id`'s action is taken: its count of reflections starts again.
    pub(super) fn reflections_spent(&mut self, id: ActorId) {
        self.reflection.made.retain(|(a, _)| *a != id);
    }

    /// `caster`'s spell `shot`, flown along `path` from `here` (a touch: the cell touched),
    /// missed `reflector` by its Shield: [`Trigger::SpellBlocked`] fires for it, and if its
    /// ability turns the spell back, what to cast back. No draw.
    pub(super) fn spell_blocked<'s>(
        &mut self,
        (reflector, caster): (ActorId, ActorId),
        (path, here): (&[Pos], Pos),
        shot: Shot<'s>,
        events: &mut Vec<Event>,
    ) -> Option<Turned<'s>> {
        self.reflection.turning = None;
        let fired = Fired {
            other: caster,
            kind: AttackKind::Spell,
            skills: self.held_skills(reflector),
            blow: Some(Blow::missed(Some(Category::Shield))),
        };
        self.weapon_trigger(reflector, Trigger::SpellBlocked, fired, events);
        self.reflection.turning.take().filter(|&r| r == reflector)?;
        let back = (path.iter().rev().skip(1).copied()).chain([here]).collect();
        Some(Turned {
            reflector,
            caster,
            path: back,
            shot,
        })
    }

    /// Casts `turned` back at its caster from the reflector's cell, said first ("Your shield
    /// turns the bolt back at the shaman."), its effect drawn along the way back; its own attack
    /// roll against the caster's Defence, as the caster cast it. The reflected flight never
    /// raises the trigger: it is never reflected again.
    pub(super) fn turn_back(&mut self, turned: &Turned<'_>, made: &mut Vec<Event>) {
        let Turned {
            reflector,
            caster,
            ref path,
            shot,
        } = *turned;
        let (Some(from), true) = (
            self.state.actors.get(&reflector).map(|a| a.pos),
            self.state.actors.contains_key(&caster),
        ) else {
            return;
        };
        if self.state.death.is_some() {
            return;
        }
        self.say_reflection(reflector, caster, &shot.def.look.noun);
        let flies = shot.def.shape != Shape::Touch;
        if flies && !shot.effect.is_empty() && !path.is_empty() {
            made.push(Event::Spell {
                effect: shot.effect.to_string(),
                origin: from,
                shape: SpellShape::Bolt { path: path.clone() },
                cells: path.clone(),
            });
        }
        // Cast as the caster cast it, but practised by no one (no marks, no triggers).
        let shot = Shot {
            skills: AttackSkills::default(),
            ..shot
        };
        self.spell_attack(reflector, caster, (from, flies), shot, made);
    }

    /// Says a spell turned back, as the player sees it: "Your shield turns the bolt back at the
    /// shaman.", "The champion's shield turns your bolt back at you."
    fn say_reflection(&mut self, reflector: ActorId, caster: ActorId, noun: &str) {
        let player = self.state.player;
        let content = self.content.clone();
        let seen = |w: &World, id: ActorId| {
            (w.state.actors.get(&id))
                .filter(|a| w.perception.sees_actors_at(a.pos))
                .map(|a| content.creature(a.creature).name.clone())
        };
        let text = if reflector == player {
            let them =
                seen(self, caster).map_or("its caster".to_string(), |n| crate::content::the(&n));
            format!("Your shield turns the {noun} back at {them}.")
        } else {
            let whose = seen(self, reflector).map(|n| crate::content::the_start(&n));
            match (whose, caster == player) {
                (Some(whose), true) => format!("{whose}'s shield turns your {noun} back at you."),
                (None, true) => format!("Your {noun} is turned back at you."),
                (Some(whose), false) => {
                    let them = seen(self, caster)
                        .map_or("its caster".to_string(), |n| crate::content::the(&n));
                    format!("{whose}'s shield turns the {noun} back at {them}.")
                }
                (None, false) => return,
            }
        };
        self.say(text);
    }
}
