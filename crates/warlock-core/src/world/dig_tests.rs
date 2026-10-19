//! Digging and breaking walls (spec §8 "Digging"): the Tool slot, how long a wall takes by its
//! hardness, the tool's power, Strength, and burden; interruptions and progress kept on the wall;
//! walls that never break; light, sight, and memory after a wall gives way; solid rock showing
//! its face; explosions and their blast; noise; saves and replays mid-dig.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::noise::SoundKind;
use crate::perception::CellView;
use crate::replay::Replay;

// A lit cellar. The player starts by a pick-axe and a mattock; north of the corridor stand packed
// earth (E), a stone wall, and granite (G); a closed door leads south; below the corridor's south
// wall lies a second room.
const CELLAR: &str = "
##########
#.E.#.G..#
#@.......#
#+########
#........#
##########
";

fn files(map: &str, light: &str, scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/cellar.toml",
        &format!(
            "name = \"Cellar\"\nambient = {light}\nmap = '''{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\", objects = [\"pick_axe\", \
             \"mattock\"] }}\n\
             \"E\" = \"packed_earth\"\n\"G\" = \"granite\"\n\
             \"l\" = {{ terrain = \"floor\", light = \"lamp\" }}\n\
             \"d\" = {{ terrain = \"floor\", creature = \"deer\", route = \"N4 S4\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\" }}\n\
             \"F\" = {{ terrain = \"floor\", trap = \"fireball\" }}\n"
        ),
    );
    files.insert(
        "scenarios/cellar.toml",
        &format!("name = \"Cellar\"\nstart_level = \"cellar\"\n{scenario}\n"),
    );
    files
}

fn content_of(map: &str, light: &str, scenario: &str) -> Arc<Content> {
    Arc::new(Content::load(&files(map, light, scenario)).expect("test content is valid"))
}

const LIT: &str = "[255, 255, 255, 0]";

/// The cellar at Strength `strength`, with the pick-axe (a) and the mattock (b) picked up and the
/// pick-axe ready in the Tool slot.
fn cellar(strength: u8) -> World {
    let stats = format!("stats = {{ strength = {strength} }}");
    let mut w = World::new(content_of(CELLAR, LIT, &stats), "cellar", None).expect("world");
    run(&mut w, "pickup equip:tool:a");
    w
}

fn run(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions parse") {
        last = w.apply(&action);
    }
    last
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

fn terrain_at(w: &World, p: Pos) -> String {
    let (level, _) = w.ground_truth();
    let t = level.terrain_at(p).expect("in bounds");
    w.content().terrain(t).id.clone()
}

/// Digs from where the player stands toward `dir`, returning the seconds it took.
fn dig(w: &mut World, dir: &str) -> u64 {
    run(w, &format!("dig:{dir}")).elapsed.as_seconds()
}

#[test]
fn work_follows_power_and_strength() {
    // A pick-axe at Strength 10 does 10 points of hardness a turn (in twentieths).
    assert_eq!(dig_work(10, 10), 10 * WORK_SCALE);
    assert_eq!(dig_work(15, 10), 15 * WORK_SCALE);
    assert_eq!(dig_work(10, 18), 280);
    assert_eq!(digs_to_break(600, 10, 10), 60);
    assert_eq!(digs_to_break(100, 10, 10), 10);
    assert_eq!(digs_to_break(3000, 10, 10), 300, "granite: half an hour");
    assert_eq!(digs_to_break(600, 15, 10), 40, "a mattock");
    assert_eq!(digs_to_break(600, 10, 18), 43, "a strong digger");
    assert_eq!(digs_to_break(600, 10, 3), 93, "a weak one");
}

#[test]
fn a_tool_goes_in_the_tool_slot_and_digs_from_there() {
    let content = content_of(CELLAR, LIT, "");
    let mut w = World::new(content.clone(), "cellar", None).expect("world");
    run(&mut w, "pickup");
    // Not ready yet: nothing to dig with, and no time passes.
    assert_eq!(dig(&mut w, "E"), 0);
    assert_eq!(last_message(&w), "You have no digging tool ready.");
    assert_eq!(w.perception().ready_to_dig(), None);
    run(&mut w, "equip:main_hand:a");
    assert_eq!(last_message(&w), "A pick-axe does not go there.");
    run(&mut w, "equip:tool:a");
    assert_eq!(last_message(&w), "You ready a pick-axe.");
    let tool = content.slot_id("tool").expect("a Tool slot");
    assert_eq!(content.slot(tool).name, "Tool");
    assert_eq!(w.perception().equipment().get(tool), Some(0));
    assert_eq!(
        crate::equipment::mark(&content, w.perception().equipment(), 0).as_deref(),
        Some("in use")
    );
    assert_eq!(w.perception().ready_to_dig(), content.object_id("pick_axe"));
    // A swap: the mattock takes its place.
    run(&mut w, "equip:tool:b");
    assert_eq!(w.perception().ready_to_dig(), content.object_id("mattock"));
}

#[test]
fn walls_take_time_by_hardness_power_and_strength() {
    // Packed earth: 10 turns of a pick-axe at Strength 10, a minute.
    let mut w = cellar(10);
    run(&mut w, "move:E");
    assert_eq!(dig(&mut w, "N"), 60);
    assert_eq!(terrain_at(&w, Pos::new(2, 1)), "rubble");
    let said: Vec<String> = (w.messages().iter().rev().take(2))
        .map(|m| m.text.clone())
        .collect();
    assert_eq!(
        said,
        [
            "After 1 minute, the packed earth crumbles.",
            "You dig into the packed earth."
        ]
    );
    let (level, _) = w.ground_truth();
    assert!(level.dug.is_empty(), "nothing half dug is left");
    // The stone wall: six minutes with the pick-axe, four with the mattock.
    run(&mut w, "move:E*2");
    assert_eq!(dig(&mut w, "N"), 6 * 60);
    assert_eq!(terrain_at(&w, Pos::new(4, 1)), "rubble");
    let mut w = cellar(10);
    run(&mut w, "equip:tool:b move:E*3");
    assert_eq!(dig(&mut w, "N"), 4 * 60);
    // Granite: tens of minutes.
    let mut w = cellar(10);
    run(&mut w, "move:E*5");
    assert_eq!(dig(&mut w, "N"), 30 * 60);
    assert_eq!(last_message(&w), "After 30 minutes, the granite crumbles.");
    // Strength: 43 turns at 18, 93 at 3.
    let mut w = cellar(18);
    run(&mut w, "move:E*3");
    assert_eq!(dig(&mut w, "N"), 43 * 6);
    let mut w = cellar(3);
    run(&mut w, "move:E*3");
    assert_eq!(dig(&mut w, "N"), 93 * 6);
}

#[test]
fn burden_slows_digging() {
    // At Strength and Toughness 3 the limit is 13.5 kg: the pick-axe alone (3 kg) is no burden,
    // with the mattock (7.5 kg in all) the digger is Burdened, and every turn takes ×1.25.
    let stats = "stats = { strength = 3, toughness = 3 }";
    let content = content_of(CELLAR, LIT, stats);
    let mut light = World::new(content.clone(), "cellar", None).expect("world");
    run(&mut light, "pickup:a equip:tool:a move:E");
    let mut heavy = World::new(content, "cellar", None).expect("world");
    run(&mut heavy, "pickup equip:tool:a move:E");
    assert_eq!(
        heavy.actor_load(heavy.player_id()).map(|l| l.burden()),
        Some(Burden::Burdened)
    );
    // Packed earth: 16 turns at Strength 3 (130 a turn against 2000).
    assert_eq!(dig(&mut light, "N"), 16 * 6);
    assert_eq!(dig(&mut heavy, "N"), 16 * 6 * 5 / 4);
}

#[test]
fn only_walls_that_break_can_be_dug() {
    let mut w = cellar(10);
    let turn = w.turn();
    for (dir, says) in [
        ("N", "There is nothing to dig there."),
        ("W", "You cannot dig through the stone wall."),
        ("S", "You cannot dig through the closed door."),
    ] {
        assert_eq!(dig(&mut w, dir), 0, "{dir}");
        assert_eq!(last_message(&w), says, "{dir}");
    }
    assert_eq!(w.turn(), turn, "refusals take no time");
    // The level's edge never breaks, whatever its terrain: the wall north of the stone wall.
    run(&mut w, "move:E*3 dig:N move:N");
    assert_eq!(w.player_pos(), Pos::new(4, 1));
    assert_eq!(dig(&mut w, "N"), 0);
    assert_eq!(last_message(&w), "You cannot dig through the stone wall.");
}

#[test]
fn digging_stops_like_resting_and_the_work_stays_on_the_wall() {
    // A deer walks up a passage into the room while the player digs: they stop when it comes
    // into view, and the wall keeps what was dug.
    const PASSAGE: &str = "
############
#..........#
#@.........#
#####.######
#####.######
#####.######
#####d######
############
";
    let content = content_of(PASSAGE, LIT, "");
    let mut w = World::new(content, "cellar", None).expect("world");
    run(&mut w, "pickup equip:tool:a");
    assert!(w.creatures_in_view().is_empty());
    let took = dig(&mut w, "S");
    assert!(took > 0 && took < 6 * 60, "{took}");
    assert_eq!(
        last_message(&w),
        format!(
            "A deer comes into view; you stop digging after {}.",
            Duration::seconds(took).spoken()
        )
    );
    let wall = Pos::new(1, 3);
    assert_eq!(terrain_at(&w, wall), "wall");
    let (level, _) = w.ground_truth();
    let done = level.dug[&wall];
    assert_eq!(u64::from(done / (10 * WORK_SCALE)) * 6, took);
    assert!(w.perception().is_dug(wall));
    // Saved and loaded mid-dig, the work stays on the wall.
    let content = w.content().clone();
    let mut loaded = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // Digging again goes on from there (the deer comes and goes on its round, so it may take
    // a few goes): six minutes in all.
    let mut total = took;
    for _ in 0..20 {
        total += dig(&mut w, "S");
        if terrain_at(&w, wall) == "rubble" {
            break;
        }
    }
    assert!((w.messages().iter()).any(|m| m.text == "You go on digging at the stone wall."));
    assert_eq!(terrain_at(&w, wall), "rubble");
    assert_eq!(total, 6 * 60);
    while terrain_at(&loaded, wall) != "rubble" {
        dig(&mut loaded, "S");
    }
    assert_eq!(
        loaded.state(),
        w.state(),
        "the loaded run digs on the same way"
    );
}

#[test]
fn digging_is_loud_and_what_it_brings_stops_it() {
    // A kobold in the next room, out of sight behind a closed door, hears the digging (muffled by
    // the door) and comes to look: the dig stops before the wall gives way.
    const NEXT_DOOR: &str = "
###########
#@.#......#
#..+k.....#
#..#......#
###########
###########
";
    let content = content_of(NEXT_DOOR, LIT, "");
    let mut w = World::new(content.clone(), "cellar", None).expect("world");
    run(&mut w, "pickup equip:tool:a move:S*2");
    let made = noise::noise(
        &content,
        w.state().current(),
        w.state().player(),
        Burden::Unburdened,
        Act::Dig,
    )
    .expect("digging is loud");
    assert_eq!(
        (made.kind, made.at, made.loudness),
        (SoundKind::Digging, w.player_pos(), noise::DIG_NOISE)
    );
    let kobold = (w.state().actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .map(|a| a.id)
        .expect("a kobold");
    assert!(w.state().actors[&kobold].mind.is_calm());
    let took = dig(&mut w, "S");
    assert!(took < 6 * 60, "interrupted: {took}");
    assert_eq!(terrain_at(&w, Pos::new(1, 4)), "wall");
    assert!(!w.state().actors[&kobold].mind.is_calm(), "it came to look");
    let text = last_message(&w);
    assert!(
        text.ends_with(&format!(
            "stop digging after {}.",
            Duration::seconds(took).spoken()
        )),
        "{text}"
    );
}

#[test]
fn a_broken_wall_lets_light_and_sight_through_and_is_remembered() {
    // Two dark rooms, each with a lamp, split by a stone wall; the player digs through it.
    const ROOMS: &str = "
###########
#...#.....#
#l.@#...l.#
#...#.....#
###########
";
    let content = content_of(ROOMS, "[0, 0, 0, 0]", "");
    let mut w = World::new(content.clone(), "cellar", None).expect("world");
    run(&mut w, "pickup equip:tool:a");
    let (wall, beyond) = (Pos::new(4, 2), Pos::new(6, 2));
    let lamp_lights = |w: &World, p: Pos| {
        (w.perception().light_sources().iter()).any(|s| {
            s.pos() == Some(Pos::new(8, 2)) && s.contributions.iter().any(|(q, _)| *q == p)
        })
    };
    assert!(!w.perception().is_visible(beyond));
    assert_eq!(w.perception().cell(beyond), CellView::Unknown);
    assert!(
        !lamp_lights(&w, w.player_pos()),
        "the far lamp is behind the wall"
    );
    let before = w.perception().light(w.player_pos()).expect("lit");
    dig(&mut w, "E");
    assert_eq!(terrain_at(&w, wall), "rubble");
    let view = w.perception();
    assert!(view.is_visible(beyond) && view.is_visible(wall));
    assert!(matches!(view.cell(wall), CellView::Visible(t) if content.terrain(t).id == "rubble"));
    assert!(
        lamp_lights(&w, w.player_pos()),
        "its light comes through the gap"
    );
    let after = view.light(w.player_pos()).expect("lit");
    assert!(after.brightness() > before.brightness());
    // Memory: the rubble and the room beyond are remembered once the player steps away.
    run(&mut w, "move:W*2");
    let view = w.perception();
    assert!(
        matches!(view.cell(wall), CellView::Remembered(t) | CellView::Visible(t)
        if content.terrain(t).id == "rubble")
    );
    assert_ne!(view.cell(beyond), CellView::Unknown);
}

#[test]
fn solid_rock_bared_by_a_dig_shows_its_face() {
    // A wall with solid rock behind it: digging through it bares the rock, which shows as walls.
    const ROCK: &str = "
#######
#@.#  #
#..#  #
#######
";
    let content = content_of(ROCK, LIT, "");
    let mut w = World::new(content, "cellar", None).expect("world");
    run(&mut w, "pickup equip:tool:a move:E");
    assert_eq!(terrain_at(&w, Pos::new(4, 1)), "void");
    dig(&mut w, "E");
    assert_eq!(terrain_at(&w, Pos::new(3, 1)), "rubble");
    for p in [Pos::new(4, 1), Pos::new(4, 2)] {
        assert_eq!(terrain_at(&w, p), "wall", "{p:?}");
    }
    assert_eq!(
        terrain_at(&w, Pos::new(5, 1)),
        "void",
        "only next to the gap"
    );
    // On through the bared rock.
    run(&mut w, "move:E");
    dig(&mut w, "E");
    assert_eq!(terrain_at(&w, Pos::new(4, 1)), "rubble");
    assert_eq!(terrain_at(&w, Pos::new(5, 1)), "wall");
}

#[test]
fn explosions_break_walls_softer_than_their_blast() {
    // A blast of 1000 in the corridor: the packed earth (100) and the stone wall (600) in its
    // burst give way, the granite (3000), the door, and the level's edge do not.
    let mut w = cellar(10);
    let mut events = Vec::new();
    let broken = w.blast(Pos::new(4, 2), 2, 1000, &mut events);
    let row = |y: i32, xs: &[i32]| xs.iter().map(move |&x| Pos::new(x, y)).collect::<Vec<_>>();
    assert_eq!(broken, [row(1, &[2, 4]), row(3, &[2, 3, 4, 5, 6])].concat());
    for p in &broken {
        assert_eq!(terrain_at(&w, *p), "rubble", "{p:?}");
    }
    assert_eq!(terrain_at(&w, Pos::new(6, 1)), "granite");
    assert_eq!(terrain_at(&w, Pos::new(4, 0)), "wall", "the edge stands");
    assert_eq!(terrain_at(&w, Pos::new(1, 3)), "door_closed");
    // The player sees it: each wall is an event, and one message says it.
    w.perceive(&mut events);
    assert_eq!(events.len(), broken.len());
    assert!((events.iter()).all(|e| matches!(
        e,
        Event::WallBroken {
            cause: Breach::Blast,
            ..
        }
    )));
    assert_eq!(last_message(&w), "The walls collapse!");
    // A weaker blast breaks only what is softer: packed earth, not stone.
    let mut w = cellar(10);
    let broken = w.blast(Pos::new(2, 2), 1, 600, &mut Vec::new());
    assert_eq!(
        broken,
        [Pos::new(2, 1)],
        "hardness 600 is not below a blast of 600"
    );
    // A blast does not reach through what it breaks: of two layers of packed earth, the far one
    // stands.
    const LAYERS: &str = "
#######
#..@..#
###E###
###E###
#.....#
#######
";
    let mut w = World::new(content_of(LAYERS, LIT, ""), "cellar", None).expect("world");
    let broken = w.blast(Pos::new(3, 1), 3, 1000, &mut Vec::new());
    assert!(broken.contains(&Pos::new(3, 2)));
    assert_eq!(terrain_at(&w, Pos::new(3, 3)), "packed_earth");
}

#[test]
fn digging_survives_saves_and_replays() {
    // Halfway through the granite: save, load, and dig on in both.
    let mut half = cellar(10);
    run(&mut half, "move:E*5");
    let granite = Pos::new(6, 1);
    half.state
        .current_mut()
        .dug
        .insert(granite, 1500 * WORK_SCALE);
    let content = half.content().clone();
    let mut loaded =
        crate::save::decode(&crate::save::encode(&half), content.clone()).expect("loads");
    assert_eq!(loaded.state(), half.state());
    assert_eq!(dig(&mut half, "N"), 15 * 60);
    assert_eq!(dig(&mut loaded, "N"), 15 * 60);
    assert_eq!(loaded.state(), half.state());
    // A replay of a run that digs verifies.
    let mut recorded = World::new(content.clone(), "cellar", None).expect("world");
    let mut replay = Replay::new(&recorded);
    for action in parse_actions("pickup equip:tool:a move:E*3 dig:N move:N dig:N").expect("parse") {
        recorded.apply(&action);
        replay.push(&action, &recorded);
    }
    assert_eq!(terrain_at(&recorded, Pos::new(4, 1)), "rubble");
    let replay = Replay::from_json(&replay.to_json()).expect("parses");
    let played = replay.run(content).expect("replay verifies");
    assert_eq!(played.state(), recorded.state());
}

#[test]
fn breaking_terrain_and_digging_tools_are_validated() {
    const TERRAIN: &str = include_str!("../../../../assets/data/terrain.toml");
    const OBJECTS: &str = include_str!("../../../../assets/data/objects.toml");
    let load = |path: &str, text: String| {
        let mut files = ContentFiles::builtin();
        files.insert(path, &text);
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let wall = "hardness = 600\nbecomes = \"rubble\"";
    for (replace, with, message) in [
        (
            wall,
            "hardness = 600",
            "`hardness` and `becomes` go together",
        ),
        (
            wall,
            "hardness = 0\nbecomes = \"rubble\"",
            "hardness must be 1 to 10000",
        ),
        (
            wall,
            "hardness = 600\nbecomes = \"granite\"",
            "which must be open ground",
        ),
        (
            wall,
            "hardness = 600\nbecomes = \"stairs_down\"",
            "which must be open ground",
        ),
        (
            "fg = \"#6a6a72\"\nbg = \"#0d0d10\"",
            "fg = \"#6a6a72\"\nbg = \"#0d0d10\"\nhardness = 5\nbecomes = \"rubble\"",
            "only solid terrain breaks",
        ),
        (
            "face = \"wall\"",
            "face = \"floor\"",
            "only solid rock has a `face`",
        ),
    ] {
        assert!(TERRAIN.contains(replace), "{replace}");
        let err = load("data/terrain.toml", TERRAIN.replacen(replace, with, 1)).unwrap_err();
        assert!(err.contains(message), "{with}: {err}");
    }
    let pick = "slot = \"tool\"\ndig = 10";
    for (with, message) in [
        ("dig = 10", "a tool goes in a `slot`"),
        ("slot = \"tool\"\ndig = 0", "power, 1 to 100"),
    ] {
        assert!(OBJECTS.contains(pick));
        let err = load("data/objects.toml", OBJECTS.replacen(pick, with, 1)).unwrap_err();
        assert!(err.contains(message), "{with}: {err}");
    }
}

#[test]
fn a_fireball_trap_blasts_the_softer_walls_open() {
    // A hidden fireball beside packed earth, a stone wall, and granite: stepping onto it, the
    // player (tough enough to live through it) sees the earth and the stone blown into rubble
    // and the granite hold; the level's edge holds too.
    const BLAST: &str = "
#########
#.E#G...#
#.@F....#
#.......#
#########
";
    let content = content_of(BLAST, LIT, "hp = 100");
    let mut w = World::new(content, "cellar", None).expect("world");
    let out = run(&mut w, "move:E");
    for (p, terrain) in [
        (Pos::new(2, 1), "rubble"),
        (Pos::new(3, 1), "rubble"),
        (Pos::new(4, 1), "granite"),
        (Pos::new(3, 0), "wall"),
    ] {
        assert_eq!(terrain_at(&w, p), terrain, "{p:?}");
    }
    let broken: Vec<Pos> = (out.events.iter())
        .filter_map(|e| match e {
            Event::WallBroken {
                pos,
                cause: Breach::Blast,
            } => Some(*pos),
            _ => None,
        })
        .collect();
    assert_eq!(broken, [Pos::new(2, 1), Pos::new(3, 1)]);
    let said: Vec<&str> = (w.messages().iter()).map(|m| m.text.as_str()).collect();
    assert!(said.contains(&"A fireball explodes!"), "{said:?}");
    assert!(said.contains(&"The walls collapse!"), "{said:?}");
}
