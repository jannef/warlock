//! Coarse turns while the player rests (spec §8 "Rest", #349): a creature away from the player
//! takes its turns in coarse steps, so a night's rest on a floor full of creatures stays fast.
//! Outside a rest nothing here applies, and near the player a rest is played turn by turn.
//!
//! As its action begins while the player rests, a creature on a local level (not the
//! overworld) with no dialogue is **away** ([`World::away`]) unless it is hostile to the player
//! and the player, or a spot its mind is set on (where it last saw a hostile, a noise it goes to
//! look into), is within its sight radius of the player (Chebyshev, which covers the circle).
//! An away creature:
//!
//! - **strides** if it is also **idle**: calm ([`super::Mind::is_calm`]), with nothing heard
//!   since it last acted that it would go to look into (it has enemies), no status effects, able
//!   to move, and no hostile actor within its sight radius. Instead of one action it takes up to
//!   [`STRIDE`] steps of its patrol route at once, priced as those steps are, each onto a plain
//!   cell (walkable, no door, no ice, no trap, nobody there, not one it shuns) from which no
//!   hostile actor is within its sight radius, stopping before the first that is not, and
//!   makes one step's noise where it ends. Without a route it waits [`STRIDE`] waits at once.
//!   Its pace is its own (the same energy for the same steps), so a patrol still walks into
//!   view and stops a rest; it never strides to where it could see the player.
//! - otherwise (busy with something away from the player: a fight with another creature, a
//!   noise to look into, the way back to its post) acts as usual, but its action costs
//!   [`STRIDE`] times its price: away from the player, the rest's world runs that much slower
//!   for it.
//!
//! What it saves: deciding (sight, paths), the noise of every step but a stride's last, and the
//! scheduler's turn for each step. What it changes, in a rest only: an away creature notices a
//! hostile or a noise up to a stride later, its steps within a stride are not heard, and the
//! fights and searches of the busy ones go slower. [`World::out_of_view_for`] counts a stride's
//! head start. Deterministic: no randomness, actors in the scheduler's order.

use super::{ActorId, Patrol, World};
use crate::action::Event;
use crate::content::{ActionKind, LevelKind, Stance, TerrainId};
use crate::geom::Pos;
use crate::noise::{self, Act};
use crate::sight;

/// How many steps (or waits) an idle creature away from a resting player takes at once, and
/// how many times its price a busy one's action costs: two minutes' walk at normal speed.
pub(super) const STRIDE: i32 = 20;

/// How a creature takes its turn as the player rests.
pub(super) enum Coarse {
    /// It strode, for this price.
    Strode(i64),
    /// It acts as usual, at [`STRIDE`] times the price.
    Slow,
    /// It acts as usual.
    Usual,
}

impl World {
    /// Creature `id`'s turn as the player rests (the module docs): a stride, done, if it is away
    /// and idle; slowed if it is away and busy; else as usual.
    pub(super) fn coarse_turn(&mut self, id: ActorId, events: &mut Vec<Event>) -> Coarse {
        if !self.away(id) {
            return Coarse::Usual;
        }
        let Some(foes) = self.idle(id) else {
            return Coarse::Slow;
        };
        self.stride(id, &foes, events)
            .map_or(Coarse::Usual, Coarse::Strode)
    }

    /// Whether creature `id` takes coarse turns now: the player rests and it is away from them
    /// (the module docs).
    fn away(&self, id: ActorId) -> bool {
        if !self.resting {
            return false;
        }
        let state = &self.state;
        let me = &state.actors[&id];
        if state.current().kind == LevelKind::Overworld || self.dialogue_of(me).is_some() {
            return false;
        }
        // One that means the player no harm may stride beside them.
        if self.stance(id, state.player) != Stance::Hostile {
            return true;
        }
        let radius = self.sight_of(id);
        let player = self.player_pos();
        let near = |p: Pos| p.chebyshev(player) <= radius;
        !near(me.pos) && !me.mind.last_seen.is_some_and(near) && !me.mind.noise.is_some_and(near)
    }

    /// Idle creature `id`'s action as a stride, kept clear of `foes` (the module docs); returns
    /// its price, or `None` when it cannot take a step of it and acts as usual.
    fn stride(&mut self, id: ActorId, foes: &[Pos], events: &mut Vec<Event>) -> Option<i64> {
        let Some(route) = self.state.actors[&id].patrol.clone() else {
            return Some(self.price_of(id, ActionKind::Wait) * i64::from(STRIDE));
        };
        let (cells, price) = self.stride_path(id, &route, foes);
        let Some((&last, _)) = cells.split_last() else {
            // Someone stands in its way: it waits, as it would, a stride's waits at once.
            let pos = self.state.actors[&id].pos;
            let next = pos.step(route.route[route.step]);
            let level = &self.state.current_level;
            return (self.state.actor_at(level, next).is_some())
                .then(|| self.price_of(id, ActionKind::Wait) * i64::from(STRIDE));
        };
        let mut from = self.state.actors[&id].pos;
        for &to in &cells {
            events.push(Event::ActorMoved {
                actor: id,
                from,
                to,
            });
            from = to;
        }
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.pos = last;
            // What it heard was nothing it would look into.
            actor.heard.clear();
            if let Some(patrol) = actor.patrol.as_mut() {
                patrol.step = (route.step + cells.len()) % route.route.len();
            }
        }
        self.make_step_noise(id, last, false);
        Some(price)
    }

    /// The cells creature `id` strides over along `route` and what the steps cost: up to
    /// [`STRIDE`] steps, each onto a plain cell (walkable, no door, ice, or trap, nobody there,
    /// not one it shuns) from which it sees none of `foes` (as far as its sight reaches).
    fn stride_path(&self, id: ActorId, route: &Patrol, foes: &[Pos]) -> (Vec<Pos>, i64) {
        let (level, content) = (self.state.current(), &self.content);
        let me = &self.state.actors[&id];
        let radius = self.sight_of(id);
        let others: Vec<Pos> = (self.state.actors.values())
            .filter(|a| a.id != id && a.level == level.id)
            .map(|a| a.pos)
            .collect();
        let shuns = self.shuns(id);
        let plain = |to: Pos| {
            (level.terrain.get(to)).is_some_and(|t| content.terrain(*t).walkable)
                && !level.is_door(content, to)
                && level.ice_at(to).is_none()
                && !level.traps.contains_key(&to)
                && !others.contains(&to)
                && !shuns(to)
                // Where the player saw creatures as they last looked, it goes turn by turn.
                && !self.perception.sees_actors_at(to)
        };
        // Every step onto the same terrain costs it the same and is as loud: it carries no
        // farther than a cell for every `STEP_LOSS` of loudness.
        let burden = self.state.load(content, me).penalty();
        let mut steps: Vec<(TerrainId, i64, i32)> = Vec::new();
        let mut step_onto = |to: Pos, terrain: TerrainId| {
            if let Some(&(_, price, reach)) = steps.iter().find(|(t, ..)| *t == terrain) {
                return (price, reach);
            }
            let price = self.step_price(id, content.terrain(terrain), false);
            let loudness =
                noise::noise(content, level, me, burden, Act::Step(to)).map_or(0, |n| n.loudness);
            let reach = i32::from(loudness / noise::STEP_LOSS);
            steps.push((terrain, price, reach));
            (price, reach)
        };
        let player = self.player_pos();
        let (mut at, mut step, mut price) = (me.pos, route.step, 0);
        let mut cells = Vec::new();
        for _ in 0..STRIDE {
            let to = at.step(route.route[step]);
            if foes.iter().any(|f| f.chebyshev(to) <= radius) || !plain(to) {
                break;
            }
            let (cost, reach) = step_onto(to, level.terrain[to]);
            // Where the player could hear its step, it goes turn by turn too.
            if to.chebyshev(player) <= reach {
                break;
            }
            price += cost;
            cells.push(to);
            (at, step) = (to, (step + 1) % route.route.len());
        }
        (cells, price)
    }

    /// How far creature `id` sees now (Chebyshev covers it: sight is a circle).
    fn sight_of(&self, id: ActorId) -> i32 {
        let def = self.content.creature(self.state.actors[&id].creature);
        sight::sight_radius(&self.state, &self.content, def.sight)
    }

    /// Where the hostile actors stand that creature `id` could come to see within a stride, if
    /// it is idle enough to stride (the module docs); `None` if it is not.
    fn idle(&self, id: ActorId) -> Option<Vec<Pos>> {
        let (state, content) = (&self.state, &self.content);
        let me = &state.actors[&id];
        let faction = me.faction(content);
        let has_enemies = faction.is_some_and(|f| content.factions().has_enemies(f));
        if !me.mind.is_calm()
            || (has_enemies && !me.heard.is_empty())
            || !me.effects.is_empty()
            || !self.load_of(id).penalty().can_move()
        {
            return None;
        }
        let radius = self.sight_of(id);
        let foes: Vec<(i32, Pos)> = (state.actors.values())
            .filter(|a| a.id != id)
            .map(|a| (a.pos.chebyshev(me.pos), a))
            .filter(|(d, a)| *d <= radius + STRIDE && a.level == me.level)
            .filter(|(_, a)| match a.id == state.player {
                // The player's stance has more to it (Beast-friend).
                true => self.stance(id, a.id) == Stance::Hostile,
                false => content.stance_of(faction, a.faction(content)) == Stance::Hostile,
            })
            .map(|(d, a)| (d, a.pos))
            .collect();
        // One it may see now is looked at the usual way.
        (foes.iter().all(|(d, _)| *d > radius)).then(|| foes.into_iter().map(|(_, p)| p).collect())
    }
}
