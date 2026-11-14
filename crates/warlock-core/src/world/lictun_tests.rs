//! Līctūn, Cēaptūn's churchyard by the south road (#1258; spec §7 "Graves"): the way in from the
//! town and out by the edge, nobody but the sexton and nothing coming, Old Dudda's words, the
//! fifty stones and their inscriptions (one in ten of the graveyard's fallen, never twice; a
//! stranger's stone the same under any graveyard), out of the state hash and kept by a save, and
//! a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Stance};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::test_support::run as act;
use crate::test_support::texts as said;
use std::sync::LazyLock;

/// Cēaptūn's cell, where the south road starts.
const TOWN_CELL: Pos = Pos::new(18, 28);
/// Līctūn's cell on the overworld, beside the south road.
const YARD_CELL: Pos = Pos::new(19, 32);
/// Where the player arrives in the yard: the road outside the gate.
const START: Pos = Pos::new(2, 12);
/// The walk from the pass to the yard's cell.
const WALK: &str = "move:E*6 move:S*4 move:E";

/// Built-in content with nobody hungering, and nobody met on the way.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.remove_encounter_tables();
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// A run of the Ascmearc in the yard, given `graveyard` as the run begins.
fn yard(graveyard: Vec<Fallen>) -> World {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    w.set_graveyard(graveyard);
    act(&mut w, WALK);
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "lictun");
    w
}

fn fallen(name: &str, race: &str, death: &str) -> Fallen {
    Fallen {
        name: name.into(),
        race: race.into(),
        petrified: false,
        death: death.into(),
    }
}

/// Twenty of the dead, more than the stones that can name them.
fn many() -> Vec<Fallen> {
    (0..20)
        .map(|i| {
            fallen(
                &format!("Hero{i}"),
                "dwarf",
                "Killed by a kobold in Fyrdelf 3",
            )
        })
        .collect()
}

fn graves(w: &World) -> BTreeMap<Pos, String> {
    w.state.levels["lictun"].graves.clone()
}

#[test]
fn the_yard_is_beside_the_south_road_and_left_by_its_edge() {
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    let at = |p: Pos| CONTENT.terrain(map.terrain[p]).id.as_str();
    for y in TOWN_CELL.y + 1..=YARD_CELL.y {
        assert_eq!(at(Pos::new(18, y)), "road", "(18, {y})");
    }
    assert_eq!(at(YARD_CELL), "graveyard");
    assert_eq!(
        YARD_CELL.chebyshev(TOWN_CELL),
        4,
        "3 to 5 cells south of the town"
    );
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, WALK);
    assert_eq!(w.player_pos(), YARD_CELL);
    assert_eq!(said(&w).last().unwrap(), "The way into Līctūn is here.");
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "lictun");
    assert_eq!(w.player_pos(), START, "on the road outside the gate");
    assert!(
        said(&w)
            .iter()
            .any(|l| l.starts_with("Līctūn, Cēaptūn's churchyard"))
    );
    let level = CONTENT.level("lictun").expect("the yard's level");
    assert!(level.stairs.is_empty(), "{:?}", level.stairs);
    let exit = level.exit.as_ref().expect("an edge exit");
    assert_eq!((exit.to.as_str(), exit.pos), ("ascmearc", YARD_CELL));
    act(&mut w, "move:W*3");
    assert_eq!(w.state.current_level, "ascmearc");
    assert_eq!(w.player_pos(), YARD_CELL);
}

#[test]
fn nobody_but_the_sexton_and_nothing_comes() {
    let mut w = yard(Vec::new());
    let player = w.state.player;
    let others: Vec<ActorId> = (w.state.actors.values())
        .filter(|a| a.level == "lictun" && a.id != player)
        .map(|a| a.id)
        .collect();
    assert_eq!(others.len(), 1, "the sexton alone");
    let dudda = others[0];
    assert_eq!(w.state.actors[&dudda].npc.as_deref(), Some("dudda"));
    assert_ne!(w.stance(dudda, player), Stance::Hostile);
    assert_eq!(w.danger("lictun"), 0, "nothing comes to the yard");
    assert!(
        w.topics(dudda).is_empty(),
        "no topics, no quest, no service"
    );
    // He talks: one of his lines.
    let at = w.state.actors[&dudda].pos;
    let id = w.state.player;
    let beside = (Dir::ALL.into_iter())
        .map(|d| at.step(d))
        .find(|&p| {
            let level = &w.state.levels["lictun"];
            CONTENT.terrain(level.terrain[p]).walkable && w.state.actor_at("lictun", p).is_none()
        })
        .expect("room beside him");
    w.state.actors.get_mut(&id).expect("player").pos = beside;
    w.perceive(&mut Vec::new());
    let to = (Dir::ALL.into_iter())
        .find(|d| beside.step(*d) == at)
        .expect("a way to him");
    let before = said(&w).len();
    act(&mut w, &format!("chat:{to}"));
    assert!(
        said(&w)[before].starts_with("Old Dudda says: \""),
        "{:?}",
        said(&w)
    );
}

#[test]
fn fifty_stones_each_cut_for_a_stranger_without_a_graveyard() {
    let w = yard(Vec::new());
    let cut = graves(&w);
    assert_eq!(cut.len(), 50);
    for (p, text) in &cut {
        let terrain = CONTENT.terrain(w.state.levels["lictun"].terrain[*p]);
        assert!(terrain.grave, "{p}");
        // A stranger's: their years, before the year the yard was made.
        assert!(text.contains('–'), "{p}: {text}");
        assert!(!text.contains('{'), "{p}: {text}");
    }
}

#[test]
fn one_stone_in_ten_names_one_of_the_fallen_never_twice() {
    let bare = graves(&yard(Vec::new()));
    let w = yard(many());
    let cut = graves(&w);
    let theirs: Vec<(&Pos, &String)> = (cut.iter())
        .filter(|(p, text)| bare[*p] != **text)
        .collect();
    // About one in ten of the fifty.
    assert!((2..=10).contains(&theirs.len()), "{theirs:?}");
    let mut named = Vec::new();
    for (p, text) in &theirs {
        let name = (text.strip_prefix("Here lies "))
            .and_then(|t| {
                t.split(" the dwarf, killed by a kobold in Fyrdelf 3.")
                    .next()
            })
            .unwrap_or_else(|| panic!("{p}: {text}"));
        assert!(
            text.ends_with(" the dwarf, killed by a kobold in Fyrdelf 3."),
            "{text}"
        );
        assert!(!named.contains(&name), "{name} twice");
        named.push(name);
    }
    // The rest read as they do with no graveyard; the names are out of the state hash.
    assert_eq!(w.state_hash(), yard(Vec::new()).state_hash());
    // A graveyard of one: at most one stone of theirs, and the same stone the roll chose.
    let one = graves(&yard(vec![fallen("Brakka", "dwarf", "Turned to stone")]));
    let differ: Vec<&Pos> = (one.iter())
        .filter(|(p, text)| bare[*p] != **text)
        .map(|(p, _)| p)
        .collect();
    assert_eq!(differ, [theirs[0].0]);
    assert_eq!(
        one[theirs[0].0],
        "Here lies Brakka the dwarf, turned to stone."
    );
}

#[test]
fn the_graveyard_given_late_cuts_the_same_and_a_save_keeps_it() {
    // Given after the yard was made (a replay's dump): cut again, the same as made with it.
    let mut late = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut late, WALK);
    act(&mut late, "descend");
    late.set_graveyard(many());
    assert_eq!(graves(&late), graves(&yard(many())));
    let loaded = crate::save::decode(&crate::save::encode(&late), CONTENT.clone()).expect("loads");
    assert_eq!(graves(&loaded), graves(&late));
    // The player reads one.
    let p = Pos::new(7, 19);
    assert_eq!(
        late.perception().epitaph(p),
        Some(graves(&late)[&p].as_str())
    );
}

#[test]
fn the_yard_on_arrival() {
    let w = yard(Vec::new());
    let rows = dump::layer_rows(&w, Layer::View).join("\n");
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!("lictun_arrival", rows);
    });
}

#[test]
fn a_run_to_the_yard_replays_exactly_under_any_graveyard() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    w.set_graveyard(many());
    let mut replay = Replay::new(&w);
    let actions = format!("{WALK} descend move:E*3 wait*5");
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    // Verified with no graveyard at all.
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
