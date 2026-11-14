//! Wedging a door shut (spec §8 "Doors"; #1027): Woodcraft's second use, the Wedge ability
//! ([`crate::ability::Ability::Wedge`], [`crate::Action::Wedge`]).
//!
//! The player spends a turn driving a wedge under the closed door next to them. The door is
//! then in the level's `wedged` map (saved, hashed; ground truth like a lock), holding the
//! points Woodcraft's `wedge` curve gave. A creature that opens doors cannot open it: it kicks it
//! instead, one kick a turn, against the terrain's `sturdiness` plus those points, as a kick
//! breaks any door ([`crate::door::kick_breaks`], its Strength score; the door then breaks for
//! good). The player opens it as usual, pulling the wedge free (the wedge is no item). The wedge
//! goes with the closed door ([`World::set_terrain`]). Refused, taking no time, without the
//! skill, a closed door that way, or a door already wedged. Marks Woodcraft ([`Use::Wedge`]).

use crate::ability::Ability;
use crate::action::Event;
use crate::content::ActionKind;
use crate::door::kick_breaks;
use crate::geom::{Dir, Pos};
use crate::noise::Act;
use crate::rng::draw;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::world::{ActorId, Spent, World};

impl World {
    /// The player wedges the door in `dir` shut (module docs).
    pub(in crate::world) fn wedge(&mut self, dir: Dir) -> Spent {
        let player = self.state.player;
        if !Ability::Wedge.had_by(self.state.player(), &self.content) {
            self.say("You do not know how to wedge a door.");
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let closed = (self.state.current().terrain.get(at))
            .is_some_and(|t| self.content.terrain(*t).is_closed_door());
        if !closed {
            self.say("There is no closed door there to wedge.");
            return Spent::Nothing;
        }
        if self.state.current().wedged.contains_key(&at) {
            self.say("That door is wedged already.");
            return Spent::Nothing;
        }
        let points = self
            .state
            .player()
            .skill_effect(&self.content, Effect::Wedge)
            .clamp(0, i32::from(u8::MAX));
        let points = u8::try_from(points).unwrap_or(u8::MAX);
        self.state.current_mut().wedged.insert(at, points);
        if self.trains(Use::Wedge) {
            self.train(Use::Wedge);
        }
        self.say("You drive a wedge under the door.");
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// What the wedge at `p` adds to the door's sturdiness; 0 where none is.
    pub(in crate::world) fn wedge_at(&self, p: Pos) -> u8 {
        self.state.current().wedged.get(&p).copied().unwrap_or(0)
    }

    /// Creature `id` kicks at the wedged door at `to` instead of opening it: a kick's price, and
    /// the door breaks with the chance of its Strength in the door's sturdiness and the wedge's
    /// points. Returns what it cost.
    pub(in crate::world) fn creature_kicks_wedge(
        &mut self,
        id: ActorId,
        to: Pos,
        events: &mut Vec<Event>,
    ) -> i64 {
        let price = self.price_of(id, ActionKind::Kick);
        let strength = self.state.actors[&id]
            .effective_stats(&self.content)
            .score(Stat::Strength);
        let wedge = self.wedge_at(to);
        let kicked_in = self
            .content
            .terrain(self.state.current().terrain[to])
            .kicked_in;
        let broken = kicked_in.and_then(|(into, sturdiness)| {
            kick_breaks(
                strength,
                sturdiness.saturating_add(wedge),
                &mut draw!(self, "wedge/kick/{},{}", to.x, to.y),
            )
            .then_some(into)
        });
        self.make_noise(id, Act::Kick(to));
        self.spring_trap(to, id, events);
        match broken {
            Some(into) => {
                self.set_terrain(to, into);
                events.push(Event::DoorBroken { pos: to });
            }
            None => events.push(Event::DoorKicked { pos: to }),
        }
        price
    }
}
