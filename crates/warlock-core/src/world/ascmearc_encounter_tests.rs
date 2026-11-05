//! The Ascmearc's encounter tables (#806; `docs/design/overworld-encounters.md` §7.4, §8;
//! `docs/dev/encounters.md`): every table names the overworld's regions and the march's land is
//! covered, the road is the safest way, a forced encounter of each outcome replays exactly, the
//! pedlar talks and tells the ways, the villagers on the road speak its lines, a `draw` entry
//! brings a branch's folk, and the tables that change as the run goes on (a quest's variant, a
//! quest under way, the days gone by).

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, EntryStance, Landscape, local_area_id};
use crate::daylight::{self, DayPhase};
use crate::replay::Replay;
use std::sync::LazyLock;

/// The road one cell east of Cēaptūn, in Cēapdenu: seven steps east of the pass.
const ROAD: Pos = Pos::new(19, 28);
const WALK_TO_ROAD: &str = "move:E*7";

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn start() -> World {
    World::new(CONTENT.clone(), "ascmearc", None).expect("a run of the Ascmearc")
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World, text: &str) -> bool {
    world.messages().iter().any(|m| m.text.contains(text))
}

/// The player set down on overworld cell `at` (no time, no roll).
fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

/// A run on the road east of the town, at noon.
fn on_the_road() -> World {
    let mut w = start();
    put_player(&mut w, ROAD);
    act(&mut w, "skip:4h");
    assert_eq!(daylight::phase(w.state.time), DayPhase::Day);
    w
}

/// The members of the encounter parties on the player's level, as actors.
fn party(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.id != w.state.player && w.party_of(a.id).is_some())
        .collect()
}

#[test]
fn every_table_names_the_overworlds_regions_and_the_march_is_covered() {
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    let names: BTreeSet<&str> = map.regions.iter().map(|r| r.name.as_str()).collect();
    let tables: Vec<_> = CONTENT.encounter_tables("ascmearc").collect();
    assert_eq!(tables.len(), 11);
    for t in &tables {
        let whens = std::iter::once(&t.when).chain(t.entries.iter().map(|e| &e.when));
        for when in whens {
            for r in &when.regions {
                assert!(names.contains(r.as_str()), "{}: {r}", t.id);
            }
            assert!(
                when.level_max.is_none(),
                "{}: no level gates (#796 Q5)",
                t.id
            );
        }
    }
    // Every land cell with a local area but the mountain wall's matches a table by day.
    let mut w = on_the_road();
    let player = w.state.player;
    for y in 0..map.terrain.height() {
        for x in 0..map.terrain.width() {
            let p = Pos::new(x, y);
            let local = CONTENT.local_area("ascmearc", &local_area_id(p));
            if local.is_none() || map.region_at(p) == Some("Grægweall") {
                continue;
            }
            w.state.actors.get_mut(&player).expect("player").pos = p;
            assert!(
                !w.encounter_tables_here().is_empty(),
                "{p} matches no table"
            );
        }
    }
}

#[test]
fn the_road_is_the_safest_way() {
    let w = on_the_road();
    let rules = CONTENT.meetings().expect("the rules");
    assert_eq!(w.encounter_tables_here(), ["roads"]);
    // 1 % an hour by day: a day's march of twelve hours meets someone about one day in eight.
    assert_eq!(w.encounter_rate_here(), Some(100));
    let road = rules.rate_of(Landscape::Road);
    for landscape in [
        Landscape::Plains,
        Landscape::Grassland,
        Landscape::Forest,
        Landscape::Hills,
        Landscape::DenseForest,
        Landscape::Swamp,
    ] {
        assert!(road < rules.rate_of(landscape), "{landscape:?}");
    }
    // The road's hostile rows are a third of its weight by day.
    let (table, _) = CONTENT
        .encounter_entry("ascmearc", "road_pedlar")
        .expect("the pedlar");
    // (The Strange Spirit, met only while a calling is open, is no one's ordinary road.)
    let by_day = |e: &&crate::content::EncounterEntry| {
        !e.when.phase.contains(&DayPhase::Night) && e.when.calling.is_none()
    };
    let day: Vec<_> = (table.entries.iter()).filter(by_day).collect();
    let all: u32 = day.iter().map(|e| e.weight).sum();
    let hostile: u32 = (day.iter())
        .filter(|e| e.stance == EntryStance::Hostile)
        .map(|e| e.weight)
        .sum();
    assert_eq!((hostile, all), (5, 12));
}

#[test]
fn exactly_the_footpads_the_duergar_and_the_cultists_are_talked_past() {
    // The owner: "be careful what to flag" (#1017; docs/design/skill-uses.md §6.2 B3).
    let mut flagged: Vec<_> = (CONTENT.encounter_tables("ascmearc"))
        .flat_map(|t| &t.entries)
        .filter(|e| e.talk_past)
        .map(|e| e.id.as_str())
        .collect();
    flagged.sort_unstable();
    assert_eq!(
        flagged,
        [
            "ascdun_duergar",
            "fen_cultists",
            "road_footpads",
            "road_footpads_night",
            "vale_footpad",
            "woods_footpads",
        ]
    );
}

#[test]
fn a_bard_may_talk_past_footpads_on_the_road() {
    let spec: crate::character::CharacterSpec = "human,male,bard".parse().expect("a bard");
    let mut w = World::with_character(CONTENT.clone(), "ascmearc", None, Some(&spec)).expect("w");
    let mut replay = Replay::new(&w);
    for action in parse_actions(&format!(
        "{WALK_TO_ROAD} meet:road_footpads:met encounter:bluff"
    ))
    .expect("actions")
    {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(said(&w, "past.") || said(&w, "fooled."));
    assert!(w.state.encounter.is_none());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn a_forced_encounter_of_each_outcome_replays_exactly() {
    for (actions, party_made) in [
        ("meet:road_footpads:drop encounter:ambush wait*2", true),
        ("meet:road_footpads:met encounter:fight wait*2", true),
        ("meet:road_footpads:ambushed wait*2", true),
        ("meet:road_footpads:pass move:E", false),
        ("meet:road_goblins:met encounter:evade", false),
        ("meet:road_pedlar encounter:hail wait", true),
        ("meet:road_villagers encounter:hail wait", true),
        ("meet:road_deer encounter:hunt wait", true),
        ("meet:road_deer encounter:pass", false),
        ("meet:downs_strays:met encounter:fight wait", true),
    ] {
        let mut w = start();
        let mut replay = Replay::new(&w);
        for (i, part) in [WALK_TO_ROAD, actions].into_iter().enumerate() {
            for action in parse_actions(part).expect("actions") {
                w.apply(&action);
                replay.push(&action, &w);
            }
            if i == 0 {
                // Nobody met on the way out of the pass (seed 1).
                assert_eq!(w.player_pos(), ROAD);
                assert!(w.state.encounter.is_none() && party(&w).is_empty());
            }
        }
        let made = !party(&w).is_empty();
        // An evade may fail and open the fight; the rest are as forced.
        if !actions.contains("evade") {
            assert_eq!(made, party_made, "{actions}");
        }
        assert!(w.state.encounter.is_none(), "{actions}: answered");
        let verified = Replay::from_json(&replay.to_json())
            .expect("json")
            .run(CONTENT.clone())
            .expect("the replay verifies");
        assert_eq!(verified.state_hash(), w.state_hash(), "{actions}");
    }
}

#[test]
fn the_pedlar_hailed_talks_and_tells_the_ways() {
    let mut w = on_the_road();
    act(&mut w, "meet:road_pedlar");
    assert!(said(&w, "You meet a pedlar on the way."));
    act(&mut w, "encounter:hail");
    assert!(said(&w, "You go to meet a pedlar."));
    let pedlar = (party(&w).first().map(|a| a.id)).expect("the pedlar");
    let topics: Vec<String> = (w.topics(pedlar).into_iter()).map(|t| t.id).collect();
    assert_eq!(topics, ["rumours", "news", "wares"]);
    // Beside him: the ways he tells mark the map.
    let me = w.player_pos();
    let level = &w.state.levels[&w.state.current_level];
    let dir = (Dir::ALL.into_iter())
        .find(|&d| {
            level.is_walkable(&CONTENT, me.step(d))
                && w.state
                    .actor_at(&w.state.current_level, me.step(d))
                    .is_none()
        })
        .expect("room beside the player");
    w.state.actors.get_mut(&pedlar).expect("pedlar").pos = me.step(dir);
    w.perceive(&mut Vec::new());
    act(&mut w, &format!("chat:{dir:?} ask:{dir:?}:rumours"));
    assert!(said(&w, super::rumour::MARKED), "the way is marked");
    let overworld = &w.state.levels["ascmearc"];
    assert!(
        overworld
            .memory
            .get(Pos::new(87, 28))
            .is_some_and(|m| m.is_some()),
        "the ford"
    );
}

#[test]
fn villagers_on_the_road_speak_its_lines() {
    let mut w = on_the_road();
    act(&mut w, "meet:road_villagers encounter:hail");
    let folk = party(&w);
    assert!(!folk.is_empty());
    for villager in folk {
        assert_eq!(villager.npc.as_deref(), Some("road_folk"));
        let dialogue = CONTENT.dialogue_of(villager.creature, villager.npc.as_deref());
        let greet = dialogue.and_then(|d| CONTENT.dialogue(d).greet.clone());
        assert_eq!(
            greet.as_deref(),
            Some("Mornin'. Fine day for the road, innit.")
        );
    }
    // Villagers in the vale's roads only.
    let (_, entry) = CONTENT
        .encounter_entry("ascmearc", "road_villagers")
        .expect("the villagers");
    assert_eq!(entry.when.regions, ["Cēapdenu", "Burnadenu"]);
}

#[test]
fn a_draw_entry_brings_the_barrows_folk() {
    let branch = CONTENT.dungeon("cealdhlaw").expect("Cealdhlāw");
    let spawning = branch.spawning.as_ref().expect("its population");
    for seed in 1..=8 {
        let mut w = World::new(CONTENT.clone(), "ascmearc", Some(seed)).expect("a run");
        put_player(&mut w, ROAD);
        act(&mut w, "meet:downs_strays:met encounter:fight");
        let folk = party(&w);
        assert!((1..=3).contains(&folk.len()), "seed {seed}: {}", folk.len());
        for a in folk {
            let c = a.creature;
            let theirs = spawning.population.contains(&c)
                || spawning.generic.contains(&c)
                || CONTENT.family(c) != c;
            assert!(theirs, "seed {seed}: {}", CONTENT.creature(c).id);
            assert!(!CONTENT.creature(c).unique);
        }
    }
}

#[test]
fn the_tables_change_as_the_run_goes_on() {
    let mut w = on_the_road();
    // Black Wystan wanted: his archers on the vale's roads; another criminal: none.
    for (variant, watched) in [("wystan", true), ("sallow", false)] {
        let json = format!(
            r#"{{"records": {{"guild_wanted": {{"stage": "active", "completed": 0,
                "roll": {{"target": "{variant}", "variant": "{variant}", "count": 1}}}}}}}}"#
        );
        w.state.quests = serde_json::from_str(&json).expect("quests");
        let here = w.encounter_tables_here();
        assert_eq!(
            here.contains(&"wanted_wystan".to_string()),
            watched,
            "{variant}"
        );
    }
    // Sister Hild's errand under way: the cult on the fen, and half as many again met there.
    let fen = Pos::new(70, 46);
    put_player(&mut w, fen);
    w.state.quests = serde_json::from_str(r#"{"records": {}}"#).expect("quests");
    assert_eq!(w.encounter_tables_here(), ["fen"]);
    let calm = w.encounter_rate_here().expect("a rate");
    w.state.quests = serde_json::from_str(
        r#"{"records": {"fen_minster": {"stage": "active", "completed": 0}}}"#,
    )
    .expect("quests");
    assert_eq!(w.encounter_tables_here(), ["fen", "fen_cult"]);
    assert_eq!(w.encounter_rate_here(), Some(calm * 3 / 2));
    // A month in, the orcs come down into Burnadenu.
    let map = CONTENT.overworld_of("ascmearc").expect("the overworld");
    let burnadenu = (0..map.terrain.height())
        .flat_map(|y| (0..map.terrain.width()).map(move |x| Pos::new(x, y)))
        .find(|&p| {
            map.region_at(p) == Some("Burnadenu")
                && CONTENT.local_area("ascmearc", &local_area_id(p)).is_some()
        })
        .expect("a cell of Burnadenu");
    put_player(&mut w, burnadenu);
    assert!(
        !w.encounter_tables_here()
            .contains(&"orcs_come_down".to_string())
    );
    w.state.time =
        crate::time::GameTime::from_seconds(w.state.time.seconds() + 30 * crate::time::DAY);
    assert!(
        w.encounter_tables_here()
            .contains(&"orcs_come_down".to_string())
    );
}
