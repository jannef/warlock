//! Shrines of a side in play (spec §7 "Special rooms", design §9.3; `docs/dev/special-rooms.md`):
//! a shrine treats everyone inside by their own side against its hidden one
//! ([`crate::content::Shrine::regard`]): one who agrees heals faster ([`World::shrine_pace`], read
//! by `World::healing_rate_at`), one opposed holds its status, Unwelcome ([`World::dwell_shrine`],
//! from `World::dwell`), a neutral one is ignored. The first entry says which
//! ([`enter_line`], from `travel.rs::enter_room`).

use super::{Actor, ActorId, LevelState, World};
use crate::action::Event;
use crate::alignment::Alignment;
use crate::content::{Content, Regard, Shrine, SpecialRoom};
use crate::geom::Pos;

impl LevelState {
    /// The shrine at `p`, if a shrine of a side is there, with its side.
    pub(crate) fn shrine_at<'c>(
        &self,
        content: &'c Content,
        p: Pos,
    ) -> Option<(&'c Shrine, Alignment)> {
        let room = self.room_at(p)?;
        let shrine = content.room_effect(room.effect).shrine.as_ref()?;
        Some((shrine, room.side?))
    }

    /// Whether the level has a shrine of a side.
    pub(crate) fn has_shrine(&self, content: &Content) -> bool {
        (self.rooms.iter()).any(|r| content.room_effect(r.effect).shrine.is_some())
    }
}

impl World {
    /// How the shrine `actor` stands in treats it, if it stands in one.
    fn shrine_regard(&self, actor: &Actor) -> Option<(&Shrine, Regard)> {
        let level = self.state.levels.get(&actor.level)?;
        let (shrine, side) = level.shrine_at(&self.content, actor.pos)?;
        Some((shrine, Shrine::regard(side, actor.nature(&self.content))))
    }

    /// Percent of the usual pace at which `actor` gets hit points back for the shrine it stands
    /// in: the shrine's `regen` for one of its side, else 100.
    pub(super) fn shrine_pace(&self, actor: &Actor) -> u32 {
        match self.shrine_regard(actor) {
            Some((shrine, Regard::Welcome)) => u32::from(shrine.regen),
            _ => 100,
        }
    }

    /// Actor `id` stays where it is: in a shrine of the other side it holds the shrine's
    /// status (Unwelcome) at one degree, renewed as a room's status is ([`World::dwell`]).
    pub(super) fn dwell_shrine(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        if let Some((shrine, Regard::Unwelcome)) = self.shrine_regard(actor) {
            let status = shrine.status;
            self.afflict(id, status, 1, None, events);
        }
    }
}

/// The line the player's first entry into `room` says: a shrine's by how it treats them, else
/// the effect's `enter`.
pub(super) fn enter_line(room: &SpecialRoom, player: &Actor, content: &Content) -> String {
    let effect = content.room_effect(room.effect);
    match (&effect.shrine, room.side) {
        (Some(shrine), Some(side)) => {
            let regard = Shrine::regard(side, player.nature(content));
            shrine.enter_line(regard, &effect.enter).to_string()
        }
        _ => effect.enter.clone(),
    }
}
