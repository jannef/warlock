//! The run's dead laid to rest (spec §7 "Graves"; #1682; `docs/dev/graves.md`): a burial
//! ground's places (`content/burial.rs`) are filled from the run's graveyard
//! (`world/graveyard.rs`) as the level is made ([`bury`], from `travel.rs::instantiate_level`,
//! and again for the levels made already when the run is given its graveyard,
//! `World::set_graveyard`).
//!
//! Every one of the dead but the petrified (statues, `graveyard.rs`) has a grave of their own,
//! the oldest death first: the churchyard's places in row order, then the first crypt's, the
//! second's, and so on down the chain. A place filled becomes its grave terrain, cut with the
//! level's `epitaph`; a level whose places are all filled opens its way down to the next. The
//! fixed graves (the legend's `epitaph`) are cut whatever the graveyard holds. No draw: the
//! same graveyard always lays the same dead in the same places.
//!
//! **Decoration**: a place and its grave are the same in every rule (checked at load), no rule
//! reads an inscription, and the state hash sees each level as it would be with no graveyard
//! ([`LevelState::unburied`]): the places empty, the way down closed. So a replay verifies under
//! any graveyard, unless it steps on or takes an opened way down. Saves keep them.

use super::{Fallen, LevelState, WorldState};
use crate::content::{Burial, Content};
use std::borrow::Cow;

/// What a death without words reads as (an entry the app gave none).
const UNTOLD: &str = "taken before their time";

/// Lays the run's dead in level `id`'s burial ground, as the module docs say; a level without
/// one is left as it is.
pub(super) fn bury(state: &mut WorldState, content: &Content, id: &str) {
    let Some((above, burial)) = content.burial_of(id) else {
        return;
    };
    let Some(level) = state.levels.get_mut(id) else {
        return;
    };
    unbury(level);
    level.graves = burial.epitaphs.clone();
    let dead: Vec<&Fallen> = (state.graveyard.iter().rev())
        .filter(|f| !f.petrified)
        .collect();
    let theirs = dead.iter().skip(above).take(burial.places.len());
    for (&p, fallen) in burial.places.iter().zip(theirs) {
        let place = level.terrain[p];
        let Some(grave) = content.terrain(place).burial else {
            continue;
        };
        change(level, p, grave);
        level.graves.insert(p, words(content, burial, fallen));
    }
    if dead.len() >= above + burial.places.len()
        && let Some(link) = burial.opened()
    {
        let stairs = burial.stairs.as_ref().expect("opened").terrain;
        change(level, link.pos, stairs);
        level.stairs.push(link);
    }
}

/// Sets cell `p` of `level` to `terrain` as burial does, keeping what it was.
fn change(level: &mut LevelState, p: crate::geom::Pos, terrain: crate::content::TerrainId) {
    let was = level.terrain[p];
    level.buried.entry(p).or_insert(was);
    level.terrain.set(p, terrain);
    if level.memory[p].is_some() {
        level.memory.set(p, Some(terrain));
    }
}

/// Undoes what [`bury`] did to `level`: the places empty, the way down closed (its link gone),
/// and the player's memory of them so. The inscriptions stay for [`bury`] to cut again.
fn unbury(level: &mut LevelState) {
    let buried = std::mem::take(&mut level.buried);
    for (&p, &was) in &buried {
        level.terrain.set(p, was);
        if level.memory[p].is_some() {
            level.memory.set(p, Some(was));
        }
    }
    level.stairs.retain(|s| !buried.contains_key(&s.pos));
}

impl LevelState {
    /// The level as the state hash sees it: as it would be with no graveyard (module docs).
    pub(crate) fn unburied(&self) -> Cow<'_, Self> {
        if self.buried.is_empty() {
            return Cow::Borrowed(self);
        }
        let mut level = self.clone();
        unbury(&mut level);
        Cow::Owned(level)
    }
}

/// One of the run's dead in `burial`'s words: "Here lies Brakka the dwarf warrior, killed by a
/// kobold in Fyrdelf 3, The Ascmearc."
fn words(content: &Content, burial: &Burial, fallen: &Fallen) -> String {
    let mut chars = fallen.death.chars();
    let death = match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => UNTOLD.to_string(),
    };
    (burial.epitaph)
        .replace("{name}", &fallen.grave_words(content))
        .replace("{death}", &death)
}
