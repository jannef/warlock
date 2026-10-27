//! Creature behavior (spec §8 "Creature behavior"): what a creature does with each action the
//! scheduler gives it. Deterministic, no randomness; outplayable by design.
//!
//! Each action, in order:
//! 1. **Look**: the hostile actors it sees, by the player's rules of sight with its own eyes and
//!    radius (`sight.rs`); the nearest is its target (ties: the lower actor id).
//! 2. **Fight**: a melee creature attacks a target next to it, else steps along a shortest path
//!    toward it; one that throws (a giant's boulders, `hurl.rs`) throws on the way whenever the
//!    target is in range with a clear line of fire. A ranged creature or caster shoots or casts
//!    when the target is within its range with a clear line of fire (no opaque cell and nobody
//!    in between), else steps toward the nearest cell that gives it one (failing that, toward
//!    the target). Leaving its routine, it remembers where it was (its post).
//! 3. **Search**: with no hostile in sight, it goes to where it last saw one.
//! 4. **Listen**: failing that, to where it last heard a noise whose source it could not see
//!    ([`crate::noise`]; what it heard since it last acted, the newest first). Only creatures
//!    with enemies (a faction hostile to someone) go to look; wildlife does not care.
//! 5. **Return**: then back to its post (waiting beside it while someone stands on it), and
//!    carries on with its patrol route (or waits). A creature the player swapped places with
//!    (`talk.rs`) has its old cell as its post, so it steps back onto its route.
//!
//! Paths (`path.rs`) go over walkable cells, for an incorporeal creature solid ones too
//! (`incorporeal.rs`, at a step's extra cost), and, for creatures that open doors, closed doors (it
//! opens the door, then steps in); a locked door only for one carrying a key that fits it (it
//! unlocks the door as it opens it; spec §8 "Doors"), else it is a wall to it. Other actors
//! block: a path may lead through one only when the way around is much longer, and then the
//! creature waits behind it. A creature hostile to the player never steps into a sanctuary (a
//! special room its effect makes creatures shun, spec §7 "Special rooms"): its paths go around
//! it, and it waits when the only way leads in. Melee attacks and shots are real (`fight.rs`,
//! `shoot.rs`: rolled against Defence, they hurt and kill; a shot spends an arrow, which flies
//! and comes down to lie where it fell); a ranged creature with nothing left to shoot fights in
//! melee. Spells stay intents until magic is designed. Each is an [`Event::Attacked`], a noise, and, if the player perceives it, a
//! message. Steps and opened doors make noise too. Creatures neither flee nor change how they
//! fight when hurt; a frightened one (a status that `flees`: Hound's bay) runs from who frightened
//! it (`sign_picks.rs`).

use super::path::{Cost, Search};
use super::stride::Coarse;
use super::{ActorId, Mind, World, beast_friend};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, Combat, Stance};
use crate::fov::radius2;
use crate::geom::{Dir, Pos};
use crate::noise::Heard;
use crate::sight::{self, LineOfSight};

/// Extra cost of a path through a cell another actor stands on, in steps: a creature walks a
/// detour of up to this many more steps around someone in its way, else waits behind them.
const CROWD: u32 = 4;
/// Extra cost of a closed door on a path: opening it takes an action.
const DOOR: u32 = 1;
/// Extra cost of a locked door on top of [`DOOR`], for a creature with the key: unlocking it.
const UNLOCK: u32 = 1;
/// Path searches look this many cells beyond the distance to their target...
const SEARCH_MARGIN: i32 = 12;
/// ...and at most this far from the creature.
const SEARCH_LIMIT: i32 = 64;

/// What a creature does with one action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Plan {
    Attack {
        target: ActorId,
        kind: AttackKind,
    },
    /// Call forth creatures of its side (`summon.rs`).
    Summon,
    Step(Dir),
    Patrol,
    Wait,
}

/// Where a path query leaves a creature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    /// It is already where it wants to be.
    There,
    Step(Dir),
    /// The way is blocked by someone for now.
    Wait,
    /// No way there.
    Unreachable,
}

/// What a step does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stepped {
    /// Moved, for this energy price.
    Moved(i64),
    /// Opened the closed door in the way instead, for this energy price (priced before the door
    /// swings: a trap on it may kill the opener).
    Opened(i64),
    /// Unlocked the locked door in the way with its key, and opened it, for this energy price.
    Unlocked(i64),
    Blocked,
}

impl World {
    /// One action of creature `id` on the current level, taken whenever the scheduler lets it act
    /// (phase 2; `schedule.rs` decides who acts when, and so how often). Returns the action's
    /// energy price, which with the creature's speed decides how soon it may act again: a step
    /// costs the step onto the destination ([`super::LevelState::step_price`]), anything else its
    /// action's cost (an attack, opening a door, waiting; [`crate::content::Content::action_cost`],
    /// where a creature type may give its attacks their own) over the 6-second base.
    ///
    /// This is the one place a creature decides and does something; whatever the action lets
    /// the player perceive (events, noise) starts here too. What the creature heard since it
    /// last acted ([`super::Actor::heard`]) is read here and spent.
    pub(super) fn creature_turn(&mut self, id: ActorId, events: &mut Vec<Event>) -> i64 {
        // Asleep (a teeming den's sleeper, `den.rs`), it takes no turn.
        if let Some(price) = self.slumbers(id) {
            return price;
        }
        // Away from a resting player, it takes coarse turns (`stride.rs`).
        let slow = match self.coarse_turn(id, events) {
            Coarse::Strode(price) => return price,
            Coarse::Slow => true,
            Coarse::Usual => false,
        };
        let heard = (self.state.actors.get_mut(&id))
            .map(|a| std::mem::take(&mut a.heard))
            .unwrap_or_default();
        // Frightened (Hound's bay, `sign_picks.rs`), it runs; held fast (Drowning grip,
        // `lucky.rs`), it fights back instead of stepping.
        let plan = self.decide(id, &heard);
        let plan = self.frightened(id, plan);
        let plan = self.held_back(id, plan);
        let price = match plan {
            Plan::Attack {
                target,
                kind: AttackKind::Ranged,
            } => self.creature_shoots(id, target, events),
            Plan::Attack {
                target,
                kind: AttackKind::Thrown,
            } => self.creature_hurls(id, target, events),
            Plan::Attack { target, kind } => {
                let skills = self.skills_of(id, kind, None);
                let with = self.state.actors[&id].weapon_item(&self.content);
                let with = with.filter(|_| kind == AttackKind::Melee);
                let price = self.attack_price(id, kind.action(), with, skills);
                self.attack(id, target, kind, events);
                price
            }
            Plan::Summon => self.creature_summons(id, events),
            Plan::Step(dir) => self.step_or_wait(id, dir, events).1,
            Plan::Patrol => self.patrol(id, events),
            Plan::Wait => self.price_of(id, ActionKind::Wait),
        };
        // Going about its routine, it may mutter to itself (`talk.rs`).
        self.mutter(id);
        if slow {
            price.saturating_mul(i64::from(super::stride::STRIDE))
        } else {
            price
        }
    }

    /// Creature `id` steps in `dir` ([`World::creature_steps`]); returns what it did and its
    /// price: the step, opening the door in the way, or waiting when blocked.
    fn step_or_wait(&mut self, id: ActorId, dir: Dir, events: &mut Vec<Event>) -> (Stepped, i64) {
        let stepped = self.creature_steps(id, dir, events);
        let price = match stepped {
            Stepped::Moved(price) | Stepped::Opened(price) | Stepped::Unlocked(price) => price,
            Stepped::Blocked => self.price_of(id, ActionKind::Wait),
        };
        (stepped, price)
    }

    /// Steps 1 to 5 of the module docs: what creature `id`, having `heard` what it heard since it
    /// last acted, does now. Updates its mind.
    fn decide(&mut self, id: ActorId, heard: &[Heard]) -> Plan {
        let seen = self.nearest_hostile_in_sight(id);
        let mut mind = self.state.actors[&id].mind;
        let pos = self.state.actors[&id].pos;
        let plan = match seen {
            Some((target, at)) => {
                mind.last_seen = Some(at);
                mind.noise = None;
                mind.post.get_or_insert(pos);
                self.fight(id, target, at)
            }
            None => {
                if let Some(at) = self.noise_to_look_into(id, heard) {
                    mind.noise = Some(at);
                    mind.post.get_or_insert(pos);
                }
                self.search_or_return(id, &mut mind)
            }
        };
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.mind = mind;
        }
        plan
    }

    /// Step 4: where creature `id` goes to look into what it heard: the newest noise whose source
    /// it does not see (by the rules of sight), if it has enemies at all.
    fn noise_to_look_into(&mut self, id: ActorId, heard: &[Heard]) -> Option<Pos> {
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let me = &state.actors[&id];
        let def = content.creature(me.creature);
        let has_enemies = me
            .faction(content)
            .is_some_and(|f| content.factions().has_enemies(f));
        if heard.is_empty() || !has_enemies {
            return None;
        }
        let radius = sight::sight_radius(state, content, def.sight);
        let level = state.current();
        let los = LineOfSight::compute(me.pos, radius, |p| level.is_opaque(content, p));
        let eye = me.eyes(content, &content.vision_profiles()[def.vision]);
        (heard.iter().rev())
            .map(|h| h.at)
            .find(|&at| !sight::sees(state, content, lights, &los, &eye, at))
    }

    /// The nearest hostile actor creature `id` sees (ties: the lower id), and where it stands.
    /// Sight is computed only if a hostile is within the creature's sight radius, nearest first,
    /// and only as far as the first one it sees: line of sight only out to its distance.
    fn nearest_hostile_in_sight(&mut self, id: ActorId) -> Option<(ActorId, Pos)> {
        let World {
            state,
            content,
            lights,
            ..
        } = self;
        let me = &state.actors[&id];
        let def = content.creature(me.creature);
        let radius = sight::sight_radius(state, content, def.sight);
        let reach2 = radius2(radius);
        let mut hostiles: Vec<(i32, ActorId, Pos)> = (state.actors.values())
            .filter(|a| a.level == me.level && a.id != id && a.pos.dist2(me.pos) <= reach2)
            .filter(|a| {
                content.stance_of(me.faction(content), a.faction(content)) == Stance::Hostile
                    && !beast_friend::befriended(state, content, me, a)
            })
            .map(|a| (a.pos.dist2(me.pos), a.id, a.pos))
            .collect();
        if hostiles.is_empty() {
            return None;
        }
        hostiles.sort_unstable();
        let level = state.current();
        let eye = me.eyes(content, &content.vision_profiles()[def.vision]);
        let mut los: Option<LineOfSight> = None;
        for (d2, other, p) in hostiles {
            let reach = LineOfSight::radius_for(d2).min(radius);
            let los = match los.take() {
                Some(l) if l.radius() >= reach => los.insert(l),
                _ => los.insert(LineOfSight::compute(me.pos, reach, |q| {
                    level.is_opaque(content, q)
                })),
            };
            if sight::sees(state, content, lights, los, &eye, p) {
                return Some((other, p));
            }
        }
        None
    }

    /// Step 2: attack the target at `at`, or get where the creature can. A ranged creature
    /// with nothing to shoot (no missile weapon ready, or no ammunition for it left) fights in
    /// melee; it shoots no farther than its missile weapon does.
    fn fight(&self, id: ActorId, target: ActorId, at: Pos) -> Plan {
        let me = &self.state.actors[&id];
        let def = self.content.creature(me.creature);
        let attack = |kind| Plan::Attack { target, kind };
        // A summoner (`summon.rs`) summons whenever it may.
        if self.summons_at(id, at) {
            return Plan::Summon;
        }
        // A creature that throws (`hurl.rs`) throws whenever it can from where it stands.
        if self.hurls_at(id, target, at) {
            return attack(AttackKind::Thrown);
        }
        let close_in = || match self.route(id, at, 1, |q| q.chebyshev(at) <= 1) {
            Route::Step(dir) => Plan::Step(dir),
            Route::There | Route::Wait | Route::Unreachable => Plan::Wait,
        };
        let shoots = (me.launcher(&self.content))
            .filter(|_| me.ammunition_item(&self.content).is_some())
            .and_then(|(launcher, _)| self.content.object(launcher).range);
        // A caster whose casting is blocked (Silenced, in a silent vault) fights in melee.
        let casts = (me.effects.blocking(&self.content, ActionKind::Cast)).is_none();
        let (kind, range) = match (def.combat, shoots) {
            (Combat::Ranged, Some(reach)) => (AttackKind::Ranged, def.range.min(reach)),
            (Combat::Caster, _) if casts => (AttackKind::Spell, def.range),
            _ if me.pos.chebyshev(at) <= 1 => return attack(AttackKind::Melee),
            _ => return close_in(),
        };
        let reach2 = radius2(range);
        let can_hit = |q: Pos| q.dist2(at) <= reach2 && self.clear_shot(q, at, id, target);
        match self.route(id, at, range, can_hit) {
            Route::There => attack(kind),
            Route::Step(dir) => Plan::Step(dir),
            Route::Wait => Plan::Wait,
            Route::Unreachable => close_in(),
        }
    }

    /// Steps 3 to 5: with no hostile in sight, go to where one was last seen, then to where a
    /// noise was heard, then back to the post, then carry on.
    fn search_or_return(&self, id: ActorId, mind: &mut Mind) -> Plan {
        for spot in [&mut mind.last_seen, &mut mind.noise] {
            let Some(at) = *spot else { continue };
            match self.go_look(id, at) {
                Some(plan) => return plan,
                None => *spot = None,
            }
        }
        if let Some(post) = mind.post {
            // Someone on it (the player who swapped places with it, perhaps): it waits beside
            // it until they are gone.
            let level = &self.state.current_level;
            let taken = (self.state.actor_at(level, post)).is_some_and(|a| a.id != id);
            let reach = i32::from(taken);
            match self.route(id, post, reach, |q| q.chebyshev(post) <= reach) {
                Route::Step(dir) => return Plan::Step(dir),
                Route::Wait => return Plan::Wait,
                Route::There if taken => return Plan::Wait,
                Route::There | Route::Unreachable => mind.post = None,
            }
        }
        Plan::Patrol
    }

    /// The way to cell `spot`, to look there; `None` once there, or if it cannot get there.
    /// When someone it cannot see stands on the spot (perhaps the one it lost in the dark), or
    /// no one can stand there (a door shut or kicked at), next to it will do.
    fn go_look(&self, id: ActorId, spot: Pos) -> Option<Plan> {
        let taken = self.state.actor_at(&self.state.current_level, spot);
        let solid = !self.state.current().is_walkable(&self.content, spot);
        let reach = i32::from(solid || taken.is_some_and(|a| a.id != id));
        match self.route(id, spot, reach, |q| q.chebyshev(spot) <= reach) {
            Route::Step(dir) => Some(Plan::Step(dir)),
            Route::Wait => Some(Plan::Wait),
            Route::There | Route::Unreachable => None,
        }
    }

    /// Whether a shot from `from` at `target` on `to` would fly clear: no opaque cell and no
    /// actor (but the shooter `shooter` itself, who would have moved) on the line between.
    pub(super) fn clear_shot(&self, from: Pos, to: Pos, shooter: ActorId, target: ActorId) -> bool {
        let level = self.state.current();
        let occupied = |p: Pos| {
            (self.state.actors.values())
                .any(|a| a.level == level.id && a.pos == p && a.id != shooter && a.id != target)
        };
        from.line_between(to)
            .all(|p| !level.is_opaque(&self.content, p) && !occupied(p))
    }

    /// How creature `id` gets to a cell `goal` accepts, all of which lie within `reach` of
    /// `target`: a shortest path over walkable cells (and closed doors, if it opens them) that
    /// no one else stands on.
    fn route(&self, id: ActorId, target: Pos, reach: i32, goal: impl Fn(Pos) -> bool) -> Route {
        let level = self.state.current();
        let me = &self.state.actors[&id];
        let opens_doors = self.content.creature(me.creature).opens_doors;
        // An incorporeal creature goes through solid terrain too (`incorporeal.rs`).
        let phases = self.content.incorporeal(me.creature).is_some();
        let phaseable = |p: Pos| phases && level.phaseable(&self.content, p);
        let others: Vec<Pos> = (self.state.actors.values())
            .filter(|a| a.level == level.id && a.id != id)
            .map(|a| a.pos)
            .collect();
        let occupied = |p: Pos| others.contains(&p);
        if goal(me.pos) {
            return Route::There;
        }
        let shuns = self.shuns(id);
        let cost = |p: Pos| {
            let Some(def) = level.terrain.get(p).map(|t| self.content.terrain(*t)) else {
                return Cost::Impassable;
            };
            if shuns(p) {
                return Cost::Impassable;
            }
            let enter = if def.walkable {
                1
            } else if phaseable(p) {
                1 + super::incorporeal::WALL
            } else if opens_doors && def.is_closed_door() {
                match level.locks.get(&p) {
                    None => 1 + DOOR,
                    Some(lock) if me.key_for(&self.content, lock).is_some() => 1 + DOOR + UNLOCK,
                    Some(_) => return Cost::Impassable,
                }
            } else {
                return Cost::Impassable;
            };
            Cost::Steps(if occupied(p) { enter + CROWD } else { enter })
        };
        let search = Search {
            from: me.pos,
            target,
            reach,
            bound: (me.pos.chebyshev(target) + SEARCH_MARGIN).min(SEARCH_LIMIT),
        };
        let free_goal = |p: Pos| {
            goal(p) && !occupied(p) && (level.is_walkable(&self.content, p) || phaseable(p))
        };
        match search.first_step(cost, free_goal) {
            Some(dir) if occupied(me.pos.step(dir)) => Route::Wait,
            Some(dir) => Route::Step(dir),
            None => Route::Unreachable,
        }
    }

    /// Which cells creature `id` keeps out of on the current level: a creature hostile to the
    /// player does not step into a special room whose effect it shuns (a sanctuary, spec §7
    /// "Special rooms") from outside it; one already inside may move about in it, and leave.
    pub(super) fn shuns(&self, id: ActorId) -> impl Fn(Pos) -> bool + '_ {
        let level = self.state.current();
        let hostile = self.stance(id, self.state.player) == Stance::Hostile;
        let inside = (self.state.actors.get(&id)).and_then(|me| level.room_at(me.pos));
        move |p: Pos| {
            hostile
                && level.room_at(p).is_some_and(|room| {
                    inside != Some(room) && self.content.room_effect(room.effect).shunned
                })
        }
    }

    /// Creature `id` steps in `dir`: onto a walkable cell no one stands on, or, for a creature
    /// that opens doors, opening a closed door there (unlocking it first if it is locked and the
    /// creature carries a key that fits; else it is blocked), which sets off a trap on the door
    /// ([`World::swing_open`]). Either makes noise. An overloaded
    /// creature opens doors but does not step. A creature hostile to the player does not step
    /// (or open a door) into a sanctuary ([`World::shuns`]): it waits.
    pub(super) fn creature_steps(
        &mut self,
        id: ActorId,
        dir: Dir,
        events: &mut Vec<Event>,
    ) -> Stepped {
        let level = self.state.current();
        let actor = &self.state.actors[&id];
        let (from, to) = (actor.pos, actor.pos.step(dir));
        let Some(def) = level.terrain.get(to).map(|t| self.content.terrain(*t)) else {
            return Stepped::Blocked;
        };
        // Held fast, it cannot step at all (a status that `blocks` steps; `lucky.rs`).
        let held = actor.effects.blocking(&self.content, ActionKind::Step);
        if self.shuns(id)(to) || held.is_some() {
            return Stepped::Blocked;
        }
        // An incorporeal creature slips through solid terrain, a closed door too, left shut.
        let phases = self.content.incorporeal(actor.creature).is_some();
        let through = phases && level.phaseable(&self.content, to);
        if !through && def.is_closed_door() && self.content.creature(actor.creature).opens_doors {
            let open = self.price_of(id, ActionKind::OpenDoor);
            let stepped = match level.locks.get(&to) {
                None => Stepped::Opened(open),
                Some(lock) if actor.key_for(&self.content, lock).is_some() => {
                    Stepped::Unlocked(self.price_of(id, ActionKind::Unlock) + open)
                }
                Some(_) => return Stepped::Blocked,
            };
            self.state.current_mut().locks.remove(&to);
            self.swing_open(id, to, events);
            return stepped;
        }
        // An overloaded creature cannot move (spec §8 "Burden"): it waits.
        let overloaded = !self.load_of(id).penalty().can_move();
        if !(def.walkable || through) || overloaded || self.state.actor_at(&level.id, to).is_some()
        {
            return Stepped::Blocked;
        }
        let fleeing = self.flees(id, to);
        // Stepping away from beside a hostile opportunist, it is struck as it turns to go
        // (`free_blow.rs`); struck down, it never steps.
        if !self.step_away(id, to, events) {
            return Stepped::Blocked;
        }
        // Ice there may give way under it (spec §5 "Ice"); never under an incorporeal one.
        let footing = if phases {
            super::ice::Footing::Plain
        } else {
            self.tread(id, from, to)
        };
        let onto = self.content.terrain(self.state.current().terrain[to]);
        let price = self.step_price(id, onto, fleeing) + self.tread_price(id, footing);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.pos = to;
        }
        events.push(Event::ActorMoved {
            actor: id,
            from,
            to,
        });
        self.after_tread(id, to, footing, events);
        self.make_step_noise(id, to, fleeing);
        // It knows nothing of traps (spec §8 "Traps"): one there goes off; a pit costs it its
        // next action. Nothing of an incorporeal one presses down.
        let climb = if phases {
            0
        } else {
            self.step_onto_trap(to, id, events)
        };
        // Stepping up beside a hostile gatekeeper, it is struck as it comes (`free_blow.rs`).
        self.step_up(id, from, events);
        Stepped::Moved(price + climb)
    }

    /// The routine: the next step of the creature's patrol route; it waits when blocked (or
    /// opens a closed door in the way, if it can) and when it has no route. Returns the price.
    fn patrol(&mut self, id: ActorId, events: &mut Vec<Event>) -> i64 {
        let Some(dir) = (self.state.actors[&id].patrol.as_ref()).map(|p| p.route[p.step]) else {
            return self.price_of(id, ActionKind::Wait);
        };
        let (stepped, price) = self.step_or_wait(id, dir, events);
        if let Stepped::Moved(_) = stepped
            && let Some(patrol) = (self.state.actors.get_mut(&id)).and_then(|a| a.patrol.as_mut())
        {
            patrol.step = (patrol.step + 1) % patrol.route.len();
        }
        price
    }
}
