//! Danger levels and creatures that keep coming (spec §7 "Danger levels", `spawn.rs`): the
//! danger of a level, where and what comes, the cap, uniques, frozen floors, the talents, and
//! determinism.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::perception::Perception;
use crate::time::{Duration, GameTime};
use std::collections::BTreeSet;

/// A dark hall of 70 by 19 floor cells (the player's lantern lights a few around them), where
/// kobolds of danger 3 live, a chief among them who comes once a run.
const HAUNT: &str = r#"name = "Haunt"
danger = 3
population = ["kobold", "kobold_archer", "kobold_shaman", "chief"]
population_count = 6
spawn = { every = "10m", chance = 100 }
map = '''
########################################################################
#@.....................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#......................................................................#
#.....................................................................>#
#......................................................................#
########################################################################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "barrow:1" }
"#;

/// A kobold chief: a unique type, of the haunt's danger.
const CHIEF: &str = r##"[[creature]]
id = "chief"
name = "kobold chief"
glyph = "k"
fg = "#c08040"
faction = "kobolds"
level = 3
xp = 25
unique = true
weight = 30000
stats = { strength = 12, dexterity = 12, toughness = 12, intelligence = 8, willpower = 10, presence = 10 }
"##;

/// A branch of danger 2 rising by 1 a floor, its third floor sudden and deep.
const BARROW: &str = r#"
[[dungeon]]
id = "barrow"
name = "Barrow"
generator = "rooms"
depth = 3
size = [80, 21]
danger = 2
population = ["kobold", "kobold_archer", "kobold_shaman"]

[[dungeon.levels]]
from = 3
danger = { base = 9, per_depth = 0 }
"#;

fn content() -> Arc<Content> {
    content_with(HAUNT)
}

/// The builtin content with the haunt as `haunt`, the chief, and the barrow.
fn content_with(haunt: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/haunt.toml", haunt);
    files.insert("data/creatures/zz_chief.toml", CHIEF);
    let dungeons = format!(
        "{}{BARROW}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    files.insert(
        "scenarios/haunt.toml",
        "name = \"Haunt\"\nstart_level = \"haunt\"\nlantern = \"lantern\"\nhp = 900\n",
    );
    Arc::new(Content::load(&files).expect("content"))
}

fn haunt(seed: u64) -> World {
    World::new(content(), "haunt", Some(seed)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// The creatures on the player's level but the player.
fn others(world: &World) -> BTreeSet<ActorId> {
    let level = &world.state.current_level;
    (world.state.actors.values())
        .filter(|a| a.level == *level && a.id != world.state.player)
        .map(|a| a.id)
        .collect()
}

#[test]
fn the_danger_is_the_rule_at_the_depth() {
    let w = haunt(1);
    assert_eq!(w.danger("haunt"), 3);
    assert_eq!(
        (
            w.danger("barrow:1"),
            w.danger("barrow:2"),
            w.danger("barrow:3")
        ),
        (2, 3, 9)
    );
    // No danger given: 0, nothing comes (the showcase's levels and its Depths).
    for id in ["hub", "arena", "depths:1", "depths:5", "wildlands"] {
        assert_eq!(w.danger(id), 0, "{id}");
        assert!(w.content.spawning(id).is_none(), "{id}");
    }
}

#[test]
fn a_player_scaled_level_follows_the_character() {
    let scaled = HAUNT.replace(
        "danger = 3",
        "danger = { base = 2, player = { percent = 100, from = 1, max = 5 } }",
    );
    let content = content_with(&scaled);
    let mut w = World::new(content, "haunt", Some(1)).expect("world");
    assert_eq!(w.danger("haunt"), 2);
    run(&mut w, "xp:2431");
    assert_eq!((w.player_level(), w.danger("haunt")), (3, 4));
    run(&mut w, "levelup:auto xp:100000");
    assert_eq!(w.danger("haunt"), 5);
}

#[test]
fn only_levels_with_a_danger_keep_a_spawner() {
    let w = haunt(1);
    assert!(w.state.current().spawner.is_some());
    // The builtin worlds: no level has one, so their states are as they were.
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut d = World::new(content, "depths", Some(3)).expect("world");
    run(&mut d, "rest:24h spawn");
    assert!(d.state.levels.values().all(|l| l.spawner.is_none()));
    assert!(said(&d).last().is_some_and(|m| m.contains("no danger")));
}

/// The property test of the design (§9.6): over many seeds, nothing comes in view or within
/// [`spawn::AWAY`] cells, nothing above the danger but out-of-depth draws within 3, a unique
/// type once, and the cap holds.
#[test]
fn what_comes_comes_out_of_sight_near_the_danger_and_within_the_cap() {
    for seed in 0..12 {
        let mut w = haunt(seed);
        let danger = w.danger("haunt");
        let mut chiefs = 0;
        for _ in 0..30 {
            let seen = Perception::compute(&w.state, &w.content, &mut w.lights);
            let Some(id) = w.attempt(true) else {
                continue;
            };
            let actor = &w.state.actors[&id];
            let def = w.content.creature(actor.creature);
            assert!(
                !seen.sees_actors_at(actor.pos),
                "seed {seed}: seen at {}",
                actor.pos
            );
            assert!(
                actor.pos.chebyshev(w.player_pos()) >= spawn::AWAY,
                "seed {seed}"
            );
            assert!(def.level <= danger + 3, "seed {seed}: {}", def.id);
            chiefs += usize::from(def.unique);
            // Its cell was free and plain.
            let level = w.state.current();
            assert!(
                level.is_walkable(&w.content, actor.pos) && level.stairs_at(actor.pos).is_none()
            );
        }
        assert!(chiefs <= 1, "seed {seed}: the chief came {chiefs} times");
        assert_eq!(chiefs == 1, w.state.progress.uniques.contains("chief"));
        // No two stand on one cell.
        let cells: BTreeSet<_> = others(&w).iter().map(|a| w.state.actors[a].pos).collect();
        assert_eq!(cells.len(), others(&w).len());
    }
    // Left to the clock, the hall refills only to its cap (6 hostiles).
    let mut w = haunt(5);
    w.state.time += Duration::hours(6);
    w.spawn_tick();
    assert_eq!(others(&w).len(), 6);
    w.state.time += Duration::hours(6);
    w.spawn_tick();
    assert_eq!(others(&w).len(), 6);
}

#[test]
fn creatures_come_with_the_clock_and_the_same_actions_bring_the_same_ones() {
    let mut a = haunt(9);
    let mut b = haunt(9);
    // Every 10 minutes (100 turns), certainly: none before the first attempt is due.
    run(&mut a, "wait*99");
    assert!(others(&a).is_empty());
    run(&mut a, "wait*2");
    assert_eq!(others(&a).len(), 1);
    run(&mut b, "wait*101");
    assert_eq!(a.state_hash(), b.state_hash());
    // A rest brings several, and stops when one comes into view or acts against the player.
    run(&mut a, "rest:2h");
    run(&mut b, "rest:2h");
    assert!(others(&a).len() > 1);
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn floors_the_player_left_stay_frozen() {
    let mut w = haunt(2);
    // Off to the barrow; a day passes there.
    let stairs = Pos::new(70, 18);
    w.player_mut().pos = stairs;
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "barrow:1");
    // Its first attempt 15 minutes after the player arrived (the stairs took a turn since).
    let barrow = w
        .state
        .current()
        .spawner
        .clone()
        .expect("the barrow has a spawner");
    let after =
        |at: GameTime, t: GameTime, every: u64| at > t && at <= t + Duration::minutes(every);
    assert!(after(barrow.at, w.state.time, 15));
    run(&mut w, "rest:24h");
    // Nothing came to the haunt meanwhile; back there, its clock starts again on arrival.
    assert!(
        (w.state.actors.values()).all(|a| a.level != "haunt"),
        "{:?}",
        w.state
            .actors
            .values()
            .map(|a| &a.level)
            .collect::<Vec<_>>()
    );
    run(&mut w, "ascend");
    assert_eq!(w.state.current_level, "haunt");
    let haunt = w
        .state
        .current()
        .spawner
        .clone()
        .expect("the haunt's spawner");
    assert!(after(haunt.at, w.state.time, 10));
}

#[test]
fn quiet_delver_puts_attempts_off_and_sixth_sense_feels_them() {
    let mut w = haunt(4);
    w.player_mut().talents.take("quiet_delver", None);
    // The first attempt was due when the hall was made; from then on, every 20 minutes.
    run(&mut w, "wait*101");
    assert_eq!(others(&w).len(), 1);
    run(&mut w, "wait*150");
    assert_eq!(others(&w).len(), 1, "every 20 minutes now, not 10");
    run(&mut w, "wait*60");
    assert_eq!(others(&w).len(), 2);
    // Sixth sense: on arrival, how the place feels; at the second rank, what stirs.
    let mut w = haunt(4);
    w.player_mut().talents.take("alert", None);
    w.player_mut().talents.take("sixth_sense", None);
    w.player_mut().pos = Pos::new(70, 18);
    run(&mut w, "descend");
    assert!(
        said(&w).iter().any(|m| m == "This place feels dangerous."),
        "{:?}",
        said(&w)
    );
    run(&mut w, "ascend");
    assert!(said(&w).iter().any(|m| m == "This place feels dangerous."));
    run(&mut w, "spawn");
    assert!(!said(&w).iter().any(|m| m == spawn::STIRS));
    w.player_mut().talents.take("sixth_sense", None);
    run(&mut w, "spawn");
    assert!(said(&w).iter().any(|m| m == spawn::STIRS));
}

#[test]
fn the_spawn_dev_action_brings_one_and_replays() {
    let mut w = haunt(6);
    let text = "spawn spawn wait*3";
    run(&mut w, text);
    assert_eq!(others(&w).len(), 2);
    let mut again = haunt(6);
    run(&mut again, text);
    assert_eq!(w.state_hash(), again.state_hash());
    // It is saved with the level.
    let json = serde_json::to_string(&w.state).expect("serialize");
    assert!(json.contains("\"spawner\""));
    // The dev dumps' header says how things stand (2 kobolds of the 6 the hall holds).
    let header = &crate::dump::layer_rows(&w, crate::dump::Layer::Actors)[0];
    assert!(
        header.starts_with("danger 3 population 2/6 spawn next "),
        "{header}"
    );
}
