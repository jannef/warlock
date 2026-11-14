//! Holtwīc, the village at the end of the Ascmearc's north road (#1140), and Ælfhām, the hidden fey
//! hall in the wood beside it: the way in from Cēaptūn by road and out by the edge, everyone at
//! peace, the wise-woman's name and topics (no reveal), the mound hidden until revealed and the
//! way down, the branch's three fey floors with the moss bed on the last, the dev scenarios, the
//! village at the start (a snapshot), and a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Stance};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::test_support::run as act;
use crate::test_support::texts as said;
use std::sync::LazyLock;

/// Cēaptūn's cell, where the north road starts.
const TOWN_CELL: Pos = Pos::new(18, 28);
/// The north road's last road cell.
const ROAD_END: Pos = Pos::new(18, 18);
/// Holtwīc's cell on the overworld.
const VILLAGE_CELL: Pos = Pos::new(18, 17);
/// The fairy mound over Ælfhām, hidden in the wood.
const MOUND: Pos = Pos::new(25, 13);
/// Where the player arrives in the village: the lane from the south.
const START: Pos = Pos::new(30, 24);
const WYNFLAED: Pos = Pos::new(54, 13);
/// The moss bed's name: the place on Ælfhām's bottom floor where the quest (#1141) puts the
/// villager.
const MOSS_BED: &str = "the moss bed";

/// Built-in content with nobody hungering, and nobody met on the way (the walks test the ways in;
/// `ascmearc_encounter_tests` meets parties).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.remove_encounter_tables();
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn village() -> World {
    World::new(CONTENT.clone(), "holtwic", None).expect("a run in Holtwīc")
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn terrain_at(at: Pos) -> &'static str {
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    CONTENT.terrain(map.terrain[at]).id.as_str()
}

#[test]
fn the_village_is_at_the_north_roads_end_and_left_by_its_edge() {
    for y in ROAD_END.y..TOWN_CELL.y {
        assert_eq!(terrain_at(Pos::new(18, y)), "road", "(18, {y})");
    }
    assert_eq!(terrain_at(VILLAGE_CELL), "town");
    // On the edge of Holegnwudu: forest two cells east.
    assert_eq!(terrain_at(Pos::new(20, 17)), "forest");
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    act(&mut w, "move:E*6 move:N*11");
    assert_eq!(w.player_pos(), VILLAGE_CELL);
    assert_eq!(said(&w).last().unwrap(), "The way into Holtwīc is here.");
    act(&mut w, "descend");
    assert_eq!(w.state.current_level, "holtwic");
    assert_eq!(w.player_pos(), START, "on the lane from the road");
    let words = said(&w);
    assert!(
        words
            .iter()
            .any(|l| l.starts_with("Holtwīc: thatch, woodsmoke"))
    );
    assert!(words.contains(&"You enter Holtwīc.".to_string()));
    let level = CONTENT.level("holtwic").expect("the village's level");
    assert!(level.stairs.is_empty(), "{:?}", level.stairs);
    let exit = level.exit.as_ref().expect("an edge exit");
    assert_eq!((exit.to.as_str(), exit.pos), ("ascmearc", VILLAGE_CELL));
    // Back down the lane and off the edge.
    act(&mut w, "move:S*2");
    assert_eq!(w.state.current_level, "ascmearc");
    assert_eq!(w.player_pos(), VILLAGE_CELL);
}

#[test]
fn everyone_in_the_village_is_at_peace_with_the_player() {
    let w = village();
    assert_eq!(w.campaign(), "ascmearc");
    let player = w.state.player;
    let others: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.level == "holtwic" && a.id != player)
        .collect();
    // Five cottars, the wise-woman, a dog, a cat, and three hens (#1248).
    assert_eq!(others.len(), 11);
    for actor in others {
        assert_ne!(w.stance(actor.id, player), Stance::Hostile);
    }
    assert_eq!(w.danger("holtwic"), 0, "nothing comes to the village");
    assert!(w.state.current().shops.is_empty(), "no shop");
}

/// The chicken pen (#1248): three hens behind a fence, its gate shut, and they stay in it while
/// the day goes by; the player opens the gate like a door.
#[test]
fn the_hens_keep_to_their_pen() {
    let mut w = village();
    let content = w.content().clone();
    let pen = |p: Pos| (51..=55).contains(&p.x) && (18..=20).contains(&p.y);
    let hens = |w: &World| -> Vec<Pos> {
        (w.state.actors.values())
            .filter(|a| a.level == "holtwic" && content.creature(a.creature).id == "chicken")
            .map(|a| a.pos)
            .collect()
    };
    assert_eq!(hens(&w).len(), 3);
    let gate = Pos::new(53, 21);
    let at = |w: &World, p: Pos| content.terrain(w.state.current().terrain[p]).id.clone();
    assert_eq!(at(&w, gate), "gate_closed");
    assert_eq!(at(&w, Pos::new(50, 19)), "fence");
    act(&mut w, "wait*200");
    assert!(hens(&w).into_iter().all(pen), "{:?}", hens(&w));
    put_player(&mut w, gate.step(Dir::S));
    act(&mut w, "open:N");
    assert_eq!(at(&w, gate), "gate_open");
}

#[test]
fn gammer_wynflaed_talks_of_the_village_the_wood_and_the_fair_folk() {
    let mut w = village();
    let id = (w.state.actor_at("holtwic", WYNFLAED))
        .expect("the wise-woman")
        .id;
    assert_eq!(w.state.actors[&id].npc.as_deref(), Some("wynflaed"));
    let topics: Vec<String> = (w.topics(id).into_iter()).map(|t| t.id).collect();
    assert_eq!(
        topics,
        ["quest-taken_under_the_hill", "holtwic", "wood", "fair_folk"]
    );
    put_player(&mut w, WYNFLAED.step(Dir::W));
    let before = said(&w).len();
    act(&mut w, "chat:E");
    // What she says first (a mutter of hers may follow it in the same turn).
    assert!(said(&w)[before].starts_with("Gammer Wynflæd says: \""));
    // Her quest is `wynflaed_quest_tests.rs`'s.
    for topic in &topics[1..] {
        let before = said(&w).len();
        act(&mut w, &format!("ask:E:{topic}"));
        assert!(said(&w)[before].starts_with("Gammer Wynflæd says: \""));
    }
    assert!(
        w.state.revealed.is_empty(),
        "her quest reveals the mound, not her talk"
    );
}

#[test]
fn the_mound_is_hidden_in_the_wood_until_revealed() {
    let (at, mound) = CONTENT
        .hidden_location("ascmearc", "aelfham")
        .expect("Ælfhām");
    assert_eq!((at, terrain_at(at)), (MOUND, "forest"));
    assert_eq!(
        (
            CONTENT.terrain(mound.terrain).id.as_str(),
            mound.name.as_str()
        ),
        ("barrow", "Ælfhām")
    );
    assert_eq!(MOUND.chebyshev(VILLAGE_CELL), 7, "near the village");
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    assert_eq!(map.region_at(MOUND), Some("Holegnwudu"));
    for stairs in &map.stairs {
        assert!(stairs.pos.chebyshev(MOUND) >= 6, "{} too close", stairs.pos);
    }
    for d in Dir::ALL {
        assert_ne!(terrain_at(MOUND.step(d)), "road", "no road leads to it");
    }
    let mut w = World::new(CONTENT.clone(), "ascmearc", None).expect("the Ascmearc");
    put_player(&mut w, MOUND.step(Dir::S));
    assert!(w.perception().is_visible(MOUND));
    assert_eq!(w.perception().leads_to(MOUND), None);
    let json = dump::json(&w, &[Layer::View, Layer::Memory, Layer::Locations], false).to_string();
    assert!(!json.contains("lfhām"), "the mound in the JSON dump");
    // The dev reveal marks it (as the quest will), and it leads down into the branch.
    act(&mut w, "reveal:aelfham");
    assert_eq!(w.perception().leads_to(MOUND).as_deref(), Some("Ælfhām"));
    put_player(&mut w, MOUND);
    act(&mut w, "descend");
    assert_eq!(w.perception().level_id(), "aelfham:1");
    assert!(said(&w).iter().any(|l| l.contains("pipes are playing")));
}

#[test]
fn aelfham_is_three_floors_of_fey_with_the_moss_bed_at_the_bottom() {
    let def = CONTENT.dungeon("aelfham").expect("Ælfhām");
    assert_eq!((def.name.as_str(), def.depth), ("Ælfhām", 3));
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    for depth in 1..=3 {
        let (rule, at, spawning) = CONTENT.spawning(&def.level_id(depth)).expect("a danger");
        assert_eq!(rule.at(at, 1), depth as u8 + 3, "depth {depth}");
        for c in &spawning.population {
            assert!(CONTENT.has_tag(*c, "fey"), "{}", CONTENT.creature(*c).id);
        }
    }
    let v = (CONTENT.vaults().iter())
        .find(|v| v.id() == "moss_bower")
        .expect("the vault");
    assert!(v.pinned, "stamped only where named");
    for seed in 1..=4 {
        let bottom = crate::mapgen::generate(&CONTENT, ("ascmearc", seed), "aelfham:3", None)
            .expect("generates");
        let beds: Vec<_> = (bottom.places.values())
            .filter(|p| p.name == MOSS_BED)
            .collect();
        assert_eq!(beds.len(), 1, "seed {seed}");
        for creature in &bottom.creatures {
            assert!(CONTENT.has_tag(creature.creature, "fey"), "seed {seed}");
        }
        for depth in [1, 2] {
            let id = format!("aelfham:{depth}");
            let floor = crate::mapgen::generate(&CONTENT, ("ascmearc", seed), &id, None)
                .expect("generates");
            assert!(floor.places.is_empty(), "seed {seed}, depth {depth}");
        }
    }
}

#[test]
fn the_fey_hall_scenario_starts_on_its_first_floor() {
    let w = World::new(CONTENT.clone(), "aelfham", None).expect("a run in Ælfhām");
    assert_eq!(w.state.current_level, "aelfham:1");
    assert_eq!(w.campaign(), "ascmearc");
}

#[test]
fn the_village_at_the_start() {
    let w = village();
    let rows = dump::layer_rows(&w, Layer::View).join("\n");
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!("holtwic_start", rows);
    });
}

#[test]
fn a_run_in_the_village_replays_exactly() {
    let mut w = village();
    let mut replay = Replay::new(&w);
    let actions = "move:N*8 move:E*11 move:NE move:N*3 move:E*11 chat:SE ask:SE:fair_folk wait*3";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
