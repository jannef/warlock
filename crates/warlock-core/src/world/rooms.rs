//! Special rooms in play (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): what the
//! player is told of them besides the first entry (`travel.rs::enter_room`): a room's `near`
//! line the first time a cell of it comes into view, and a whole level's `pervade` line on each
//! arrival; and the pace a room gives healing and mana as an actor's talents make it (Hallowed
//! rest). The rules' one question is [`World::effects_at`].

use super::{Actor, World, WorldState};
use crate::content::Content;

impl World {
    /// Phase 4, after perception: each special room of the current level of which the player
    /// sees a cell for the first time is glimpsed, and its `near` line said, unless they are
    /// inside it (its `enter` line says it).
    pub(super) fn glimpse_rooms(&mut self) {
        let here = self.state.player().pos;
        let perception = &self.perception;
        let content = &self.content;
        let level = self.state.current_mut();
        let mut lines = Vec::new();
        for room in level.rooms.iter_mut().filter(|r| !r.glimpsed) {
            if !room.cells().into_iter().any(|p| perception.is_visible(p)) {
                continue;
            }
            room.glimpsed = true;
            if room.entered || room.contains(here) {
                continue;
            }
            lines.extend(content.room_effect(room.effect).near.clone());
        }
        for line in lines {
            self.say(line);
        }
    }

    /// The percent `room` (the pace a special room gives healing or mana, 100 none) is for
    /// `actor`: where it quickens, Hallowed rest quickens it more (spec §8 "Talents").
    pub(super) fn room_pace(&self, actor: &Actor, room: u32) -> u32 {
        if room <= 100 || actor.talents.is_empty() {
            return room;
        }
        let more = actor.talent_mods(&self.content).room_pace_percent;
        let more = u32::try_from((100 + more).max(0)).unwrap_or(0);
        100 + (room - 100) * more / 100
    }
}

/// The line said as the player arrives on the current level when an effect covers it whole: its
/// `pervade` line, and with Sense of place what the effect is.
pub(super) fn pervade_line(state: &WorldState, content: &Content) -> Option<String> {
    let effect = content.room_effect(state.current().pervading?);
    let line = effect.pervade.clone()?;
    let player = state.player();
    let knows = !player.talents.is_empty() && player.talent_mods(content).sense_of_place;
    Some(if knows {
        format!("{line} The whole level is like {}.", effect.with_article())
    } else {
        line
    })
}
