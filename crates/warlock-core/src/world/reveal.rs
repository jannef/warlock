//! Hidden locations (spec §7 "Hidden locations"): a location on the overworld that is, until a
//! quest reveals it, the plain terrain its cell shows, in every way (as a secret door is the wall
//! it looks like). Revealing it turns the cell into its entrance, links the entrance to the
//! level it leads to, and marks it on the player's map (their memory of the overworld), wherever
//! they are. `world/quest.rs` reveals at a quest's stage ([`crate::content::RevealWhen`]); the
//! dev action `reveal:ID` ([`crate::action::Action::Reveal`]) at once. `world/mod.rs` has only
//! the dispatch line.

use super::{Spent, World, WorldState};
use crate::content::{StairsDir, StairsLink};
use crate::geom::Pos;

impl World {
    /// Reveals hidden location `id` of the overworld: if the overworld has been made, its cell
    /// becomes the entrance, leading on, and is marked on the player's map; if not, it will be
    /// as it is made ([`apply_revealed`]). Returns whether anything changed: `false` for a
    /// location revealed before, or one the overworld does not have.
    pub(super) fn reveal_location(&mut self, id: &str) -> bool {
        let Some(overworld) = self
            .content
            .overworld_of(&self.state.campaign)
            .map(|o| o.id.clone())
        else {
            return false;
        };
        if self
            .content
            .hidden_location(&self.state.campaign, id)
            .is_none()
            || !self.state.revealed.insert(id.into())
        {
            return false;
        }
        if let Some(at) = reveal_on(&mut self.state, &overworld, id)
            && self.state.current_level == overworld
        {
            let level = self.state.current();
            self.lights.terrain_changed(level, &self.content, at);
        }
        true
    }

    /// The dev action `reveal:ID` ([`crate::action::Action::Reveal`]): reveals hidden location
    /// `id` as a quest would, marking it on the map. Takes no time.
    pub(super) fn reveal_dev(&mut self, id: &str) -> Spent {
        let Some((_, location)) = self.content.hidden_location(&self.state.campaign, id) else {
            self.say(format!("There is no hidden location `{id}`."));
            return Spent::Nothing;
        };
        let name = location.name.clone();
        if !self.reveal_location(id) {
            self.say(format!("{name} is no secret any more."));
            return Spent::Nothing;
        }
        self.say(format!("{name} is marked on your map (dev)."));
        Spent::NoTime
    }
}

/// The overworld `level_id` as it is made: the locations revealed before it was are revealed on
/// it now (called by `instantiate_level`; no other level has hidden locations).
pub(super) fn apply_revealed(state: &mut WorldState, level_id: &str) {
    let revealed: Vec<String> = state.revealed.iter().cloned().collect();
    for id in revealed {
        reveal_on(state, level_id, &id);
    }
}

/// Reveals hidden location `id` on level `level_id`, if it lies there unrevealed: the cell
/// becomes its entrance, linked to where it leads under the location's name, and the player
/// remembers it. Returns the cell.
fn reveal_on(state: &mut WorldState, level_id: &str, id: &str) -> Option<Pos> {
    let level = state.levels.get_mut(level_id)?;
    let at = (level.hidden.iter()).find_map(|(p, h)| (h.id == id).then_some(*p))?;
    let location = level.hidden.remove(&at)?;
    level.terrain.set(at, location.terrain);
    level.memory.set(at, Some(location.terrain));
    level.stairs.push(StairsLink {
        pos: at,
        dir: StairsDir::Down,
        to: location.to,
        name: Some(location.name),
    });
    Some(at)
}
