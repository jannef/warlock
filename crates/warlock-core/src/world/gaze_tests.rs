//! Gazes (`gaze.rs`) and averted eyes (`avert.rs`): a gaze takes only where the eyes meet, both
//! ways round, never through a wall or from the dark, on its cooldown, at its chance; a gaze
//! marked `averted` makes the player avert their eyes while they see its gazer, derived from the
//! snapshot and out of the hash: −4 to hit, Dodge halved, no free blows, and a glance's chance.

use super::*;
use crate::action::{Event, parse_actions};
use crate::combat::{Category, Defence};
use crate::content::ContentFiles;
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

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a08040"
walkable = false
transparent = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "dead"
name = "the dead"
hostile = ["player", "wild"]

[[faction]]
id = "wild"
name = "wildlife"
"##;

const STATUS: &str = r##"
[[status]]
id = "gaze"
name = "Gaze spent"
max = 100
stacking = "refresh"
eases = 6

[[status]]
id = "transfixed"
name = "Transfixed"
seen = "transfixed"
max = 4
stacking = "refresh"
eases = 12
start = "You are transfixed."
end = "You shake off the stare."
"##;

/// `{gaze}` is the starer's `[creature.gaze]` body, `{eyes}` the basilisk's.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "starer"
name = "vampire"
glyph = "V"
fg = "#a01060"
weight = 70000
faction = "dead"
vision = "darkvision"
base_hp = 1000
level = 10
[creature.gaze]
{gaze}
[creature.attack]
damage = "1"
name = "bite"
verb = "bites"

[[creature]]
id = "basilisk"
name = "basilisk"
glyph = "B"
fg = "#40a040"
weight = 70000
faction = "dead"
base_hp = 1000
level = 10
[creature.gaze]
{eyes}
[creature.attack]
damage = "1"
name = "bite"
verb = "bites"

[[creature]]
id = "deer"
name = "deer"
glyph = "d"
fg = "#a08040"
weight = 50000
faction = "wild"
base_hp = 1000

[[creature]]
id = "golem"
name = "golem"
glyph = "g"
fg = "#a08040"
weight = 50000
faction = "wild"
base_hp = 1000
vitals = false
"##;

const ACTIONS: &str = "melee = 1000\ncast = 1000\nwait = 1000\nstep = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"@" = { terrain = "floor", spawn = "player" }
"V" = { terrain = "floor", creature = "starer" }
"B" = { terrain = "floor", creature = "basilisk" }
"d" = { terrain = "floor", creature = "deer" }
"g" = { terrain = "floor", creature = "golem" }
"##;

/// The starer's gaze: sure, a degree of Transfixed, within 4.
const STARE: &str = "range = 4\nriders = [{ status = \"transfixed\", chance = 100, severity = 2 }]";
/// The basilisk's: sure but averted, a glance its whole chance.
const AVERTED: &str = "range = 4\naverted = true\nglance = 100\n\
riders = [{ status = \"transfixed\", chance = 100, severity = 1 }]";

/// A lit hall: the player at the west end, the vampire three cells east.
const HALL: &str = "#########\n#@..V...#\n#.......#\n#########\n";

fn files(gaze: &str, eyes: &str, map: &str, ambient: u8) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    let creatures = CREATURES.replace("{gaze}", gaze).replace("{eyes}", eyes);
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/status.toml", STATUS),
        ("data/creatures.toml", creatures.as_str()),
        ("data/actions.toml", ACTIONS),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "data/vision.toml",
        "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n\
         [[vision]]\nid = \"darkvision\"\nname = \"Darkvision\"\nthreshold = 40\ndarkvision = 8\n",
    );
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    files
}

fn load(gaze: &str, eyes: &str, map: &str, ambient: u8) -> Result<World, String> {
    let content = Content::load(&files(gaze, eyes, map, ambient)).map_err(|e| e.to_string())?;
    World::new(Arc::new(content), "hall", None).map_err(|e| format!("{e:?}"))
}

fn world_with(gaze: &str, map: &str) -> World {
    load(gaze, AVERTED, map, 255).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Event> {
    let mut events = Vec::new();
    for action in parse_actions(actions).expect("actions") {
        events.extend(world.apply(&action).events);
    }
    events
}

fn first_of<'w>(w: &'w World, kind: &str) -> &'w Actor {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .expect("the creature")
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn degrees(w: &World, id: ActorId, status: &str) -> u16 {
    let status = w.content().status_id(status).expect("status");
    (w.state.actors[&id].effects.get(status)).map_or(0, |e| e.severity)
}

#[test]
fn a_gaze_takes_at_sight_both_ways_and_then_cools() {
    let mut w = world_with(STARE, HALL);
    let me = w.state.player;
    run(&mut w, "wait");
    let said = said(&w);
    assert!(
        said.iter().any(|l| l == "The vampire's eyes find yours."),
        "{said:?}"
    );
    assert!(said.iter().any(|l| l == "You are transfixed."), "{said:?}");
    assert!(degrees(&w, me, "transfixed") > 0);
    // 12 s of cooldown at 6 s a degree: two, a degree of which may have eased since.
    let vampire = first_of(&w, "starer").id;
    assert!((1..=2).contains(&degrees(&w, vampire, "gaze")));
    // While it cools it closes in: no second gaze before the cooldown is off.
    let far = first_of(&w, "starer").pos.chebyshev(w.player_pos());
    let before = w.messages().len();
    run(&mut w, "wait");
    assert!(first_of(&w, "starer").pos.chebyshev(w.player_pos()) < far);
    let again = w.messages().iter().skip(before);
    assert!(
        !again
            .map(|m| m.display())
            .any(|l| l.contains("eyes find yours")),
        "the gaze is spent"
    );
}

#[test]
fn out_of_its_range_it_comes_closer_before_it_gazes() {
    let far = "###########\n#@......V.#\n###########\n";
    let mut w = world_with(STARE, far);
    let me = w.state.player;
    run(&mut w, "wait");
    assert_eq!(degrees(&w, me, "transfixed"), 0, "seven cells off");
    run(&mut w, "wait*3");
    assert!(degrees(&w, me, "transfixed") > 0, "within four");
}

#[test]
fn never_through_a_wall_or_a_door() {
    let door = "###########\n#@.+..V...#\n###########\n";
    let mut w = world_with(STARE, door);
    let me = w.state.player;
    run(&mut w, "wait*6");
    assert_eq!(degrees(&w, me, "transfixed"), 0);
    let vampire = first_of(&w, "starer").id;
    assert_eq!(degrees(&w, vampire, "gaze"), 0, "it never gazed");
    assert!(!said(&w).iter().any(|l| l == gaze::UNSEEN));
}

#[test]
fn a_gazer_in_the_dark_is_felt_and_cannot_catch_the_eye() {
    // An unlit hall: the vampire's darkvision finds the player; the player does not see it.
    let mut w = load(STARE, AVERTED, HALL, 0).expect("world");
    let me = w.state.player;
    run(&mut w, "wait");
    assert!(said(&w).iter().any(|l| l == gaze::UNSEEN), "{:?}", said(&w));
    assert_eq!(degrees(&w, me, "transfixed"), 0, "the eyes never met");
    let vampire = first_of(&w, "starer").id;
    assert!(degrees(&w, vampire, "gaze") > 0, "the stare was spent");
}

#[test]
fn a_gaze_at_a_creature_needs_its_eyes_too() {
    // The deer (wildlife, hostile to the dead) is beside the vampire; the player is far off,
    // out of its range.
    let map = "##############\n#@.......Vd..#\n##############\n";
    let mut w = world_with(STARE, map);
    run(&mut w, "wait");
    let deer = first_of(&w, "deer").id;
    assert!(degrees(&w, deer, "transfixed") > 0);
    assert!(
        said(&w)
            .iter()
            .any(|l| l == "The vampire stares at the deer."),
        "{:?}",
        said(&w)
    );
    // No eyes to meet: a golem (no vitals) is never gazed at.
    let map = "##############\n#@.......Vg..#\n##############\n";
    let mut w = world_with(STARE, map);
    run(&mut w, "wait");
    let golem = first_of(&w, "golem").id;
    assert_eq!(degrees(&w, golem, "transfixed"), 0);
    let vampire = first_of(&w, "starer").id;
    assert_eq!(degrees(&w, vampire, "gaze"), 0);
}

#[test]
fn a_drain_rider_meets_its_check() {
    // A will drain no one can resist (difficulty 40).
    let drain = "range = 4\nriders = [{ drain = \"willpower\", points = 2, chance = 100, \
                 difficulty = 40 }]";
    let mut w = world_with(drain, HALL);
    run(&mut w, "wait");
    assert_eq!(w.state.player().drained[Stat::Willpower], 2);
    // A gaze that rarely catches the eye: its riders fall only when it does.
    let rare = "range = 4\nchance = 1\nriders = [{ status = \"transfixed\", chance = 100, \
                severity = 1 }]";
    let mut w = world_with(rare, HALL);
    run(&mut w, "wait");
    let said = said(&w);
    assert!(
        said.iter()
            .any(|l| l == "You tear your eyes from the vampire's."),
        "{said:?}"
    );
    assert_eq!(degrees(&w, w.state.player, "transfixed"), 0);
}

#[test]
fn an_averted_gaze_makes_the_player_avert_their_eyes_while_they_see_it() {
    let map = "#########\n#@..B...#\n#.......#\n#########\n";
    let mut w = world_with(STARE, map);
    let me = w.state.player;
    assert!(w.perception().averted(), "seen from the start");
    // The fight half-blind: −4 to hit, Dodge halved, no free blows.
    let profile = w.state.player().melee(w.content());
    assert_eq!(
        w.averted_blow(me, profile.clone()).to_hit,
        profile.to_hit + w.content().tuning().averted.to_hit
    );
    let basilisk = first_of(&w, "basilisk").id;
    assert_eq!(
        w.averted_blow(basilisk, profile.clone()).to_hit,
        profile.to_hit,
        "only the player averts"
    );
    let defence = Defence::from_bonuses([(Category::Dodge, 5)]);
    assert_eq!(w.averted_defence(me, defence).bonus(Category::Dodge), 2);
    let mut blows = w.state.player().clone();
    blows.free_blows.riposte = true;
    blows.free_blows.opportunist = true;
    assert!(w.free_blows_of(&blows).is_none());
    // Its glance (here its whole chance) still finds the eyes.
    run(&mut w, "wait");
    assert!(
        said(&w)
            .iter()
            .any(|l| l == "The basilisk's eyes find yours.")
    );
    assert!(w.perception().averted());
    // Derived from what the player sees, never saved: a world rebuilt from the state is
    // averted as well, with the same hash.
    let rebuilt = World::from_state(w.content.clone(), (*w.state).clone()).expect("rebuilt");
    assert!(rebuilt.perception().averted());
    assert_eq!(rebuilt.state_hash(), w.state_hash());
}

#[test]
fn averted_eyes_mostly_escape_and_another_cover_ends_the_averting() {
    // A gaze of 4 % at a quarter against averted eyes: 1 %.
    let eyes = "range = 4\nchance = 4\naverted = true\n\
                riders = [{ status = \"transfixed\", chance = 100, severity = 1 }]";
    let map = "#########\n#@..B...#\n#.......#\n#########\n";
    let mut w = load(STARE, eyes, map, 255).expect("world");
    run(&mut w, "wait");
    let said = said(&w);
    assert!(
        said.iter()
            .any(|l| l == "You keep your eyes from the basilisk's."),
        "{said:?}"
    );
    assert_eq!(degrees(&w, w.state.player, "transfixed"), 0);
    // In the dark the player does not see it: not averted, and its stare is only felt.
    let mut w = load(STARE, eyes, map, 0).expect("world");
    assert!(!w.perception().averted());
    run(&mut w, "wait");
    assert!(!w.perception().averted());
}

#[test]
fn a_gaze_is_checked_as_it_loads() {
    let refused = |gaze: &str, says: &str| {
        let Err(e) = load(gaze, AVERTED, HALL, 255) else {
            panic!("`{gaze}` loads");
        };
        assert!(e.contains(says), "{e}");
    };
    let rider = "riders = [{ status = \"transfixed\", chance = 100, severity = 1 }]";
    refused(
        &format!("range = 3\nglance = 25\n{rider}"),
        "`glance` goes with",
    );
    refused(
        "range = 3\nriders = [{ status = \"transfixed\", chance = 100 }]",
        "needs a `severity`",
    );
}
