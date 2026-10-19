//! Traps (spec §8 "Traps"): what a trap does when it goes off.
//!
//! A trap lies on a cell of a level ([`crate::content::Trap`], in the level state), hidden until
//! the player sets it off or sees a creature do it; then it is known, shown, and remembered.
//! Whoever steps onto a trap sets it off, the player and creatures alike (creatures know nothing
//! of traps yet, so they walk onto them, and luring them is a tactic). [`World::spring_trap`] is
//! the one way a trap goes off: it spends a charge (a trap with none left is spent and gone),
//! makes the trap's noise on its cell, and does what the trap does
//! ([`crate::content::TrapEffect`]):
//!
//! - **shot**: a missile shoots out of a hidden slit, the nearest wall in one of the eight
//!   directions (ties go to the direction a hash of the level and the cell picks first), at the
//!   victim: it flies like any missile, attacking whoever stands in its way in turn (rolled
//!   against their Defence with the trap's to-hit and damage) until it hits, and comes down to
//!   lie where it fell;
//! - **strike**: an attack roll against the victim;
//! - **burst**: everyone on the cells within the radius that the blast reaches from the trap
//!   (shadowcast: walls stop it) takes one roll of the damage, half of it (at least 1) beyond
//!   radius − 1; no roll to hit; then the walls it reaches that are softer than its `blast` break
//!   ([`World::blast`], spec §8 "Digging");
//! - **bolt**: a line from the trap in its direction (the level's `dir`, else the longest clear
//!   line from it, ties to the earlier of N, NE, ... NW) until a wall, at most its length:
//!   everyone on it takes one roll of the damage;
//! - **pit**: the victim takes the damage and loses its next action climbing out (the action's
//!   price is a `wait` more);
//! - **alarm**: only its noise;
//! - **teleport**: the victim is moved to a free cell of the level (walkable, no one there, no
//!   trap), picked by a hash of the seed, the level, the turn, the victim, and the trap's cell,
//!   never by the gameplay random stream. A creature goes on from where it lands.
//!
//! Every roll comes from the gameplay random stream. Events say what happened
//! ([`Event::TrapSprung`], then a missile, a spell, what the trap did to whom, and deaths), and
//! perception decides what the player perceived of it and says so.

use super::fight::Cause;
use super::{ActorId, World, fnv1a};
use crate::action::{Event, MissileEnd, SpellShape};
use crate::combat::{self, Blow, Profile, Strike};
use crate::content::{ActionKind, ObjectId, TrapEffect, TrapId};
use crate::dice::Dice;
use crate::fov::{compute_fov, radius2};
use crate::geom::{Dir, Pos};
use crate::noise::{self, Noise};
use crate::perception::VIEW_RADIUS;
use std::collections::BTreeSet;

/// How far past its victim a trap's missile flies on, at most, when it misses.
const OVERSHOOT: usize = 8;

impl World {
    /// The trap on cell `at` of the current level goes off at actor `victim`, who stepped onto it
    /// (module docs). Pushes [`Event::TrapSprung`] and what came of it. Returns the energy the
    /// victim pays on top of its action: a pit's climb out, else 0. Nothing happens if no trap
    /// lies there.
    pub(super) fn spring_trap(&mut self, at: Pos, victim: ActorId, events: &mut Vec<Event>) -> i64 {
        let content = self.content.clone();
        let level = self.state.current_mut();
        let Some(trap) = level.traps.get_mut(&at) else {
            return 0;
        };
        let (kind, dir) = (trap.kind, trap.dir);
        if let Some(charges) = &mut trap.charges {
            *charges = charges.saturating_sub(1);
            if *charges == 0 {
                level.traps.remove(&at);
            }
        }
        self.sprung += 1;
        events.push(Event::TrapSprung {
            trap: kind,
            at,
            victim,
        });
        let def = content.trap(kind);
        if let Some(sound) = def.noise {
            let noise = Noise {
                kind: sound.sound,
                at,
                loudness: sound.loudness,
                maker: None,
            };
            noise::make(&mut self.state, &content, noise);
        }
        match &def.effect {
            TrapEffect::Shot { object, attack } => {
                self.trap_shot(kind, at, victim, *object, Profile::bare(attack), events);
            }
            TrapEffect::Strike { attack } => {
                self.trap_strikes(kind, victim, &Profile::bare(attack), events);
            }
            TrapEffect::Burst {
                spell,
                radius,
                damage,
                blast,
            } => {
                self.trap_burst(kind, at, spell, *radius, *damage, events);
                if *blast > 0 {
                    self.blast(at, *radius, *blast, events);
                }
            }
            TrapEffect::Bolt {
                spell,
                length,
                damage,
            } => self.trap_bolt(kind, at, dir, (spell, *length), *damage, events),
            TrapEffect::Pit { damage } => {
                if !self.state.actors.contains_key(&victim) {
                    return 0;
                }
                let climb = self.price_of(victim, ActionKind::Wait);
                let fall = damage.roll(&mut self.state.rng);
                self.trap_hurts(kind, victim, fall, events);
                return climb;
            }
            TrapEffect::Alarm => {}
            TrapEffect::Teleport => self.teleport(victim, at, events),
        }
        0
    }

    /// Trap `trap` attacks `target` with `profile`, rolled against its Defence: the event and the
    /// damage, which may kill.
    fn trap_strikes(
        &mut self,
        trap: TrapId,
        target: ActorId,
        profile: &Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let to = self.state.actors.get(&target)?.pos;
        let defence = self.defence_of(target);
        let blow = combat::strike(profile, &defence, &mut self.state.rng);
        events.push(Event::TrapStruck {
            trap,
            target,
            to,
            strike: blow.strike,
        });
        if blow.damage > 0 {
            self.hurt(target, blow.damage, Cause::Trap(trap), events);
        }
        Some(blow)
    }

    /// Trap `trap` does `damage` to `target` with no roll to hit (a blast, a fall).
    fn trap_hurts(&mut self, trap: TrapId, target: ActorId, damage: i32, events: &mut Vec<Event>) {
        let Some(to) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        events.push(Event::TrapStruck {
            trap,
            target,
            to,
            strike: Strike::Hit { critical: false },
        });
        if damage > 0 {
            self.hurt(target, damage, Cause::Trap(trap), events);
        }
    }

    /// A shot trap on `at` shoots `object` out of its slit at `victim` (module docs).
    fn trap_shot(
        &mut self,
        trap: TrapId,
        at: Pos,
        victim: ActorId,
        object: ObjectId,
        profile: Profile,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let target = self.state.actors.get(&victim).map_or(at, |a| a.pos);
        let slit = self.slit(at);
        let reach = slit.chebyshev(target).max(1) as usize + OVERSHOOT;
        let mut path = Vec::new();
        let mut struck_events = Vec::new();
        let mut struck = false;
        for cell in slit.ray(target).take(reach) {
            let level = self.state.current();
            if level.is_opaque(&content, cell) {
                break;
            }
            path.push(cell);
            let someone = self.state.actor_at(&level.id, cell).map(|a| a.id);
            if let Some(someone) = someone {
                let blow = self.trap_strikes(trap, someone, &profile, &mut struck_events);
                if blow.is_some_and(|b| b.strike.is_hit()) {
                    struck = true;
                    break;
                }
            }
        }
        let end = path.last().copied().unwrap_or(target);
        let breaks = content.object(object).breaks;
        let broke = struck && breaks > 0 && self.state.rng.chance(u32::from(breaks));
        if !broke && self.state.current().is_walkable(&content, end) {
            self.come_down(end, object);
        }
        events.push(Event::Missile {
            object,
            from: Some(slit),
            path,
            end: if struck {
                MissileEnd::Struck
            } else {
                MissileEnd::Fell
            },
        });
        events.extend(struck_events);
    }

    /// Where a shot trap on `at` shoots from: the nearest opaque cell in the eight directions
    /// (outside the level counts), within [`VIEW_RADIUS`]; ties go to the direction a hash of the
    /// level and the cell picks first. With none, the cell that far in that direction.
    fn slit(&self, at: Pos) -> Pos {
        let level = self.state.current();
        let mut key = level.id.as_bytes().to_vec();
        key.extend_from_slice(&at.x.to_le_bytes());
        key.extend_from_slice(&at.y.to_le_bytes());
        let first = (fnv1a(&key) % 8) as usize;
        let mut best: Option<(i32, Pos)> = None;
        for k in 0..Dir::ALL.len() {
            let dir = Dir::ALL[(first + k) % Dir::ALL.len()];
            let mut p = at;
            for d in 1..=VIEW_RADIUS {
                p = p.step(dir);
                if level.is_opaque(&self.content, p) {
                    if best.is_none_or(|(nearest, _)| d < nearest) {
                        best = Some((d, p));
                    }
                    break;
                }
            }
        }
        let far = || {
            let (dx, dy) = Dir::ALL[first].delta();
            Pos::new(at.x + dx * VIEW_RADIUS, at.y + dy * VIEW_RADIUS)
        };
        best.map_or_else(far, |(_, p)| p)
    }

    /// The transparent cells within `radius` of `at` that a blast from there reaches (it does not
    /// go through walls), in position order.
    pub(super) fn burst_cells(&self, at: Pos, radius: i32) -> Vec<Pos> {
        let level = self.state.current();
        let opaque = |p: Pos| level.is_opaque(&self.content, p);
        let mut cells = Vec::new();
        compute_fov(at, radius, opaque, |p| {
            if !opaque(p) {
                cells.push(p);
            }
        });
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    /// A burst trap on `at` goes off (module docs).
    fn trap_burst(
        &mut self,
        trap: TrapId,
        at: Pos,
        spell: &str,
        radius: i32,
        damage: Dice,
        events: &mut Vec<Event>,
    ) {
        let cells = self.burst_cells(at, radius);
        events.push(Event::Spell {
            effect: spell.to_string(),
            origin: at,
            shape: SpellShape::Burst { center: at, radius },
            cells: cells.clone(),
        });
        let roll = damage.roll(&mut self.state.rng);
        let inner = radius2(radius - 1);
        let level = self.state.current_level.clone();
        let caught: Vec<(ActorId, Pos)> = (self.state.actors.values())
            .filter(|a| a.level == level && cells.binary_search(&a.pos).is_ok())
            .map(|a| (a.id, a.pos))
            .collect();
        for (id, p) in caught {
            let edge = p.dist2(at) > inner;
            let damage = if edge { (roll / 2).max(1) } else { roll };
            self.trap_hurts(trap, id, damage, events);
        }
    }

    /// The cells a bolt from `at` runs over in `dir`: the trap's cell, then on until a wall or
    /// `length` cells in all.
    pub(super) fn bolt_path(&self, at: Pos, dir: Dir, length: i32) -> Vec<Pos> {
        let level = self.state.current();
        let mut path = vec![at];
        let mut p = at;
        while path.len() < length.max(1) as usize {
            p = p.step(dir);
            if level.is_opaque(&self.content, p) {
                break;
            }
            path.push(p);
        }
        path
    }

    /// A bolt trap on `at` goes off along `dir`, or with none along the longest clear line from
    /// it (ties: the earlier direction, N first).
    fn trap_bolt(
        &mut self,
        trap: TrapId,
        at: Pos,
        dir: Option<Dir>,
        (spell, length): (&str, i32),
        damage: Dice,
        events: &mut Vec<Event>,
    ) {
        let dir = dir.unwrap_or_else(|| {
            let mut best = (0, Dir::N);
            for d in Dir::ALL {
                let run = self.bolt_path(at, d, length).len();
                if run > best.0 {
                    best = (run, d);
                }
            }
            best.1
        });
        let path = self.bolt_path(at, dir, length);
        events.push(Event::Spell {
            effect: spell.to_string(),
            origin: at,
            shape: SpellShape::Bolt { path: path.clone() },
            cells: path.clone(),
        });
        let roll = damage.roll(&mut self.state.rng);
        let level = self.state.current_level.clone();
        let mut caught = Vec::new();
        for p in &path {
            caught.extend(self.state.actor_at(&level, *p).map(|a| a.id));
        }
        for id in caught {
            self.trap_hurts(trap, id, roll, events);
        }
    }

    /// Moves `victim` from the teleport trap on `at` to a free cell of the level (module docs).
    fn teleport(&mut self, victim: ActorId, at: Pos, events: &mut Vec<Event>) {
        let state = &self.state;
        let Some(from) = state.actors.get(&victim).map(|a| a.pos) else {
            return;
        };
        let level = state.current();
        let taken: BTreeSet<Pos> = (state.actors.values())
            .filter(|a| a.level == level.id)
            .map(|a| a.pos)
            .collect();
        let free: Vec<Pos> = (level.terrain.iter())
            .map(|(p, _)| p)
            .filter(|p| {
                level.is_walkable(&self.content, *p)
                    && !level.traps.contains_key(p)
                    && !taken.contains(p)
            })
            .collect();
        if free.is_empty() {
            return;
        }
        let mut key = state.seed.to_le_bytes().to_vec();
        key.extend_from_slice(level.id.as_bytes());
        key.extend_from_slice(&state.turn.to_le_bytes());
        key.extend_from_slice(&victim.0.to_le_bytes());
        key.extend_from_slice(&at.x.to_le_bytes());
        key.extend_from_slice(&at.y.to_le_bytes());
        let to = free[(fnv1a(&key) % free.len() as u64) as usize];
        // A creature goes on from where it lands (back to its post, if it had left one).
        if let Some(actor) = self.state.actors.get_mut(&victim) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: victim,
            from,
            to,
        });
    }
}
