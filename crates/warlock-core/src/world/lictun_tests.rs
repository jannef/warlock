//! Līctūn, Cēaptūn's churchyard by the south road, and its crypts (#1258, #1682; spec §7
//! "Graves"): the way in from the town and out by the edge, nobody but the sexton and nothing
//! coming, Old Dudda's words, the ten old graves and the forty plots, the run's dead laid oldest
//! first (never the petrified), the way down opening once a level is full, the crypts filled in
//! turn, all out of the state hash and kept by a save, and a replay verified without the
//! graveyard.

use super::*;
use crate::action::parse_actions;
use crate::content::{Burial, ContentFiles, Stance};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::test_support::{run as act, texts};
use std::sync::LazyLock;

/// Cēaptūn's cell, where the south road starts.
const TOWN_CELL: Pos = Pos::new(18, 28);
/// Līctūn's cell on the overworld, beside the south road.
const YARD_CELL: Pos = Pos::new(19, 32);
/// Where the player arrives in the yard: the road outside the gate.
const START: Pos = Pos::new(2, 13);
/// The walk from the pass to the yard's cell.
const WALK: &str = "move:E*6 move:S*4 move:E";
/// The chain's levels, the yard first.
const CHAIN: [&str; 12] = [
    "lictun",
    "lictun_crypt_01",
    "lictun_crypt_02",
    "lictun_crypt_03",
    "lictun_crypt_04",
    "lictun_crypt_05",
    "lictun_crypt_06",
    "lictun_crypt_07",
    "lictun_crypt_08",
    "lictun_crypt_09",
    "lictun_crypt_10",
    "lictun_crypt_11",
];

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
        class: "warrior".into(),
        petrified: false,
        death: death.into(),
    }
}

/// `n` of the dead, the most recent first: `Hero{n-1}` died first.
fn many(n: usize) -> Vec<Fallen> {
    (0..n)
        .rev()
        .map(|i| {
            fallen(
                &format!("Hero{i}"),
                "dwarf",
                "Killed by a kobold in Fyrdelf 3",
            )
        })
        .collect()
}

fn burial(level: &str) -> &'static Burial {
    CONTENT
        .level(level)
        .and_then(|l| l.burial.as_ref())
        .expect("a burial ground")
}

fn graves(w: &World, level: &str) -> BTreeMap<Pos, String> {
    w.state.levels[level].graves.clone()
}

/// Whether level `level`'s way down is open in `w`.
fn open(w: &World, level: &str) -> bool {
    let at = burial(level).stairs.as_ref().expect("a way down").pos;
    let state = &w.state.levels[level];
    state.stairs.iter().any(|s| s.pos == at) && CONTENT.terrain(state.terrain[at]).stairs.is_some()
}

/// Takes level `from`'s open way down.
fn go_down(w: &mut World, from: &str) {
    let at = burial(from).stairs.as_ref().expect("a way down").pos;
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = at;
    act(w, "descend");
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
    assert_eq!(texts(&w).last().unwrap(), "The way into Līctūn is here.");
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "lictun");
    assert_eq!(w.player_pos(), START, "on the road outside the gate");
    assert!(
        texts(&w)
            .iter()
            .any(|l| l.starts_with("Līctūn, Cēaptūn's churchyard"))
    );
    let level = CONTENT.level("lictun").expect("the yard's level");
    assert!(level.stairs.is_empty(), "{:?}", level.stairs);
    assert!(w.state.current().stairs.is_empty(), "no way down yet");
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
    let before = texts(&w).len();
    act(&mut w, &format!("chat:{to}"));
    assert!(
        texts(&w)[before].starts_with("Old Dudda says: \""),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn the_chain_is_the_yard_then_its_crypts_each_quiet_with_room_for_statues() {
    assert_eq!(CONTENT.burial_chain("lictun"), CHAIN);
    for (depth, id) in CHAIN.iter().enumerate() {
        let b = burial(id);
        assert_eq!((b.chain.as_str(), usize::from(b.depth)), ("lictun", depth));
        let level = CONTENT.level(id).expect("the level");
        assert!(level.spawning.is_none(), "{id}: no danger");
        assert!(level.creatures.len() <= 1, "{id}: nobody but the sexton");
        let stairs = b.stairs.as_ref().expect("a way down");
        assert_eq!(stairs.to.as_deref(), CHAIN.get(depth + 1).copied(), "{id}");
        if depth > 0 {
            assert!(
                (10..=40).contains(&b.places.len()),
                "{id}: {}",
                b.places.len()
            );
            assert!(b.statue_places.len() >= 4, "{id}: room for statues");
            let up = (level.stairs.iter()).find(|s| s.to.as_deref() == Some(CHAIN[depth - 1]));
            assert!(up.is_some(), "{id}: stairs up to the level above");
        }
    }
    let yard = burial("lictun");
    assert_eq!((yard.places.len(), yard.epitaphs.len()), (40, 10));
}

#[test]
fn ten_old_graves_and_forty_empty_plots_without_a_graveyard() {
    let w = yard(Vec::new());
    let cut = graves(&w, "lictun");
    assert_eq!(cut, burial("lictun").epitaphs, "the town's own, as written");
    assert!(
        cut.values()
            .any(|t| t.starts_with("Here lies Ælfric atte Welle"))
    );
    let level = &w.state.levels["lictun"];
    for p in &burial("lictun").places {
        assert_eq!(CONTENT.terrain(level.terrain[*p]).id, "grave_plot", "{p}");
    }
    assert!(!open(&w, "lictun"));
}

#[test]
fn the_dead_fill_the_plots_oldest_first_and_the_petrified_are_statues() {
    let mut dead = vec![
        fallen(
            "Osk",
            "halfling",
            "Killed by a goblin in Cealdhlāw 2, The Ascmearc",
        ),
        Fallen {
            petrified: true,
            death: "Turned to stone".into(),
            ..fallen("Brakka", "dwarf", "")
        },
        fallen("Tamsin", "elf", ""),
    ];
    dead[2].class = String::new();
    let w = yard(dead);
    let places = &burial("lictun").places;
    let cut = graves(&w, "lictun");
    assert_eq!(
        cut[&places[0]],
        "Here lies Tamsin the elf, taken before their time."
    );
    assert_eq!(
        cut[&places[1]],
        "Here lies Osk the halfling warrior, killed by a goblin in Cealdhlāw 2, The Ascmearc."
    );
    assert!(!cut.contains_key(&places[2]), "Brakka is a statue's");
    assert_eq!(cut.len(), 12);
    let level = &w.state.levels["lictun"];
    let at = |p: Pos| CONTENT.terrain(level.terrain[p]).id.as_str();
    assert_eq!((at(places[0]), at(places[1])), ("gravestone", "gravestone"));
    assert_eq!(at(places[2]), "grave_plot");
    // The player reads one beside it.
    let mut w = w;
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = places[0].step(Dir::N);
    w.perceive(&mut Vec::new());
    assert_eq!(
        w.perception().epitaph(places[0]),
        Some(cut[&places[0]].as_str())
    );
}

#[test]
fn a_full_yard_opens_the_way_down_and_the_crypts_fill_in_turn() {
    let (yard_places, first) = (40, burial(CHAIN[1]).places.len());
    assert!(
        !open(&yard(many(yard_places - 1)), "lictun"),
        "one plot left"
    );
    assert!(open(&yard(many(yard_places)), "lictun"), "full");
    // Enough to fill the yard and every crypt but the deepest, and one more for that.
    let deepest = CHAIN[CHAIN.len() - 1];
    let above: usize = (CHAIN[..CHAIN.len() - 1].iter())
        .map(|id| burial(id).places.len())
        .sum();
    let n = above + 1;
    let mut dead = many(n);
    // A petrified one among them takes no place.
    dead.insert(
        5,
        Fallen {
            petrified: true,
            ..fallen("Stone", "dwarf", "Turned to stone")
        },
    );
    let mut w = yard(dead);
    assert!(open(&w, "lictun"));
    let names = |w: &World, level: &str| -> Vec<String> {
        let cut = graves(w, level);
        (burial(level).places.iter())
            .filter_map(|p| cut.get(p).cloned())
            .collect()
    };
    let in_yard = names(&w, "lictun");
    assert_eq!(in_yard.len(), yard_places);
    assert!(
        in_yard[0].starts_with("Here lies Hero0 the dwarf warrior,"),
        "{in_yard:?}"
    );
    assert!(in_yard[39].starts_with("Here lies Hero39 "), "{in_yard:?}");
    go_down(&mut w, "lictun");
    assert_eq!(w.state.current_level, CHAIN[1]);
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "You descend the stairs to Līctūn crypt 1."),
        "{:?}",
        texts(&w)
    );
    let below = names(&w, CHAIN[1]);
    assert_eq!(below.len(), first);
    assert!(below[0].starts_with("Here lies Hero40 "), "{below:?}");
    assert!(below[0].ends_with(". Laid in the undercroft."), "{below:?}");
    let level = &w.state.levels[CHAIN[1]];
    for p in &burial(CHAIN[1]).places {
        assert!(CONTENT.terrain(level.terrain[*p]).grave, "{p} filled");
    }
    assert!(open(&w, CHAIN[1]));
    // Each crypt below full in turn, its own words on its graves, and its way down open.
    let mut laid = yard_places + first;
    for (depth, id) in CHAIN.iter().enumerate().skip(2) {
        go_down(&mut w, CHAIN[depth - 1]);
        assert_eq!(w.state.current_level, *id);
        let here = names(&w, id);
        if *id == deepest {
            break;
        }
        let places = burial(id).places.len();
        assert_eq!(here.len(), places, "{id}");
        assert!(
            here[0].starts_with(&format!("Here lies Hero{laid} ")),
            "{here:?}"
        );
        let words = &burial(id).epitaph;
        let tail = &words[words.find("{death}").expect("{death}") + "{death}".len()..];
        assert!(here[0].ends_with(tail), "{id}: {here:?}");
        laid += places;
        assert!(open(&w, id), "{id} full");
    }
    assert_eq!(w.state.current_level, deepest);
    let last = names(&w, deepest);
    assert_eq!(last.len(), 1);
    assert!(last[0].starts_with(&format!("Here lies Hero{} ", n - 1)));
    assert!(!open(&w, deepest), "the deepest's way down leads nowhere");
    // And back up, arriving on the way down.
    act(&mut w, "ascend");
    let up = CHAIN[CHAIN.len() - 2];
    assert_eq!(w.state.current_level, up);
    assert_eq!(
        w.player_pos(),
        burial(up).stairs.as_ref().expect("way down").pos
    );
}

#[test]
fn the_dead_are_out_of_the_hash_the_same_every_time_and_a_save_keeps_them() {
    let full = yard(many(45));
    assert!(open(&full, "lictun"));
    assert_eq!(full.state_hash(), yard(Vec::new()).state_hash());
    assert_eq!(graves(&full, "lictun"), graves(&yard(many(45)), "lictun"));
    // Given after the yard was made (a replay's dump): laid the same as made with it, and
    // given again with none, undone.
    let mut late = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut late, WALK);
    act(&mut late, "descend");
    late.set_graveyard(many(45));
    assert_eq!(graves(&late, "lictun"), graves(&full, "lictun"));
    assert_eq!(late.state.levels["lictun"], full.state.levels["lictun"]);
    let loaded = crate::save::decode(&crate::save::encode(&late), CONTENT.clone()).expect("loads");
    assert_eq!(loaded.state.levels["lictun"], late.state.levels["lictun"]);
    assert_eq!(loaded.state_hash(), late.state_hash());
    late.set_graveyard(Vec::new());
    assert!(!open(&late, "lictun"));
    assert_eq!(
        late.state.levels["lictun"],
        yard(Vec::new()).state.levels["lictun"]
    );
}

#[test]
fn the_yard_on_arrival() {
    let w = yard(many(3));
    let rows = dump::layer_rows(&w, Layer::View).join("\n");
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!("lictun_arrival", rows);
    });
}

#[test]
fn a_run_to_the_yard_replays_exactly_under_any_graveyard() {
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    w.set_graveyard(many(50));
    let mut replay = Replay::new(&w);
    let actions = format!("{WALK} descend move:E*3 wait*5 move:E*12");
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(open(&w, "lictun"));
    // Verified with no graveyard at all.
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
