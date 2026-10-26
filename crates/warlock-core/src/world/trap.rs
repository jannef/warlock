//! Traps (spec §8 "Traps"): what a trap does when it goes off.
//!
//! A trap lies on a cell of a level ([`crate::content::Trap`], in the level state), hidden until
//! the player sets it off or sees a creature do it; then it is known, shown, and remembered.
//! Whoever steps onto a trap sets it off, the player and creatures alike (creatures know nothing
//! of traps yet, so they walk onto them, and luring them is a tactic): [`World::step_onto_trap`].
//! A trap on a door (spec §8 "Trapped doors") is the door's, not the floor's: walking through
//! the doorway never sets it off; opening the door does, and kicking it (`world/door.rs`), at
//! whoever opened or kicked it from the next cell. [`World::spring_trap`] is the one way a trap
//! goes off: it spends a charge (a trap with none left is spent and gone), makes the trap's noise
//! on its cell, and does what the trap does ([`crate::content::TrapEffect`]):
//!
//! - **shot**: a missile shoots out of a hidden slit, the nearest wall in one of the eight
//!   directions (ties go to the direction a hash of the level and the cell picks first), at the
//!   victim: it flies like any missile, attacking whoever stands in its way in turn (rolled
//!   against their Defence with the trap's to-hit and damage) until it hits, and comes down to
//!   lie where it fell;
//! - **strike**: an attack roll against the victim;
//! - **burst**: everyone on the cells within the radius that the blast reaches from the trap
//!   (shadowcast: walls stop it, but not the trap's own cell, so a door's blast bursts out on
//!   both sides of it, shut or open) takes one roll of the damage, half of it (at least 1) beyond
//!   radius − 1; no roll to hit; then the walls it reaches that are softer than its `blast` break
//!   ([`World::blast`], spec §8 "Digging"); then a hot one melts the ice it reaches, and whoever
//!   stood on it falls in ([`World::shock`]), and a cold one freezes the water it reaches
//!   ([`World::frost`], spec §5 "Ice"). The same burst a scroll casts ([`World::cast_burst`]);
//! - **bolt**: a line from the trap in its direction (the level's `dir`; else, set off from the
//!   next cell (a door's), toward the victim; else the longest clear line from it, ties to the
//!   earlier of N, NE, ... NW) until a wall, at most its length: everyone on it takes one roll of
//!   the damage ([`World::cast_bolt`]);
//! - **pit**: the victim takes the damage and loses its next action climbing out (the action's
//!   price is a `wait` more);
//! - **alarm**: only its noise;
//! - **teleport**: the victim is moved to a free cell of the level (walkable, no one there, no
//!   trap, no ice: spec §5 "Ice"), picked by a hash of the seed, the level, the turn, the victim, and the trap's cell,
//!   never by the gameplay random stream. A creature goes on from where it lands.
//!
//! Every roll comes from the gameplay random stream. Events say what happened
//! ([`Event::TrapSprung`], then a missile, a spell, what the trap did to whom, and deaths), and
//! perception decides what the player perceived of it and says so.

use super::fight::Harm;
use super::spell::Caster;
use super::{ActorId, World, fnv1a};
use crate::action::{Event, MissileEnd};
use crate::check::check;
use crate::combat::{self, Blow, Profile, Strike};
use crate::content::StatUse;
use crate::content::{ActionKind, Bolt, ObjectId, TrapEffect, TrapId, article};
use crate::damage::Damage;
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use crate::perception::VIEW_RADIUS;
use crate::stats::Stat;
use crate::talent::TalentMods;
use std::collections::BTreeSet;

/// How far past its victim a trap's missile flies on, at most, when it misses.
const OVERSHOOT: usize = 8;

impl World {
    /// Actor `victim` has just stepped onto `at`: a trap there goes off ([`World::spring_trap`]),
    /// unless it is on a door, which only opening or kicking the door sets off (spec §8 "Trapped
    /// doors"). Returns the energy a pit's climb costs on top of the step.
    pub(super) fn step_onto_trap(
        &mut self,
        at: Pos,
        victim: ActorId,
        events: &mut Vec<Event>,
    ) -> i64 {
        if self.state.current().is_door(&self.content, at) || self.steps_lightly(at, victim) {
            return 0;
        }
        self.spring_trap(at, victim, events)
    }

    /// Whether the player, `victim`, steps onto the trap at `at` without setting it off (Light
    /// step, spec §8 "Talents"): a pit never catches them at its second rank; else a Dexterity
    /// check against the trap's difficulty (two draws of the gameplay stream, only with the
    /// talent and a trap there). Either way the trap stays, and is found.
    fn steps_lightly(&mut self, at: Pos, victim: ActorId) -> bool {
        if victim != self.state.player {
            return false;
        }
        let Some(kind) = self.state.current().traps.get(&at).map(|t| t.kind) else {
            return false;
        };
        let content = self.content.clone();
        let player = self.state.player();
        let talents = player.talent_mods(&content);
        if !talents.sure_step && !talents.pit_proof {
            return false;
        }
        let def = content.trap(kind);
        let pit = matches!(def.effect, TrapEffect::Pit { .. });
        let light = (talents.pit_proof && pit)
            || (talents.sure_step && {
                let dexterity = player.effective_stats(&content).modifier(Stat::Dexterity)
                    + talents.check(Stat::Dexterity);
                check(&mut self.state.rng, dexterity, i32::from(def.difficulty))
            });
        if light {
            self.know_trap(at);
            self.state.current_mut().trap_memory.insert(at, kind);
            self.say(format!(
                "You step lightly off {} {}.",
                article(&def.name),
                def.name
            ));
        }
        light
    }

    /// The trap on cell `at` of the current level goes off at actor `victim`, who stepped onto it
    /// (module docs); a trap set off from elsewhere (a trapped door, opened or kicked from the
    /// next cell) aims its missile or blades at the victim where it stands, its bolt (with no
    /// `dir` of its own) through them, and its blast at `at`. Pushes
    /// [`Event::TrapSprung`] and what came of it. Returns the energy the victim pays on top of its
    /// action: a pit's climb out, else 0. Nothing happens if no trap lies there.
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
        self.sound_at(def.noise, at);
        match &def.effect {
            TrapEffect::Shot { object, attack } => {
                self.trap_shot(kind, at, victim, *object, Profile::bare(attack), events);
            }
            TrapEffect::Strike { attack } => {
                let profile = Profile::bare(attack);
                self.trap_strikes(kind, victim, (at, false), &profile, events);
            }
            TrapEffect::Burst(burst) => {
                self.cast_burst(Caster::Trap(kind), (at, at), burst, events);
            }
            TrapEffect::Bolt(bolt) => {
                // Set off from the next cell (a door's trap), it runs at whoever did.
                let from = self.state.actors.get(&victim).map(|a| a.pos);
                let toward = from.filter(|p| p.chebyshev(at) == 1);
                let dir = dir.or_else(|| toward.and_then(|p| Dir::toward(at, p)));
                self.trap_bolt(kind, at, dir, bolt, events);
            }
            TrapEffect::Pit { damage, kind: of } => {
                if !self.state.actors.contains_key(&victim) {
                    return 0;
                }
                let climb = self.price_of(victim, ActionKind::Wait);
                let fall = Damage::new(damage.roll(&mut self.state.rng), *of);
                let from = self.state.actors.get(&victim).map_or(at, |a| a.pos);
                self.trap_hurts(kind, victim, (from, false), fall, events);
                if victim == self.state.player {
                    self.train_stat(StatUse::Climb);
                }
                return climb;
            }
            TrapEffect::Alarm => {}
            TrapEffect::Teleport => self.teleport(victim, at, events),
        }
        0
    }

    /// Trap `trap` attacks `target` with `profile` from `source` (a cell, and whether a missile
    /// flew from it), rolled against its Defence and soaked by its Protection like any blow: the
    /// event and the blow ([`World::trap_deals`]).
    fn trap_strikes(
        &mut self,
        trap: TrapId,
        target: ActorId,
        source: (Pos, bool),
        profile: &Profile,
        events: &mut Vec<Event>,
    ) -> Option<Blow> {
        let to = self.state.actors.get(&target)?.pos;
        let struck = self.target_against(target, source.1);
        let blow = combat::strike(profile, &struck, &mut self.state.rng);
        events.push(Event::TrapStruck {
            trap,
            target,
            to,
            strike: blow.strike,
        });
        self.trap_deals(trap, target, source, blow, events);
        Some(blow)
    }

    /// Trap `trap` does `damage` to `target` from `source` with no roll to hit (a blast, a bolt,
    /// a fall), less what the target resists of its kind (spec §8 "Damage types"; nothing left
    /// of it is [`Strike::Resisted`]).
    pub(super) fn trap_hurts(
        &mut self,
        trap: TrapId,
        target: ActorId,
        source: (Pos, bool),
        damage: Damage,
        events: &mut Vec<Event>,
    ) {
        let Some(to) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        let taken = self.resisted(target, damage);
        let mut blow = Blow::plain(taken, damage.kind);
        if damage.amount > 0 && taken <= 0 {
            blow.strike = Strike::Resisted;
        }
        events.push(Event::TrapStruck {
            trap,
            target,
            to,
            strike: blow.strike,
        });
        self.trap_deals(trap, target, source, blow, events);
    }

    /// Trap `trap` deals `blow` to `target`, from `source`, the way every blow is dealt (spec
    /// §8 "Blood", "Status effects"): a big hit's blood, the damage, which may kill (a trap's
    /// kill earns no one anything), and if it lives the statuses its wound causes. The target's
    /// talents scale the damage first (Trap sense: half, rounded down, at least 1).
    fn trap_deals(
        &mut self,
        trap: TrapId,
        target: ActorId,
        source: (Pos, bool),
        mut blow: Blow,
        events: &mut Vec<Event>,
    ) {
        if blow.damage <= 0 {
            return;
        }
        let percent = (self.state.actors.get(&target))
            .map_or(0, |a| a.talent_mods(&self.content).trap_damage_percent);
        if percent != 0 {
            let scaled = TalentMods::scaled(u64::try_from(blow.damage).unwrap_or(0), percent);
            blow.damage = i32::try_from(scaled).unwrap_or(i32::MAX).max(1);
        }
        self.spill(u32::from(trap.0), target, source, blow);
        if !self.hurt(target, blow.damage, Harm::Trap(trap), events) {
            self.wounded(target, &blow, None, events);
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
                let source = (slit, true);
                let blow = self.trap_strikes(trap, someone, source, &profile, &mut struck_events);
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
            self.come_down(end, Stack::one(object));
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
    /// it (ties: the earlier direction, N first): the trap's own cell is struck too.
    fn trap_bolt(
        &mut self,
        trap: TrapId,
        at: Pos,
        dir: Option<Dir>,
        bolt: &Bolt,
        events: &mut Vec<Event>,
    ) {
        let length = bolt.length;
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
        self.cast_bolt(Caster::Trap(trap), at, &path, bolt, events);
    }

    /// Moves `victim` from the teleport trap on `at` to a free cell of the level (module docs).
    pub(super) fn teleport(&mut self, victim: ActorId, at: Pos, events: &mut Vec<Event>) {
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
                    && level.ice_at(*p).is_none()
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

    /// Traps the player saw go off are known from now on (spec §8 "Traps"): those they set off
    /// themselves, and those set off where they see (a creature they see stepping onto one).
    pub(super) fn learn_traps(&mut self, events: &[Event]) {
        let player = self.state.player;
        for event in events {
            if let Event::TrapSprung { at, victim, .. } = *event
                && (victim == player || self.perception.sees_actors_at(at))
            {
                self.know_trap(at);
            }
        }
    }

    /// The trap at `at` on the current level is known from now on: the player saw it go off
    /// ([`World::learn_traps`]) or found it (spec §8 "Searching"). Returns its type, if a trap
    /// lies there.
    pub(super) fn know_trap(&mut self, at: Pos) -> Option<TrapId> {
        let trap = self.state.current_mut().traps.get_mut(&at)?;
        trap.known = true;
        Some(trap.kind)
    }
}
