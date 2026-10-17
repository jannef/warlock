use super::*;
use crate::content::ContentFiles;
use crate::perception::CellView;

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

[[terrain]]
id = "stairs_down"
name = "staircase down"
glyph = ">"
fg = "#ffffff"
stairs = "down"

[[terrain]]
id = "stairs_up"
name = "staircase up"
glyph = "<"
fg = "#ffffff"
stairs = "up"
"##;

const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"

[[creature]]
id = "rat"
name = "rat"
glyph = "r"
fg = "#806040"
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"+" = "door_closed"
"'" = "door_open"
"<" = "stairs_up"
"##;

// Room A (x 1..3) and room B (x 5..9) separated by a closed door at (4, 2).
// A rat patrols room B. Stairs down at (1, 3) lead to level "below".
const TOP: &str = r##"
name = "Top"
map = '''
###########
#@..#.....#
#...+.r...#
#>..#.....#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
">" = { terrain = "stairs_down", to = "below" }
"r" = { terrain = "floor", creature = "rat", route = "E2 W2" }
"##;

const BELOW: &str = r##"
name = "Below"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "top" }
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    files.insert("levels/top.toml", TOP);
    files.insert("levels/below.toml", BELOW);
    files.insert(
        "scenarios/test.toml",
        "name = \"Test\"\nstart_level = \"top\"\nseed = 7\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "test", None).expect("world")
}

fn terrain_name(world: &World, p: Pos) -> String {
    let (level, _) = world.ground_truth();
    world.content().terrain(level.terrain[p]).id.clone()
}

fn last_message(world: &World) -> String {
    world
        .messages()
        .iter()
        .last()
        .map(|m| m.text.clone())
        .unwrap_or_default()
}

#[test]
fn new_world_starts_at_player_start_with_scenario_seed() {
    let w = world();
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(w.seed(), 7);
    assert_eq!(w.turn(), 0);
    let view = w.perception();
    assert!(matches!(view.cell(Pos::new(2, 1)), CellView::Visible(_)));
    assert_eq!(view.actors().len(), 1, "the rat is behind a closed door");
}

#[test]
fn explicit_seed_overrides_scenario_seed() {
    let w = World::new(content(), "test", Some(99)).unwrap();
    assert_eq!(w.seed(), 99);
}

#[test]
fn unknown_scenario_is_an_error() {
    assert_eq!(
        World::new(content(), "nope", None).unwrap_err(),
        WorldError::UnknownScenario("nope".into())
    );
}

#[test]
fn walking_into_a_wall_takes_no_time() {
    let mut w = world();
    let outcome = w.apply(&Action::Move(Dir::N));
    assert!(!outcome.took_turn);
    assert_eq!(w.turn(), 0);
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(last_message(&w), "The wall blocks your way.");
}

#[test]
fn moving_takes_a_turn_and_reports_the_move() {
    let mut w = world();
    let outcome = w.apply(&Action::Move(Dir::E));
    assert!(outcome.took_turn);
    assert_eq!(w.turn(), 1);
    assert_eq!(w.player_pos(), Pos::new(2, 1));
    assert!(outcome.events.contains(&Event::ActorMoved {
        actor: w.player_id(),
        from: Pos::new(1, 1),
        to: Pos::new(2, 1)
    }));
}

#[test]
fn bumping_a_closed_door_opens_it_and_reveals_the_room() {
    let mut w = world();
    w.apply(&Action::Move(Dir::SE)); // (2, 2)
    w.apply(&Action::Move(Dir::E)); // (3, 2), next to the door
    let outcome = w.apply(&Action::Move(Dir::E));
    assert!(outcome.took_turn);
    assert_eq!(w.player_pos(), Pos::new(3, 2), "opening does not move");
    assert_eq!(terrain_name(&w, Pos::new(4, 2)), "door_open");
    assert!(outcome.events.contains(&Event::DoorOpened {
        pos: Pos::new(4, 2)
    }));
    assert_eq!(w.perception().actors().len(), 2, "the rat is now in view");
}

#[test]
fn open_and_close_doors_explicitly() {
    let mut w = world();
    w.apply(&Action::Move(Dir::SE));
    w.apply(&Action::Move(Dir::E));
    assert!(w.apply(&Action::Open(Dir::E)).took_turn);
    assert!(!w.apply(&Action::Open(Dir::E)).took_turn);
    assert_eq!(last_message(&w), "That door is already open.");
    assert!(w.apply(&Action::Close(Dir::E)).took_turn);
    assert_eq!(terrain_name(&w, Pos::new(4, 2)), "door_closed");
    assert!(!w.apply(&Action::Close(Dir::N)).took_turn);
    assert_eq!(last_message(&w), "There is nothing to close there.");
}

#[test]
fn closing_the_door_hides_the_rat_but_memory_keeps_terrain() {
    let mut w = world();
    w.apply(&Action::Move(Dir::SE));
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Open(Dir::E));
    let room_b = Pos::new(8, 2);
    assert!(matches!(w.perception().cell(room_b), CellView::Visible(_)));
    w.apply(&Action::Close(Dir::E));
    let view = w.perception();
    assert!(matches!(view.cell(room_b), CellView::Remembered(_)));
    assert_eq!(
        view.actors().len(),
        1,
        "remembered cells never show creatures"
    );
}

#[test]
fn rat_patrols_and_waits_when_blocked() {
    let mut w = world();
    let rat = ActorId(1);
    let rat_pos = |w: &World| w.state().actors[&rat].pos;
    assert_eq!(rat_pos(&w), Pos::new(6, 2));
    w.apply(&Action::Wait);
    assert_eq!(rat_pos(&w), Pos::new(7, 2));
    w.apply(&Action::Wait);
    assert_eq!(rat_pos(&w), Pos::new(8, 2));
    w.apply(&Action::Wait);
    assert_eq!(rat_pos(&w), Pos::new(7, 2));
}

#[test]
fn unseen_creature_moves_are_not_reported() {
    let mut w = world();
    let outcome = w.apply(&Action::Wait);
    assert!(outcome.events.is_empty(), "{:?}", outcome.events);
}

#[test]
fn stairs_travel_and_levels_persist() {
    let mut w = world();
    // Open the door first so we can check it persists.
    w.apply(&Action::Move(Dir::SE));
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Open(Dir::E));
    w.apply(&Action::Move(Dir::W));
    w.apply(&Action::Move(Dir::SW)); // (1, 3): stairs down
    assert_eq!(w.player_pos(), Pos::new(1, 3));
    assert!(!w.apply(&Action::Ascend).took_turn);

    let outcome = w.apply(&Action::Descend);
    assert!(outcome.took_turn);
    assert_eq!(w.perception().level_id(), "below");
    assert_eq!(
        w.player_pos(),
        Pos::new(1, 1),
        "arrive on the stairs leading back"
    );

    assert!(w.apply(&Action::Ascend).took_turn);
    assert_eq!(w.perception().level_id(), "top");
    assert_eq!(w.player_pos(), Pos::new(1, 3));
    assert_eq!(
        terrain_name(&w, Pos::new(4, 2)),
        "door_open",
        "level state persisted"
    );
    assert!(matches!(
        w.perception().cell(Pos::new(2, 1)),
        CellView::Visible(_)
    ));
}

#[test]
fn creatures_on_other_levels_do_not_act() {
    let mut w = world();
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::Move(Dir::S));
    w.apply(&Action::Descend);
    let rat = ActorId(1);
    let before = w.state().actors[&rat].pos;
    w.apply(&Action::Wait);
    w.apply(&Action::Wait);
    assert_eq!(w.state().actors[&rat].pos, before);
}

#[test]
fn same_actions_give_identical_states() {
    let actions = [
        Action::Move(Dir::SE),
        Action::Move(Dir::E),
        Action::Move(Dir::E),
        Action::Wait,
        Action::Close(Dir::E),
        Action::Move(Dir::W),
    ];
    let mut a = world();
    let mut b = world();
    for action in &actions {
        a.apply(action);
        b.apply(action);
    }
    assert_eq!(a.state(), b.state());
    assert_eq!(a.state_hash(), b.state_hash());
}

#[test]
fn state_hash_tracks_state() {
    let mut w = world();
    let start = w.state_hash();
    assert_eq!(start, world().state_hash());
    w.apply(&Action::Wait);
    assert_ne!(w.state_hash(), start);
}

#[test]
fn restoring_state_recomputes_perception() {
    let mut w = world();
    w.apply(&Action::Move(Dir::E));
    let state = w.state().clone();
    let restored = World::from_state(w.content().clone(), state).unwrap();
    assert_eq!(restored.player_pos(), Pos::new(2, 1));
    assert_eq!(
        restored.perception().actors().len(),
        w.perception().actors().len()
    );
}

#[test]
fn inconsistent_state_is_rejected() {
    let w = world();
    let mut state = w.state().clone();
    state.current_level = "missing".into();
    assert!(matches!(
        World::from_state(w.content().clone(), state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn flow_is_perceived_only_on_visible_running_water() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/stream.toml",
        "name = \"Stream\"\nambient = [255, 255, 255, 0]\nmap = '''\n\
         #########\n#@.↓#.↓.#\n#..↓+.↓.#\n#########\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/stream.toml",
        "name = \"S\"\nstart_level = \"stream\"\n",
    );
    let content = Content::load(&files).expect("content loads");
    let mut w = World::new(Arc::new(content), "stream", None).expect("world");
    let south = crate::content::Flow::new(Dir::S, 100);
    let view = w.perception();
    assert_eq!(view.flow(Pos::new(3, 1)), Some(south));
    assert_eq!(view.flow(Pos::new(2, 1)), None, "floor does not flow");
    assert_eq!(view.flow(Pos::new(6, 1)), None, "behind the closed door");
    // Seen once, then out of sight again: remembered running water has no flow.
    w.apply(&Action::Move(Dir::SE));
    w.apply(&Action::Move(Dir::E));
    w.apply(&Action::Open(Dir::E));
    assert_eq!(w.perception().flow(Pos::new(6, 2)), Some(south));
    w.apply(&Action::Close(Dir::E));
    assert!(matches!(
        w.perception().cell(Pos::new(6, 2)),
        CellView::Remembered(_)
    ));
    assert_eq!(w.perception().flow(Pos::new(6, 2)), None);
}
