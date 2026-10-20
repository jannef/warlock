//! Doors (spec §8 "Doors"): locks and keys, kicking doors in, kicking creatures, creatures and
//! locked doors, and locks in saves and replays.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::content::ContentFiles;
use crate::replay::Replay;

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
toggle_noise = 60
muffle = 30
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

[[terrain]]
id = "iron_door"
name = "iron door"
glyph = "+"
fg = "#a0a0a0"
walkable = false
transparent = false
opens_to = "door_open"
muffle = 30
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
hostile = ["player"]
"##;

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
hearing = 5

[[creature]]
id = "kobold"
name = "kobold"
glyph = "k"
fg = "#60a040"
weight = 1000
base_hp = 500
faction = "kobolds"
opens_doors = true
"##;

const OBJECTS: &str = r##"
[[object]]
id = "iron_key"
name = "iron key"
glyph = "("
fg = "#b4b8bc"
class = "tools"
materials = ["iron"]
weight = 40
value = 0
key = "pantry"

[[object]]
id = "brass_key"
name = "brass key"
glyph = "("
fg = "#d8b048"
class = "tools"
materials = ["iron"]
weight = 40
value = 0
key = "cellar"

[[object]]
id = "skeleton_key"
name = "skeleton key"
glyph = "("
fg = "#e0dccc"
class = "tools"
materials = ["iron"]
weight = 30
value = 0
key = "*"

[[object]]
id = "maul"
name = "maul"
glyph = ")"
fg = "#c0c0c0"
class = "melee_weapons"
materials = ["iron"]
weight = 5000
value = 0
slot = "main_hand"
damage = "40"
"##;

const SLOTS: &str = r##"
[[slot]]
id = "main_hand"
name = "Main hand"
action = "wield"
mark = "wielded"
hand = true
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"=" = "bars"
"+" = "door_closed"
"'" = "door_open"
"I" = "iron_door"
"@" = { terrain = "floor", spawn = "player" }
"P" = { terrain = "door_closed", lock = "pantry" }
"L" = { terrain = "door_closed", locked = true }
"r" = { terrain = "floor", creature = "rat" }
"k" = { terrain = "floor", creature = "kobold" }
"q" = { terrain = "floor", creature = "kobold", carry = ["iron_key"] }
"##;

/// Levels by name, each with its scenario (and the scenario's extra lines: the player's stats).
const LEVELS: &[(&str, &str, &str)] = &[
    // A door locked with the lock `pantry` east of the player, a plain locked door beyond.
    ("pantry", "#######\n#@P.L.#\n#######\n", ""),
    // A closed door east of a strong player, and of a weak one.
    (
        "strong",
        "#####\n#@+.#\n#####\n",
        "stats = { strength = 30 }\n",
    ),
    (
        "weak",
        "#####\n#@+.#\n#####\n",
        "stats = { strength = 1 }\n",
    ),
    (
        "average",
        "#####\n#@+.#\n#####\n",
        "stats = { strength = 10 }\n",
    ),
    // A locked door and a strong kicker.
    (
        "locked_strong",
        "#####\n#@P.#\n#####\n",
        "stats = { strength = 30 }\n",
    ),
    // An iron door no kick breaks, and a rat round the corner beyond it, out of sight.
    (
        "iron",
        "#######\n#@I...#\n#####.#\n#####.#\n#####r#\n#######\n",
        "stats = { strength = 30 }\n",
    ),
    // A rat next to the player.
    ("brawl", "######\n#@r..#\n######\n", ""),
    // A kobold sees the player through bars; the way round leads through a locked door. In
    // `bars_key` the kobold carries the key.
    (
        "bars",
        "#########\n#...#...#\n#k..=..@#\n#...#...#\n#...P...#\n#########\n",
        "",
    ),
    (
        "bars_key",
        "#########\n#...#...#\n#q..=..@#\n#...#...#\n#...P...#\n#########\n",
        "",
    ),
];

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    files.insert("data/terrain.toml", TERRAIN);
    files.insert("data/factions.toml", FACTIONS);
    files.insert("data/creatures.toml", CREATURES);
    files.insert("data/objects.toml", OBJECTS);
    files.insert("data/slots.toml", SLOTS);
    files.insert("data/legend.toml", LEGEND);
    files.insert(
        "data/materials.toml",
        "[[material]]\nid = \"iron\"\nname = \"iron\"\n",
    );
    for (name, map, extra) in LEVELS {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!("name = \"{name}\"\nmap = '''\n{map}'''\n"),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\nhp = 500\n{extra}"),
        );
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn last_message(world: &World) -> String {
    (world.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn terrain(world: &World, p: Pos) -> String {
    let (level, _) = world.ground_truth();
    world.content().terrain(level.terrain[p]).id.clone()
}

fn lock(world: &World, p: Pos) -> Option<Option<String>> {
    let (level, _) = world.ground_truth();
    level.lock_at(p).map(|l| l.id.clone())
}

/// Puts one `object` in the player's pack.
fn give(world: &mut World, object: &str) {
    let object = world.content().object_id(object).expect(object);
    world.player_mut().inventory.push(Stack::one(object));
}

const EAST: Pos = Pos::new(2, 1);

#[test]
fn a_locked_door_opens_only_to_a_key_that_fits() {
    let mut w = world("pantry");
    assert_eq!(lock(&w, EAST), Some(Some("pantry".into())));
    // Without a key it is locked, whether opened or bumped; no time passes.
    for action in [Action::Open(Dir::E), Action::Move(Dir::E)] {
        let out = w.apply(&action);
        assert!(!out.took_turn && out.elapsed.is_zero());
        assert_eq!(last_message(&w), "The door is locked.");
    }
    // A key to another lock does not fit.
    give(&mut w, "brass_key");
    assert!(!w.apply(&Action::Open(Dir::E)).took_turn);
    assert_eq!(terrain(&w, EAST), "door_closed");
    // The right one unlocks and opens it in one action: `unlock` plus `open_door`.
    give(&mut w, "iron_key");
    let out = w.apply(&Action::Open(Dir::E));
    assert_eq!(out.elapsed, Duration::seconds(12));
    assert_eq!(
        last_message(&w),
        "You unlock the door with your iron key and open it."
    );
    assert!(out.events.contains(&Event::DoorOpened { pos: EAST }));
    assert_eq!(
        (terrain(&w, EAST), lock(&w, EAST)),
        ("door_open".into(), None)
    );
    // Closed again, it stays unlocked; the key is kept.
    run(&mut w, "close:E");
    let out = w.apply(&Action::Open(Dir::E));
    assert_eq!(out.elapsed, Duration::seconds(6));
    assert_eq!(last_message(&w), "You open the door.");
    assert_eq!(w.state().player().inventory.len(), 2);
}

#[test]
fn bumping_a_locked_door_with_its_key_unlocks_and_opens_it() {
    let mut w = world("pantry");
    give(&mut w, "iron_key");
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(out.elapsed, Duration::seconds(12));
    assert_eq!(
        w.player_pos(),
        Pos::new(1, 1),
        "it opens; the next step goes through"
    );
    assert_eq!(terrain(&w, EAST), "door_open");
}

#[test]
fn skeleton_keys_open_every_lock_and_only_they_open_one_without_an_id() {
    let mut w = world("pantry");
    give(&mut w, "iron_key");
    run(&mut w, "open:E move:E move:E");
    // The plain lock (no id): the iron key does not fit it.
    let plain = Pos::new(4, 1);
    assert_eq!(lock(&w, plain), Some(None));
    w.apply(&Action::Open(Dir::E));
    assert_eq!(last_message(&w), "The door is locked.");
    give(&mut w, "skeleton_key");
    w.apply(&Action::Open(Dir::E));
    assert_eq!(
        last_message(&w),
        "You unlock the door with your skeleton key and open it."
    );
    // A skeleton key opens a lock with an id too, but the key made for it is used first.
    let mut w = world("pantry");
    give(&mut w, "skeleton_key");
    give(&mut w, "iron_key");
    w.apply(&Action::Open(Dir::E));
    assert_eq!(
        last_message(&w),
        "You unlock the door with your iron key and open it."
    );
    let mut w = world("pantry");
    give(&mut w, "skeleton_key");
    w.apply(&Action::Open(Dir::E));
    assert_eq!(terrain(&w, EAST), "door_open");
}

#[test]
fn a_strong_kick_breaks_a_door_for_good() {
    let mut w = world("strong");
    let out = w.apply(&Action::Kick(Dir::E));
    assert_eq!(out.elapsed, Duration::seconds(6), "a kick takes `kick`");
    assert_eq!(last_message(&w), "The door crashes open!");
    assert!(out.events.contains(&Event::DoorBroken { pos: EAST }));
    assert_eq!(terrain(&w, EAST), "door_broken");
    // Open for good: it cannot be closed, kicking it kicks the air, and one walks through.
    w.apply(&Action::Close(Dir::E));
    assert_eq!(last_message(&w), "There is nothing to close there.");
    let out = w.apply(&Action::Kick(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You kick at empty space.");
    // The player sees through it.
    assert!(w.perception().is_visible(Pos::new(3, 1)));
    run(&mut w, "move:E move:E");
    assert_eq!(w.player_pos(), Pos::new(3, 1));
}

#[test]
fn a_kicked_in_locked_door_loses_its_lock() {
    let mut w = world("locked_strong");
    w.apply(&Action::Kick(Dir::E));
    assert_eq!(
        (terrain(&w, EAST), lock(&w, EAST)),
        ("door_broken".into(), None)
    );
}

#[test]
fn a_door_that_holds_says_whamm_and_an_unbreakable_one_always_holds() {
    let mut w = world("weak");
    // Strength 1 against sturdiness 30: one kick in thirty breaks it.
    let mut kicks = 0;
    while terrain(&w, EAST) == "door_closed" {
        let out = w.apply(&Action::Kick(Dir::E));
        assert!(out.took_turn);
        kicks += 1;
        if terrain(&w, EAST) == "door_closed" {
            assert_eq!(last_message(&w), "WHAMM!");
            assert!(out.events.contains(&Event::DoorKicked { pos: EAST }));
        }
        assert!(kicks < 500, "it breaks sooner or later");
    }
    // An iron door has no sturdiness: it never breaks, and no die is rolled.
    let mut w = world("iron");
    let rng = w.state().rng.clone();
    for _ in 0..20 {
        assert!(w.apply(&Action::Kick(Dir::E)).took_turn);
        assert_eq!(last_message(&w), "WHAMM!");
    }
    assert_eq!(terrain(&w, EAST), "iron_door");
    assert_eq!(w.state().rng, rng);
}

#[test]
fn the_chance_to_break_a_door_is_strength_in_sturdiness() {
    // Strength 10 against sturdiness 30: a third of first kicks break it.
    let content = content();
    let seeds = 600;
    let broke = (0..seeds)
        .filter(|&seed| {
            let mut w = World::new(content.clone(), "average", Some(seed)).expect("world");
            w.apply(&Action::Kick(Dir::E));
            terrain(&w, EAST) == "door_broken"
        })
        .count();
    assert!((160..=240).contains(&broke), "{broke} of {seeds}");
    // The same seed kicks the same way.
    let kick = |seed| {
        let mut w = World::new(content.clone(), "average", Some(seed)).expect("world");
        run(&mut w, "kick:E kick:E kick:E");
        (terrain(&w, EAST), w.state_hash())
    };
    assert_eq!(kick(5), kick(5));
}

#[test]
fn athletics_helps_kick_a_door_in_and_kicking_doors_trains_it() {
    let mut files = files();
    files.insert(
        "data/skills.toml",
        include_str!("../../../../assets/data/skills.toml"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let athletics = content.skill_id("athletics").expect("athletics");
    // Strength 10 and athletics 100 (+8 Strength) against sturdiness 30: 60 % of first kicks
    // break the door, not a third.
    let seeds = 600;
    let broke = (0..seeds)
        .filter(|&seed| {
            let mut w = World::new(content.clone(), "average", Some(seed)).expect("world");
            w.player_mut().skills.add(athletics, 100);
            w.apply(&Action::Kick(Dir::E));
            terrain(&w, EAST) == "door_broken"
        })
        .count();
    assert!((310..=410).contains(&broke), "{broke} of {seeds}");
    // Kicking at a door that holds earns athletics marks; kicking a creature trains unarmed
    // fighting instead.
    let mut w = World::new(content.clone(), "iron", None).expect("world");
    w.player_mut().skills.add(athletics, 0);
    run(&mut w, "kick:E*20");
    let marks = |w: &World| w.state.player().skills.get(athletics).map(|t| t.marks);
    assert!(marks(&w) > Some(0), "{:?}", marks(&w));
    let mut w = World::new(content.clone(), "brawl", None).expect("world");
    w.player_mut().skills.add(athletics, 0);
    run(&mut w, "kick:E*20");
    assert_eq!(marks(&w), Some(0));
}

#[test]
fn kicks_are_loud() {
    // The rat round the corner hears the banging on the iron door and comes to look.
    let mut w = world("iron");
    let rat = ActorId(1);
    let start = w.state().actors[&rat].pos;
    w.apply(&Action::Kick(Dir::E));
    assert_eq!(w.state().actors[&rat].mind.noise, Some(EAST));
    assert_eq!(w.state().actors[&rat].pos, start.step(Dir::N), "it comes");
}

#[test]
fn kicking_a_creature_is_an_unarmed_blow() {
    let mut w = world("brawl");
    let rat = ActorId(1);
    // Wielding a maul (40 damage a blow) changes nothing: a kick is bare hands, 1d3.
    give(&mut w, "maul");
    run(&mut w, "equip:main_hand:a");
    let mut hits = 0;
    for _ in 0..30 {
        let before = w.state().actors[&rat].hp;
        let out = w.apply(&Action::Kick(Dir::E));
        assert_eq!(out.elapsed, Duration::seconds(6));
        let kicks: Vec<_> = (out.events.iter())
            .filter_map(|e| match *e {
                Event::Attacked {
                    attacker: ActorId(0),
                    target,
                    kind,
                    ..
                } => Some((target, kind)),
                _ => None,
            })
            .collect();
        assert_eq!(kicks, [(rat, AttackKind::Melee)]);
        let lost = before - w.state().actors[&rat].hp;
        assert!(lost <= 6, "a kick did {lost}");
        let said: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
        let said = (said.into_iter().rev())
            .find(|t| t.contains("kick") || t.starts_with("You miss"))
            .unwrap_or_default();
        assert!(
            said.starts_with("You kick the giant rat")
                || said == "You miss the giant rat."
                || said.ends_with("your kick."),
            "{said}"
        );
        hits += usize::from(lost > 0);
    }
    assert!(hits > 0, "some kicks land");
    // A blow with the maul does its 40.
    let before = w.state().actors[&rat].hp;
    for _ in 0..10 {
        w.apply(&Action::Move(Dir::E));
    }
    assert!(before - w.state().actors[&rat].hp >= 40, "the maul hits");
}

#[test]
fn kicking_at_nothing_takes_no_time() {
    let mut w = world("brawl");
    for (dir, said) in [
        (Dir::N, "Kicking the wall would only hurt your foot."),
        (Dir::SE, "Kicking the wall would only hurt your foot."),
    ] {
        let out = w.apply(&Action::Kick(dir));
        assert!(!out.took_turn && out.elapsed.is_zero());
        assert_eq!(last_message(&w), said);
    }
    let mut w = world("pantry");
    give(&mut w, "iron_key");
    run(&mut w, "open:E");
    let out = w.apply(&Action::Kick(Dir::E));
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You kick at empty space.");
}

#[test]
fn creatures_open_locked_doors_only_with_the_key() {
    let door = Pos::new(4, 4);
    // Without the key the kobold cannot get round: the locked door is a wall to it.
    let mut w = world("bars");
    let kobold = ActorId(1);
    let start = w.state().actors[&kobold].pos;
    run(&mut w, "wait*12");
    assert_eq!(terrain(&w, door), "door_closed");
    assert_eq!(lock(&w, door), Some(Some("pantry".into())));
    assert!(w.state().actors[&kobold].pos.chebyshev(start) <= 4);
    // With it, it unlocks the door, opens it, and comes round.
    let mut w = world("bars_key");
    let mut opened = false;
    for _ in 0..14 {
        let out = w.apply(&Action::Wait);
        opened |= out.events.contains(&Event::DoorOpened { pos: door });
    }
    assert!(opened, "the player saw the door open");
    assert_eq!(
        (terrain(&w, door), lock(&w, door)),
        ("door_open".into(), None)
    );
    assert_eq!(w.state().actors[&kobold].pos.chebyshev(w.player_pos()), 1);
}

#[test]
fn locks_survive_a_save_and_count_in_the_state_hash() {
    let w = world("pantry");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(lock(&loaded, EAST), Some(Some("pantry".into())));
    // Unlocking changes the hash even with the door shut again.
    let mut unlocked = world("pantry");
    give(&mut unlocked, "iron_key");
    let mut plain = world("pantry");
    give(&mut plain, "iron_key");
    run(&mut unlocked, "open:E close:E");
    run(&mut plain, "wait wait");
    assert_eq!(terrain(&unlocked, EAST), terrain(&plain, EAST));
    assert_ne!(
        unlocked.state().current().locks,
        plain.state().current().locks
    );
}

#[test]
fn kicks_and_keys_replay_exactly() {
    let content = content();
    for (scenario, actions) in [
        ("average", "kick:E kick:E kick:E kick:E move:E move:E"),
        ("brawl", "kick:E kick:E kick:E kick:N"),
        ("bars_key", "wait*8 kick:W"),
    ] {
        let mut w = World::new(content.clone(), scenario, None).expect("world");
        let mut replay = Replay::new(&w);
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        let (verified, divergence) = replay.play(content.clone()).expect("replays");
        assert_eq!(divergence, None, "{scenario}");
        assert_eq!(verified.state_hash(), w.state_hash());
    }
}

#[test]
fn the_hubs_pantry_is_locked_and_its_key_lies_in_the_storeroom() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let mut w = World::new(content, "showcase", None).expect("world");
    // From the start room up the door corridor to the pantry's west door.
    run(&mut w, "move:N*3 open:N move:N*2 move:E*16 open:S move:S*4");
    assert_eq!(w.player_pos(), Pos::new(23, 14));
    w.apply(&Action::Open(Dir::E));
    assert_eq!(last_message(&w), "The door is locked.");
    // The key lies just inside the storeroom across the corridor, on the pick-axe.
    run(&mut w, "open:W move:W*2 pickup:a");
    assert_eq!(last_message(&w), "You pick up an iron key.");
    run(&mut w, "move:E*2 open:E");
    assert_eq!(
        last_message(&w),
        "You unlock the door with your iron key and open it."
    );
}

/// `files` with `path` replaced by `text`, loaded.
fn load_with(path: &str, text: &str) -> Result<Content, String> {
    let mut files = files();
    files.insert(path, text);
    Content::load(&files).map_err(|e| e.message)
}

#[test]
fn bad_locks_and_keys_are_reported() {
    let level = |legend: &str| {
        format!("name = \"x\"\nmap = '''\n#####\n#@X.#\n#####\n'''\n[legend]\n\"X\" = {legend}\n")
    };
    for (legend, expected) in [
        (
            "{ terrain = \"floor\", lock = \"pantry\" }",
            "which is no closed door",
        ),
        ("{ terrain = \"door_closed\", lock = \"*\" }", "no lock id"),
        ("{ terrain = \"door_closed\", lock = \"\" }", "no lock id"),
        (
            "{ terrain = \"door_closed\", lock = \"a\", locked = false }",
            "`locked = false`",
        ),
    ] {
        let err = load_with("levels/pantry.toml", &level(legend)).unwrap_err();
        assert!(err.contains(expected), "{legend}: {err}");
    }
    let key = OBJECTS.replace("key = \"cellar\"", "key = \" \"");
    let err = load_with("data/objects.toml", &key).unwrap_err();
    assert!(err.contains("`key`"), "{err}");
    for (from, to, expected) in [
        ("sturdiness = 30\n", "", "go together"),
        (
            "sturdiness = 30\n",
            "sturdiness = 0\n",
            "`sturdiness` must be",
        ),
        (
            "breaks_to = \"door_broken\"",
            "breaks_to = \"door_open\"",
            "walkable and no door",
        ),
    ] {
        let err = load_with("data/terrain.toml", &TERRAIN.replacen(from, to, 1)).unwrap_err();
        assert!(err.contains(expected), "{to}: {err}");
    }
    let floor_breaks = TERRAIN.replace(
        "fg = \"#808080\"\n",
        "fg = \"#808080\"\nbreaks_to = \"door_broken\"\nsturdiness = 5\n",
    );
    let err = load_with("data/terrain.toml", &floor_breaks).unwrap_err();
    assert!(err.contains("only closed doors"), "{err}");
}

#[test]
fn a_vaults_locks_need_their_keys_within_reach() {
    // The entrance at the bottom opens west of the locked door (`P`, the default legend's).
    let vault = |inner: &str| {
        format!(
            "name = \"v\"\nmap = '''\n#######\n#{inner}#\n##+####\n'''\n[legend]\n\
             \"K\" = {{ terrain = \"floor\", objects = [\"iron_key\"] }}\n\
             \"B\" = {{ terrain = \"floor\", objects = [\"brass_key\"] }}\n\
             \"S\" = {{ terrain = \"floor\", objects = [\"skeleton_key\"] }}\n"
        )
    };
    // A key lies, or is carried (`q`), on this side of the lock: fine.
    for inner in ["K.P..", "q.P..", "S.P.."] {
        load_with("vaults/v.toml", &vault(inner)).unwrap_or_else(|e| panic!("{inner}: {e}"));
    }
    // No key, a key to another lock, or the key only behind the lock itself.
    for inner in ["..P..", "B.P..", "..P.K"] {
        let err = load_with("vaults/v.toml", &vault(inner)).unwrap_err();
        assert!(err.contains("no key to the lock"), "{inner}: {err}");
    }
}
