//! Experience in the world (spec §8 "Experience"): only the player's own kills are worth
//! anything, levels follow from the total and the character's factor, each level up to the cap
//! grows the hit points, and the first kill of a type and the first arrival on a level are worth
//! a little more.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, StairsDir};
use crate::replay::Replay;
use crate::stats::Stat;
use std::sync::LazyLock;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "pests"
name = "pests"
hostile = ["player"]

[[faction]]
id = "hunters"
name = "hunters"
hostile = ["pests"]
"##;

// The pup dies of a scratch (two of them, the first worth twice, make level 2); the elder is a
// higher-level monster, the hunter a strong killer.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "pup"
name = "pup"
glyph = "p"
fg = "#806040"
weight = 5000
faction = "pests"
stats = { toughness = 1, dexterity = 1 }
level = 1
xp = 300

[[creature]]
id = "elder"
name = "elder"
glyph = "e"
fg = "#806040"
weight = 5000
faction = "pests"
stats = { toughness = 1, dexterity = 1 }
level = 5
xp = 50

[[creature]]
id = "hunter"
name = "hunter"
glyph = "h"
fg = "#60a040"
weight = 20000
faction = "hunters"
stats = { strength = 30, dexterity = 20 }
attack = { damage = "1d10", name = "claw", verb = "claws" }
"##;

const VISION: &str = "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n";

const OBJECTS: &str = r##"
[[object]]
id = "bow"
name = "short bow"
glyph = "}"
fg = "#c0c0c0"
class = "missile_weapons"
materials = ["wood"]
weight = 700
value = 0
slot = "missile_weapon"
launches = "arrow"
damage = "1d8"
strength = 10
range = 8

[[object]]
id = "arrow"
name = "arrow"
glyph = "/"
fg = "#c0c0c0"
class = "missiles"
materials = ["wood"]
weight = 30
value = 0
stackable = true
slot = "missiles"
ammo = "arrow"
"##;

const SLOTS: &str = r##"
[[slot]]
id = "missile_weapon"
name = "Missile weapon"
action = "ready"
mark = "readied"

[[slot]]
id = "missiles"
name = "Missiles"
action = "ready"
mark = "quivered"
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"p" = { terrain = "floor", creature = "pup" }
"e" = { terrain = "floor", creature = "elder" }
"h" = { terrain = "floor", creature = "hunter" }
"@" = { terrain = "floor", spawn = "player", objects = ["bow", "arrow*30"] }
"##;

/// The experience rules as shipped.
const EXPERIENCE: &str = include_str!("../../../../assets/data/experience.toml");

/// A world on `map`, fully lit, with the scenario's `hp` (the stand-in for class hit points) 8.
fn world_on(map: &str) -> World {
    World::new(
        content_with(map, CREATURES).expect("test content is valid"),
        "xp",
        None,
    )
    .expect("world")
}

/// This file's content with `creatures` for its creatures, on `map`.
fn content_with(map: &str, creatures: &str) -> Result<Arc<Content>, crate::content::ContentError> {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", creatures),
        ("data/vision.toml", VISION),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/experience.toml", EXPERIENCE),
        ("data/legend.toml", LEGEND),
        (
            "data/materials.toml",
            "[[material]]\nid = \"wood\"\nname = \"wood\"\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/xp.toml",
        &format!("name = \"Xp\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/xp.toml",
        "name = \"Xp\"\nstart_level = \"xp\"\nhp = 8\nstats = { strength = 30 }\n",
    );
    Content::load(&files).map(Arc::new)
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn pups_left(world: &World) -> usize {
    let pup = world.content().creature_id("pup").expect("pup");
    (world.state().actors.values())
        .filter(|a| a.creature == pup)
        .count()
}

/// What one level adds for this world's player: half the scenario's 8, plus the Toughness modifier.
fn per_level(world: &World) -> i32 {
    (4 + world.state().player().stats.modifier(Stat::Toughness)).max(1)
}

/// Kills pups to the east until `kills` more have died.
fn kill_pups(world: &mut World, kills: usize) {
    let target = pups_left(world) - kills;
    for _ in 0..200 {
        if pups_left(world) == target {
            return;
        }
        run(world, "move:E");
    }
    panic!("the pups did not die");
}

#[test]
fn melee_kills_are_worth_their_xp_and_levels_come_with_messages() {
    let mut w = world_on("########\n#@pppp.#\n########\n");
    let (hp0, max0) = {
        let p = w.state().player();
        (p.hp, p.max_hp(w.content()))
    };
    assert_eq!((w.player_level(), w.player_xp()), (1, 0));
    assert_eq!(w.perception().own_experience().next, Some(192));
    // Each pup is level 1 and worth 300, the first one twice (a first kill): 600, past the 192
    // of level 2.
    kill_pups(&mut w, 1);
    assert_eq!((w.player_level(), w.player_xp()), (2, 600));
    let log = said(&w);
    let i = log.iter().rposition(|m| m == "You feel more experienced.");
    let i = i.expect("level up said");
    assert_eq!(log[i + 1], "Welcome to level 2.");
    let dies = log
        .iter()
        .rposition(|m| m == "The pup dies.")
        .expect("dies");
    let learn = log
        .iter()
        .rposition(|m| m == "You learn something of pups.");
    assert!(dies < i, "after the death: {log:?}");
    assert_eq!(learn, Some(dies + 1), "{log:?}");
    let p = w.state().player();
    assert_eq!(p.max_hp(w.content()), max0 + per_level(&w));
    assert_eq!(p.hp, hp0 + per_level(&w), "the current hit points rose too");
    // A level 1 pup is below a level 2 player: four fifths of its XP, 840 in all, past the 691
    // of level 3.
    kill_pups(&mut w, 1);
    assert_eq!((w.player_level(), w.player_xp()), (3, 840));
    let log = said(&w);
    assert!(log.iter().any(|m| m == "Welcome to level 3."), "{log:?}");
    let learned = log.iter().filter(|m| m.starts_with("You learn")).count();
    assert_eq!(learned, 1, "the first kill only: {log:?}");
    // Two levels below a level 3 player: three fifths.
    kill_pups(&mut w, 2);
    assert_eq!(w.player_xp(), 840 + 180 + 180);
    assert_eq!(w.perception().own_experience().next, Some(1_619));
}

#[test]
fn shots_are_worth_xp_too() {
    let mut w = world_on("##########\n#@.....p.#\n##########\n");
    run(&mut w, "pickup equip:missile_weapon:a equip:missiles:b");
    for _ in 0..30 {
        if pups_left(&w) == 0 {
            break;
        }
        w.apply(&Action::Fire(Pos::new(7, 1)));
    }
    assert_eq!(pups_left(&w), 0, "the pup was shot");
    assert_eq!(
        w.player_xp(),
        600,
        "its XP, and once more for the first of its kind"
    );
}

#[test]
fn kills_by_others_are_worth_nothing() {
    // A hunter kills a pup in a room the player cannot see into.
    let mut w = world_on("#######\n#@...##\n#######\n#######\n#hp...#\n#######\n");
    for _ in 0..200 {
        if pups_left(&w) == 0 {
            break;
        }
        w.apply(&Action::Wait);
    }
    assert_eq!(pups_left(&w), 0, "the hunter killed the pup");
    assert_eq!((w.player_level(), w.player_xp()), (1, 0));
    assert!(
        w.state.progress.is_empty(),
        "nothing learned of a kill watched"
    );
}

#[test]
fn a_hand_set_xp_percent_scales_the_kill_and_the_first_kill_bonus() {
    // The elder, worth 50, four levels above the player: +20 %, and as much again the first
    // time; with xp_percent its 50 becomes 50 × percent first.
    let gained = |percent: &str| {
        let creatures = CREATURES.replace("xp = 50", &format!("xp = 50\nxp_percent = {percent}"));
        let content = content_with("#####\n#@..#\n#####\n", &creatures).expect("valid");
        let mut w = World::new(content, "xp", None).expect("world");
        let elder = w.content().creature_id("elder").expect("elder");
        assert_eq!(w.content().creature(elder).xp, 50, "the base stays");
        w.earn_kill(elder);
        w.player_xp()
    };
    assert_eq!(gained("100"), 60 + 50, "as without it");
    assert_eq!(gained("150"), 90 + 75, "75: +20 %, and 75 for the first");
    assert_eq!(gained("1000"), 600 + 500, "no ceiling");
    assert_eq!(gained("0"), 0, "worth nothing, and nothing learned");
    // A negative percent is refused at load.
    let creatures = CREATURES.replace("xp = 50", "xp = 50\nxp_percent = -10");
    assert!(content_with("#####\n#@..#\n#####\n", &creatures).is_err());
}

#[test]
fn a_higher_level_monster_is_worth_more() {
    let mut w = world_on("#####\n#@..#\n#####\n");
    let elder = w.content().creature_id("elder").expect("elder");
    w.earn_kill(elder);
    assert_eq!(
        w.player_xp(),
        60 + 50,
        "50 and 4 levels above: +20%, and 50 for the first"
    );
    w.earn_kill(elder);
    assert_eq!(w.player_xp(), 110 + 60, "the second: no more to learn");
}

#[test]
fn a_first_kill_pays_its_full_xp_whatever_the_levels() {
    let mut w = world_on("#####\n#@..#\n#####\n");
    let pup = w.content().creature_id("pup").expect("pup");
    let rules = w.content().experience().clone();
    // At level 10 a level 1 pup is grey: the kill gives nothing, the first kill still 300.
    w.state.experience = rules.threshold(10);
    w.earn_kill(pup);
    assert_eq!(w.player_xp(), rules.threshold(10) + 300);
    w.earn_kill(pup);
    assert_eq!(w.player_xp(), rules.threshold(10) + 300, "grey, and known");
    w.say_news();
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("You learn something of pups.")
    );
    let kills: Vec<&str> = (w.state.progress.discovered.kills.iter())
        .map(String::as_str)
        .collect();
    assert_eq!(kills, ["pup"]);
}

#[test]
fn several_levels_come_at_once_and_hit_points_grow_up_to_the_cap() {
    let mut w = world_on("#####\n#@..#\n#####\n");
    let (hp0, max0) = {
        let p = w.state().player();
        (p.hp, p.max_hp(w.content()))
    };
    let step = per_level(&w);
    let rules = w.content().experience().clone();
    w.grant_xp(rules.threshold(4));
    assert_eq!(w.player_level(), 4, "levels 2, 3, and 4 at once");
    let p = w.state().player();
    assert_eq!(
        (p.max_hp(w.content()), p.hp),
        (max0 + 3 * step, hp0 + 3 * step)
    );
    w.say_news();
    let log = said(&w);
    assert_eq!(
        &log[log.len() - 2..],
        ["You feel more experienced.", "Welcome to level 4."]
    );

    // To level 49, and over the last growing level: level 50 grows, level 51 does not.
    w.state.experience = rules.threshold(49);
    let max49 = w.state().player().max_hp(w.content());
    w.grant_xp(rules.threshold(50) - rules.threshold(49));
    assert_eq!(w.player_level(), 50);
    assert_eq!(w.state().player().max_hp(w.content()), max49 + step);
    let max50 = w.state().player().max_hp(w.content());
    w.grant_xp(rules.threshold(51) - rules.threshold(50));
    assert_eq!(w.player_level(), 51);
    assert_eq!(
        w.state().player().max_hp(w.content()),
        max50,
        "no more hit points past level 50"
    );
    // The top: 60, and no further.
    w.grant_xp(u32::MAX);
    assert_eq!(w.player_level(), 60);
    assert_eq!(w.perception().own_experience().next, None);
}

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// The showcase as `character`.
fn showcase_as(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

#[test]
fn race_and_class_factors_scale_the_thresholds() {
    // An elf wizard needs 175 % of the XP (140 % × 125 %), a half-orc commoner 72 % (90 % ×
    // 80 %): level 2 at 336 and at 138.
    let mut wizard = showcase_as("elf,male,wizard");
    let mut commoner = showcase_as("half_orc,female,commoner");
    assert_eq!(wizard.perception().own_experience().next, Some(336));
    assert_eq!(commoner.perception().own_experience().next, Some(138));
    for w in [&mut wizard, &mut commoner] {
        w.grant_xp(300);
    }
    assert_eq!(wizard.player_level(), 1);
    assert_eq!(commoner.player_level(), 2);
    let next = |w: &World| w.perception().own_experience().next;
    assert_eq!(next(&commoner), Some(498), "691 × 72 %");
    // The showcase's human thief: 95 %.
    assert_eq!(
        next(&World::new(BUILTIN.clone(), "showcase", None).expect("world")),
        Some(182)
    );
}

/// Puts the player on the current level's first stairs going `dir`, then takes them.
fn take_stairs(world: &mut World, dir: StairsDir) {
    let at = (world.state.current().stairs.iter())
        .find(|s| s.dir == dir)
        .map(|s| s.pos)
        .expect("stairs");
    world
        .state
        .actors
        .get_mut(&world.state.player)
        .expect("player")
        .pos = at;
    world.apply(&match dir {
        StairsDir::Down => Action::Descend,
        StairsDir::Up => Action::Ascend,
    });
}

#[test]
fn the_first_arrival_on_a_level_is_worth_its_item_level() {
    // The Depths start on their first level, which gives nothing; the second is worth 5 × 2.
    let mut w = World::new(BUILTIN.clone(), "depths", Some(3)).expect("world");
    take_stairs(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, "depths:2");
    assert_eq!(w.player_xp(), 10);
    let log = said(&w);
    let i = (log.iter())
        .position(|m| m == "You gain 10 experience for finding your way here.")
        .expect("said");
    assert!(log[i - 1].starts_with("You descend the stairs"), "{log:?}");
    // Back up, and down again: both levels are known.
    take_stairs(&mut w, StairsDir::Up);
    take_stairs(&mut w, StairsDir::Down);
    assert_eq!(w.state.current_level, "depths:2");
    assert_eq!(w.player_xp(), 10, "known levels give nothing");
    assert!(w.state.progress.is_empty(), "arrivals need no memory");
}

#[test]
fn the_overworld_and_its_local_areas_give_nothing() {
    // A local area of the wildlands is never worth anything, even the first time.
    let mut w = World::new(BUILTIN.clone(), "world", None).expect("world");
    w.apply(&Action::Descend);
    assert!(
        w.state.current_level.starts_with("local:"),
        "{}",
        w.state.current_level
    );
    assert_eq!(w.player_xp(), 0);
    // Beside the Hall: the Hub below it is a hand-made level, worth 5 × 1.
    let mut w = World::new(BUILTIN.clone(), "world", None).expect("world");
    let hall = (w.state.current().stairs.iter())
        .find(|s| s.to.as_deref() == Some("hub"))
        .map(|s| s.pos)
        .expect("the Hall");
    w.state.actors.get_mut(&w.state.player).expect("player").pos = hall;
    w.apply(&Action::Descend);
    assert_eq!(w.state.current_level, "hub");
    assert_eq!(w.player_xp(), 5);
    // Back out to the overworld: known.
    let out = (w.state.current().stairs.iter())
        .find(|s| s.to.as_deref() == Some("overworld"))
        .map(|s| s.pos)
        .expect("the way out");
    w.state.actors.get_mut(&w.state.player).expect("player").pos = out;
    w.apply(&Action::Ascend);
    assert_eq!(w.state.current_level, "overworld");
    assert_eq!(w.player_xp(), 5);
}

#[test]
fn experience_survives_a_save_and_replays_exactly() {
    let play = || {
        let mut w = world_on("########\n#@pppp.#\n########\n");
        let mut replay = Replay::new(&w);
        for _ in 0..40 {
            w.apply(&Action::Move(Dir::E));
            replay.push(&Action::Move(Dir::E), &w);
        }
        (w, replay)
    };
    let (a, replay) = play();
    assert!(a.player_xp() > 0, "something died");
    assert!(!a.state.progress.is_empty(), "the first kill is remembered");
    let content = a.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), a.state_hash());
    let bytes = crate::save::encode(&a);
    let loaded = crate::save::decode(&bytes, content).expect("loads");
    assert_eq!(loaded.state(), a.state());
    assert_eq!(loaded.player_xp(), a.player_xp());
    assert_eq!(loaded.player_level(), a.player_level());
}
