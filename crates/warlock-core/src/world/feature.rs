//! A special feature of the ground the player arrives at (an altar, a faint rune, a worn statue;
//! terrain with `announce`): the message line names it, once per arrival, however they came (a
//! step, stairs, a fall, a teleport, or the start of a run). Waiting, resting, or searching there
//! says nothing more. Walkable features are named when the player arrives on them; one that
//! cannot be stood on (a statue) when they arrive beside it, and not again while they move about
//! beside it.

use super::{World, WorldState};
use crate::Pos;
use crate::content::Content;
use crate::geom::Dir;

/// Where the player last stood when the world perceived (not saved: a loaded world stands where
/// it was, its arrival already told).
pub(super) type StoodOn = (String, Pos);

/// Where the player stands now.
pub(super) fn standing(state: &WorldState) -> StoodOn {
    (state.current_level.clone(), state.player().pos)
}

/// What the player is told of the feature at `p`, if the ground there is one: "There is an
/// altar here.", with its look-mode description after, if it has one, and the note of the place
/// there, if any (an inscription).
fn feature_at(state: &WorldState, content: &Content, p: Pos) -> Option<String> {
    let level = state.current();
    let &terrain = level.terrain.get(p)?;
    let def = content.terrain(terrain);
    if !def.announce {
        return None;
    }
    let name = &def.name;
    let a = crate::text::article(name);
    let mut line = format!("There is {a} {name} here.");
    let note = level
        .place(content, p)
        .and_then(|place| place.note.as_deref());
    for more in [def.description.as_deref(), note].into_iter().flatten() {
        line.push(' ');
        line.push_str(more);
    }
    Some(line)
}

/// What the player is told of the features at their arrival from `from` (the cell they last
/// stood on, on this level; `None` from elsewhere): the walkable one under them, then each one
/// beside them that cannot be stood on and was not beside `from`, in [`Dir::ALL`]'s order.
pub(super) fn feature_lines(
    state: &WorldState,
    content: &Content,
    from: Option<Pos>,
) -> Vec<String> {
    let here = state.player().pos;
    let level = state.current();
    let standable = |p: Pos| {
        level
            .terrain
            .get(p)
            .is_some_and(|&t| content.terrain(t).walkable)
    };
    let beside = (Dir::ALL.into_iter())
        .map(|d| here.step(d))
        .filter(|&p| !standable(p))
        .filter(|&p| from.is_none_or(|f| f.chebyshev(p) > 1));
    (std::iter::once(here).filter(|&p| standable(p)))
        .chain(beside)
        .filter_map(|p| feature_at(state, content, p))
        .collect()
}

impl World {
    /// If the player has come to another cell (or level) since it was last called, names the
    /// features they arrive at. A step calls it before saying what lies there (`travel.rs`);
    /// `perceive` calls it after every action for every other way of arriving.
    pub(super) fn announce_feature(&mut self) {
        let here = standing(&self.state);
        if self.stood_on == here {
            return;
        }
        let from = (self.stood_on.0 == here.0).then_some(self.stood_on.1);
        self.stood_on = here;
        for text in feature_lines(&self.state, &self.content, from) {
            self.say(text);
        }
    }
}
