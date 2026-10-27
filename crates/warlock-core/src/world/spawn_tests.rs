//! Danger levels and creatures that keep coming (spec §7 "Danger levels", `spawn.rs`): the
//! danger of a level, where and what comes, the cap, uniques, frozen floors, the talents, and
//! determinism.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, LevelTemplate};
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
# A teeming den needs a danger, so it would move the map against the floor without one.
room_effects = { banned = ["teeming"] }

[[dungeon.levels]]
from = 3
danger = { base = 9, per_depth = 0 }

[[dungeon]]
id = "lair"
name = "Lair"
generator = "caves"
depth = 1
size = [100, 40]
danger = 3
population = ["chief", "kobold"]
population_count = 30
"#;

fn content() -> Arc<Content> {
    content_with(HAUNT)
}

/// The builtin content with the haunt as `haunt`, the chief, and the barrow.
fn content_with(haunt: &str) -> Arc<Content> {
    Arc::new(Content::load(&files_with(haunt)).expect("content"))
}

/// The files of [`content_with`].
fn files_with(haunt: &str) -> ContentFiles {
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
    files
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
fn an_incorporeal_creature_is_never_placed_inside_a_wall() {
    // The hall with a wall pillar in every other cell, haunted by stone haunts only.
    let hall = HAUNT
        .replace(
            r#"population = ["kobold", "kobold_archer", "kobold_shaman", "chief"]"#,
            r#"population = ["stone_haunt"]"#,
        )
        .replace("......", ".#.#.#");
    for seed in 0..6 {
        let mut w = World::new(content_with(&hall), "haunt", Some(seed)).expect("world");
        for _ in 0..20 {
            w.attempt(true);
        }
        let level = w.state.current();
        assert!(!others(&w).is_empty(), "seed {seed}: haunts came");
        for id in others(&w) {
            let at = w.state.actors[&id].pos;
            assert!(
                level.is_walkable(&w.content, at),
                "seed {seed}: in a wall at {at}"
            );
        }
    }
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

/// The barrow's first floor for the run with `seed`.
fn barrow_floor(content: &Content, seed: u64) -> LevelTemplate {
    crate::mapgen::generate(content, ("showcase", seed), "barrow:1", None).expect("barrow:1")
}

/// The builtin content with the barrow but no danger anywhere: the same maps, no population.
fn without_danger() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_chief.toml", CHIEF);
    let plain: String = (BARROW.lines())
        .filter(|l| !l.starts_with("danger") && !l.starts_with("population"))
        .map(|l| format!("{l}\n"))
        .collect();
    let dungeons = format!(
        "{}{plain}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    Arc::new(Content::load(&files).expect("content"))
}

/// The design's §9.3 and §9.6: a generated floor of a branch with a danger is laid down full,
/// by its walkable cells, away from the stairs, near the danger, from its own stream after the
/// loot (the map, traps, secrets, and loot are the same with any population), and its creatures
/// roll their gear from the level's gear stream after it.
#[test]
fn a_generated_floor_is_laid_down_full_and_nothing_else_moves() {
    let content = content();
    let plain = without_danger();
    let mut files = files_with(HAUNT);
    files.edit_table("creatures", super::gear_tests::without_gear);
    let ungeared = Content::load(&files).expect("valid without gear");
    let floor = content.terrain_id("floor").expect("floor");
    for seed in 0..10 {
        let with = barrow_floor(&content, seed);
        let without = barrow_floor(&plain, seed);
        assert_eq!(with.terrain, without.terrain, "seed {seed}");
        assert_eq!(with.traps, without.traps, "seed {seed}");
        assert_eq!(with.secrets, without.secrets, "seed {seed}");
        assert_eq!(with.objects, without.objects, "seed {seed}");
        // The generator's own creatures (patrols, a vault's) come first, as they were.
        let (own, population) = with.creatures.split_at(without.creatures.len());
        assert_eq!(own, without.creatures.as_slice(), "seed {seed}");
        let walkable = (with.terrain.iter())
            .filter(|(_, t)| content.terrain(**t).walkable)
            .count();
        let (_, _, spawning) = content.spawning("barrow:1").expect("rules");
        let count = spawning
            .count
            .for_cells(u32::try_from(walkable).expect("cells"));
        assert_eq!(population.len(), count as usize, "seed {seed}");
        let danger = 2;
        let mut cells: BTreeSet<_> = own.iter().map(|s| s.pos).collect();
        for spawn in population {
            let def = content.creature(spawn.creature);
            let deepest = danger + 1 + spawning.odds.deeper_cap;
            assert!(def.level <= deepest, "seed {seed}: {}", def.id);
            let kinds = [&spawning.population, &spawning.generic];
            assert!(kinds.iter().any(|k| k.contains(&spawn.creature)));
            assert_eq!(with.terrain.get(spawn.pos), Some(&floor), "seed {seed}");
            assert!(
                (with.stairs.iter()).all(|s| s.pos.chebyshev(spawn.pos) >= 8),
                "seed {seed}: {} by the stairs",
                spawn.pos
            );
            assert!(cells.insert(spawn.pos), "seed {seed}: two on {}", spawn.pos);
            // Its route loops and never steps on a trap.
            let mut p = spawn.pos;
            for dir in &spawn.route {
                p = p.step(*dir);
                assert!(!with.traps.contains_key(&p), "seed {seed}: a trap at {p}");
            }
            assert_eq!(p, spawn.pos);
            assert!(!spawn.rolls_gear);
        }
        // The population rolls its gear too, from the level's gear stream after the generator's
        // creatures (whose gear is as it was, above); without gear tables, it is the same
        // population, bare.
        let bare = |t: &LevelTemplate| -> Vec<_> {
            (t.creatures.iter())
                .map(|s| (s.creature, s.pos, s.route.clone()))
                .collect()
        };
        assert_eq!(bare(&barrow_floor(&ungeared, seed)), bare(&with));
        assert!(
            population.iter().any(|s| !s.carry.is_empty()),
            "seed {seed}: no kobold armed"
        );
        // The same seed lays down the same population.
        assert_eq!(barrow_floor(&content, seed).creatures, with.creatures);
    }
}

/// A unique type comes once a run: once on a floor, never on a floor made after it was met.
#[test]
fn a_unique_comes_once_in_a_population_too() {
    let content = content();
    let chief = |t: &LevelTemplate| {
        (t.creatures.iter())
            .filter(|s| content.creature(s.creature).unique)
            .count()
    };
    let lair = crate::mapgen::generate(&content, ("showcase", 3), "lair:1", None).expect("lair");
    assert_eq!(chief(&lair), 1);
    let met = BTreeSet::from(["chief".to_string()]);
    let visit = crate::mapgen::Visit {
        uniques: Some(&met),
        ..Default::default()
    };
    let lair = crate::mapgen::generate_for(&content, ("showcase", 3), "lair:1", None, &visit)
        .expect("lair");
    assert_eq!(chief(&lair), 0);
    assert!(lair.creatures.len() >= 30);
    // In play, the chief laid down on a floor is met: the spawns never bring him again.
    let mut w = haunt(3);
    w.travel("lair:1", None).expect("to the lair");
    assert!(w.state.progress.uniques.contains("chief"));
}

/// What comes in play rolls its type's gear from its own stream, `gear/<actor id>`.
#[test]
fn a_spawn_rolls_its_gear_from_its_own_stream() {
    let mut armed = 0;
    for seed in 0..8 {
        let mut w = haunt(seed);
        let Some(id) = w.attempt(true) else {
            continue;
        };
        let actor = &w.state.actors[&id];
        let mut rng = crate::rng::Rng::gear(w.state.seed, &id.0.to_string());
        let pack = crate::gear::roll(&w.content, &mut rng, actor.creature, 1);
        assert_eq!(
            (&actor.inventory, &actor.equipment),
            (&pack.items, &pack.equipment),
            "seed {seed}"
        );
        armed += usize::from(!actor.inventory.is_empty());
    }
    assert!(armed > 0, "no spawn carried anything");
}
