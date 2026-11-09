//! Rooted creatures (#1294, `rooted.rs`): a rooted type never moves, by its own turn or anyone
//! else's doing, and still strikes and summons from where it stands.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::geom::Dir;

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
id = "wild"
name = "wild"
hostile = ["player"]

[[faction]]
id = "folk"
name = "folk"
"##;

const STATUS: &str = r##"
[[status]]
id = "summoning"
name = "Summoning spent"
max = 100
stacking = "refresh"
eases = 6
"##;

const DAMAGE: &str = r##"
[[type]]
id = "slashing"
name = "slashing"
verb = "slashes"
you = "slash"
physical = true
"##;

/// A rooted thorn that claws, a rooted gate that calls imps, a rooted shrub of the peaceful
/// folk, and a wolf that walks as any creature does.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "thorn"
name = "thorn"
glyph = "T"
fg = "#40a040"
weight = 70000
faction = "wild"
sight = 20
base_hp = 1000
rooted = true
[[creature.attack]]
damage = "3"
damage_type = "slashing"
name = "thorn"
verb = "pricks"
to_hit = 30

[[creature]]
id = "imp"
name = "imp"
glyph = "i"
fg = "#e04020"
weight = 5000
faction = "wild"
level = 1
speed = 100

[[creature]]
id = "gate"
name = "gate"
glyph = "G"
fg = "#c02020"
weight = 70000
faction = "wild"
tags = ["summoner"]
sight = 20
base_hp = 1000
rooted = true
[creature.summon]
creatures = ["imp"]
count = "1"
cooldown = 6
most = 2
verb = "lets through"

[[creature]]
id = "shrub"
name = "shrub"
glyph = "s"
fg = "#40a040"
weight = 70000
faction = "folk"
rooted = true

[[creature]]
id = "wolf"
name = "wolf"
glyph = "w"
fg = "#a0a0a0"
weight = 40000
faction = "wild"
sight = 20
base_hp = 1000
"##;

const ALLIES: &str = include_str!("../../../../assets/data/allies.toml");

const ACTIONS: &str = "melee = 1000\ncast = 1000\nwait = 1000\nstep = 1000\n";

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"T" = { terrain = "floor", creature = "thorn" }
"G" = { terrain = "floor", creature = "gate" }
"s" = { terrain = "floor", creature = "shrub" }
"w" = { terrain = "floor", creature = "wolf" }
"##;

fn world(map: &str) -> World {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/status.toml", STATUS),
        ("data/damage_types.toml", DAMAGE),
        ("data/creatures.toml", CREATURES),
        ("data/actions.toml", ACTIONS),
        ("data/legend.toml", LEGEND),
        ("data/allies.toml", ALLIES),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("valid content"));
    World::new(content, "hall", None).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

/// The first actor of creature type `kind`.
fn first_of(w: &World, kind: &str) -> ActorId {
    let id = w.content().creature_id(kind).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == id)
        .expect("one of the kind")
        .id
}

fn pos(w: &World, id: ActorId) -> Pos {
    w.state.actors[&id].pos
}

#[test]
fn a_rooted_hostile_never_steps_toward_the_player() {
    let mut w = world("##########\n#@.....T.#\n#........#\n##########\n");
    let thorn = first_of(&w, "thorn");
    assert!(w.content().creature(w.state.actors[&thorn].creature).rooted);
    let at = pos(&w, thorn);
    run(&mut w, "wait*20");
    assert_eq!(pos(&w, thorn), at, "it stayed where it stands");
}

#[test]
fn a_rooted_hostile_strikes_whoever_comes_beside_it() {
    let mut w = world("######\n#@T..#\n######\n");
    let hp = w.state.player().hp;
    run(&mut w, "wait*10");
    assert!(w.state.player().hp < hp, "it struck from where it stands");
}

#[test]
fn a_rooted_gate_summons_to_its_cap_and_stays() {
    let mut w = world("############\n#@.......G.#\n#..........#\n#..........#\n############\n");
    let gate = first_of(&w, "gate");
    let at = pos(&w, gate);
    run(&mut w, "wait*30");
    let called = (w.state.actors.values())
        .filter(|a| a.summoner == Some(gate))
        .count();
    assert_eq!(called, 2, "called to its `most`");
    assert_eq!(pos(&w, gate), at, "the gate never moved");
}

#[test]
fn a_rooted_gate_with_no_attack_strikes_no_blow() {
    // No free cell to call into: beside the player, it would strike with bare hands if it could.
    let mut w = world("####\n#@G#\n####\n");
    let hp = w.state.player().hp;
    run(&mut w, "wait*20");
    assert_eq!(
        w.state.player().hp,
        hp,
        "a gate has no hands to strike with"
    );
}

#[test]
fn a_rooted_one_never_gives_way_to_the_player() {
    let mut w = world("######\n#@s..#\n######\n");
    let shrub = first_of(&w, "shrub");
    assert!(w.refusal(shrub).is_some(), "it never budges");
    run(&mut w, "move:E");
    assert_eq!(pos(&w, shrub), Pos::new(2, 1), "no swap");
    assert_eq!(w.player_pos(), Pos::new(1, 1));
}

#[test]
fn a_rooted_one_is_never_driven_or_teleported() {
    let mut w = world("#######\n#@.T..#\n#.....#\n#######\n");
    let thorn = first_of(&w, "thorn");
    let at = pos(&w, thorn);
    let mut events = Vec::new();
    assert_eq!(w.drive_back(thorn, Dir::E, &mut events), None);
    w.teleport_to(thorn, (at, Pos::new(5, 2)), &mut events);
    w.teleport(thorn, at, &mut events);
    assert_eq!(pos(&w, thorn), at);
    assert!(events.is_empty(), "nothing moved: {events:?}");
    assert!(w.rooted(thorn), "nothing drives it on");
}

#[test]
fn a_rooted_hostile_never_follows_on_stairs_but_a_wolf_does() {
    let mut w = world("######\n#Tw..#\n#@...#\n######\n");
    w.perceive(&mut Vec::new());
    let (thorn, wolf) = (first_of(&w, "thorn"), first_of(&w, "wolf"));
    let followers = w.stair_followers();
    assert!(followers.contains(&wolf), "{followers:?}");
    assert!(!followers.contains(&thorn), "{followers:?}");
}

#[test]
fn a_rooted_ally_stays_behind_when_the_player_leaves() {
    let mut w = world("##########\n#@.......#\n#........#\n##########\n");
    run(&mut w, "ally:shrub ally:wolf");
    let player = w.state.player;
    let of = |w: &World, kind: &str| {
        let id = w.content().creature_id(kind).expect("a type");
        (w.state.actors.values())
            .find(|a| a.creature == id && a.master() == Some(player))
            .expect("an ally")
            .id
    };
    let (shrub, wolf) = (of(&w, "shrub"), of(&w, "wolf"));
    let at = pos(&w, shrub);
    let (old, arrival) = (w.player_pos(), Pos::new(7, 2));
    w.follow_along("hall", "hall", old, arrival);
    assert_eq!(pos(&w, shrub), at, "the rooted ally stayed");
    assert!(pos(&w, wolf).chebyshev(arrival) <= 1, "the wolf came along");
}
