//! Trapped doors (spec §8 "Trapped doors"): a trap on a door goes off at whoever opens, unlocks,
//! or kicks the door, never at whoever walks through the doorway; it opens and shuts with the
//! door and goes when the door is kicked in; its blast and its bolt start at the door, its arrow
//! and its blade at the victim; creatures set it off as they open the door; searching finds it;
//! levels may not put a pit on a door nor a trap on a secret one.

use super::*;
use crate::action::{SpellShape, parse_actions};
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

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"
toggle_noise = 60
breaks_to = "door_broken"
sturdiness = 30

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"
toggle_noise = 70

[[terrain]]
id = "door_broken"
name = "broken door"
glyph = ","
fg = "#a06030"

# A door any kick breaks.
[[terrain]]
id = "flimsy_door"
name = "flimsy door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"
breaks_to = "door_broken"
sturdiness = 1

# A door no kick breaks.
[[terrain]]
id = "iron_door"
name = "iron door"
glyph = "+"
fg = "#a0a0a0"
walkable = false
transparent = false
opens_to = "door_open"
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// The dummy takes a lot and never fights; the porter goes its rounds and opens doors.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "dummy"
name = "training dummy"
glyph = "D"
fg = "#806040"
weight = 30000
faction = "wildlife"
base_hp = 1000

[[creature]]
id = "porter"
name = "porter"
glyph = "p"
fg = "#60a040"
weight = 30000
faction = "wildlife"
base_hp = 1000
opens_doors = true

[[creature]]
id = "moth"
name = "moth man"
glyph = "m"
fg = "#a0a080"
weight = 30000
faction = "wildlife"
base_hp = 1
opens_doors = true
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40
"##;

const OBJECTS: &str = r##"
[[object]]
id = "arrow"
name = "arrow"
glyph = "/"
fg = "#c0c0c0"
material = "wood"
weight = 30
value = 0
stackable = true

[[object]]
id = "iron_key"
name = "iron key"
glyph = "("
fg = "#b4b8bc"
material = "wood"
weight = 40
value = 0
key = "pantry"
"##;

const TRAPS: &str = r##"
[[trap]]
id = "arrow"
name = "arrow trap"
difficulty = 1
fg = "#c8b48c"
message = "An arrow shoots out of the wall!"
kind = "shot"
object = "arrow"
damage = "1d6"
to_hit = 20

[[trap]]
id = "blade"
name = "blade trap"
fg = "#b4b8c4"
message = "A blade springs out!"
kind = "strike"
noun = "blade"
damage = "1d8"
to_hit = 20

[[trap]]
id = "fireball"
name = "fireball trap"
fg = "#ff7a3a"
message = "A fireball explodes!"
charges = 1
kind = "burst"
spell = "fireball"
noun = "fireball"
verb = "burns"
radius = 2
damage = "3d6"

[[trap]]
id = "lightning"
name = "lightning trap"
fg = "#a8c8ff"
message = "Lightning arcs out!"
kind = "bolt"
spell = "lightning_bolt"
noun = "lightning bolt"
verb = "strikes"
length = 12
damage = "4d6"

[[trap]]
id = "pit"
name = "pit"
fg = "#9a7a58"
kind = "pit"
damage = "1d6"

[[trap]]
id = "alarm"
name = "alarm trap"
fg = "#e8d060"
message = "A loud bell rings!"
kind = "alarm"
noise = { sound = "bell", loudness = 300 }
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "door_closed", trap = "arrow" }
"b" = { terrain = "door_closed", trap = "blade" }
"o" = { terrain = "door_open", trap = "blade" }
"I" = { terrain = "iron_door", trap = "blade" }
"L" = { terrain = "door_closed", trap = "blade", lock = "pantry" }
"f" = { terrain = "flimsy_door", trap = "blade" }
"i" = { terrain = "iron_door", trap = "fireball" }
"l" = { terrain = "door_closed", trap = "lightning" }
"r" = { terrain = "door_closed", trap = "alarm" }
"R" = { terrain = "door_open", trap = "alarm" }
"F" = { terrain = "door_open", trap = "fireball" }
"D" = { terrain = "floor", creature = "dummy" }
"p" = { terrain = "floor", creature = "porter", route = "E4 W4" }
"m" = { terrain = "floor", creature = "moth", route = "E4 W4" }
"##;

fn files(map: &str, scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/vision.toml", VISION),
        ("data/objects.toml", OBJECTS),
        ("data/traps.toml", TRAPS),
        ("data/legend.toml", LEGEND),
        (
            "data/materials.toml",
            "[[material]]\nid = \"wood\"\nname = \"wood\"\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n{scenario}\n"),
    );
    files
}

fn content(map: &str, scenario: &str) -> Arc<Content> {
    Arc::new(Content::load(&files(map, scenario)).expect("test content is valid"))
}

fn world_with(map: &str, scenario: &str) -> World {
    World::new(content(map, scenario), "hall", None).expect("world")
}

fn world(map: &str) -> World {
    world_with(map, "")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// Who set off a trap at `at` in `out`, in order.
fn sprung_at(out: &[Outcome], at: Pos) -> Vec<ActorId> {
    (out.iter().flat_map(|o| &o.events))
        .filter_map(|e| match *e {
            Event::TrapSprung { at: p, victim, .. } if p == at => Some(victim),
            _ => None,
        })
        .collect()
}

fn terrain(world: &World, p: Pos) -> String {
    let (level, _) = world.ground_truth();
    world.content().terrain(level.terrain[p]).id.clone()
}

/// Hit points actor `id` has lost.
fn lost(world: &World, id: ActorId) -> i32 {
    let a = &world.state.actors[&id];
    a.max_hp(&world.content) - a.hp
}

fn creature_at(world: &World, p: Pos) -> ActorId {
    (world.state.actors.values())
        .find(|a| a.pos == p)
        .map(|a| a.id)
        .expect("someone stands there")
}

const DOOR: Pos = Pos::new(2, 1);

#[test]
fn opening_a_trapped_door_sets_it_off_at_the_opener_and_it_is_known() {
    let mut w = world("#####\n#@b.#\n#####\n");
    let player = w.state.player;
    let out = run(&mut w, "open:E");
    assert!(out[0].took_turn);
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
    assert_eq!(terrain(&w, DOOR), "door_open");
    let struck = (out[0].events.iter())
        .any(|e| matches!(e, Event::TrapStruck { target, .. } if *target == player));
    assert!(
        struck,
        "the blade strikes at the opener: {:?}",
        out[0].events
    );
    let blade = w.content().trap_id("blade").expect("trap");
    assert_eq!(w.perception().known_traps(), vec![(DOOR, blade)]);
    let said = texts(&w);
    assert!(said.contains(&"You open the door.".to_string()), "{said:?}");
    assert!(
        said.contains(&"A blade springs out!".to_string()),
        "{said:?}"
    );
}

#[test]
fn walking_through_the_doorway_never_sets_it_off_but_opening_it_again_does() {
    let mut w = world("######\n#@b..#\n######\n");
    let player = w.state.player;
    // Open, walk through and on, back into the doorway and out again: once.
    let out = run(&mut w, "open:E move:E move:E move:W move:W");
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
    // Shut, it is set again: opening it, or walking into it, sets it off again.
    let out = run(&mut w, "close:E move:E");
    assert_eq!(terrain(&w, DOOR), "door_open");
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
}

#[test]
fn a_trap_on_an_open_door_waits_until_the_door_is_shut_and_opened() {
    let mut w = world("#####\n#@o.#\n#####\n");
    let player = w.state.player;
    let out = run(&mut w, "move:E move:E move:W move:W");
    assert!(sprung_at(&out, DOOR).is_empty());
    let out = run(&mut w, "close:E open:E");
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
}

#[test]
fn unlocking_a_trapped_door_sets_it_off_and_a_locked_one_stays_shut_and_quiet() {
    let map = "#####\n#@L.#\n#####\n";
    let mut w = world(map);
    let out = run(&mut w, "move:E");
    assert!(!out[0].took_turn, "locked, no key");
    assert!(sprung_at(&out, DOOR).is_empty());
    let mut w = world_with(map, "carry = [\"iron_key\"]");
    let player = w.state.player;
    let out = run(&mut w, "move:E");
    assert_eq!(terrain(&w, DOOR), "door_open");
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
}

#[test]
fn kicking_a_trapped_door_sets_it_off_and_a_door_kicked_in_takes_its_trap_along() {
    // A door that holds: the trap goes off at every kick.
    let mut w = world("#####\n#@I.#\n#####\n");
    let player = w.state.player;
    let out = run(&mut w, "kick:E kick:E");
    assert_eq!(terrain(&w, DOOR), "iron_door", "no kick breaks it");
    assert_eq!(sprung_at(&out, DOOR), vec![player, player]);
    // A door any kick breaks: the trap goes off, then is gone with the door.
    let mut w = world("#####\n#@f.#\n#####\n");
    let out = run(&mut w, "kick:E");
    assert_eq!(terrain(&w, DOOR), "door_broken");
    assert_eq!(sprung_at(&out, DOOR), vec![player]);
    assert!(w.ground_truth().0.trap_at(DOOR).is_none());
    assert!(
        w.perception().known_traps().is_empty(),
        "gone before it was known"
    );
    let out = run(&mut w, "move:E move:E");
    assert_eq!(w.player_pos(), Pos::new(3, 1));
    assert!(sprung_at(&out, DOOR).is_empty());
}

#[test]
fn a_doors_fireball_bursts_out_on_both_sides_even_while_it_is_shut() {
    // An iron door no kick breaks: the fireball on it goes off with the door shut, and burns the
    // kicker and the dummy beyond alike.
    let mut w = world("#######\n#.@iD.#\n#######\n");
    let player = w.state.player;
    let dummy = creature_at(&w, Pos::new(4, 1));
    let door = Pos::new(3, 1);
    let out = run(&mut w, "kick:E");
    assert_eq!(terrain(&w, door), "iron_door");
    assert_eq!(sprung_at(&out, door), vec![player]);
    let cells = (out[0].events.iter()).find_map(|e| match e {
        Event::Spell { cells, .. } => Some(cells.clone()),
        _ => None,
    });
    // The player sees their side of it; the dummy beyond the door is burnt all the same.
    let cells = cells.expect("the burst is seen");
    assert_eq!(cells, vec![Pos::new(1, 1), Pos::new(2, 1), door]);
    assert!(lost(&w, player) > 0 && lost(&w, dummy) > 0);
    assert!(w.ground_truth().0.trap_at(door).is_none(), "spent");
}

#[test]
fn a_doors_lightning_runs_through_whoever_set_it_off() {
    // The longest clear line from the door runs south past the dummy, but the bolt runs north at
    // the player who opened it.
    let map = "#########\n#...@...#\n####l####\n#...D...#\n#.......#\n#########\n";
    let mut w = world(map);
    let player = w.state.player;
    let dummy = creature_at(&w, Pos::new(4, 3));
    let out = run(&mut w, "open:S");
    let path = (out[0].events.iter()).find_map(|e| match e {
        Event::Spell {
            shape: SpellShape::Bolt { path },
            ..
        } => Some(path.clone()),
        _ => None,
    });
    assert_eq!(path, Some(vec![Pos::new(4, 1)]));
    assert!(lost(&w, player) > 0);
    assert_eq!(lost(&w, dummy), 0);
}

#[test]
fn a_doors_arrow_shoots_from_the_wall_beside_it_at_the_opener() {
    let mut w = world("#####\n#...#\n##a##\n#.@.#\n#####\n");
    let out = run(&mut w, "open:N");
    let door = Pos::new(2, 2);
    let shot = (out[0].events.iter()).find_map(|e| match e {
        Event::Missile { from, path, .. } => Some((*from, path.clone())),
        _ => None,
    });
    let (from, path) = shot.expect("an arrow flies");
    let slit = from.expect("from a slit");
    assert_eq!(slit.y, door.y, "the wall beside the door: {slit}");
    assert_eq!(slit.chebyshev(door), 1);
    assert!(path.contains(&w.player_pos()), "{path:?}");
}

#[test]
fn creatures_set_it_off_as_they_open_the_door_and_never_as_they_walk_through() {
    // The porter's rounds lead east through an open door and back (a patrol never crosses a
    // closed door): it walks through, and nothing happens.
    let map = "#######\n#p.R..#\n#..@..#\n#######\n";
    let mut w = world(map);
    let porter = creature_at(&w, Pos::new(1, 1));
    let door = Pos::new(3, 1);
    let out = run(&mut w, "wait*9");
    assert_eq!(
        w.state.actors[&porter].pos,
        Pos::new(2, 1),
        "on its way back"
    );
    assert!(sprung_at(&out, door).is_empty());
    // The player shuts the door before it: it opens the door, which sets the trap off, and
    // then goes on through the open doorway, back and forth.
    let mut w = world(map);
    let out = run(&mut w, "close:N wait*16");
    assert_eq!(
        sprung_at(&out, door),
        vec![porter],
        "once, as it opened the door"
    );
    assert_eq!(terrain(&w, door), "door_open");
    let alarm = w.content().trap_id("alarm").expect("trap");
    assert_eq!(w.perception().known_traps(), vec![(door, alarm)]);
    assert!(texts(&w).contains(&"A loud bell rings!".to_string()));
}

#[test]
fn a_creature_the_door_kills_as_it_opens_it_dies_there() {
    // Shut on the moth man's rounds, the door's fireball goes off as it opens it, and kills it.
    let mut w = world("########\n#m..F..#\n#...@..#\n########\n");
    let moth = creature_at(&w, Pos::new(1, 1));
    let door = Pos::new(4, 1);
    let out = run(&mut w, "close:N wait*4");
    assert_eq!(sprung_at(&out, door), vec![moth]);
    assert!(!w.state.actors.contains_key(&moth));
    let died = (out.iter().flat_map(|o| &o.events))
        .any(|e| matches!(e, Event::Died { actor, .. } if *actor == moth));
    assert!(died);
    assert_eq!(terrain(&w, door), "door_open");
}

#[test]
fn searching_finds_a_doors_trap() {
    let mut w = world("#####\n#.@.#\n##a##\n#...#\n#####\n");
    let door = Pos::new(2, 2);
    run(&mut w, "search:20");
    let arrow = w.content().trap_id("arrow").expect("trap");
    assert_eq!(w.perception().known_traps(), vec![(door, arrow)]);
    let said = texts(&w);
    let found = "You notice that the door is trapped: an arrow trap.".to_string();
    assert!(said.contains(&found), "{said:?}");
}

/// The trap hall's closets: the west door shows its arrow trap; the east door stands open on a
/// kobold's rounds until the player shuts it, and the kobold rings its alarm opening it again;
/// the arrow goes off as the player opens the west door.
#[test]
fn the_trap_halls_closets_have_trapped_doors() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut w = World::new(content, "traps", None).expect("the trap hall");
    let player = w.state.player;
    let (west, east) = (Pos::new(2, 4), Pos::new(9, 4));
    let trap = |w: &World, id: &str| w.content().trap_id(id).expect("trap");
    assert_eq!(w.perception().trap(west), Some(trap(&w, "arrow")));
    assert_eq!(terrain(&w, east), "door_open");
    let out = run(&mut w, "move:NW*14 move:W*13 close:NW wait*6");
    assert_eq!(w.player_pos(), Pos::new(10, 5));
    let rang = sprung_at(&out, east);
    assert_eq!(rang.len(), 1, "once, as the kobold opened the door");
    assert_eq!(
        w.content().creature(w.state.actors[&rang[0]].creature).id,
        "kobold"
    );
    assert_eq!(w.perception().trap(east), Some(trap(&w, "alarm")));
    assert!(texts(&w).contains(&"A loud bell rings!".to_string()));
    let out = run(&mut w, "move:SW move:W*6 move:NW open:N");
    assert_eq!(w.player_pos(), Pos::new(2, 5));
    assert_eq!(sprung_at(&out, west), vec![player]);
    assert!(texts(&w).contains(&"An arrow shoots out of the wall!".to_string()));
}

#[test]
fn door_traps_save_and_replay() {
    let map = "#######\n#@b.r.#\n#######\n";
    let content = content(map, "");
    let mut w = World::new(content.clone(), "hall", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("open:E move:E move:E kick:E close:W").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

fn load(legend: &str) -> Result<Content, String> {
    let mut files = files("#####\n#@x.#\n#####\n", "");
    files.insert("data/legend.toml", &format!("{LEGEND}{legend}\n"));
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn levels_put_traps_on_doors_but_no_pit_and_none_on_a_secret_door() {
    for ok in [
        "\"x\" = { terrain = \"door_closed\", trap = \"alarm\", known = true }",
        "\"x\" = { terrain = \"door_open\", trap = \"fireball\" }",
        "\"x\" = { terrain = \"door_closed\", trap = \"lightning\", dir = \"E\" }",
    ] {
        assert!(load(ok).is_ok(), "{ok}: {:?}", load(ok).err());
    }
    let pit = load("\"x\" = { terrain = \"door_closed\", trap = \"pit\" }").unwrap_err();
    assert!(
        pit.contains("places a pit on terrain `door_closed`"),
        "{pit}"
    );
    let pit = load("\"x\" = { terrain = \"door_open\", trap = \"pit\" }").unwrap_err();
    assert!(pit.contains("places a pit on terrain `door_open`"), "{pit}");
    let secret =
        load("\"x\" = { terrain = \"door_closed\", trap = \"blade\", secret = true }").unwrap_err();
    assert!(
        secret.contains("a secret door is never trapped"),
        "{secret}"
    );
    let wall = load("\"x\" = { terrain = \"wall\", trap = \"blade\" }").unwrap_err();
    assert!(wall.contains("not walkable floor"), "{wall}");
}
