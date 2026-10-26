//! Incorporeal creatures (spec §8 "Incorporeal creatures", `incorporeal.rs`): through walls and
//! closed doors but never the level's edge or lava, sheltered by the wall in Defence, struck at
//! there when seen, striking out of it, setting off no trap, dropping what it carried outside,
//! and replaying exactly.

use super::*;
use crate::action::parse_actions;
use crate::combat::Category;
use crate::content::ContentFiles;

const TERRAIN: &str = r##"
[[terrain]]
id = "void"
name = "rock"
glyph = " "
fg = "#000000"
walkable = false
transparent = false

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
id = "lava"
name = "lava"
glyph = "~"
fg = "#ff4000"
walkable = false

[[terrain]]
id = "door_closed"
name = "closed door"
glyph = "+"
fg = "#a06030"
walkable = false
transparent = false
opens_to = "door_open"

[[terrain]]
id = "door_open"
name = "open door"
glyph = "'"
fg = "#a06030"
closes_to = "door_closed"
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "undead"
name = "undead"
hostile = ["player"]
"##;

// Everyone at normal speed: each acts once a turn, after the player. The haunt's touch is weak
// and the player tough, so fights go on.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "haunt"
name = "stone haunt"
glyph = "G"
fg = "#a8a49c"
weight = 500
base_hp = 500
faction = "undead"
loot = 0
attack = { damage = "1", name = "touch", verb = "touches" }
incorporeal = { wall_defence = 4 }

[[creature]]
id = "ghoul"
name = "ghoul"
glyph = "z"
fg = "#80a080"
weight = 50000
base_hp = 500
faction = "undead"
opens_doors = true
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40
"##;

const TRAPS: &str = r##"
[[trap]]
id = "spear"
name = "spear trap"
fg = "#b4b8c4"
message = "Spears spring from the floor!"
kind = "strike"
noun = "spear"
damage = "1d8"
to_hit = 20
"##;

const OBJECTS: &str = r##"
[[object]]
id = "bone"
name = "bone"
glyph = "-"
fg = "#e0e0d0"
class = "miscellaneous"
materials = ["bone"]
weight = 100
value = 0
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"~" = "lava"
"+" = "door_closed"
"'" = "door_open"
"@" = { terrain = "floor", spawn = "player" }
"h" = { terrain = "floor", creature = "haunt" }
"H" = { terrain = "wall", creature = "haunt", carry = ["bone"] }
"z" = { terrain = "floor", creature = "ghoul" }
"^" = { terrain = "floor", trap = "spear" }
"##;

/// A world on `map`, fully lit unless `dark`.
fn world(map: &str, dark: bool) -> World {
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
        ("data/traps.toml", TRAPS),
        ("data/objects.toml", OBJECTS),
        ("data/legend.toml", LEGEND),
        (
            "data/materials.toml",
            "[[material]]\nid = \"bone\"\nname = \"bone\"\n",
        ),
    ] {
        files.insert(path, text);
    }
    let ambient = if dark { 0 } else { 255 };
    files.insert(
        "levels/crypt.toml",
        &format!(
            "name = \"Crypt\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n"
        ),
    );
    files.insert(
        "scenarios/crypt.toml",
        "name = \"Crypt\"\nstart_level = \"crypt\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "crypt", Some(1)).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

/// The first creature of type `id` on the level.
fn first(world: &World, id: &str) -> ActorId {
    let creature = world.content.creature_id(id).expect(id);
    (world.state.actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .expect("there")
}

fn pos(world: &World, id: ActorId) -> Pos {
    world.state.actors[&id].pos
}

fn terrain_at(world: &World, p: Pos) -> String {
    let level = world.state.current();
    world.content.terrain(level.terrain[p]).id.clone()
}

fn last_message(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

#[test]
fn solid_cells_off_the_edge_are_the_ones_it_passes_through() {
    let w = world("#######\n#@.#~+#\n#.....#\n#######\n", false);
    let level = w.state.current();
    let phaseable = |x, y| level.phaseable(&w.content, Pos::new(x, y));
    assert!(phaseable(3, 1), "a wall");
    assert!(phaseable(5, 1), "a closed door");
    assert!(!phaseable(4, 1), "lava is no wall");
    assert!(!phaseable(2, 1), "floor is walked, not passed through");
    assert!(
        !phaseable(0, 1) && !phaseable(3, 0) && !phaseable(6, 2),
        "the edge"
    );
    assert!(!phaseable(-1, 1) && !phaseable(3, 9), "off the level");
}

#[test]
fn it_cuts_through_a_wall_the_ghoul_cannot_cross() {
    // The haunt sees the player across the lava; the only way over is the wall below it.
    let map = "#########\n#@..~..h#\n#...#..z#\n#########\n";
    let mut w = world(map, false);
    let (haunt, ghoul) = (first(&w, "haunt"), first(&w, "ghoul"));
    let mut cells = vec![pos(&w, haunt)];
    for _ in 0..8 {
        run(&mut w, "wait");
        cells.push(pos(&w, haunt));
    }
    assert!(
        cells.iter().any(|&p| p.x == 4),
        "through the wall: {cells:?}"
    );
    assert_eq!(
        pos(&w, haunt).chebyshev(w.player_pos()),
        1,
        "beside the player: {cells:?}"
    );
    assert!(pos(&w, ghoul).x > 4, "the ghoul stays walled off");
    // Never inside the level's edge.
    assert!(cells.iter().all(|p| p.y > 0 && p.y < 3));
}

#[test]
fn a_closed_door_is_left_shut_as_it_slips_through() {
    let map = "#########\n#@..~..h#\n#...+...#\n#########\n";
    let mut w = world(map, false);
    let haunt = first(&w, "haunt");
    let mut through = false;
    for _ in 0..6 {
        run(&mut w, "wait");
        through |= pos(&w, haunt) == Pos::new(4, 2);
        assert_eq!(terrain_at(&w, Pos::new(4, 2)), "door_closed");
    }
    assert!(through, "it passed the door's cell");
}

#[test]
fn neither_lava_nor_the_edge_lets_it_by() {
    // The only ways round the lava are the edge's walls.
    let mut w = world("#####\n#@~h#\n#####\n", false);
    let haunt = first(&w, "haunt");
    run(&mut w, "wait*5");
    assert_eq!(pos(&w, haunt), Pos::new(3, 1));
}

#[test]
fn the_wall_shelters_it_and_only_while_it_is_inside() {
    let map = "#######\n#@.H.h#\n#######\n";
    let mut w = world(map, false);
    let level = w.state.current().id.clone();
    let inside = w
        .state
        .actor_at(&level, Pos::new(3, 1))
        .expect("in the wall")
        .id;
    let outside = w
        .state
        .actor_at(&level, Pos::new(5, 1))
        .expect("on the floor")
        .id;
    assert!(w.in_wall(inside) && !w.in_wall(outside));
    let (din, dout) = (w.defence_of(inside), w.defence_of(outside));
    assert_eq!(din.bonus(Category::Wall), 4);
    assert_eq!(dout.bonus(Category::Wall), 0);
    assert_eq!(din.total(), dout.total() + 4);
    // A dig or blast that opens the wall around it leaves it on open ground, unsheltered.
    w.state.current_mut().terrain.set(
        Pos::new(3, 1),
        w.content.terrain_id("floor").expect("floor"),
    );
    assert!(!w.in_wall(inside));
    assert_eq!(w.defence_of(inside).bonus(Category::Wall), 0);
}

#[test]
fn the_player_strikes_at_one_they_see_in_a_wall_and_it_strikes_back_from_there() {
    let map = "#####\n#@H.#\n#...#\n#####\n";
    let mut w = world(map, false);
    let haunt = first(&w, "haunt");
    assert!(w.perception().is_visible(Pos::new(2, 1)));
    assert!(w.perception().actors().iter().any(|a| a.id == haunt));
    let mut struck_back = false;
    for _ in 0..20 {
        let out = w.apply(&Action::Move(Dir::E));
        assert!(out.took_turn, "a blow: {}", last_message(&w));
        assert_eq!(w.player_pos(), Pos::new(1, 1), "no step into the wall");
        struck_back |= (out.events.iter())
            .any(|e| matches!(e, Event::Attacked { attacker, .. } if *attacker == haunt));
    }
    assert_eq!(pos(&w, haunt), Pos::new(2, 1), "it fights on from the wall");
    assert!(struck_back, "it strikes out of the wall");
    assert!(w.state.actors[&haunt].hp < w.state.actors[&haunt].max_hp(&w.content));
}

#[test]
fn unseen_in_a_dark_wall_it_is_just_the_wall() {
    let map = "#####\n#@H.#\n#...#\n#####\n";
    let mut w = world(map, true);
    let haunt = first(&w, "haunt");
    assert!(!w.perception().actors().iter().any(|a| a.id == haunt));
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "The wall blocks your way.");
}

#[test]
fn it_sets_off_no_trap() {
    let map = "#########\n#@..^^^h#\n#########\n";
    let mut w = world(map, false);
    let haunt = first(&w, "haunt");
    let hp = w.state.actors[&haunt].hp;
    let mut crossed = false;
    for _ in 0..6 {
        run(&mut w, "wait");
        crossed |= (4..=6).contains(&pos(&w, haunt).x) && pos(&w, haunt).y == 1;
    }
    assert!(crossed, "it walked the trapped cells");
    assert_eq!(w.state.actors[&haunt].hp, hp, "no spear struck it");
    assert!(w.state.current().traps.values().all(|t| !t.known));
}

#[test]
fn what_it_carried_falls_out_of_the_wall() {
    let map = "#####\n#@H.#\n#...#\n#####\n";
    let mut w = world(map, false);
    let haunt = first(&w, "haunt");
    w.state.actors.get_mut(&haunt).expect("haunt").hp = 1;
    let mut died = false;
    for _ in 0..40 {
        run(&mut w, "move:E");
        if !w.state.actors.contains_key(&haunt) {
            died = true;
            break;
        }
    }
    assert!(died, "the haunt fell");
    let level = w.state.current();
    assert!(
        level.objects_at(Pos::new(2, 1)).is_empty(),
        "nothing in the wall"
    );
    // The nearest walkable cell, in row order: the player's own, west of it.
    assert_eq!(level.objects_at(Pos::new(1, 1)).len(), 1);
}

#[test]
fn a_walk_through_walls_replays_and_survives_a_save() {
    let map = "#########\n#@..~..h#\n#...#...#\n#########\n";
    let mut w = world(map, false);
    let text = "wait*2 move:E wait*5";
    let actions = parse_actions(text).expect("actions");
    for action in &actions {
        w.apply(action);
    }
    let mut again = world(map, false);
    for action in &actions {
        again.apply(action);
    }
    assert_eq!(w.state_hash(), again.state_hash());
    // Partway, it stood inside the wall: the state saved there holds that cell.
    let mut partway = world(map, false);
    let haunt = first(&partway, "haunt");
    for _ in 0..6 {
        if partway.in_wall(haunt) {
            break;
        }
        run(&mut partway, "wait");
    }
    assert!(partway.in_wall(haunt), "inside at {}", pos(&partway, haunt));
    let json = serde_json::to_string(&partway.state).expect("serialize");
    let back: WorldState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.actors[&haunt].pos, pos(&partway, haunt));
    let json = serde_json::to_string(&w.state).expect("serialize");
    let back: WorldState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        serde_json::to_string(&back).expect("serialize"),
        json,
        "positions inside walls round-trip"
    );
}
