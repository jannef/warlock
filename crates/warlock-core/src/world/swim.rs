//! Deep water (spec §8 "Skills": Swim; #1019): the one step `walkable = false` lets through, for a
//! trained swimmer, and what the water asks of them. Everything else keeps out of it because it
//! is not walkable: paths, travel, spawning, teleports, blinks, and landings.
//!
//! **Going in** (from anything else): presentation asks first
//! (`PerceptionView::dives_at`); the rules refuse the untrained and the Strained with no
//! time ([`World::at_the_shore`]), and stop the player, with no time, at water they did not know
//! was there, so that the next step can ask. It needs no check, and it puts out the ready light
//! ([`World::dive_in`]).
//!
//! **A step in it**, one that starts in the water wherever it ends, is a check
//! ([`World::stroke_fails`]; `data/swim.toml`): made, the step; failed, the turn goes on treading
//! water, with no move, and the cold takes a hit point, never the last. Two gameplay draws (the
//! 2d12), and [`Use::Stroke`] whatever came of it.
//!
//! **Afloat**: dropping, picking up, resting, searching, digging, and lighting a light are refused
//! with no time ([`World::refused_afloat`]); the rest goes on as anywhere. Nothing drags a swimmer
//! down.
//!
//! **Creatures** stay out and close in on the shore ([`World::route_to_shore`]); allies too, and
//! none is swapped into the water (`Refusal::Water`).

use super::ai::Route;
use super::fight::Harm;
use super::{Actor, ActorId, Event, Spent, World};
use crate::burden::Burden;
use crate::content::Content;
use crate::damage::Damage;
use crate::geom::Pos;
use crate::object::LightMode;
use crate::skills::{Effect, Use};
use crate::stats::Stat;

/// How far from a swimmer a creature that cannot reach them looks for a place to wait.
const SHORE_REACH: i32 = 16;

/// Why `player`, under `burden`, may not dive in, if they may not: the rules' words. Read by
/// presentation too, which asks no question of one the rules would refuse.
pub(crate) fn shore_refusal(
    content: &Content,
    player: &Actor,
    burden: Burden,
) -> Option<&'static str> {
    if content.swim().is_none() || !player.skills.trains_effect(content, Effect::Swim) {
        return Some("You cannot swim.");
    }
    (burden_above(burden) >= 2).then_some("You would sink.")
}

/// How many levels `burden` is above Unburdened.
fn burden_above(burden: Burden) -> i32 {
    match burden {
        Burden::Unburdened => 0,
        Burden::Burdened => 1,
        Burden::Strained => 2,
        Burden::Overloaded => 3,
    }
}

impl World {
    /// Whether the player is in deep water.
    pub(super) fn swimming(&self) -> bool {
        let level = self.state.current();
        (level.terrain.get(self.player_pos())).is_some_and(|t| self.content.terrain(*t).swim)
    }

    /// `said`, with no time, if the player is in deep water. Whether they are.
    pub(super) fn refused_afloat(&mut self, said: &str) -> bool {
        let swimming = self.swimming();
        if swimming {
            self.say(said);
        }
        swimming
    }

    /// A step from `from` onto deep water at `to`: what it comes to instead, if it does not go
    /// ahead. From in the water, nothing stops it here. From the shore: water the player did not
    /// know of is found, with no time, so they can be asked before they step in (a wall in the dark
    /// is found the same way, by walking into it); and the untrained and the Strained are refused.
    pub(super) fn at_the_shore(&mut self, from: Pos, to: Pos) -> Option<Spent> {
        let level = self.state.current();
        if self.content.terrain(level.terrain[from]).swim {
            return None;
        }
        if level.memory.get(to).is_none_or(Option::is_none) {
            let terrain = level.terrain[to];
            self.state.current_mut().memory.set(to, Some(terrain));
            self.say("Your foot finds no bottom: deep water lies ahead.");
            return Some(Spent::NoTime);
        }
        let burden = self.load_of(self.state.player).penalty();
        let said = shore_refusal(&self.content, self.state.player(), burden)?;
        self.say(said);
        Some(Spent::Nothing)
    }

    /// A step that starts where the player stands (`player_moves`, once nothing else stands in its
    /// way): in deep water it is a check (the module docs). The price of the turn spent treading
    /// water, if it failed; `None` if the step goes ahead.
    pub(super) fn stroke_fails(&mut self, events: &mut Vec<Event>) -> Option<i64> {
        if !self.swimming() {
            return None;
        }
        let content = self.content.clone();
        let rules = content.swim()?;
        let player = self.state.player;
        let me = self.state.player();
        let modifier =
            me.check_modifier(&content, Stat::Toughness) + me.skill_effect(&content, Effect::Swim);
        let burden = burden_above(self.load_of(player).penalty());
        let difficulty = rules.difficulty + rules.per_burden * burden;
        let made = crate::check::check(&mut self.state.rng, modifier, difficulty);
        if self.trains(Use::Stroke) {
            self.train(Use::Stroke);
        }
        if made {
            return None;
        }
        let here = content.terrain(self.state.current().terrain[self.player_pos()]);
        let price = self.step_price(player, here, false);
        self.say("You tread water and make no headway, and the cold bites.");
        let taken = self.resisted(player, Damage::new(rules.cold, Some(rules.damage_type)));
        self.hurt(player, taken, Harm::Spared, events);
        Some(price)
    }

    /// The player, just moved from `from` to `to`: going into deep water from anything else puts
    /// out their ready light.
    pub(super) fn dive_in(&mut self, from: Pos, to: Pos) {
        let level = self.state.current();
        let swim = |p: Pos| self.content.terrain(level.terrain[p]).swim;
        if swim(to) && !swim(from) {
            self.say("You dive into the deep water.");
            self.douse_afloat();
        }
    }

    /// In deep water, the player's ready light goes out, if it is burning.
    pub(super) fn douse_afloat(&mut self) {
        if !self.swimming() {
            return;
        }
        let me = self.state.player();
        let Some(item) = me.ready_light(&self.content) else {
            return;
        };
        if me.inventory[item].light_mode() != LightMode::Unlit {
            let name = me.inventory[item].title(&self.content);
            self.snuff_light();
            self.say(format!("Your {name} goes out."));
        }
    }

    /// How creature `id` closes to within `reach` of `at`, as [`World::route`] does. A swimmer may
    /// have no ground that near: then it goes to the nearest there is, and waits on the shore. Only
    /// a target in deep water searches more than once.
    pub(super) fn route_to_shore(&self, id: ActorId, at: Pos, reach: i32) -> Route {
        let route = |r: i32| self.route(id, at, r, |q| q.chebyshev(at) <= r);
        let first = route(reach);
        let level = self.state.current();
        let afloat = (level.terrain.get(at)).is_some_and(|t| self.content.terrain(*t).swim);
        if !afloat || !matches!(first, Route::Unreachable) {
            return first;
        }
        ((reach + 1)..=SHORE_REACH)
            .map(route)
            .find(|r| !matches!(r, Route::Unreachable))
            .unwrap_or(first)
    }
}
