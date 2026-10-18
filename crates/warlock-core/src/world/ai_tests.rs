//! Factions and creature behavior (spec §7 "Factions", §8 "Creature behavior"): who is hostile
//! to whom, how creatures see, approach, shoot, cast, search, and go back to their routine, and
//! what the player perceives of it.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::content::{ContentFiles, Stance};
use crate::replay::Replay;
use std::sync::LazyLock;

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
id = "bars"
name = "iron bars"
glyph = "="
fg = "#a0a0a0"
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
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "kobolds"
name = "kobolds"
hostile = ["player", "vermin"]

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// Everyone at normal speed, so each creature acts once a turn, in id order, after the player.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 1000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"

[[creature]]
id = "nearsighted_rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"
sight = 2

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#60a040"
weight = 1000
base_hp = 500
faction = "kobolds"
opens_doors = true

[[creature]]
id = "archer"
name = "kobold archer"
glyph = "k"
fg = "#a0a040"
weight = 1000
base_hp = 500
faction = "kobolds"
combat = "ranged"
range = 5

[[creature]]
id = "shaman"
name = "kobold shaman"
glyph = "k"
fg = "#a040a0"
weight = 1000
base_hp = 500
faction = "kobolds"
combat = "caster"
range = 4

[[creature]]
id = "deer"
name = "deer"
glyph = "d"
fg = "#c09060"
weight = 1000
base_hp = 500
faction = "wildlife"
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"=" = "bars"
"+" = "door_closed"
"'" = "door_open"
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"n" = { terrain = "floor", creature = "nearsighted_rat" }
"k" = { terrain = "floor", creature = "kobold" }
"a" = { terrain = "floor", creature = "archer" }
"s" = { terrain = "floor", creature = "shaman" }
"d" = { terrain = "floor", creature = "deer" }
"##;

/// Levels by name; each gets a scenario of the same name. No vision file: every eye needs no
/// light, so creatures see whatever is in their line of sight.
const LEVELS: &[(&str, &str)] = &[
    // A giant rat six cells east of the player.
    ("duel", "##########\n#@.....r.#\n#........#\n##########\n"),
    // A kobold between two nearsighted rats (ids 1 and 3, both four cells away, too far to see
    // it); the player is walled off.
    (
        "between",
        "###########\n#n...k...n#\n###########\n#@........#\n###########\n",
    ),
    // The same, but the second rat is a cell nearer.
    (
        "nearer",
        "###########\n#n...k..n.#\n###########\n#@........#\n###########\n",
    ),
    // The kobold sees the player through bars; the way round leads through a closed door.
    (
        "bars",
        "#########\n#...#...#\n#k..=..@#\n#...#...#\n#...+...#\n#########\n",
    ),
    // The same with a rat, which does not open doors.
    (
        "bars_rat",
        "#########\n#...#...#\n#r..=..@#\n#...#...#\n#...+...#\n#########\n",
    ),
    // An archer (range 5) four cells from the player, a deer in the line of fire.
    (
        "blocked_shot",
        "###########\n#.........#\n#..a.d.@..#\n#.........#\n###########\n",
    ),
    // An archer eight cells from the player: out of range.
    ("far_shot", "############\n#a.......@.#\n############\n"),
    // A shaman (range 4) six cells from the player.
    ("caster", "##########\n#s.....@.#\n##########\n"),
    // A kobold fighting a rat behind a wall; the player cannot see them.
    (
        "hidden_fight",
        "#########\n#@......#\n#########\n#..kr...#\n#########\n",
    ),
    // A deer next to the player, and a rat beyond.
    ("bump", "#######\n#@d...#\n#.....#\n#######\n"),
];

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/legend.toml", LEGEND);
    for (name, map) in LEVELS {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!("name = \"{name}\"\nmap = '''\n{map}'''\n"),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\nhp = 500\n"),
        );
    }
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world(scenario: &str) -> World {
    World::new(CONTENT.clone(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn pos(world: &World, id: ActorId) -> Pos {
    world.state().actors[&id].pos
}

fn messages(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn last_message(world: &World) -> String {
    messages(world).pop().unwrap_or_default()
}

/// Whether `message` reports a melee blow (no weapon: fists) by `who` on `whom`, hit or miss.
/// The test fixture's eyes see no Defence categories on an unarmoured player, so every miss is
/// plain.
fn blow(message: &str, who: &str, whom: &str) -> bool {
    let message = message.split(" (×").next().unwrap_or_default();
    [
        format!("{who} hits {whom}."),
        format!("{who} hits {whom} hard!"),
        format!("{who} misses {whom}."),
        format!("{who} hit {whom}."),
        format!("{who} hit {whom} hard!"),
        format!("{who} miss {whom}."),
    ]
    .contains(&message.to_string())
}

/// The attacks among the events: (attacker, target, kind).
fn attacks(out: &Outcome) -> Vec<(ActorId, ActorId, AttackKind)> {
    out.events
        .iter()
        .filter_map(|e| match *e {
            Event::Attacked {
                attacker,
                target,
                kind,
                ..
            } => Some((attacker, target, kind)),
            _ => None,
        })
        .collect()
}

const PLAYER: ActorId = ActorId(0);

#[test]
fn builtin_factions_set_who_fights_whom() {
    let content = Content::builtin().expect("builtin content");
    let id = |name: &str| content.creature_id(name).expect(name);
    let stance = |a, b| content.stance(id(a), id(b));
    let hostile = [
        ("rat", "player"),
        ("kobold", "player"),
        ("kobold", "rat"),
        ("kobold_archer", "rat"),
        ("kobold_shaman", "player"),
    ];
    for (a, b) in hostile {
        assert_eq!(stance(a, b), Stance::Hostile, "{a} and {b}");
        assert_eq!(stance(b, a), Stance::Hostile, "{b} and {a}");
    }
    for wild in ["deer", "bat", "beetle"] {
        for other in ["player", "rat", "kobold", "deer"] {
            assert_eq!(stance(wild, other), Stance::Neutral, "{wild} and {other}");
        }
    }
    assert_eq!(stance("kobold", "kobold_archer"), Stance::Neutral);
    assert_eq!(stance("rat", "rat"), Stance::Neutral);
    // Every creature type but the player names a faction.
    assert!(content.creature_defs().iter().all(|c| c.faction.is_some()));
    let player = content
        .creature(id("player"))
        .faction
        .expect("the player's");
    assert_eq!(content.factions().def(player).id, "player");
}

#[test]
fn creatures_need_known_factions_and_sound_combat() {
    let with = |creatures: &str| {
        let mut files = files();
        files.insert("data/creatures.toml", &creatures.replace("\n\n", "\n"));
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    assert_eq!(with(CREATURES), Ok(()));
    let cases = [
        (
            CREATURES.replace("faction = \"wildlife\"\n", ""),
            "creature `deer` needs a `faction`",
        ),
        (
            CREATURES.replace("\"wildlife\"", "\"elves\""),
            "creature `deer`: unknown faction `elves`",
        ),
        (
            CREATURES.replacen(
                "fg = \"#ffffff\"",
                "fg = \"#ffffff\"\nfaction = \"kobolds\"",
                1,
            ),
            "the player belongs to the faction `player`",
        ),
        (
            CREATURES.replace("range = 5\n", ""),
            "creature `archer` needs a `range`",
        ),
        (
            CREATURES.replace("range = 5", "range = 11"),
            "range 11 is out of range",
        ),
        (
            CREATURES.replace("opens_doors = true", "opens_doors = true\nrange = 3"),
            "only `ranged` and `caster` creatures have a `range`",
        ),
        (
            CREATURES.replace("sight = 2", "sight = 41"),
            "sight 41 is out of range",
        ),
        (
            CREATURES.replace("sight = 2", "vision = \"x-ray\""),
            "unknown vision profile `x-ray`",
        ),
    ];
    for (creatures, expected) in cases {
        let err = with(&creatures).expect_err(expected);
        assert!(err.contains(expected), "{expected}: {err}");
    }
    // Without a factions file there are none: everyone is neutral, and none may be named.
    let mut files = files();
    files.insert("data/factions.toml", "");
    assert!(Content::load(&files).is_err(), "an empty factions file");
    let mut bare = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/legend.toml", LEGEND),
        ("levels/duel.toml", "name = \"d\"\nmap = '''\n#@#\n'''\n"),
        (
            "scenarios/duel.toml",
            "name = \"d\"\nstart_level = \"duel\"\n",
        ),
    ] {
        bare.insert(path, text);
    }
    bare.insert(
        "data/creatures.toml",
        "[[creature]]\nid = \"player\"\nname = \"you\"\nglyph = \"@\"\nfg = \"#ffffff\"\n\
         weight = 1\n[[creature]]\nid = \"rat\"\nname = \"rat\"\nglyph = \"r\"\n\
         fg = \"#806040\"\nweight = 1\n",
    );
    let content = Content::load(&bare).expect("no factions at all");
    let (player, rat) = (content.creature_id("player"), content.creature_id("rat"));
    assert_eq!(
        content.stance(player.unwrap(), rat.unwrap()),
        Stance::Neutral
    );
    bare.insert(
        "data/creatures.toml",
        "[[creature]]\nid = \"player\"\nname = \"you\"\nglyph = \"@\"\nfg = \"#ffffff\"\n\
         weight = 1\nfaction = \"player\"\n",
    );
    let err = Content::load(&bare).unwrap_err();
    assert!(
        err.message.contains("no data/factions.toml"),
        "{}",
        err.message
    );
}

#[test]
fn a_melee_creature_closes_in_and_attacks() {
    let mut w = world("duel");
    let rat = ActorId(1);
    assert_eq!(w.stance(rat, PLAYER), Stance::Hostile);
    // It steps west each turn until it stands next to the player, then attacks.
    let outs = run(&mut w, "wait*5");
    assert_eq!(pos(&w, rat), Pos::new(2, 1));
    assert!(outs.iter().all(|o| attacks(o).is_empty()));
    let out = w.apply(&Action::Wait);
    assert_eq!(attacks(&out), [(rat, PLAYER, AttackKind::Melee)]);
    assert!(blow(&last_message(&w), "The giant rat", "you"));
    // It stays and keeps attacking.
    let out = w.apply(&Action::Wait);
    assert_eq!(attacks(&out).len(), 1);
    assert!(blow(&last_message(&w), "The giant rat", "you"));
    assert_eq!(pos(&w, rat), Pos::new(2, 1));
    // It remembers where it left its routine.
    assert_eq!(w.state().actors[&rat].mind.post, Some(Pos::new(7, 1)));
}

#[test]
fn the_nearest_hostile_is_the_target_ties_go_to_the_lower_id() {
    // Equally far: rat 1 (west) before rat 3 (east).
    let mut w = world("between");
    let kobold = ActorId(2);
    w.apply(&Action::Wait);
    assert_eq!(pos(&w, kobold), Pos::new(4, 1));
    // One nearer: that one (east).
    let mut w = world("nearer");
    w.apply(&Action::Wait);
    assert_eq!(pos(&w, kobold), Pos::new(6, 1));
    // Neither rat saw the kobold: too far for their sight; they stayed put.
    assert_eq!(pos(&w, ActorId(1)), Pos::new(1, 1));
}

#[test]
fn paths_lead_around_walls_and_through_doors_for_those_that_open_them() {
    let mut w = world("bars");
    let kobold = ActorId(1);
    let mut opened = false;
    let mut attacked = false;
    for _ in 0..14 {
        let out = w.apply(&Action::Wait);
        opened |= out.events.contains(&Event::DoorOpened {
            pos: Pos::new(4, 4),
        });
        attacked |= attacks(&out).contains(&(kobold, PLAYER, AttackKind::Melee));
    }
    let (level, _) = w.ground_truth();
    assert_eq!(
        w.content().terrain(level.terrain[Pos::new(4, 4)]).id,
        "door_open"
    );
    assert!(opened, "the player saw the door open");
    assert!(attacked, "it came round and attacked");
    assert_eq!(pos(&w, kobold).chebyshev(w.player_pos()), 1);

    // A rat sees the player just as well, but cannot open the door: it waits.
    let mut w = world("bars_rat");
    let start = pos(&w, ActorId(1));
    run(&mut w, "wait*10");
    assert!(pos(&w, ActorId(1)).chebyshev(start) <= 4);
    let (level, _) = w.ground_truth();
    assert_eq!(
        w.content().terrain(level.terrain[Pos::new(4, 4)]).id,
        "door_closed"
    );
    assert!(!messages(&w).iter().any(|m| blow(m, "The giant rat", "you")));
}

#[test]
fn ranged_creatures_shoot_with_a_clear_line_of_fire_else_find_one() {
    let mut w = world("blocked_shot");
    let archer = ActorId(1);
    // In range, but the deer stands in the line: it steps aside (NE, the first of two equally
    // good cells), then shoots.
    let out = w.apply(&Action::Wait);
    assert!(attacks(&out).is_empty());
    assert_eq!(pos(&w, archer), Pos::new(4, 1));
    let out = w.apply(&Action::Wait);
    assert_eq!(attacks(&out), [(archer, PLAYER, AttackKind::Ranged)]);
    // It has no bow ready: a shot announced, nothing rolled.
    assert_eq!(last_message(&w), "The kobold archer shoots at you.");
    assert_eq!(
        pos(&w, archer),
        Pos::new(4, 1),
        "it shoots from where it is"
    );

    // Out of range: it closes in until the player is within 5 cells, then shoots.
    let mut w = world("far_shot");
    let mut shots = Vec::new();
    for _ in 0..6 {
        let out = w.apply(&Action::Wait);
        shots.extend(attacks(&out));
    }
    assert_eq!(pos(&w, ActorId(1)), Pos::new(4, 1), "5 cells away");
    assert_eq!(
        shots,
        [
            (ActorId(1), PLAYER, AttackKind::Ranged),
            (ActorId(1), PLAYER, AttackKind::Ranged),
            (ActorId(1), PLAYER, AttackKind::Ranged)
        ]
    );
}

#[test]
fn casters_cast_from_within_their_range() {
    let mut w = world("caster");
    let shaman = ActorId(1);
    // Six cells away with a range of 4: two steps closer, then spells.
    run(&mut w, "wait*2");
    assert_eq!(pos(&w, shaman), Pos::new(3, 1));
    let out = w.apply(&Action::Wait);
    assert_eq!(attacks(&out), [(shaman, PLAYER, AttackKind::Spell)]);
    assert_eq!(last_message(&w), "The kobold shaman casts a spell at you.");
}

#[test]
fn bumping_a_hostile_attacks_it_and_a_neutral_one_is_in_the_way() {
    let mut w = world("bump");
    let deer = ActorId(1);
    assert_eq!(w.stance(PLAYER, deer), Stance::Neutral);
    let out = w.apply(&Action::Move(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "The deer is in the way.");

    let mut w = world("duel");
    run(&mut w, "wait*5");
    let rat = ActorId(1);
    assert_eq!(pos(&w, rat), Pos::new(2, 1));
    let out = w.apply(&Action::Move(Dir::E));
    assert!(out.took_turn, "an attack takes a turn");
    assert_eq!(w.player_pos(), Pos::new(1, 1));
    assert_eq!(
        attacks(&out),
        [
            (PLAYER, rat, AttackKind::Melee),
            (rat, PLAYER, AttackKind::Melee)
        ]
    );
    let shown = messages(&w);
    assert!(
        blow(&shown[shown.len() - 2], "You", "the giant rat"),
        "{shown:?}"
    );
    assert!(
        blow(&shown[shown.len() - 1], "The giant rat", "you"),
        "{shown:?}"
    );
}

#[test]
fn attacks_the_player_does_not_perceive_leave_no_trace() {
    let mut w = world("hidden_fight");
    let (kobold, rat) = (ActorId(1), ActorId(2));
    let said = w.messages().len();
    // They fight (the raw events say so), but behind the wall: no event, no message.
    let mut raw = Vec::new();
    let wait = w.price_of(w.state.player, ActionKind::Wait);
    w.run_turn(wait, &mut raw);
    assert!(raw.iter().any(|e| matches!(
        *e,
        Event::Attacked {
            attacker,
            target,
            kind: AttackKind::Melee,
            from,
            to,
            strike: Some(_),
        } if (attacker, target, from, to) == (kobold, rat, Pos::new(3, 3), Pos::new(4, 3))
    )));
    let mut perceived = raw.clone();
    w.perceive(&mut perceived);
    assert!(perceived.is_empty());
    assert_eq!(w.messages().len(), said);
    for out in run(&mut w, "wait*3") {
        assert!(out.events.is_empty());
    }
    assert_eq!(w.messages().len(), said);
}

#[test]
fn search_the_last_seen_spot_then_go_back_and_patrol_on() {
    // A rat that has lost sight of a hostile: it last saw one at (7, 2) and left its patrol at
    // (1, 1), where the route carries on east. The player is walled off.
    let map = "##########\n#........#\n#........#\n##########\n#@.......#\n##########\n";
    let mut files = files();
    files.insert(
        "levels/search.toml",
        &format!(
            "name = \"s\"\nmap = '''\n{map}'''\n[legend]\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\", route = \"E2 W2\" }}\n"
        ),
    );
    files.insert(
        "scenarios/search.toml",
        "name = \"s\"\nstart_level = \"search\"\nhp = 500\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "search", None).expect("world");
    let player = w.state.player;
    let rat = ActorId(w.state.next_actor_id);
    w.state.next_actor_id += 1;
    let creature = w.content().creature_id("rat").expect("rat");
    w.state.actors.insert(
        rat,
        Actor {
            id: rat,
            kind: ActorKind::Creature,
            creature,
            level: "search".into(),
            pos: Pos::new(4, 1),
            patrol: Some(Patrol {
                route: vec![Dir::E, Dir::E, Dir::W, Dir::W],
                step: 0,
            }),
            energy: 0,
            heard: Vec::new(),
            mind: Mind {
                last_seen: Some(Pos::new(7, 2)),
                noise: None,
                post: Some(Pos::new(1, 1)),
            },
            inventory: Vec::new(),
            equipment: Default::default(),
            stats: Default::default(),
            hp: 10,
            hp_base: 0,
            regen: 0,
        },
    );
    assert_ne!(player, rat);
    let mut path = Vec::new();
    for _ in 0..14 {
        w.apply(&Action::Wait);
        path.push(pos(&w, rat));
    }
    // To the spot (three steps), back to its post (six), then on along its route: E E W W.
    assert_eq!(path[2], Pos::new(7, 2));
    assert_eq!(path[8], Pos::new(1, 1));
    assert_eq!(
        path[9..13],
        [
            Pos::new(2, 1),
            Pos::new(3, 1),
            Pos::new(2, 1),
            Pos::new(1, 1)
        ]
    );
    assert!(w.state().actors[&rat].mind.is_calm());
}

#[test]
fn rest_stops_when_a_hostile_attacks() {
    let mut w = world("duel");
    let start = w.time();
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    // The rat was in view from the start, so its approach did not stop the rest; its attack did.
    assert!(w.time().since(start) < Duration::minutes(2));
    let shown = messages(&w);
    assert!(
        shown.iter().any(|m| blow(m, "The giant rat", "you")),
        "{shown:?}"
    );
    assert!(
        shown
            .last()
            .is_some_and(|m| m.starts_with("You stop resting"))
    );
}

// ---------------------------------------------------------------------------------------------
// Hearing: creatures look into noises they cannot see the source of; attacks are heard.

/// The fixture with ears: the player's steps and the creatures' ears (spec §5 "Hearing").
fn listening_world(map: &str) -> World {
    let creatures = CREATURES
        .replacen(
            "fg = \"#ffffff\"",
            "fg = \"#ffffff\"\nfootsteps = 60\nhearing = 10",
            1,
        )
        .replace(
            "faction = \"vermin\"\n\n",
            "faction = \"vermin\"\nhearing = 5\n\n",
        )
        .replace(
            "faction = \"wildlife\"",
            "faction = \"wildlife\"\nhearing = 5",
        );
    let mut files = files();
    files.insert("data/creatures.toml", &creatures);
    files.insert(
        "levels/listen.toml",
        &format!("name = \"l\"\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/listen.toml",
        "name = \"l\"\nstart_level = \"listen\"\nhp = 500\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    World::new(content, "listen", None).expect("world")
}

#[test]
fn creatures_with_enemies_go_to_look_where_they_heard_something_unseen() {
    // A rat and a deer west of a pillar, the player east of it: they cannot see each other.
    let mut w =
        listening_world("#########\n#.......#\n#.r.#.@.#\n#.d.#...#\n#.......#\n#########\n");
    let (rat, deer) = (ActorId(1), ActorId(2));
    w.apply(&Action::Wait);
    assert!(w.state().actors[&rat].mind.is_calm(), "a wait is silent");
    // A step behind the pillar is heard; the rat goes to look, the deer (wildlife) does not care.
    w.apply(&Action::Move(Dir::S));
    let mind = w.state().actors[&rat].mind;
    assert_eq!(mind.noise, Some(Pos::new(6, 3)));
    assert_eq!(mind.post, Some(Pos::new(2, 2)));
    assert_ne!(pos(&w, rat), Pos::new(2, 2), "it set off");
    let mut attacked = false;
    for _ in 0..4 {
        let out = w.apply(&Action::Wait);
        attacked |= attacks(&out).contains(&(rat, PLAYER, AttackKind::Melee));
    }
    assert!(attacked, "it found the player");
    assert_eq!(pos(&w, deer), Pos::new(2, 3));
    assert!(w.state().actors[&deer].mind.is_calm());
}

#[test]
fn a_fight_out_of_sight_is_heard() {
    // A kobold and a rat fighting round a corner from the player.
    let mut w = listening_world("#####\n#@..#\n###.#\n#kr.#\n#####\n");
    let out = w.apply(&Action::Wait);
    assert!(out.events.is_empty(), "nothing seen");
    assert!(
        messages(&w)
            .iter()
            .any(|m| m == "You hear fighting to the south."),
        "{:?}",
        messages(&w)
    );
    let fought = |m: &String| m.contains("hit") || m.contains("miss") || m.contains("dies");
    assert!(!messages(&w).iter().any(fought));
}

// ---------------------------------------------------------------------------------------------
// Sight by light: creatures see by the player's rules, under gameplay light.

const LIT_TERRAIN: &str = r##"
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
"##;

const LIT_CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 1000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"
sight = 10

[[creature]]
id = "lurker"
name = "lurker"
glyph = "l"
fg = "#806040"
weight = 1000
base_hp = 500
faction = "vermin"
vision = "darkvision"
"##;

const LIT_LEVEL: &str = r##"
name = "Dark"
map = '''
############
#..........#
#r........@#
#..........#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", route = "N1 S1" }
"l" = { terrain = "floor", creature = "lurker" }
"##;

/// The dark room above, with lights and eyes: the player (normal eyes) has an unlit lantern.
fn dark_world(level: &str) -> World {
    let mut files = ContentFiles::default();
    files.insert("data/terrain.toml", LIT_TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures.toml", LIT_CREATURES);
    files.insert(
        "data/legend.toml",
        "[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n",
    );
    files.insert(
        "data/lights.toml",
        "[[light]]\nid = \"lantern\"\ncolor = [250, 215, 160, 0]\nradius = 5\n",
    );
    files.insert(
        "data/vision.toml",
        "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n\
         [[vision]]\nid = \"darkvision\"\nname = \"Darkvision\"\nthreshold = 40\ndarkvision = 2\n",
    );
    files.insert("levels/dark.toml", level);
    files.insert(
        "scenarios/dark.toml",
        "name = \"Dark\"\nstart_level = \"dark\"\nlantern = \"lantern\"\nhp = 500\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    World::new(content, "dark", None).expect("world")
}

#[test]
fn creatures_see_by_light_and_search_where_they_last_saw_you() {
    let mut w = dark_world(LIT_LEVEL);
    let rat = ActorId(1);
    // In the dark the rat does not see the player and keeps to its patrol.
    run(&mut w, "wait*2");
    assert_eq!(pos(&w, rat), Pos::new(1, 2));
    assert!(w.state().actors[&rat].mind.is_calm());
    // Lit, the player is seen nine cells away: it comes.
    w.apply(&Action::ToggleLantern);
    assert_eq!(w.state().actors[&rat].mind.last_seen, Some(Pos::new(10, 2)));
    assert_eq!(pos(&w, rat), Pos::new(2, 2));
    // Doused, and a step aside in the dark: it heads for where it saw the player, does not see
    // them there, and goes back.
    run(&mut w, "lantern move:N");
    let mut path = Vec::new();
    for _ in 0..18 {
        let out = w.apply(&Action::Wait);
        assert!(attacks(&out).is_empty(), "it never finds the player");
        path.push(pos(&w, rat));
    }
    assert!(path.contains(&Pos::new(10, 2)), "{path:?}");
    let back = path.iter().rposition(|p| *p == Pos::new(1, 2));
    assert!(back.is_some_and(|i| i > 8), "{path:?}");
    // Its route: N, then S back onto its post.
    assert!(
        path.ends_with(&[Pos::new(1, 1), Pos::new(1, 2)])
            || path.ends_with(&[Pos::new(1, 2), Pos::new(1, 1)]),
        "{path:?}"
    );
}

#[test]
fn something_unseen_attacks_you_in_the_dark() {
    // A lurker with darkvision next to the player in the dark: it sees the player (within its
    // darkvision); the player does not see it.
    let level = LIT_LEVEL.replace("r........@", "........l@");
    let mut w = dark_world(&level);
    let lurker = ActorId(1);
    assert!(w.perception().actors().iter().all(|a| a.id != lurker));
    let out = w.apply(&Action::Wait);
    assert!(out.events.is_empty(), "no event names the unseen attacker");
    assert!(blow(&last_message(&w), "Something", "you"));
    // Bumping into it is an attack on something.
    w.apply(&Action::Move(Dir::W));
    assert!(messages(&w).iter().any(|m| blow(m, "You", "something")));
}

// ---------------------------------------------------------------------------------------------
// The built-in arena and sky-lit levels.

fn builtin() -> Arc<Content> {
    static BUILTIN: LazyLock<Arc<Content>> =
        LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));
    BUILTIN.clone()
}

#[test]
fn creature_sight_matches_the_players_under_the_sun_and_the_moon() {
    for (scenario, actions) in [
        ("courtyard", "wait"),
        ("courtyard", "skip:5h wait"),
        ("courtyard", "skip:14h wait lantern"),
        // The night of the full moon: moonlight and moon shadows.
        ("courtyard", "move:E*20 skip:113h skip:28m wait"),
        ("forest", "skip:113h skip:30m wait"),
        ("world", "wait"),
        ("world", "skip:10h wait"),
        ("forest", "wait"),
        ("arena", "wait*3"),
    ] {
        let mut w = World::new(builtin(), scenario, None).expect(scenario);
        run(&mut w, actions);
        super::light_tests::check_creature_sight(&w)
            .unwrap_or_else(|e| panic!("{scenario} after {actions}: {e}"));
    }
}

#[test]
fn the_arena_fights_the_same_way_every_time() {
    let content = builtin();
    let play = || {
        let mut w = World::new(content.clone(), "arena", None).expect("arena");
        let mut replay = Replay::new(&w);
        for action in parse_actions("wait*30 move:E*3 wait*10").expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        (w, replay)
    };
    let (a, replay) = play();
    let (b, _) = play();
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.messages(), b.messages());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), a.state_hash());
    // They did fight, and the player saw some of it: blows, shots, spells, and deaths.
    let shown = messages(&a);
    for what in [
        "bites the kobold",
        "hits the giant rat",
        "kobold archer",
        "casts a spell at the",
        "dies.",
    ] {
        assert!(shown.iter().any(|m| m.contains(what)), "{what}: {shown:?}");
    }
    // Saved mid-fight and loaded, the run goes on identically.
    let bytes = crate::save::encode(&a);
    let mut loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), a.state());
    let mut a = a;
    run(&mut a, "wait*5");
    run(&mut loaded, "wait*5");
    assert_eq!(loaded.state_hash(), a.state_hash());
}
