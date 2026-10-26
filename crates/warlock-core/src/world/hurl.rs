//! Creatures' throws (spec §8 "Ranged combat": "Creatures' throws"): a natural ranged attack
//! with no ammunition, `[creature.throw]` in its type (a giant's boulders).
//!
//! **When**: a creature with a throw that is not next to its target throws whenever the target
//! is within the throw's range with a clear line of fire (no opaque cell and nobody in between,
//! as an archer's); otherwise it closes in, and next to its target it fights in melee. Nothing
//! is drawn at random to decide it.
//!
//! **What**: a thrown attack ([`crate::world::Actor::hurled`]: Dexterity to hit, Strength to
//! damage), flying as any missile ([`World::fly`]) as the throw's object, which never lands: the
//! supply is endless, and the throw's price (its `cost`, else `throw`'s) is what limits it. It
//! makes a throw's noise once. Its `on_hit` riders apply as a natural attack's do.
//!
//! **Words**: the player who sees the thrower is told it throws ("The hill giant hurls a
//! boulder."); its blow is said as a trap's missile is, by what flies ("The boulder strikes
//! you.", "You dodge the boulder."): what an unseen thrower did is said only to its target.

use super::fight::{End, Guard, Words, attack_message};
use super::shoot::Flight;
use super::{Actor, ActorId, World};
use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::{ActionKind, Content, DamageTypeId, ObjectId, Throw, article};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::noise::Act;

impl World {
    /// Whether creature `id` throws at actor `target`, standing at `at`, now: it has a throw,
    /// is not next to it, and has it in range with a clear line of fire.
    pub(super) fn hurls_at(&self, id: ActorId, target: ActorId, at: Pos) -> bool {
        let me = &self.state.actors[&id];
        let Some(throw) = self.content.creature(me.creature).throw.as_ref() else {
            return false;
        };
        me.pos.chebyshev(at) > 1
            && me.pos.dist2(at) <= radius2(throw.range)
            && self.clear_shot(me.pos, at, id, target)
    }

    /// Creature `id` throws at actor `target` (its AI saw to it that it can). Returns the
    /// price: its throw's cost (priced as the throw begins, as every attack is).
    pub(super) fn creature_hurls(
        &mut self,
        id: ActorId,
        target: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let me = &self.state.actors[&id];
        let at = self.state.actors.get(&target).map(|a| a.pos);
        let throw = content.creature(me.creature).throw.as_ref();
        let (Some(throw), Some(profile), Some(at)) = (throw, me.hurled(&content), at) else {
            return self.price_of(id, ActionKind::Wait);
        };
        let skills = self.skills_of(id, AttackKind::Thrown, Some(throw.object));
        let price = self.attack_price(id, ActionKind::Throw, None, skills);
        self.make_noise(id, Act::Attack(AttackKind::Thrown));
        let flight = Flight {
            kind: AttackKind::Thrown,
            with: throw.object,
            flies: throw.object,
            profile,
            range: throw.range,
            lands: None,
            breaks: 0,
        };
        self.fly(id, at, flight, events);
        price
    }
}

/// The throw of `attacker`'s type, if an attack of `kind` made `with` an object was one.
pub(super) fn thrown_by<'c>(
    content: &'c Content,
    attacker: &Actor,
    kind: AttackKind,
    with: Option<ObjectId>,
) -> Option<&'c Throw> {
    let throw = content.creature(attacker.creature).throw.as_ref()?;
    (kind == AttackKind::Thrown && with == Some(throw.object)).then_some(throw)
}

/// What the player is told of a creature's throw (module docs): by `thrower` (as they see it),
/// at `target`, what came of it, the kind of damage it did, and the target's guard.
pub(super) fn throw_messages(
    content: &Content,
    throw: &Throw,
    (thrower, target): (End<'_>, End<'_>),
    strike: Option<Strike>,
    damage: Option<DamageTypeId>,
    guard: Guard<'_>,
) -> Vec<String> {
    if thrower == End::Unseen && target == End::Unseen {
        return Vec::new();
    }
    let noun = throw.attack.name.as_str();
    let mut said = Vec::new();
    if let End::Seen(name) = thrower {
        let verb = &throw.attack.verb;
        said.push(format!("The {name} {verb} {} {noun}.", article(noun)));
    }
    let verb = damage.map_or("hits", |d| content.damage_type(d).verb.as_str());
    let words = Words::new(noun, verb);
    let blow = attack_message(
        End::Trap(noun),
        target,
        &words,
        AttackKind::Thrown,
        strike,
        guard,
    );
    said.extend(blow);
    said
}
