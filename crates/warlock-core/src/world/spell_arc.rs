//! How a spell of a type that arcs or conducts spreads from those it harmed (the spell types
//! design `docs/design/spell-types-and-new-spells.md` §3, §4, §7 entry 1; `arc` and `conducts` in
//! `[types]`, `data/spells.toml`): [`World::spell_type_spreads`], one line in `cast_damage`, before
//! `spell_lands` says and walks all it struck. It reads the cast's events in order, each new strike
//! appended and read in its turn, so every creature harmed conducts and then arcs once, however it
//! came to be struck.
//!
//! - **Conducting** ([`World::water_body`]): a creature harmed on open water strikes everyone
//!   standing in that body of water not yet struck, in actor id order. Ice is not water.
//! - **Arcing** ([`World::arc_from`]): the creature harmed strikes the nearest beside it not yet
//!   struck, ties by actor id, by the type's chance, or Conductor's, or for certain while the
//!   spell's `arcs` last (a spell with `arcs` reaches [`ARC_REACH`] cells, never through what stops
//!   sight). A ray, a bolt, a touch, or a line arcs; a burst and a cone only conduct.
//!
//! It spares no one (the owner's Q14): the caster and their allies are struck like anyone. Every
//! strike is [`World::spell_strikes`] at the roll the cast settled, with no roll to hit and no
//! Evasion. A spell turned back and a mishap never come here, so they spread nothing. One that
//! took no hit points (a miss, one that resisted it all) spreads nothing either.
//!
//! Determinism: one `below(100)` per creature harmed with someone to jump to, only for a chance
//! below 100 and past the certain jumps, in event order. Saves: nothing is stored.

use super::{ActorId, World};
use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::{ARC_REACH, DamageTypeId, Shape, SpellData};
use crate::damage::Damage;
use crate::geom::Pos;
use std::collections::BTreeSet;

/// One cast's spreading so far.
struct Spread {
    by: ActorId,
    kind: DamageTypeId,
    /// The harm the cast settled, before what each target resists.
    rolled: i32,
    /// Everyone it has struck at, harmed or not.
    struck: BTreeSet<ActorId>,
    /// The bodies of water it has been through, each filled once.
    bodies: Vec<BTreeSet<Pos>>,
    /// The percent chance of an arc (`None`: it does not arc).
    chance: Option<u8>,
    /// How far an arc reaches.
    reach: i32,
    /// The jumps still taken without a draw.
    jumps: i32,
}

impl World {
    /// `by`'s spell `def`, with `jumps` certain jumps ([`crate::world::Reach`]), rolled `rolled`
    /// and made the events `made`: it spreads from everyone it harmed (module docs), each new
    /// strike's events added to `made`.
    pub(super) fn spell_type_spreads(
        &mut self,
        by: ActorId,
        (def, jumps): (&SpellData, Option<i32>),
        rolled: i32,
        made: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let (Some(kind), Some(rule)) = (def.damage_type, content.spell_type(def.damage_type))
        else {
            return;
        };
        let single = matches!(
            def.shape,
            Shape::Ray | Shape::Bolt | Shape::Touch | Shape::Line
        );
        let own =
            (self.state.actors.get(&by)).map_or(0, |a| a.talent_mods(&content).arc_chance(kind));
        let chance = rule.arc.filter(|_| single).map(|c| c.max(own));
        if chance.is_none() && !rule.conducts {
            return;
        }
        let mut spread = Spread {
            by,
            kind,
            rolled,
            struck: (made.iter())
                .filter_map(|e| spell_blow(e, by).map(|(target, ..)| target))
                .collect(),
            bodies: Vec::new(),
            chance,
            reach: if def.effect.arcs.is_some() {
                ARC_REACH
            } else {
                1
            },
            jumps: jumps.unwrap_or(0),
        };
        let mut next = 0;
        while next < made.len() && self.state.death.is_none() {
            let harmed = match spell_blow(&made[next], by) {
                Some((_, to, Strike::Hit { .. })) => Some(to),
                _ => None,
            };
            next += 1;
            let Some(cell) = harmed else {
                continue;
            };
            if rule.conducts {
                self.conduct(cell, &mut spread, made);
            }
            self.arc_from(cell, &mut spread, made);
        }
    }

    /// A creature was harmed on `cell`: everyone standing in its body of water not yet struck is
    /// struck, in actor id order.
    fn conduct(&mut self, cell: Pos, spread: &mut Spread, made: &mut Vec<Event>) {
        let known = spread.bodies.iter().position(|b| b.contains(&cell));
        let body = match known {
            Some(i) => i,
            None => {
                let body = self.water_body(cell);
                if body.is_empty() {
                    return;
                }
                spread.bodies.push(body);
                spread.bodies.len() - 1
            }
        };
        let level = &self.state.current_level;
        let waders: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == *level && !a.in_retinue())
            .filter(|a| !spread.struck.contains(&a.id) && spread.bodies[body].contains(&a.pos))
            .map(|a| a.id)
            .collect();
        for id in waders {
            if self.state.death.is_none() {
                self.spread_strike(id, (cell, false), spread, made);
            }
        }
    }

    /// The body of water of `cell` on the current level: the open water connected to it over 8
    /// neighbours, none if it is no open water. Ice is not water, so it parts a pond.
    pub(super) fn water_body(&self, cell: Pos) -> BTreeSet<Pos> {
        let level = self.state.current();
        let water = |p: Pos| level.is_open_water(&self.content, p);
        let mut body = BTreeSet::new();
        if !water(cell) {
            return body;
        }
        body.insert(cell);
        let mut open = vec![cell];
        while let Some(p) = open.pop() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let near = Pos::new(p.x + dx, p.y + dy);
                    if water(near) && body.insert(near) {
                        open.push(near);
                    }
                }
            }
        }
        body
    }

    /// A creature was harmed on `from`: the nearest within reach not yet struck, ties by actor
    /// id, is struck if the draw says so (no draw with no one there, for a certain jump, or at a
    /// chance of 100).
    fn arc_from(&mut self, from: Pos, spread: &mut Spread, made: &mut Vec<Event>) {
        let Some(chance) = spread.chance else {
            return;
        };
        let level = self.state.current();
        let content = &self.content;
        let clear = |to: Pos| from.line_between(to).all(|p| !level.is_opaque(content, p));
        let nearest = (self.state.actors.values())
            .filter(|a| a.level == level.id && !a.in_retinue() && !spread.struck.contains(&a.id))
            .filter(|a| from.chebyshev(a.pos) <= spread.reach && clear(a.pos))
            .map(|a| (from.chebyshev(a.pos), a.id))
            .min();
        let Some((_, id)) = nearest else {
            return;
        };
        let goes = if spread.jumps > 0 {
            spread.jumps -= 1;
            true
        } else {
            chance >= 100 || self.state.rng.below(100) < u32::from(chance)
        };
        if goes {
            self.spread_strike(id, (from, true), spread, made);
        }
    }

    /// The cast strikes `target` from `source`, at the roll it settled.
    fn spread_strike(
        &mut self,
        target: ActorId,
        source: (Pos, bool),
        spread: &mut Spread,
        made: &mut Vec<Event>,
    ) {
        spread.struck.insert(target);
        let damage = Damage::new(spread.rolled, Some(spread.kind));
        let hit = Strike::Hit { critical: false };
        self.spell_strikes(spread.by, target, source, (damage, hit), made);
    }
}

/// `event`, if it is a blow of `by`'s spell: whom it struck at, on what cell, and how.
fn spell_blow(event: &Event, by: ActorId) -> Option<(ActorId, Pos, Strike)> {
    match *event {
        Event::Attacked {
            attacker,
            target,
            kind: AttackKind::Spell,
            to,
            strike: Some(strike),
            ..
        } if attacker == by => Some((target, to, strike)),
        _ => None,
    }
}
