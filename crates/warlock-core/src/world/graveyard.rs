//! The petrified dead of past runs, as statues' names (spec §8 "Statues"; #1137, design
//! `docs/design/monster-dread.md` §9.5).
//!
//! The app keeps the graveyard file (every character who has died, `crates/warlock/src/
//! graveyard.rs`); the core never reads it. When a run begins the app gives the run the
//! petrified among them, the most recent first ([`World::set_graveyard`]), and the run keeps
//! that list in its state (`WorldState::graveyard`), so a resumed run sees the list it began
//! with. Each level, as it is made ([`name_statues`], from `travel.rs::instantiate_level`),
//! names its statues from it: one name a statue, the most recent first, never twice on a
//! floor; which statue gets which is drawn from the level's own stream `statue-names/<level>`
//! (the level's seed), so nothing else drawn moves.
//!
//! **The names are decoration**: nothing of the rules reads one, and both the list and a
//! statue's name are left out of the state hash (`hash.rs::unhashed`), so a replay made on a
//! machine with another graveyard verifies. Saves keep them.

use super::{World, WorldState};
use crate::content::Content;
use crate::geom::Pos;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// One petrified character of a past run, as a statue names them: "Brakka the dwarf".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallen {
    /// The character's name.
    pub name: String,
    /// Their race, by id (`dwarf`); one this content lacks is named by its id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub race: String,
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

impl World {
    /// The run's petrified dead, the most recent first (module docs): the app's, read from the
    /// graveyard file as the run begins. The statues of the levels made already are named now,
    /// as each level made later will be. The names are out of the state hash.
    pub fn set_graveyard(&mut self, fallen: Vec<Fallen>) {
        self.state.graveyard = fallen;
        let ids: Vec<String> = self.state.levels.keys().cloned().collect();
        for id in ids {
            name_statues(&mut self.state, &self.content, &id);
        }
    }

    /// The run's petrified dead, as given by [`World::set_graveyard`].
    pub fn graveyard(&self) -> &[Fallen] {
        &self.state.graveyard
    }
}

/// Names the unnamed statues of level `id` from the run's graveyard (module docs).
pub(super) fn name_statues(state: &mut WorldState, content: &Content, id: &str) {
    if state.graveyard.is_empty() {
        return;
    }
    let seed = state.level_seed(content, id);
    let Some(level) = state.levels.get_mut(id) else {
        return;
    };
    let named: Vec<&Fallen> = (level.statues.values())
        .filter_map(|s| s.name.as_ref())
        .collect();
    let mut names: Vec<&Fallen> = Vec::new();
    for fallen in &state.graveyard {
        if !named.contains(&fallen) && !names.contains(&fallen) {
            names.push(fallen);
        }
    }
    let mut cells: Vec<Pos> = (level.statues.iter())
        .filter(|(_, s)| s.name.is_none())
        .map(|(&p, _)| p)
        .collect();
    if cells.is_empty() || names.is_empty() {
        return;
    }
    let mut rng = Rng::stream(seed, &format!("statue-names/{id}"));
    rng.shuffle(&mut cells);
    let names: Vec<Fallen> = names.into_iter().cloned().collect();
    for (p, fallen) in cells.into_iter().zip(names) {
        if let Some(statue) = level.statues.get_mut(&p) {
            statue.name = Some(fallen);
        }
    }
}
