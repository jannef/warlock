//! What is cut in gravestones (spec §7 "Graves"; #1258): a hand-made level's `[graves]` words
//! (`content::Graves`) give every cell of a `grave` terrain one inscription as the level is made
//! ([`carve_graves`], from `travel.rs::instantiate_level`, and again for the levels made already
//! when the run is given its graveyard, `World::set_graveyard`).
//!
//! Each stone, in row order, draws from the level's own stream `graves/<level>` (the level's
//! seed): a roll of one in the `[graves]` `fallen_one_in` of `data/rules/tuning.toml`, then a
//! stranger of the level's folk (a given name, a byname, an epitaph, how many years ago they died
//! and how old they were, before the year the level is made). Every stone draws all of that
//! whatever the graveyard holds, so a stranger's stone reads the same with any graveyard. Where the
//! roll comes up and one of the run's graveyard (`world/fixtures/graveyard.rs`) is left, the stone is theirs
//! instead: which one is drawn from the stream `graves/<level>/fallen`, never twice on a level.
//!
//! **Decoration**: no rule reads an inscription, and they are left out of the state hash
//! (`LevelState::graves`), so a replay verifies under any graveyard. Saves keep them.

use crate::content::{Content, Graves};
use crate::geom::Pos;
use crate::rng::Rng;
use crate::world::{Fallen, WorldState};
use std::collections::BTreeMap;

/// What a death without words reads as (an entry the app gave none).
const UNTOLD: &str = "taken before their time";

/// Cuts the inscriptions of level `id`'s gravestones (module docs); a level without `graves`
/// words keeps its stones blank.
pub(in crate::world) fn carve_graves(state: &mut WorldState, content: &Content, id: &str) {
    let Some(graves) = content.level(id).and_then(|t| t.graves.as_ref()) else {
        return;
    };
    let seed = state.level_seed(content, id);
    let year = state.time.year();
    let Some(level) = state.levels.get(id) else {
        return;
    };
    let terrain = &level.terrain;
    let mut cells = Vec::new();
    for y in 0..terrain.height() {
        for x in 0..terrain.width() {
            let p = Pos::new(x, y);
            if content.terrain(terrain[p]).grave {
                cells.push(p);
            }
        }
    }
    let carved = carve(content, graves, &cells, (seed, id), year, &state.graveyard);
    if let Some(level) = state.levels.get_mut(id) {
        level.graves = carved;
    }
}

/// The inscriptions of the stones on `cells` (in the order given) of level `id` in a run with
/// `seed`, made in `year`, with the run's `graveyard` (module docs).
pub(crate) fn carve(
    content: &Content,
    graves: &Graves,
    cells: &[Pos],
    (seed, id): (u64, &str),
    year: u64,
    graveyard: &[Fallen],
) -> BTreeMap<Pos, String> {
    let mut rng = Rng::stream(seed, &format!("graves/{id}"));
    let mut pick = Rng::stream(seed, &format!("graves/{id}/fallen"));
    let mut left: Vec<&Fallen> = Vec::new();
    for fallen in graveyard {
        if !left.contains(&fallen) {
            left.push(fallen);
        }
    }
    let mut carved = BTreeMap::new();
    for &p in cells {
        let theirs = rng.below(content.tuning().graves.fallen_one_in) == 0;
        let stranger = stranger(graves, &mut rng, year);
        let text = if theirs && !left.is_empty() {
            let fallen = left.remove(pick.index(left.len()));
            fallen_words(content, graves, fallen)
        } else {
            stranger
        };
        carved.insert(p, text);
    }
    carved
}

/// A stranger of the level's folk: the same draws every time (module docs).
fn stranger(graves: &Graves, rng: &mut Rng, year: u64) -> String {
    let given = rng.pick(&graves.names).map_or("", String::as_str);
    let byname = rng.pick(&graves.bynames);
    let epitaph = rng.pick(&graves.epitaphs).map_or("", String::as_str);
    let ago = rng.range(graves.died[0].into(), graves.died[1].into());
    let age = rng.range(graves.age[0].into(), graves.age[1].into());
    let name = match byname {
        Some(byname) => format!("{given} {byname}"),
        None => given.to_string(),
    };
    let year = i64::try_from(year).unwrap_or(i64::MAX);
    let died = (year - i64::from(ago)).max(1);
    let born = (died - i64::from(age)).max(1);
    epitaph
        .replace("{name}", &name)
        .replace("{born}", &born.to_string())
        .replace("{died}", &died.to_string())
}

/// One of the run's graveyard: "Here lies Brakka the dwarf, killed by a kobold in Fyrdelf 3."
fn fallen_words(content: &Content, graves: &Graves, fallen: &Fallen) -> String {
    let mut chars = fallen.death.chars();
    let death = match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => UNTOLD.to_string(),
    };
    (graves.fallen)
        .replace("{name}", &fallen.words(content))
        .replace("{death}", &death)
}
