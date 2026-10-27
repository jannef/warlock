//! A special feature of the ground the player arrives on (an altar, a faint rune; terrain with
//! `announce`): the message line names it, once per arrival, however they came (a step, stairs,
//! a fall, a teleport, or the start of a run). Waiting, resting, or searching there says nothing
//! more.

use super::{World, WorldState};
use crate::Pos;
use crate::content::Content;

/// Where the player last stood when the world perceived (not saved: a loaded world stands where
/// it was, its arrival already told).
pub(super) type StoodOn = (String, Pos);

/// Where the player stands now.
pub(super) fn standing(state: &WorldState) -> StoodOn {
    (state.current_level.clone(), state.player().pos)
}

/// What the player is told of the feature under them, if the ground there is one: "There is
/// an altar here.", with its look-mode description after, if it has one.
pub(super) fn feature_line(state: &WorldState, content: &Content) -> Option<String> {
    let &terrain = state.current().terrain.get(state.player().pos)?;
    let def = content.terrain(terrain);
    if !def.announce {
        return None;
    }
    let name = &def.name;
    let a = crate::content::article(name);
    Some(match &def.description {
        Some(description) => format!("There is {a} {name} here. {description}"),
        None => format!("There is {a} {name} here."),
    })
}

impl World {
    /// If the player has come to another cell (or level) since it was last called, names the
    /// feature they stand on. A step calls it before saying what lies there (`travel.rs`);
    /// `perceive` calls it after every action for every other way of arriving.
    pub(super) fn announce_feature(&mut self) {
        let here = standing(&self.state);
        if self.stood_on == here {
            return;
        }
        self.stood_on = here;
        if let Some(text) = feature_line(&self.state, &self.content) {
            self.say(text);
        }
    }
}
