//! Tempting hoards (spec §7 "Special rooms", design §9.7): a tempting hoard's items lie unguarded
//! until the player lifts one. Then the room **seals**: every door of it shuts (an open one, unless
//! something stands in the doorway) and locks with a lock of the floor's level (no key fits it but
//! a skeleton key; it is picked or kicked as any other, `world/lockpick.rs`, `world/door.rs`), each
//! with a slam as loud as a kick, and "The doors slam shut!"; and `hoard.guardians` hostiles of
//! the level's danger + `hoard.danger` come at the room's edges, on cells the player does not see
//! where there are any. A cave chamber has no doors: only the guardians come. A room seals once
//! ([`SpecialRoom::sealed`], saved and hashed with the room).
//!
//! What seals it is the player's pick-up inside it (one line in `carry.rs::pick_up`,
//! [`World::lift_in_hoard`]): each stack lifted counts ([`SpecialRoom::lifted`]), and the room
//! seals once the count passes the player's grace (none; Old delver, one stack).
//!
//! The guardians are drawn from the gameplay stream: how many, then for each its type (the
//! branch's population and strays by the danger, [`danger::draw`], hostile to the player and
//! never unique, as a den's sleepers) and its cell. They are ordinary creatures from then on and
//! roll their gear as creatures that come do.

use super::{ActorId, World, travel};
use crate::content::danger::{self, Candidate};
use crate::content::{CreatureSpawn, SpecialRoom, Stance, TrapNoise, Treasure};
use crate::door::Lock;
use crate::geom::{Dir, Pos};
use crate::noise::{KICK_NOISE, SoundKind};

/// What the player is told when a hoard's doors slam shut.
pub const SLAM: &str = "The doors slam shut!";

impl World {
    /// The player has lifted `stacks` stacks at `at`: inside a tempting hoard not yet sealed, they
    /// count toward its sealing, and it seals once they pass the player's grace (module docs).
    pub(super) fn lift_in_hoard(&mut self, at: Pos, stacks: usize) {
        let grace = self.state.player().talent_mods(&self.content).hoard_grace;
        let content = self.content.clone();
        let level = self.state.current_mut();
        let Some(room) = level.rooms.iter_mut().find(|r| r.contains(at)) else {
            return;
        };
        let Some(hoard) = content.room_effect(room.effect).hoard else {
            return;
        };
        if room.sealed {
            return;
        }
        let lifted = u8::try_from(stacks).unwrap_or(u8::MAX);
        room.lifted = room.lifted.saturating_add(lifted);
        if i32::from(room.lifted) <= grace.max(0) {
            return;
        }
        room.sealed = true;
        let room = room.clone();
        self.seal(&room, hoard);
    }

    /// `room` seals: its doors shut and lock, then its guardians come.
    fn seal(&mut self, room: &SpecialRoom, hoard: Treasure) {
        let mut slammed = Vec::new();
        for p in room.cells() {
            let Some(&t) = self.state.current().terrain.get(p) else {
                continue;
            };
            let def = self.content.terrain(t);
            if def.is_open_door() {
                let free = self.state.actor_at(&self.state.current_level, p).is_none()
                    && self.state.current().objects_at(p).is_empty();
                let Some(closed) = def.closes_to.filter(|_| free) else {
                    continue;
                };
                self.set_terrain(p, closed);
            } else if !def.is_closed_door() {
                continue;
            }
            let level = self.state.current_mut();
            level.locks.entry(p).or_insert_with(|| Lock::new(None));
            slammed.push(p);
        }
        for &p in &slammed {
            let slam = TrapNoise {
                sound: SoundKind::Banging,
                loudness: KICK_NOISE,
            };
            self.sound_at(Some(slam), p);
        }
        if !slammed.is_empty() {
            self.say(SLAM);
        }
        self.guard(room, hoard);
    }

    /// The guardians of `room` come (module docs). Returns them, in the order they came.
    fn guard(&mut self, room: &SpecialRoom, hoard: Treasure) -> Vec<ActorId> {
        let id = self.state.current_level.clone();
        let Some((_, _, spawning)) = self.content.spawning(&id) else {
            return Vec::new();
        };
        let spawning = spawning.clone();
        let danger = self.danger(&id).saturating_add(hoard.danger);
        if danger == 0 {
            return Vec::new();
        }
        let player = self.content.creature_id("player");
        let content = self.content.clone();
        let fit = |list: Vec<Candidate>| -> Vec<Candidate> {
            (list.into_iter())
                .filter(|&(c, _, _)| !content.creature(c).unique)
                .filter(|&(c, _, _)| {
                    player.is_some_and(|p| content.stance(c, p) == Stance::Hostile)
                })
                .collect()
        };
        let once = &self.state.progress.uniques;
        let own = fit(content.candidates(&spawning.population, once));
        let generic = fit(content.candidates(&spawning.generic, once));
        let (cells, edge) = self.guard_cells(room);
        let mut unseen: Vec<Pos> = (edge.iter().copied())
            .filter(|&p| !self.perception.sees_actors_at(p))
            .collect();
        let mut seen: Vec<Pos> = (edge.iter().copied())
            .filter(|&p| self.perception.sees_actors_at(p))
            .collect();
        let mut inner: Vec<Pos> = (cells.into_iter()).filter(|p| !edge.contains(p)).collect();
        let [least, most] = hoard.guardians;
        let rng = &mut self.state.rng;
        let count = rng.range(i32::from(least), i32::from(most));
        let mut came = Vec::new();
        for _ in 0..count {
            let rng = &mut self.state.rng;
            let Some(creature) = danger::draw(rng, &own, &generic, danger, &spawning.odds) else {
                break;
            };
            let from = [&mut unseen, &mut seen, &mut inner]
                .into_iter()
                .find(|cells| !cells.is_empty());
            let Some(cells) = from else {
                break;
            };
            let pos = cells.swap_remove(rng.index(cells.len()));
            let spawn = CreatureSpawn {
                creature,
                faction: None,
                alignment: None,
                npc: None,
                pos,
                route: Vec::new(),
                carry: Vec::new(),
                equipment: Default::default(),
                rolls_gear: true,
                asleep: false,
            };
            came.push(travel::make_creature(
                &mut self.state,
                &self.content,
                &id,
                &spawn,
            ));
        }
        came
    }

    /// The cells of `room` a guardian may come on (walkable, free, no door or stairs), in cell
    /// order, and of them those on its edge: next to a cell that is outside it or not walkable.
    pub(super) fn guard_cells(&self, room: &SpecialRoom) -> (Vec<Pos>, Vec<Pos>) {
        let level = self.state.current();
        let content = &self.content;
        let walkable = |p: Pos| room.contains(p) && level.is_walkable(content, p);
        let cells: Vec<Pos> = (room.cells().into_iter())
            .filter(|&p| walkable(p))
            .filter(|&p| self.state.actor_at(&level.id, p).is_none())
            .filter(|&p| !level.is_door(content, p) && level.stairs_at(p).is_none())
            .collect();
        let edge = (cells.iter().copied())
            .filter(|&p| Dir::ALL.iter().any(|d| !walkable(p.step(*d))))
            .collect();
        (cells, edge)
    }
}
