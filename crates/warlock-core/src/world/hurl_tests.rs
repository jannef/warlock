//! Creatures' throws (spec §8 "Ranged combat": "Creatures' throws", `hurl.rs`): a giant hurls
//! boulders from afar, fights in melee up close, closes in when out of range or blocked, and
//! nothing it throws lands.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::content::ContentFiles;
use crate::replay::Replay;

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
id = "giant"
name = "giants"
hostile = ["player", "kobolds"]

[[faction]]
id = "kobolds"
name = "kobolds"

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// The giant throws boulders up to 6 cells, a throw costing two actions; the deadeye never
// misses but on a fumble and kills what it hits. The kobold is frail and hardly moves.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "post"
name = "training post"
glyph = "P"
fg = "#806040"
weight = 30000
faction = "wildlife"
base_hp = 1000

[[creature]]
id = "giant"
name = "hill giant"
glyph = "H"
fg = "#a8865a"
weight = 500000
faction = "giant"
sight = 20
base_hp = 1000
[creature.throw]
object = "boulder"
damage = "1d4"
damage_type = "bludgeoning"
range = 6
cost = 2000
verb = "hurls"

[[creature]]
id = "deadeye"
name = "stone giant"
glyph = "H"
fg = "#9a9a92"
weight = 500000
faction = "giant"
sight = 20
base_hp = 1000
[creature.throw]
object = "boulder"
name = "block of ice"
damage = "10d10"
damage_type = ["bludgeoning", "cold"]
to_hit = 20
range = 6
verb = "hurls"

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#60a040"
weight = 20000
speed = 100
faction = "kobolds"
stats = { toughness = 1 }
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40
"##;

const OBJECTS: &str = r##"
[[object]]
id = "boulder"
name = "boulder"
glyph = "*"
fg = "#8e887c"
class = "miscellaneous"
materials = ["stone"]
weight = 40000
value = 0
"##;

const DAMAGE: &str = r##"
[[type]]
id = "bludgeoning"
name = "bludgeoning"
physical = true
verb = "strikes"
you = "strike"
unarmed = true

[[type]]
id = "cold"
name = "cold"
verb = "chills"
you = "chill"
"##;

const ACTIONS: &str = "melee = 1000\nthrow = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"P" = { terrain = "floor", creature = "post" }
"H" = { terrain = "floor", creature = "giant" }
"D" = { terrain = "floor", creature = "deadeye" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

/// The player stands on `@` of `map`, fully lit; they take a lot.
fn files(map: &str) -> ContentFiles {
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
        ("data/actions.toml", ACTIONS),
        ("data/damage_types.toml", DAMAGE),
        ("data/legend.toml", LEGEND),
        (
            "data/materials.toml",
            "[[material]]\nid = \"stone\"\nname = \"stone\"\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/range.toml",
        &format!("name = \"Range\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/range.toml",
        "name = \"Range\"\nstart_level = \"range\"\nhp = 1000\n",
    );
    files
}

fn world(map: &str) -> World {
    let content = Arc::new(Content::load(&files(map)).expect("test content is valid"));
    World::new(content, "range", None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// The flights of the outcomes' missiles, as the player saw them: their paths.
fn flights(outs: &[Outcome]) -> Vec<Vec<Pos>> {
    (outs.iter().flat_map(|o| o.events.iter()))
        .filter_map(|e| match e {
            Event::Missile { path, .. } => Some(path.clone()),
            _ => None,
        })
        .collect()
}

fn attacks(outs: &[Outcome]) -> Vec<AttackKind> {
    (outs.iter().flat_map(|o| o.events.iter()))
        .filter_map(|e| match e {
            Event::Attacked { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect()
}

fn objects_on_level(world: &World) -> usize {
    let (level, _) = world.ground_truth();
    (0..12)
        .flat_map(|x| (0..5).map(move |y| Pos::new(x, y)))
        .map(|p| level.objects_at(p).len())
        .sum()
}

#[test]
fn a_giant_hurls_boulders_from_afar_and_nothing_lands() {
    let mut w = world("##########\n#@.....H.#\n##########\n");
    let hp = w.state().player().hp;
    let outs = run(&mut w, "wait*6");
    let flown = flights(&outs);
    assert!(!flown.is_empty(), "the giant threw");
    // Along the row, from beside the giant to the player.
    assert_eq!(flown[0].first(), Some(&Pos::new(6, 1)));
    assert_eq!(flown[0].last(), Some(&Pos::new(1, 1)));
    assert!(attacks(&outs).iter().all(|k| *k == AttackKind::Thrown));
    // It stood its ground: never a step.
    let (_, mut actors) = w.ground_truth();
    assert!(actors.any(|a| a.pos == Pos::new(7, 1)));
    assert_eq!(objects_on_level(&w), 0, "nothing lands");
    let said = said(&w);
    assert!(said.iter().any(|m| m == "The hill giant hurls a boulder."));
    let blows = ["The boulder strikes you.", "The boulder strikes you hard!"];
    let hit = said.iter().any(|m| blows.contains(&m.as_str()));
    let missed = said.iter().any(|m| m == "You dodge the boulder.");
    assert!(hit || missed, "{said:?}");
    if hit {
        assert!(w.state().player().hp < hp);
    }
}

#[test]
fn a_throw_costs_its_own_and_half_as_many_fly() {
    let w = world("##########\n#@.....H.#\n##########\n");
    let content = w.content();
    let giant = content.creature_id("giant").expect("giant");
    let deadeye = content.creature_id("deadeye").expect("deadeye");
    assert_eq!(content.action_cost(giant, ActionKind::Throw), 2000);
    assert_eq!(content.action_cost(giant, ActionKind::Melee), 1000);
    assert_eq!(content.action_cost(deadeye, ActionKind::Throw), 1000);
    // Twelve player turns, six throws: one every two turns.
    let mut w = w;
    let outs = run(&mut w, "wait*12");
    assert_eq!(flights(&outs).len(), 6);
}

#[test]
fn up_close_it_fights_in_melee() {
    let mut w = world("##########\n#@H......#\n##########\n");
    let outs = run(&mut w, "wait*3");
    assert!(flights(&outs).is_empty());
    let kinds = attacks(&outs);
    assert!(!kinds.is_empty() && kinds.iter().all(|k| *k == AttackKind::Melee));
}

#[test]
fn out_of_range_or_with_someone_in_between_it_closes_in() {
    // Eight cells off, its throw reaches six: it steps in, then throws.
    let mut w = world("############\n#@.......H.#\n############\n");
    let outs = run(&mut w, "wait");
    assert!(flights(&outs).is_empty());
    let (_, mut actors) = w.ground_truth();
    assert!(actors.any(|a| a.pos == Pos::new(8, 1)));
    // A post in the way: no clear line, so it never throws (in a corridor it cannot go round).
    let mut w = world("##########\n#@P....H.#\n##########\n");
    let outs = run(&mut w, "wait*4");
    assert!(flights(&outs).is_empty());
}

#[test]
fn a_boulder_kills_and_the_player_who_sees_it_is_told() {
    let mut w = world("##########\n#@.k...D.#\n##########\n");
    let mut killed = false;
    for _ in 0..10 {
        run(&mut w, "wait");
        let (_, mut actors) = w.ground_truth();
        if !actors.any(|a| a.pos == Pos::new(3, 1)) {
            killed = true;
            break;
        }
    }
    assert!(killed, "the kobold was crushed");
    let said = said(&w);
    let at = said
        .iter()
        .position(|m| m == "The stone giant hurls a block of ice.");
    let at = at.expect("the throw is said");
    // Crushing or freezing, whichever hurts it more.
    let blow = &said[at + 1];
    assert!(
        blow.starts_with("The block of ice ") && blow.contains(" the kobold"),
        "{said:?}"
    );
    assert_eq!(said[at + 2], "The kobold dies.");
}

#[test]
fn throwing_fights_replay_exactly() {
    let files = files("##########\n#@.k...H.#\n##########\n");
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content.clone(), "range", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait*8 move:E wait*4").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn a_throw_needs_a_known_object_and_a_range_it_can_fly() {
    let bad = |from: &str, to: &str| {
        let mut files = files("#@#\n");
        files.insert("data/creatures.toml", &CREATURES.replacen(from, to, 1));
        Content::load(&files).expect_err("refused").to_string()
    };
    let err = bad("object = \"boulder\"", "object = \"pebble\"");
    assert!(err.contains("throw: unknown object `pebble`"), "{err}");
    let err = bad("range = 6", "range = 1");
    assert!(err.contains("throw: `range` must be 2 to 20"), "{err}");
}
