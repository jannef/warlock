//! Haunted rooms (spec §7 "Special rooms", design §9.8): no place to sleep. While the player
//! rests inside a room whose effect haunts ([`crate::content::Here::haunt`]), each time the clock
//! passes a full multiple of the haunt's `every` (every full hour of the day) a creature of the
//! haunt's faction comes inside: of the floor's danger, drawn as a cursed room's dead rise
//! ([`super::curse::rises_as`], the nearest level when none has it), on a free cell of the room
//! the player does not see where there is one, else on any free cell of it. It comes for the
//! sleeper (its mind's `last_seen` is where the player lies) and counts as a newcomer
//! (`World::spawned`), so the rest looks up at once and stops as soon as it comes into view, as
//! for any arrival. Counting by the clock rather than from the rest's start keeps
//! it stateless and means short rests strung together are haunted alike.
//!
//! The hook is one line at the end of `schedule.rs::run_turn` ([`World::haunt_tick`]), with the
//! time the turn began. Only the player's rests (`World::resting`) are haunted, not digging,
//! eating, or prayer, and only on a level with a danger. Draws (the type, the cell) come from the
//! gameplay stream, and only when a haunting is due.

use super::{World, curse, travel};
use crate::content::CreatureSpawn;
use crate::geom::Pos;
use crate::time::GameTime;

impl World {
    /// The end of a turn that began at `began`: if the player rests inside a haunted room and the
    /// turn passed a full multiple of its `every`, something comes (module docs).
    pub(super) fn haunt_tick(&mut self, began: GameTime) {
        if !self.resting || self.state.death.is_some() {
            return;
        }
        let content = self.content.clone();
        let now = self.state.time;
        let player = self.player_pos();
        let level = self.state.current();
        let Some(room) = level.rooms.iter().find(|r| r.contains(player)).cloned() else {
            return;
        };
        let Some(haunt) = content.room_effect(room.effect).haunt.clone() else {
            return;
        };
        let every = haunt.every.as_seconds().max(1);
        if began.seconds() / every == now.seconds() / every {
            return;
        }
        let id = self.state.current_level.clone();
        let danger = self.danger(&id);
        if danger == 0 {
            return;
        }
        let Some(faction) = content.factions().id(&haunt.faction) else {
            return;
        };
        let (cells, _) = self.guard_cells(&room);
        let unseen: Vec<Pos> = (cells.iter().copied())
            .filter(|&p| !self.perception.sees_actors_at(p))
            .collect();
        let cells = if unseen.is_empty() { cells } else { unseen };
        if cells.is_empty() {
            return;
        }
        let campaign = self.state.campaign.clone();
        let rng = &mut self.state.rng;
        let Some(creature) = curse::rises_as(&content, rng, faction, &campaign, danger) else {
            return;
        };
        let pos = cells[self.state.rng.index(cells.len())];
        let spawn = CreatureSpawn {
            creature,
            faction: None,
            alignment: None,
            npc: None,
            pos,
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Default::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        let came = travel::make_creature(&mut self.state, &content, &id, &spawn);
        // It comes for the sleeper: it goes to where the player lies, as to one it lost sight of.
        if let Some(actor) = self.state.actors.get_mut(&came) {
            actor.mind.last_seen = Some(player);
        }
        // A newcomer: a rest looks up at once.
        self.spawned += 1;
    }
}
