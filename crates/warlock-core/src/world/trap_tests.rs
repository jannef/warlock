//! Traps in the world (spec §8 "Traps"): hidden until sprung, then known and remembered; arrows,
//! spears, fireballs, lightning, pits, alarms, and teleports; charges; deaths; saves; replays.

use super::*;
use crate::action::{MissileEnd, SpellShape, parse_actions};
use crate::content::{ContentFiles, TrapId};
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
id = "kobolds"
name = "kobolds"
hostile = ["player"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// Dummies take a lot and never fight; the guard, hostile, hears well and sees little.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
hearing = 10

[[creature]]
id = "dummy"
name = "training dummy"
glyph = "D"
fg = "#806040"
weight = 30000
faction = "wildlife"
base_hp = 1000

[[creature]]
id = "guard"
name = "kobold"
glyph = "k"
fg = "#60a040"
weight = 20000
faction = "kobolds"
hearing = 5
sight = 3
base_hp = 1000
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
"##;

const TRAPS: &str = r##"
[[trap]]
id = "arrow"
name = "arrow trap"
fg = "#c8b48c"
message = "An arrow shoots out of the wall!"
kind = "shot"
object = "arrow"
damage = "1d6"
to_hit = 20
noise = { sound = "bowstring", loudness = 40 }

[[trap]]
id = "spear"
name = "spear trap"
fg = "#b4b8c4"
message = "Spears spring from the floor!"
kind = "strike"
noun = "spear"
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
noise = { sound = "explosion", loudness = 150 }

[[trap]]
id = "lightning"
name = "lightning trap"
fg = "#a8c8ff"
message = "Lightning arcs across the floor!"
charges = 2
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

[[trap]]
id = "teleport"
name = "teleport trap"
fg = "#c890ff"
kind = "teleport"
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", trap = "arrow" }
"s" = { terrain = "floor", trap = "spear" }
"S" = { terrain = "floor", trap = "spear", known = true }
"f" = { terrain = "floor", trap = "fireball" }
"F" = { terrain = "floor", trap = "fireball", known = true }
"l" = { terrain = "floor", trap = "lightning" }
"e" = { terrain = "floor", trap = "lightning", dir = "E" }
"p" = { terrain = "floor", trap = "pit" }
"b" = { terrain = "floor", trap = "alarm" }
"x" = { terrain = "floor", trap = "teleport" }
"D" = { terrain = "floor", creature = "dummy" }
"W" = { terrain = "floor", creature = "dummy", route = "E2 W2" }
"k" = { terrain = "floor", creature = "guard" }
"##;

fn files(map: &str, dark: bool, scenario: &str) -> ContentFiles {
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
    let ambient = if dark { 0 } else { 255 };
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{map}'''\n"
        ),
    );
    let scenario = if scenario.is_empty() {
        "hp = 1000"
    } else {
        scenario
    };
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\n{scenario}\n"),
    );
    files
}

fn content(map: &str, dark: bool, scenario: &str) -> Arc<Content> {
    Arc::new(Content::load(&files(map, dark, scenario)).expect("test content is valid"))
}

fn world(map: &str) -> World {
    World::new(content(map, false, ""), "hall", None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn trap_id(world: &World, id: &str) -> TrapId {
    world.content().trap_id(id).expect("trap")
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

#[test]
fn builtin_traps_load_and_bad_ones_are_reported() {
    let content = Content::builtin().expect("builtin content");
    let ids: Vec<&str> = content.trap_defs().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "arrow",
            "spear",
            "fireball",
            "frost",
            "lightning_bolt",
            "pit",
            "alarm",
            "teleport"
        ]
    );
    assert!(content.glyphs().contains(&'^'));
    let fireball = content.trap(content.trap_id("fireball").expect("fireball"));
    assert_eq!(
        (fireball.charges, fireball.name.as_str()),
        (Some(1), "fireball trap")
    );
    // The showcase is safe: no trap on any of its levels; the trap hall has them.
    for id in ["hub", "courtyard", "vast", "arena", "overworld"] {
        let level = content.level(id).expect("level");
        assert!(level.traps.is_empty(), "{id} has no traps");
    }
    assert!(
        content
            .level("trap_hall")
            .is_some_and(|l| l.traps.len() >= 7)
    );
    // A burst names a burst effect of data/effects.toml, a bolt a bolt one.
    let mut builtin = ContentFiles::builtin();
    let traps = include_str!("../../../../assets/data/traps.toml")
        .replace("spell = \"fireball\"", "spell = \"lightning_bolt\"");
    builtin.insert("data/traps.toml", &traps);
    let err = Content::load(&builtin).expect_err("a burst drawn as a bolt");
    assert!(err.to_string().contains("burst effect"), "{err}");

    let bad = |traps: &str, legend: &str| {
        let map = if legend.is_empty() {
            "#####\n#@..#\n#####\n"
        } else {
            "#####\n#@.q#\n#####\n"
        };
        let mut f = files(map, false, "");
        f.insert("data/traps.toml", traps);
        f.insert("data/legend.toml", &format!("{LEGEND}{legend}"));
        Content::load(&f).map(|_| ()).map_err(|e| e.to_string())
    };
    let trap =
        |body: &str| format!("[[trap]]\nid = \"t\"\nname = \"t trap\"\nfg = \"#ffffff\"\n{body}");
    for (body, why) in [
        (
            "kind = \"burst\"\nspell = \"x\"\nnoun = \"x\"\nradius = 2\n",
            "damage",
        ),
        (
            "kind = \"burst\"\nspell = \"x\"\nnoun = \"x\"\nradius = 9\ndamage = \"1d6\"\n",
            "radius",
        ),
        (
            "kind = \"bolt\"\nnoun = \"x\"\nlength = 5\ndamage = \"1d6\"\n",
            "spell",
        ),
        (
            "kind = \"shot\"\nobject = \"rock\"\ndamage = \"1d6\"\n",
            "object",
        ),
        ("kind = \"pit\"\ndamage = \"1d6\"\nto_hit = 2\n", "to_hit"),
        ("kind = \"alarm\"\n", "noise"),
        ("kind = \"teleport\"\ncharges = 0\n", "charges"),
        (
            "kind = \"alarm\"\nnoise = { sound = \"bell\", loudness = 999 }\n",
            "loudness",
        ),
        (
            "kind = \"burst\"\nspell = \"x\"\nnoun = \"x\"\nradius = 2\ndamage = \"1d6\"\n\
             heat = true\nfrost = 50\n",
            "not both",
        ),
        (
            "kind = \"burst\"\nspell = \"x\"\nnoun = \"x\"\nradius = 2\ndamage = \"1d6\"\n\
             frost = 0\n",
            "frost",
        ),
        ("kind = \"pit\"\ndamage = \"1d6\"\nheat = true\n", "heat"),
    ] {
        let err = bad(&format!("{TRAPS}{}", trap(body)), "").expect_err(body);
        assert!(err.contains(why), "{body}: {err}");
    }
    for (legend, why) in [
        (
            "\"q\" = { terrain = \"wall\", trap = \"spear\" }\n",
            "not walkable",
        ),
        (
            "\"q\" = { terrain = \"floor\", trap = \"spear\", dir = \"E\" }\n",
            "bolt",
        ),
        (
            "\"q\" = { terrain = \"floor\", trap = \"dragon\" }\n",
            "unknown trap",
        ),
        (
            "\"q\" = { terrain = \"floor\", known = true }\n",
            "no `trap`",
        ),
    ] {
        let err = bad(TRAPS, legend).expect_err(legend);
        assert!(err.contains(why), "{legend}: {err}");
    }
}

#[test]
fn a_hidden_trap_goes_off_underfoot_and_is_known_from_then_on() {
    let mut w = world("#######\n#@.s..#\n#######\n");
    let at = Pos::new(3, 1);
    run(&mut w, "move:E");
    assert_eq!(
        w.perception().trap(at),
        None,
        "hidden, though in plain sight"
    );
    let before = lost(&w, w.player_id());
    let out = w.apply(&Action::Move(Dir::E));
    let spear = trap_id(&w, "spear");
    assert!(out.events.contains(&Event::TrapSprung {
        trap: spear,
        at,
        victim: w.player_id()
    }));
    assert!(
        out.events
            .iter()
            .any(|e| matches!(e, Event::TrapStruck { target, .. } if *target == w.player_id()))
    );
    assert_eq!(w.perception().trap(at), Some(spear));
    let said = texts(&w);
    let i = said
        .iter()
        .position(|t| t == "Spears spring from the floor!");
    let hit = i.and_then(|i| said.get(i + 1)).cloned().unwrap_or_default();
    assert!(hit.starts_with("The spear hits you"), "{said:?}");
    assert!(lost(&w, w.player_id()) > before);
    // Unlimited: it goes off again.
    let again = run(&mut w, "move:W move:E");
    assert!(
        again[1]
            .events
            .iter()
            .any(|e| matches!(e, Event::TrapSprung { .. }))
    );
}

#[test]
fn traps_creatures_set_off_are_known_only_if_the_player_saw_it() {
    let map = "#########\n#@...W.s#\n#########\n";
    let at = Pos::new(7, 1);
    for dark in [false, true] {
        let mut w = World::new(content(map, dark, ""), "hall", None).expect("world");
        let outs = run(&mut w, "wait wait");
        let (level, _) = w.ground_truth();
        assert!(level.trap_at(at).is_some(), "the spear trap is still there");
        let sprung = outs[1]
            .events
            .iter()
            .any(|e| matches!(e, Event::TrapSprung { .. }));
        let said = texts(&w);
        if dark {
            assert_eq!(w.perception().trap(at), None, "unseen in the dark");
            assert!(
                !sprung && !said.iter().any(|t| t.contains("spear")),
                "{said:?}"
            );
            assert!(!w.state.current().traps[&at].known);
        } else {
            assert_eq!(w.perception().trap(at), Some(trap_id(&w, "spear")));
            assert!(sprung);
            assert!(said.contains(&"Spears spring from the floor!".to_string()));
            assert!(
                said.iter()
                    .any(|t| t.starts_with("The spear hits the training dummy")),
                "{said:?}"
            );
        }
        let dummy = creature_at(&w, at);
        assert!(lost(&w, dummy) > 0, "it hurt the dummy either way");
    }
}

#[test]
fn an_arrow_shoots_out_of_the_nearest_wall_and_lands() {
    let mut w = world("#########\n#@.a....#\n#.......#\n#########\n");
    let at = Pos::new(3, 1);
    let outs = run(&mut w, "move:E move:E");
    let (from, path, end) = (outs[1].events.iter())
        .find_map(|e| match e {
            Event::Missile {
                from, path, end, ..
            } => Some((*from, path.clone(), *end)),
            _ => None,
        })
        .expect("an arrow flew");
    let slit = from.expect("the player sees the wall it came from");
    assert_eq!(slit.chebyshev(at), 1, "the nearest wall: {slit}");
    assert!(w.state.current().is_opaque(w.content(), slit));
    assert_eq!(path.first(), Some(&at));
    assert_eq!(end, MissileEnd::Struck, "to-hit 20 hits");
    let (level, _) = w.ground_truth();
    let arrow = w.content().object_id("arrow").expect("arrow");
    assert_eq!(level.objects_at(at).first().map(|s| s.object), Some(arrow));
    let said = texts(&w);
    assert!(said.contains(&"An arrow shoots out of the wall!".to_string()));
    assert!(
        said.iter().any(|t| t.starts_with("The arrow hits you")),
        "{said:?}"
    );
}

#[test]
fn a_fireball_burns_everyone_its_blast_reaches_less_at_the_edge() {
    let map = "#########\n#.......#\n#.@fDD..#\n#..#....#\n#..D....#\n#########\n";
    let mut w = world(map);
    let (near, edge, behind) = (
        creature_at(&w, Pos::new(4, 2)),
        creature_at(&w, Pos::new(5, 2)),
        creature_at(&w, Pos::new(3, 4)),
    );
    let out = w.apply(&Action::Move(Dir::E));
    let center = Pos::new(3, 2);
    let spell = (out.events.iter())
        .find_map(|e| match e {
            Event::Spell {
                effect,
                origin,
                shape,
                cells,
            } => Some((effect.clone(), *origin, shape.clone(), cells.clone())),
            _ => None,
        })
        .expect("the fireball is seen");
    assert_eq!(spell.0, "fireball");
    assert_eq!(spell.1, center);
    assert_eq!(spell.2, SpellShape::Burst { center, radius: 2 });
    assert!(spell.3.contains(&Pos::new(5, 2)) && spell.3.contains(&Pos::new(1, 2)));
    assert!(!spell.3.contains(&Pos::new(3, 4)), "the wall stops it");
    assert!(!spell.3.contains(&Pos::new(3, 3)), "walls are not burned");
    let roll = lost(&w, w.player_id());
    assert!((3..=18).contains(&roll), "3d6 at the center: {roll}");
    assert_eq!(lost(&w, near), roll, "next to it: all of it");
    assert_eq!(lost(&w, edge), (roll / 2).max(1), "at the edge: half");
    assert_eq!(lost(&w, behind), 0, "behind the wall: nothing");
    let said = texts(&w);
    for line in [
        "A fireball explodes!",
        "The fireball burns you.",
        "The fireball burns the training dummy. (×2)",
    ] {
        assert!(said.contains(&line.to_string()), "{line}: {said:?}");
    }
    // One charge: spent and gone, and forgotten, for it was seen gone.
    let (level, _) = w.ground_truth();
    assert!(level.trap_at(center).is_none());
    assert_eq!(w.perception().trap(center), None);
}

#[test]
fn lightning_runs_until_a_wall_strikes_everyone_on_it_and_runs_out() {
    let map = "###########\n#@e...D.D.#\n#.....D...#\n###########\n";
    let mut w = world(map);
    let (first, second, aside) = (
        creature_at(&w, Pos::new(6, 1)),
        creature_at(&w, Pos::new(8, 1)),
        creature_at(&w, Pos::new(6, 2)),
    );
    let out = w.apply(&Action::Move(Dir::E));
    let path: Vec<Pos> = (3..=9).map(|x| Pos::new(x, 1)).collect();
    assert!(out.events.iter().any(|e| matches!(
        e,
        Event::Spell { effect, shape: SpellShape::Bolt { path: p }, .. }
            if effect == "lightning_bolt" && *p == path
    )));
    let roll = lost(&w, w.player_id());
    assert!((4..=24).contains(&roll));
    assert_eq!((lost(&w, first), lost(&w, second)), (roll, roll));
    assert_eq!(lost(&w, aside), 0);
    let said = texts(&w);
    assert!(
        said.contains(&"The lightning bolt strikes the training dummy. (×2)".to_string()),
        "{said:?}"
    );
    // Two charges: once more, then it is spent and gone.
    let sprung = |out: &Outcome| {
        out.events
            .iter()
            .any(|e| matches!(e, Event::TrapSprung { .. }))
    };
    let outs = run(&mut w, "move:W move:E move:W move:E");
    assert!(sprung(&outs[1]) && !sprung(&outs[3]));
    assert!(w.ground_truth().0.trap_at(Pos::new(2, 1)).is_none());

    // Without a direction, the bolt runs along the longest clear line.
    let mut w = world("#########\n#@l.....#\n#########\n");
    let out = w.apply(&Action::Move(Dir::E));
    let east: Vec<Pos> = (3..=7).map(|x| Pos::new(x, 1)).collect();
    assert!(out.events.iter().any(
        |e| matches!(e, Event::Spell { shape: SpellShape::Bolt { path }, .. } if *path == east)
    ));
}

#[test]
fn a_pit_hurts_and_costs_the_next_action() {
    let mut w = world("######\n#@p..#\n######\n");
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(
        out.elapsed,
        Duration::seconds(12),
        "the step and the climb out"
    );
    assert!((1..=6).contains(&lost(&w, w.player_id())));
    assert!(texts(&w).contains(&"You fall into a pit.".to_string()));
    assert_eq!(
        w.perception().trap(Pos::new(2, 1)),
        Some(trap_id(&w, "pit"))
    );
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(out.elapsed, Duration::seconds(6), "out of it");
}

#[test]
fn an_alarm_rings_and_draws_creatures_that_hear_it() {
    let map =
        "###############\n#@b...........#\n#############.#\n#k............#\n###############\n";
    let mut w = world(map);
    let guard = creature_at(&w, Pos::new(1, 3));
    w.apply(&Action::Move(Dir::E));
    assert!(texts(&w).contains(&"A loud bell rings!".to_string()));
    assert_eq!(
        w.state.actors[&guard].mind.noise,
        Some(Pos::new(2, 1)),
        "it goes to look"
    );
}

#[test]
fn a_teleport_moves_to_a_free_cell_picked_by_a_hash() {
    let map = "##########\n#@x......#\n#........#\n##########\n";
    let to = |w: &mut World| {
        let rng = w.state.rng.clone();
        let out = w.apply(&Action::Move(Dir::E));
        assert_eq!(w.state.rng, rng, "not the gameplay random stream");
        assert!(out.events.iter().any(|e| matches!(
            e,
            Event::ActorMoved { from, .. } if *from == Pos::new(2, 1)
        )));
        w.player_pos()
    };
    let mut w = world(map);
    let landed = to(&mut w);
    assert_ne!(landed, Pos::new(2, 1));
    assert!(w.state.current().is_walkable(w.content(), landed));
    assert!(w.ground_truth().0.trap_at(landed).is_none());
    assert!(texts(&w).contains(&"You are wrenched elsewhere.".to_string()));
    assert_eq!(to(&mut world(map)), landed, "the same every time");
    assert_eq!(
        w.perception().trap(Pos::new(2, 1)),
        Some(trap_id(&w, "teleport"))
    );
}

#[test]
fn a_trap_can_kill_and_the_death_names_it() {
    let map = "######\n#@f..#\n######\n";
    let content = content(map, false, "hp = 0\nstats = { toughness = 1 }");
    let mut w = World::new(content, "hall", None).expect("world");
    w.apply(&Action::Move(Dir::E));
    let death = w.death().expect("dead");
    assert_eq!(
        (death.cause, death.by, death.of),
        (Some(Cause::Trap(trap_id(&w, "fireball"))), None, None)
    );
    assert_eq!(
        death.cause.map(|c| c.epitaph(w.content())).as_deref(),
        Some("Killed by a fireball trap")
    );
    assert!(texts(&w).contains(&"You die...".to_string()));
}

#[test]
fn known_traps_show_and_stay_in_memory_out_of_sight() {
    let mut w = world("#########\n#@......#\n#S#####.#\n#########\n");
    let at = Pos::new(1, 2);
    let spear = trap_id(&w, "spear");
    assert_eq!(w.perception().trap(at), Some(spear), "known from the start");
    run(&mut w, "move:E*6 move:S");
    assert!(!w.perception().is_visible(at));
    assert_eq!(w.perception().trap(at), Some(spear), "remembered");
    assert_eq!(w.perception().known_traps(), vec![(at, spear)]);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Traps);
    assert_eq!(rows, ["(1, 2) spear trap (remembered)"]);
}

#[test]
fn a_rest_stops_when_a_trap_goes_off_in_view() {
    let mut w = world("##########\n#@...W.s.#\n##########\n");
    let out = w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert!(out.elapsed < Duration::minutes(1), "{:?}", out.elapsed);
    let said = texts(&w);
    assert!(said.contains(&"Spears spring from the floor!".to_string()));
    assert!(
        said.last()
            .is_some_and(|t| t.starts_with("You stop resting"))
    );
}

/// The trap hall's creatures walk into its hidden traps in view within fifteen turns, which are
/// known from then on; the beetle dies of the lightning, which earns the player nothing.
#[test]
fn the_trap_hall_shows_hidden_traps_going_off() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut w = World::new(content, "traps", None).expect("the trap hall");
    let known = |w: &World| w.perception().known_traps().len();
    assert_eq!(known(&w), 8, "the proving row, and the west closet's door");
    run(&mut w, "wait*15");
    assert_eq!(
        known(&w),
        11,
        "the spear, the lightning, and the arrow; the fireball is spent"
    );
    let said = texts(&w);
    for line in [
        "Spears spring from the floor!",
        "The lightning bolt strikes the cave beetle.",
        "The cave beetle dies.",
        "A fireball explodes!",
        "An arrow shoots out of the wall!",
    ] {
        assert!(said.contains(&line.to_string()), "{line}: {said:?}");
    }
    assert_eq!(w.state.experience, 0, "a trap's kill is not the player's");
}

#[test]
fn traps_save_and_replay() {
    let map = "##########\n#@.s.l.F.#\n#........#\n##########\n";
    let content = content(map, false, "");
    let mut w = World::new(content.clone(), "hall", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E move:E move:E move:E wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    for x in 1..9 {
        let p = Pos::new(x, 1);
        assert_eq!(loaded.perception().trap(p), w.perception().trap(p));
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn whoever_makes_way_onto_a_trap_sets_it_off() {
    // The player walks into a hidden pit and on into the dummy beyond, who makes way onto the
    // pit's cell, and falls in too.
    let mut w = world("######\n#@pD.#\n######\n");
    let dummy = creature_at(&w, Pos::new(3, 1));
    let pit = trap_id(&w, "pit");
    let out = run(&mut w, "move:E move:E");
    assert_eq!(w.state.actors[&dummy].pos, Pos::new(2, 1));
    let sprung = (out[1].events.iter()).any(
        |e| matches!(e, Event::TrapSprung { trap, victim, .. } if *trap == pit && *victim == dummy),
    );
    assert!(sprung, "{:?}", out[1].events);
    assert!(lost(&w, dummy) > 0, "the fall hurt it");
}
