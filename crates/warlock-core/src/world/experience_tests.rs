//! Experience in the world (spec §8 "Experience"): only the player's own kills are worth
//! anything, levels follow from the total, and each level up to the cap grows the hit points.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::stats::Stat;

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

// The pup dies of a scratch; the elder is a higher-level monster, the hunter a strong killer.
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
xp = 10

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
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
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
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "xp", None).expect("world")
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

#[test]
fn melee_kills_are_worth_their_xp_and_levels_come_with_messages() {
    let mut w = world_on("########\n#@pppp.#\n########\n");
    let (hp0, max0) = {
        let p = w.state().player();
        (p.hp, p.max_hp(w.content()))
    };
    assert_eq!((w.player_level(), w.player_xp()), (1, 0));
    // Each pup is level 1 and worth 10: two of them make level 2.
    let mut kills = 0;
    for _ in 0..200 {
        let before = pups_left(&w);
        run(&mut w, "move:E");
        kills += before - pups_left(&w);
        if kills == 2 {
            break;
        }
    }
    assert_eq!((w.player_level(), w.player_xp()), (2, 20));
    let log = said(&w);
    let i = log.iter().rposition(|m| m == "You feel more experienced.");
    let i = i.expect("level up said");
    assert_eq!(log[i + 1], "Welcome to level 2.");
    assert_eq!(log[i - 1], "The pup dies.", "after the death: {log:?}");
    let p = w.state().player();
    assert_eq!(p.max_hp(w.content()), max0 + per_level(&w));
    assert_eq!(p.hp, hp0 + per_level(&w), "the current hit points rose too");
    // A level 1 pup is below a level 2 player: four fifths of its XP.
    for _ in 0..200 {
        if pups_left(&w) == 0 {
            break;
        }
        run(&mut w, "move:E");
    }
    assert_eq!(w.player_xp(), 20 + 8 + 8);
    assert_eq!(w.perception().own_experience().next, Some(60));
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
    assert_eq!(w.player_xp(), 10);
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
}

#[test]
fn a_higher_level_monster_is_worth_more() {
    let mut w = world_on("#####\n#@..#\n#####\n");
    let elder = w.content().creature_id("elder").expect("elder");
    w.earn_kill(elder);
    assert_eq!(w.player_xp(), 60, "50 and 4 levels above: +20%");
}

#[test]
fn several_levels_come_at_once_and_hit_points_grow_up_to_the_cap() {
    let mut w = world_on("#####\n#@..#\n#####\n");
    let (hp0, max0) = {
        let p = w.state().player();
        (p.hp, p.max_hp(w.content()))
    };
    let step = per_level(&w);
    w.grant_xp(120);
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

    // To level 19, and over the last growing level: level 20 grows, level 21 does not.
    let rules = w.content().experience().clone();
    w.state.experience = rules.threshold(19);
    let max19 = w.state().player().max_hp(w.content());
    w.grant_xp(rules.threshold(20) - rules.threshold(19));
    assert_eq!(w.player_level(), 20);
    assert_eq!(w.state().player().max_hp(w.content()), max19 + step);
    let max20 = w.state().player().max_hp(w.content());
    w.grant_xp(rules.threshold(21) - rules.threshold(20));
    assert_eq!(w.player_level(), 21);
    assert_eq!(
        w.state().player().max_hp(w.content()),
        max20,
        "no more hit points past level 20"
    );
    // The top: 50, and no further.
    w.grant_xp(1_000_000);
    assert_eq!(w.player_level(), 50);
    assert_eq!(w.perception().own_experience().next, None);
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
