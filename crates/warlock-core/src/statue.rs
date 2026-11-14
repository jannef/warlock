//! Statues of the stoned as level data (spec §8 "Statues"): a [`Statue`] is what a statue cell
//! holds, and the [`Fallen`] it may be named for. Plain data, so both a hand-made or generated
//! level's template (`content::LevelTemplate::statues`, laid by `mapgen/leavings.rs`) and the
//! running level (`world::LevelState::statues`) hold it; the rules that lay, kick, and name
//! statues are the world's (`world/creatures/statue.rs`, `world/fixtures/graveyard.rs`).

use crate::content::{Content, CreatureId};
use crate::object::Stack;
use crate::unhashed::unhashed_option;
use serde::{Deserialize, Serialize};

/// A statue on a cell (ground truth but for what it was, which the player sees).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statue {
    /// The creature type it was.
    pub of: CreatureId,
    /// What it held, top first: what the creature carried, then the pile it stood on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holds: Vec<Stack>,
    /// The past character it is of, a name from the graveyard (`world/fixtures/graveyard.rs`):
    /// decoration, read by no rule and left out of the state hash.
    #[serde(default, skip_serializing_if = "unhashed_option")]
    pub name: Option<Fallen>,
}

impl Statue {
    /// Whether its creature type is in `content` (a guard for loaded saves; what it holds is
    /// checked with the piles).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        usize::from(self.of.0) < content.creature_defs().len()
    }
}

/// One character of a past run who died, as a statue or a gravestone names them: "Brakka the
/// dwarf".
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallen {
    /// The character's name.
    pub name: String,
    /// Their race, by id (`dwarf`); one this content lacks is named by its id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub race: String,
    /// Whether they were turned to stone: only these name statues.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub petrified: bool,
    /// How and where they died, in the app's words, without a full stop: "Killed by a kobold
    /// in Fyrdelf 3, The Ascmearc"; empty when unknown.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub death: String,
}

impl Fallen {
    /// "Brakka the dwarf" (the race's name in lower case), or "Brakka" without a race.
    pub fn words(&self, content: &Content) -> String {
        let race = (content.race(&self.race)).map_or(self.race.clone(), |r| r.name.to_lowercase());
        if race.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {race}", self.name)
        }
    }
}
