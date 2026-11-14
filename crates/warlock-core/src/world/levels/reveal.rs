//! Hidden locations (spec §7 "Hidden locations"): a location on the overworld (or a hand-made
//! level: Gimmwīc's crystal portal) that is, until a quest reveals it, the plain terrain its cell
//! shows, in every way (as a secret door is the wall it looks like). Revealing it turns the cell into its entrance, links the entrance to the
//! level it leads to, and marks it on the player's map (their memory of the overworld), wherever
//! they are. Climbing out through one reveals it too ([`World::reveal_way_out`], called by
//! `world/levels/travel.rs::take_stairs`). `world/people/quest.rs` reveals at a quest's stage ([`crate::content::RevealWhen`]); the
//! dev action `reveal:ID` ([`crate::action::Action::Reveal`]) at once. `world/mod.rs` has only
//! the dispatch line.

use crate::content::{StairsDir, StairsLink};
use crate::geom::Pos;
use crate::world::{Spent, World, WorldState};

impl World {
    /// Reveals hidden location `id` of the run's campaign (on its overworld or a hand-made
    /// level): if its level has been made, its cell becomes the entrance, leading on, and is
    /// marked on the player's map; if not, it will be as it is made ([`apply_revealed`]).
    /// Returns whether anything changed: `false` for a location revealed before, or one the
    /// campaign does not have.
    pub(in crate::world) fn reveal_location(&mut self, id: &str) -> bool {
        let Some(level) = (self.content.hidden_locations(&self.state.campaign))
            .find(|(_, _, h)| h.id == id)
            .map(|(level, _, _)| level.to_string())
        else {
            return false;
        };
        if !self.state.revealed.insert(id.into()) {
            return false;
        }
        if let Some(at) = reveal_on(&mut self.state, &level, id)
            && self.state.current_level == level
        {
            let level = self.state.current();
            self.lights.terrain_changed(level, &self.content, at);
        }
        true
    }

    /// Climbing out through a hidden way reveals it (spec §7 "Ways up"): before the stairs from
    /// level `from` take the player to `to`, a hidden location of the overworld `to` that leads
    /// to `from` and is still hidden is revealed, so they arrive on it. Returns its name.
    pub(super) fn reveal_way_out(&mut self, from: &str, to: &str) -> Option<String> {
        let overworld = self.content.overworld_of(&self.state.campaign)?;
        if overworld.id != to {
            return None;
        }
        let location = (overworld.hidden.values())
            .find(|h| h.to.as_deref() == Some(from) && !self.state.revealed.contains(&h.id))?;
        let (id, name) = (location.id.clone(), location.name.clone());
        self.reveal_location(&id).then_some(name)
    }

    /// The dev action `reveal:ID` ([`crate::action::Action::Reveal`]): reveals hidden location
    /// `id` as a quest would, marking it on the map. Takes no time.
    pub(in crate::world) fn reveal_dev(&mut self, id: &str) -> Spent {
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

/// Level `level_id` as it is made: the locations revealed before it was are revealed on
/// it now (called by `instantiate_level`; a layer boss's lair keeps its hidden stairs under the
/// boss's id, never revealed this way: `world/levels/layer_boss.rs`).
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
