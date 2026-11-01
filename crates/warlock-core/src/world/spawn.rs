//! Danger levels and creatures that keep coming (spec §7 "Danger levels"): every level has a
//! danger that can always be computed ([`World::danger`]), and on a level whose data gives one
//! creatures of about that danger keep coming while the player is there.
//!
//! Each such level keeps a [`Spawner`] in its state: when the next attempt is due and its own
//! random stream (`spawn/<level id>`, apart from `gameplay` and every generation stream, so a
//! fight rolls the same with or without a spawn between two blows). At the end of every turn
//! ([`World::spawn_tick`], from `run_turn`) the attempts whose time has come on the **current**
//! level are made, several if a rest passed hours; levels the player is not on stay frozen (the
//! next attempt is put off to `every` after the player's arrival, `travel.rs`). An attempt is
//! skipped while the level holds as many creatures hostile to the player as its cap; else, with
//! its chance, it draws a type of the level's population by the danger, now and then a stray
//! of the generic creature list or one from deeper ([`crate::content::danger::draw`]: a unique
//! type once a run), a cell out of the player's view
//! by the perception rules and at least [`AWAY`] cells from them (walkable, free, no stairs,
//! door, shop floor, sanctuary, or sealed room; up to [`TRIES`] draws, else nothing comes), and a short route
//! where one fits, else a post. The newcomer is an ordinary creature from then on.
//!
//! Danger is hidden in the game; dev dumps and the wiki state it.

use super::{ActorId, World, travel};
use crate::content::danger::{self, Spawning};
use crate::content::{CreatureSpawn, Stance};
use crate::geom::Pos;
use crate::perception::Perception;
use crate::rng::Rng;
use crate::time::{Duration, GameTime};
use serde::{Deserialize, Serialize};

/// The fewest cells (Chebyshev) between the player and a creature that comes.
pub const AWAY: i32 = 8;
/// How many cells an attempt tries before it gives up.
const TRIES: u32 = 32;

/// What comes to a level over time: when the next attempt is due and the level's own stream.
/// Kept in the level's state (saved, hashed) only on a level with a danger rule, so every other
/// level's state is as it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spawner {
    /// When the next attempt is due.
    pub(crate) at: GameTime,
    /// The level's spawn stream, `spawn/<level id>`.
    pub(crate) rng: Rng,
}

impl Spawner {
    /// The spawner of level `id` made at `now`: its first attempt `every` from now.
    pub(crate) fn new(seed: u64, id: &str, now: GameTime, every: Duration) -> Self {
        Self {
            at: now + every,
            rng: Rng::spawn(seed, id),
        }
    }
}

impl World {
    /// The danger of level `id` now (spec §7 "Danger levels"): its branch's or its own rule at
    /// its depth, with the player's level for a player-scaled area; 0 where nothing is given.
    /// Computed when asked, never stored.
    pub fn danger(&self, id: &str) -> u8 {
        (self.content.spawning(id))
            .map_or(0, |(rule, depth, _)| rule.at(depth, self.player_level()))
    }

    /// Dev: the danger of the current level, its creatures hostile to the player against its
    /// cap, and when the next attempt is due, for the dumps' headers; `None` on a level with no
    /// danger rule.
    pub fn spawn_status(&self) -> Option<(u8, u32, u32, GameTime)> {
        let level = self.state.current();
        let (_, _, spawning) = self.content.spawning(&level.id)?;
        let at = level.spawner.as_ref()?.at;
        let cap = spawning.cap_for(self.walkable_cells());
        Some((self.danger(&level.id), self.hostiles_here(), cap, at))
    }

    /// The walkable cells of the current level (what its population is counted by).
    fn walkable_cells(&self) -> u32 {
        let level = self.state.current();
        let content = &self.content;
        let n = (level.terrain.iter())
            .filter(|(_, t)| content.terrain(**t).walkable)
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    /// The creatures on the current level hostile to the player.
    fn hostiles_here(&self) -> u32 {
        let (player, level) = (self.state.player, &self.state.current_level);
        let here = (self.state.actors.values()).filter(|a| a.level == *level && a.id != player);
        let n = here
            .filter(|a| self.stance(player, a.id) == Stance::Hostile)
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    /// The end of a turn: every attempt due on the current level is made (spec §7 "Danger
    /// levels"). Quiet delver puts each next attempt off longer.
    pub(super) fn spawn_tick(&mut self) {
        let id = self.state.current_level.clone();
        let Some((_, _, spawning)) = self.content.spawning(&id) else {
            return;
        };
        let every = self.spawn_every(spawning);
        while self.state.death.is_none() {
            let now = self.state.time;
            let Some(spawner) = self.state.current_mut().spawner.as_mut() else {
                return;
            };
            if spawner.at > now {
                return;
            }
            spawner.at += every;
            self.attempt(false);
        }
    }

    /// The time between attempts on a level for the player as they are (Quiet delver: longer).
    fn spawn_every(&self, spawning: &Spawning) -> Duration {
        let slower = self
            .state
            .player()
            .talent_mods(&self.content)
            .spawn_every_percent;
        let percent = u64::try_from(100 + slower.max(0)).unwrap_or(100);
        let every = spawning.every.as_seconds().saturating_mul(percent) / 100;
        Duration::seconds(every.max(1))
    }

    /// When the player arrives on level `id`: its next attempt is `every` from now, as if no
    /// time had passed there (floors the player is not on stay frozen).
    pub(super) fn arrive_spawner(&mut self, id: &str) {
        let Some((_, _, spawning)) = self.content.spawning(id) else {
            return;
        };
        let at = self.state.time + self.spawn_every(spawning);
        if let Some(spawner) = (self.state.levels.get_mut(id)).and_then(|l| l.spawner.as_mut()) {
            spawner.at = at;
        }
    }

    /// One attempt on the current level; `forced` (the dev action) skips the cap and the chance.
    /// Returns the creature that came, if one did.
    pub(super) fn attempt(&mut self, forced: bool) -> Option<ActorId> {
        let id = self.state.current_level.clone();
        let (_, _, spawning) = self.content.spawning(&id)?;
        let spawning = spawning.clone();
        let danger = self.danger(&id);
        if danger == 0 {
            return None;
        }
        if !forced && self.hostiles_here() >= spawning.cap_for(self.walkable_cells()) {
            return None;
        }
        let mut spawner = self.state.current_mut().spawner.take()?;
        let came = self.draw_spawn(&mut spawner.rng, &spawning, danger, forced);
        self.state.current_mut().spawner = Some(spawner);
        let (creature, pos, route) = came?;
        let spawn = CreatureSpawn {
            creature,
            faction: None,
            alignment: None,
            npc: None,
            pos,
            route,
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: true,
            asleep: false,
        };
        let actor = travel::make_creature(&mut self.state, &self.content, &id, &spawn);
        self.spawned += 1;
        Some(actor)
    }

    /// The draws of one attempt from the level's stream: the chance (unless `forced`), the type,
    /// the cell, and the route.
    fn draw_spawn(
        &mut self,
        rng: &mut Rng,
        spawning: &Spawning,
        danger: u8,
        forced: bool,
    ) -> Option<(crate::content::CreatureId, Pos, Vec<crate::geom::Dir>)> {
        if !forced && !rng.chance(spawning.chance.into()) {
            return None;
        }
        let once = &self.state.progress.uniques;
        let own = self.content.candidates(&spawning.population, once);
        let generic = self.content.candidates(&spawning.generic, once);
        let creature = danger::draw(rng, &own, &generic, danger, &spawning.odds)?;
        let seen = Perception::compute(&self.state, &self.content, &mut self.lights);
        let level = self.state.current();
        let (content, player) = (&self.content, self.state.player().pos);
        let free = |p: Pos| {
            level.is_walkable(content, p)
                && self.state.actor_at(&level.id, p).is_none()
                && level.stairs_at(p).is_none()
                && !level.is_door(content, p)
                && !level.shops.iter().any(|s| s.contains(p))
                && !(level.room_at(p)).is_some_and(|r| {
                    // Nothing comes into a sealed room (a greater treasure room) either.
                    let effect = content.room_effect(r.effect);
                    effect.shunned || effect.sealed.is_some()
                })
        };
        let (w, h) = (level.width(), level.height());
        let mut cell = None;
        for _ in 0..TRIES {
            let p = Pos::new(rng.range(0, w - 1), rng.range(0, h - 1));
            if p.chebyshev(player) >= AWAY && !seen.sees_actors_at(p) && free(p) {
                cell = Some(p);
                break;
            }
        }
        let pos = cell?;
        let len = rng.range(4, 10);
        let route = crate::mapgen::out_and_back(rng, pos, len, |p| level.is_walkable(content, p))
            .unwrap_or_default();
        Some((creature, pos, route))
    }

    /// The dev action `spawn` ([`crate::action::Action::Spawn`]): one attempt on the current
    /// level now, whatever its cap and chance. Takes no time.
    pub(super) fn spawn_dev(&mut self) -> super::Spent {
        if self.danger(&self.state.current_level.clone()) == 0 {
            self.say("Nothing comes here: the level has no danger (dev).");
            return super::Spent::Nothing;
        }
        match self.attempt(true) {
            Some(actor) => {
                let name = self
                    .content
                    .creature(self.state.actors[&actor].creature)
                    .name
                    .clone();
                self.say(format!("A {name} comes (dev)."));
            }
            None => self.say("Nothing could come: no cell out of sight (dev)."),
        }
        super::Spent::NoTime
    }
}
