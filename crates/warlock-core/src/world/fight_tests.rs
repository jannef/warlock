//! Combat in the world (spec §8 "Combat"): attacks hurt and kill, creatures drop what they
//! carry, hit points come back with time, and the player's death ends the run.

use super::fight::{End, Words, attack_message, death_message};
use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Category, Health, Strike};
use crate::content::ContentFiles;
use crate::replay::Replay;

#[test]
fn messages_say_what_came_of_an_attack_without_numbers() {
    let spear = Words::test("spear", "hits");
    let bite = Words::test("bite", "bites");
    let say = |a, t, words: &Words, strike| {
        attack_message(a, t, words, AttackKind::Melee, Some(strike), Some("shell"))
            .unwrap_or_default()
    };
    let (kobold, rat, beetle) = (
        End::Seen("kobold"),
        End::Seen("giant rat"),
        End::Seen("cave beetle"),
    );
    let hit = Strike::Hit { critical: false };
    let crit = Strike::Hit { critical: true };
    let by = |c| Strike::Missed { by: Some(c) };
    for (text, expected) in [
        (say(kobold, End::You, &spear, hit), "The kobold hits you."),
        (
            say(kobold, End::You, &spear, crit),
            "The kobold hits you hard!",
        ),
        (
            say(rat, kobold, &bite, hit),
            "The giant rat bites the kobold.",
        ),
        (
            say(End::You, rat, &spear, crit),
            "You hit the giant rat hard!",
        ),
        (
            say(kobold, End::You, &spear, Strike::Missed { by: None }),
            "The kobold misses you.",
        ),
        (
            say(kobold, End::You, &spear, by(Category::Dexterity)),
            "You dodge the kobold's spear.",
        ),
        (
            say(kobold, End::You, &spear, by(Category::Armour)),
            "The kobold's spear glances off your armour.",
        ),
        (
            say(kobold, End::You, &spear, by(Category::Shield)),
            "You block the kobold's spear with your shield.",
        ),
        (
            say(kobold, End::You, &spear, by(Category::Parry)),
            "You parry the kobold's spear.",
        ),
        (
            say(End::You, beetle, &spear, by(Category::Natural)),
            "Your spear glances off the cave beetle's shell.",
        ),
        (
            say(rat, kobold, &bite, by(Category::Shield)),
            "The kobold blocks the giant rat's bite with its shield.",
        ),
        (
            say(End::You, rat, &spear, by(Category::Dodge)),
            "The giant rat dodges your spear.",
        ),
        (
            say(kobold, rat, &spear, by(Category::Deflection)),
            "The kobold's spear turns aside before it reaches the giant rat.",
        ),
        // Why it missed needs both ends seen.
        (
            say(End::Unseen, End::You, &spear, by(Category::Armour)),
            "Something misses you.",
        ),
        (
            say(End::You, End::Unseen, &spear, by(Category::Natural)),
            "You miss something.",
        ),
        (
            say(End::Unseen, rat, &bite, hit),
            "Something bites the giant rat.",
        ),
        (
            say(kobold, End::Unseen, &spear, crit),
            "The kobold hits something hard!",
        ),
    ] {
        assert_eq!(text, expected);
    }
    // Shots, and intents (a spell; a shot with no missile weapon).
    let arrow = Words::test("arrow", "shoots");
    let archer = End::Seen("kobold archer");
    let shot = |a, t, strike| attack_message(a, t, &arrow, AttackKind::Ranged, strike, None);
    assert_eq!(
        shot(archer, rat, Some(hit)).as_deref(),
        Some("The kobold archer shoots the giant rat.")
    );
    assert_eq!(
        shot(archer, End::You, Some(by(Category::Dexterity))).as_deref(),
        Some("You dodge the kobold archer's arrow.")
    );
    assert_eq!(
        shot(archer, End::You, None).as_deref(),
        Some("The kobold archer shoots at you.")
    );
    let spell = Words::test("spell", "casts a spell at");
    assert_eq!(
        attack_message(
            End::Seen("kobold shaman"),
            rat,
            &spell,
            AttackKind::Spell,
            None,
            None
        )
        .as_deref(),
        Some("The kobold shaman casts a spell at the giant rat.")
    );
    // The player sees neither end of it, and was not its target: nothing.
    assert_eq!(
        attack_message(
            End::Unseen,
            End::Unseen,
            &spear,
            AttackKind::Melee,
            Some(hit),
            None
        ),
        None
    );
    assert_eq!(death_message(rat), "The giant rat dies.");
    assert_eq!(death_message(End::You), "You die...");
}

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
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "kobolds"
name = "kobolds"
hostile = ["player", "vermin"]
"##;

// The rat is frail and slow to anger; the brute hits hard and often; the turtle is all shell.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 4000
faction = "vermin"
stats = { toughness = 4 }
attack = { damage = "1d2", name = "bite", verb = "bites" }

[[creature]]
id = "brute"
name = "kobold brute"
glyph = "k"
fg = "#60a040"
weight = 30000
faction = "kobolds"
stats = { strength = 30, toughness = 20 }
base_hp = 20

[[creature]]
id = "turtle"
name = "turtle"
glyph = "t"
fg = "#60a040"
weight = 30000
faction = "vermin"
natural = { defence = 15, name = "shell" }
base_hp = 1000

[[creature]]
id = "lurker"
name = "lurker"
glyph = "l"
fg = "#60a040"
weight = 30000
faction = "kobolds"
vision = "darkvision"
stats = { strength = 30, toughness = 20 }
base_hp = 20
"##;

const VISION: &str = r##"
[[vision]]
id = "normal"
name = "Normal"
threshold = 40

[[vision]]
id = "darkvision"
name = "Darkvision"
threshold = 40
darkvision = 3
"##;

const OBJECTS: &str = r##"
[[object]]
id = "sword"
name = "sword"
glyph = ")"
fg = "#c0c0c0"
material = "steel"
weight = 1000
slot = "main_hand"
damage = "1d8"
threat = 20

[[object]]
id = "mail"
name = "mail shirt"
glyph = "["
fg = "#c0c0c0"
material = "steel"
weight = 9000
slot = "body"
armour = 5
max_dex = 1

[[object]]
id = "buckler"
name = "buckler"
glyph = "["
fg = "#c0c0c0"
material = "steel"
weight = 2000
slot = "off_hand"
shield = 2

[[object]]
id = "staff"
name = "staff"
glyph = ")"
fg = "#c0c0c0"
material = "steel"
weight = 2000
slot = "main_hand"
damage = "1d6"
parry = 3

[[object]]
id = "bow"
name = "bow"
glyph = "}"
fg = "#c0c0c0"
material = "steel"
weight = 700
slot = "missile_weapon"
launches = "arrow"
damage = "1d8+2"
crit = 3
strength = 14
range = 8

[[object]]
id = "arrow"
name = "arrow"
glyph = "/"
fg = "#c0c0c0"
material = "steel"
weight = 30
stackable = true
slot = "missiles"
ammo = "arrow"

[[object]]
id = "coin"
name = "coin"
glyph = "$"
fg = "#f0c040"
material = "steel"
weight = 10
stackable = true
"##;

const SLOTS: &str = r##"
[[slot]]
id = "body"
name = "Body"
action = "body_armor"
mark = "being worn"

[[slot]]
id = "main_hand"
name = "Main hand"
action = "wield"
mark = "wielded"
hand = true

[[slot]]
id = "off_hand"
name = "Off hand"
action = "wield"
mark = "in off hand"
hand = true

[[slot]]
id = "missile_weapon"
name = "Missile weapon"
action = "ready"
mark = "readied"

[[slot]]
id = "missiles"
name = "Missiles"
action = "ready"
mark = "quivered"
"##;

// Distinct numbers, so the tests do not follow the tuned file.
const STANCES: &str = r##"
[[stance]]
id = "aggressive"
name = "Aggressive"
to_hit = 1
damage = 2
dodge = -3
weapon_mark = 125

[[stance]]
id = "neutral"
name = "Neutral"

[[stance]]
id = "defensive"
name = "Defensive"
to_hit = -2
damage = -1
dodge = 2
parry = 1

[[stance]]
id = "coward"
name = "Coward"
to_hit = -4
damage = -3
dodge = 4
attack_cost = 125
flee_cost = 80
flee_noise = 50
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", carry = ["coin*7", "sword"], wield = "sword" }
"k" = { terrain = "floor", creature = "brute" }
"t" = { terrain = "floor", creature = "turtle" }
"l" = { terrain = "floor", creature = "lurker" }
"##;

/// The test content's files, with a level and a scenario named "fight": the level on `map`
/// with `level` header lines, the scenario with `scenario` lines.
fn files(map: &str, level: &str, scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/vision.toml", VISION),
        ("data/stances.toml", STANCES),
        (
            "data/materials.toml",
            "[[material]]\nid = \"steel\"\nname = \"steel\"\n",
        ),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/fight.toml",
        &format!("name = \"The Pit\"\n{level}\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/fight.toml",
        &format!("name = \"Fight\"\nstart_level = \"fight\"\n{scenario}\n"),
    );
    files
}

/// A test world on `map`, fully lit; the scenario has `extra` lines (the player's `hp` beyond
/// their Toughness, their stats).
fn world_on(map: &str, extra: &str) -> World {
    let files = files(map, "ambient = [255, 255, 255, 0]", extra);
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "fight", None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn messages(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn actor(world: &World, id: u32) -> Option<&Actor> {
    world.state().actors.get(&ActorId(id))
}

#[test]
fn hit_points_are_toughness_plus_race_and_class_or_the_creatures_base() {
    let w = world_on(
        "#####\n#@.r#\n#.k.#\n#####\n",
        "hp = 7\nstats = { toughness = 14 }",
    );
    let player = w.state().player();
    assert_eq!((player.hp, player.max_hp(), player.hp_base), (21, 21, 7));
    assert_eq!(w.perception().own_hp(), (21, 21));
    let rat = actor(&w, 1).expect("rat");
    assert_eq!((rat.hp, rat.max_hp()), (4, 4), "Toughness 4, no base");
    let brute = actor(&w, 2).expect("brute");
    assert_eq!((brute.hp, brute.max_hp()), (40, 40));
    assert_eq!(rat.health(), Health::Unhurt);
}

#[test]
fn the_players_hit_points_come_from_toughness_race_and_class() {
    let content = Arc::new(Content::builtin().expect("builtin content"));
    let hp = |spec: &str| {
        let spec: crate::character::CharacterSpec = spec.parse().expect("a character");
        let w =
            World::with_character(content.clone(), "showcase", None, Some(&spec)).expect("world");
        let player = w.state().player();
        let toughness = i32::from(player.stats.score(crate::stats::Stat::Toughness));
        assert_eq!(player.hp, player.max_hp(), "{spec}: full at the start");
        (player.hp_base, player.max_hp() - toughness)
    };
    // #113's tables: dwarf 6 + barbarian 8; human 4 + warrior 7; elf 2 + wizard 0.
    assert_eq!(hp("dwarf,male,barbarian"), (14, 14));
    assert_eq!(hp("human,female,warrior"), (11, 11));
    assert_eq!(hp("elf,other,wizard"), (2, 2));
    assert_eq!(hp("halfling,male,thief"), (5, 5));
}

#[test]
fn defence_stacks_by_category_and_armour_caps_dexterity() {
    let mut w = world_on(
        "#####\n#@..#\n#####\n",
        "stats = { dexterity = 18 }\nhp = 10",
    );
    let content = w.content().clone();
    let id = |name: &str| content.object_id(name).expect(name);
    // Dexterity 18: +4 on its own.
    assert_eq!(w.perception().own_defence(), 14);
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("player");
    me.inventory = ["mail", "buckler", "staff"]
        .iter()
        .map(|o| crate::object::Stack::one(id(o)))
        .collect();
    let slot = |s: &str| content.slot_id(s).expect(s);
    me.equipment.put(&[slot("body")], 0);
    me.equipment.put(&[slot("off_hand")], 1);
    me.equipment.put(&[slot("main_hand")], 2);
    let dv = me.defence(&content, Burden::Unburdened);
    // The mail caps Dexterity at +1; armour 5, shield 2, parry 3.
    assert_eq!(dv.bonus(Category::Dexterity), 1);
    assert_eq!(dv.bonus(Category::Armour), 5);
    assert_eq!(dv.bonus(Category::Shield), 2);
    assert_eq!(dv.bonus(Category::Parry), 3);
    assert_eq!(dv.total(), 21);
    // A clumsy player keeps the whole penalty under armour (and a load).
    let mut clumsy = me.clone();
    clumsy.stats = crate::stats::Stats::from_data(
        &std::collections::BTreeMap::from([(crate::stats::Stat::Dexterity, 6)]),
        &Default::default(),
    )
    .expect("stats");
    assert_eq!(
        clumsy
            .defence(&content, Burden::Burdened)
            .bonus(Category::Dexterity),
        -2
    );
    // The turtle's shell is natural armour.
    let w = world_on("#####\n#@t.#\n#####\n", "hp = 10");
    let turtle = actor(&w, 1).expect("turtle");
    assert_eq!(
        turtle
            .defence(w.content(), Burden::Unburdened)
            .bonus(Category::Natural),
        15
    );
}

#[test]
fn a_creature_killed_drops_what_it_carried_and_is_gone() {
    // A brute next to a frail rat that wields a sword and carries coins.
    let mut w = world_on("######\n#@...#\n#..kr#\n######\n", "hp = 1000");
    let (brute, rat) = (ActorId(1), ActorId(2));
    let mut died = None;
    for _ in 0..20 {
        let out = w.apply(&Action::Wait);
        if let Some(at) = out.events.iter().find_map(|e| match *e {
            Event::Died { actor, at } if actor == rat => Some(at),
            _ => None,
        }) {
            died = Some(at);
            break;
        }
    }
    let at = died.expect("the brute killed the rat");
    assert_eq!(at, Pos::new(4, 2));
    assert!(actor(&w, rat.0).is_none(), "it is gone");
    assert!(actor(&w, brute.0).is_some());
    let content = w.content().clone();
    let pile: Vec<String> = (w.state().current().objects_at(at).iter())
        .map(|s| s.describe(&content))
        .collect();
    assert_eq!(pile, ["7 coins", "a sword"]);
    let shown = messages(&w);
    assert!(
        shown.iter().any(|m| m == "The giant rat dies."),
        "{shown:?}"
    );
    assert!(
        shown
            .iter()
            .any(|m| m.starts_with("The kobold brute hits the giant rat")),
        "{shown:?}"
    );
    // The pile is seen, and remembered.
    assert_eq!(w.perception().objects(at).len(), 2);
    // Nothing more comes from the dead rat; the brute turns on the player.
    let mut later = Vec::new();
    for out in run(&mut w, "wait*5") {
        later.extend(out.events);
    }
    assert!(
        later
            .iter()
            .all(|e| !matches!(e, Event::Attacked { attacker, .. } if *attacker == rat))
    );
}

#[test]
fn the_player_kills_what_they_hit_and_hears_of_it_at_once() {
    // The frail rat next to the player, who is strong.
    let mut w = world_on(
        "#####\n#@r.#\n#####\n",
        "hp = 1000\nstats = { strength = 30 }",
    );
    let rat = ActorId(1);
    for _ in 0..20 {
        if actor(&w, rat.0).is_none() {
            break;
        }
        w.apply(&Action::Move(Dir::E));
    }
    assert!(actor(&w, rat.0).is_none(), "the rat died");
    let shown = messages(&w);
    let i = shown.iter().position(|m| m == "The giant rat dies.");
    let i = i.expect("its death was said");
    assert!(
        shown[i - 1].starts_with("You hit the giant rat"),
        "{shown:?}"
    );
    // Its sword and coins lie where it fell; the player steps there and sees them.
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), Pos::new(2, 1));
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("There are several objects here.")
    );
}

#[test]
fn hit_points_come_back_with_time_by_toughness() {
    // Toughness 10: a hit point every 240 s of game time.
    let mut w = world_on("#####\n#@..#\n#####\n", "hp = 10");
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").hp = 5;
    // 39 turns of 6 s: 234 s, not yet; the 40th makes 240 s.
    run(&mut w, "wait*39");
    assert_eq!(w.state().player().hp, 5);
    assert_eq!(w.state().player().regen, 234);
    w.apply(&Action::Wait);
    assert_eq!(w.state().player().hp, 6);
    assert_eq!(w.state().player().regen, 0);
    // An hour's rest: 15 more, up to the maximum of 20, and no more.
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(w.state().player().hp, 20);
    assert_eq!(w.state().player().regen, 0, "nothing banked at full health");
    // Toughness 25: a minute each.
    let mut w = world_on(
        "#####\n#@..#\n#####\n",
        "hp = 10\nstats = { toughness = 25 }",
    );
    w.state.actors.get_mut(&player).expect("player").hp = 1;
    run(&mut w, "wait*10");
    assert_eq!(w.state().player().hp, 2);
    run(&mut w, "wait*20");
    assert_eq!(w.state().player().hp, 4);
}

/// The player, weak and with few hit points, next to the brute.
fn doomed() -> World {
    world_on("#####\n#@k.#\n#####\n", "hp = 0\nstats = { toughness = 1 }")
}

#[test]
fn the_players_death_ends_the_run() {
    let mut w = doomed();
    let mut outs = Vec::new();
    for _ in 0..50 {
        let out = w.apply(&Action::Wait);
        let done = w.death().is_some();
        outs.push(out);
        if done {
            break;
        }
    }
    let death = w.death().expect("the brute killed the player").clone();
    assert_eq!(death.place, "The Pit");
    assert_eq!(death.turn, w.turn());
    assert_eq!(death.time, w.time());
    let brute = w.content().creature_id("brute");
    assert_eq!(death.by, brute, "seen");
    let last = outs.last().expect("a turn");
    assert!(last.events.contains(&Event::Died {
        actor: w.player_id(),
        at: Pos::new(1, 1)
    }));
    assert_eq!(messages(&w).last().map(String::as_str), Some("You die..."));
    assert!(w.state().player().hp <= 0);
    assert_eq!(w.perception().own_hp().0, 0);
    // Nothing more happens: every action is refused, and no time passes.
    let (turn, time, hash) = (w.turn(), w.time(), w.state_hash());
    for action in parse_actions("wait move:E rest:1h skip:1h").expect("actions") {
        assert_eq!(w.apply(&action), Outcome::default());
    }
    assert_eq!((w.turn(), w.time(), w.state_hash()), (turn, time, hash));
    // A save of it still loads, dead (a dev tool may keep one; the game deletes it).
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.death(), Some(&death));
}

#[test]
fn something_unseen_kills_the_player_in_the_dark() {
    // A lurker with darkvision next to the player in the dark: the death names no killer.
    let files = files(
        "#####\n#@l.#\n#####\n",
        "ambient = [0, 0, 0, 0]",
        "hp = 0\nstats = { toughness = 1 }",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "fight", None).expect("world");
    let mut last = Outcome::default();
    for _ in 0..50 {
        last = w.apply(&Action::Wait);
        if w.death().is_some() {
            break;
        }
    }
    assert_eq!(w.death().map(|d| d.by), Some(None));
    let shown = messages(&w);
    assert!(
        shown.iter().any(|m| m.starts_with("Something hits you")),
        "{shown:?}"
    );
    assert_eq!(shown.last().map(String::as_str), Some("You die..."));
    // The player's own death reaches presentation; the unseen blow does not.
    assert!(
        last.events
            .iter()
            .all(|e| !matches!(e, Event::Attacked { .. }))
    );
    assert!(last.events.iter().any(|e| matches!(e, Event::Died { .. })));
}

#[test]
fn fights_replay_exactly_and_survive_a_save() {
    let content = {
        let w = world_on("######\n#@...#\n#..kr#\n######\n", "hp = 1000");
        w.content().clone()
    };
    let play = |n: usize| {
        let mut w = World::new(content.clone(), "fight", None).expect("world");
        let mut replay = Replay::new(&w);
        for _ in 0..n {
            w.apply(&Action::Wait);
            replay.push(&Action::Wait, &w);
        }
        (w, replay)
    };
    let (a, replay) = play(12);
    let (b, _) = play(12);
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.messages(), b.messages());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), a.state_hash());
    // Saved mid-fight (hurt actors, banked regeneration) and loaded, it goes on the same.
    let (mut a, _) = play(3);
    let bytes = crate::save::encode(&a);
    let mut loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), a.state());
    run(&mut a, "wait*8");
    run(&mut loaded, "wait*8");
    assert_eq!(loaded.state_hash(), a.state_hash());
}

#[test]
fn creatures_shoot_with_the_bow_they_have_ready() {
    // A brute with a bow and arrows ready, the player across the room: its shots are rolled.
    let legend = "\"a\" = { terrain = \"floor\", creature = \"archer\", carry = [\"bow\", \
                  \"arrow*9\"], wield = [\"bow\", \"arrow\"] }\n";
    let mut files = ContentFiles::default();
    let creatures = format!(
        "{CREATURES}\n[[creature]]\nid = \"archer\"\nname = \"kobold archer\"\nglyph = \"k\"\n\
         fg = \"#a0a040\"\nweight = 20000\nfaction = \"kobolds\"\ncombat = \"ranged\"\n\
         range = 6\nstats = {{ strength = 8, dexterity = 16 }}\n"
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", creatures.as_str()),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/vision.toml", VISION),
        (
            "data/materials.toml",
            "[[material]]\nid = \"steel\"\nname = \"steel\"\n",
        ),
        ("data/legend.toml", &format!("{LEGEND}{legend}")),
        (
            "levels/shot.toml",
            "name = \"S\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#@...a#\n#######\n'''\n",
        ),
        (
            "scenarios/shot.toml",
            "name = \"S\"\nstart_level = \"shot\"\nhp = 1000\n",
        ),
    ] {
        files.insert(path, text);
    }
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content.clone(), "shot", None).expect("world");
    let archer = ActorId(1);
    let profile = actor(&w, 1)
        .and_then(|a| a.ranged(&content))
        .expect("a bow");
    // Dexterity 16 (+3), 6 Strength short of the bow's 14: −6 to hit and to damage.
    assert_eq!((profile.to_hit, profile.bonus, profile.crit), (-3, -6, 3));
    let mut strikes = Vec::new();
    let mut blows = 0;
    for _ in 0..30 {
        for e in w.apply(&Action::Wait).events {
            match e {
                Event::Attacked {
                    attacker,
                    kind: AttackKind::Ranged,
                    strike,
                    with,
                    ..
                } if attacker == archer => {
                    assert_eq!(with, content.object_id("arrow"), "an arrow flies");
                    strikes.push(strike);
                }
                Event::Attacked {
                    attacker,
                    kind: AttackKind::Melee,
                    ..
                } if attacker == archer => blows += 1,
                _ => {}
            }
        }
    }
    // Nine arrows, nine shots, then it comes to fight with its fists.
    assert!(
        strikes.len() == 9 && strikes.iter().all(Option::is_some),
        "{strikes:?}"
    );
    assert!(strikes.iter().any(|s| s.is_some_and(Strike::is_hit)));
    assert!(blows > 0, "out of arrows, it fights in melee");
    let archer_now = actor(&w, 1).expect("the archer");
    assert!(archer_now.ammunition(&content).is_none());
    assert_eq!(archer_now.inventory.len(), 1, "the bow is all it has left");
    // Every arrow came down where the player stands, hit or miss (the wall is behind them).
    let (level, _) = w.ground_truth();
    let arrow = content.object_id("arrow").expect("arrow");
    assert_eq!(
        level.objects_at(Pos::new(1, 1)),
        [Stack {
            object: arrow,
            count: 9
        }]
    );
    let hurt = w.state().player().hp < w.state().player().max_hp();
    assert!(hurt, "an arrow found its mark");
    let shown = messages(&w);
    assert!(
        shown
            .iter()
            .any(|m| m == "The kobold archer shoots you."
                || m == "The kobold archer shoots you hard!"),
        "{shown:?}"
    );
}

fn take(world: &mut World, stance: CombatStance) {
    world.apply(&Action::SetStance(stance));
}

#[test]
fn each_stance_changes_to_hit_damage_and_defence_as_its_data_says() {
    let mut w = world_on("#####\n#@..#\n#####\n", "");
    let content = w.content().clone();
    let neutral = w.state().player().melee(&content);
    let dv = w.perception().own_defence();
    for (stance, to_hit, damage, dodge) in [
        (CombatStance::Aggressive, 1, 2, -3),
        (CombatStance::Neutral, 0, 0, 0),
        (CombatStance::Defensive, -2, -1, 2),
        (CombatStance::Coward, -4, -3, 4),
    ] {
        take(&mut w, stance);
        assert_eq!(w.perception().own_stance(), stance);
        let profile = w.state().player().melee(&content);
        assert_eq!(profile.to_hit - neutral.to_hit, to_hit, "{stance}");
        assert_eq!(profile.bonus - neutral.bonus, damage, "{stance}");
        assert_eq!(profile.damage, neutral.damage);
        let shown = w.perception().own_defence();
        assert_eq!(shown, w.defence_of(ActorId(0)).total());
        // The Defence the attacks roll against: Dodge stacks, Parry only if it beats the weapon.
        let parry = i32::from(stance == CombatStance::Defensive);
        assert_eq!(shown - dv, dodge + parry, "{stance}");
    }
}

#[test]
fn stance_parry_counts_only_when_it_beats_the_weapons() {
    let mut w = world_on("#####\n#@..#\n#####\n", "");
    let content = w.content().clone();
    let staff = content.object_id("staff").expect("staff");
    let slot = content.slot_id("main_hand").expect("slot");
    let me = w.state.actors.get_mut(&ActorId(0)).expect("player");
    me.inventory = vec![crate::object::Stack::one(staff)];
    me.equipment.put(&[slot], 0);
    let dv = w.perception().own_defence();
    take(&mut w, CombatStance::Defensive);
    // Dodge +2; the staff's parry of 3 beats the stance's 1.
    assert_eq!(w.perception().own_defence() - dv, 2);
}

#[test]
fn creatures_stay_neutral_and_the_stance_is_the_players() {
    let mut w = world_on("######\n#@.kr#\n######\n", "");
    take(&mut w, CombatStance::Coward);
    for id in [1, 2] {
        assert_eq!(
            actor(&w, id).expect("creature").stance,
            CombatStance::Neutral
        );
    }
}

#[test]
fn a_coward_s_attacks_cost_more_and_other_actions_do_not() {
    let mut w = world_on("#####\n#@r.#\n#####\n", "");
    let price = |w: &World, kind| w.price_of(ActorId(0), kind);
    let base = [
        ActionKind::Melee,
        ActionKind::Shoot,
        ActionKind::Throw,
        ActionKind::Wait,
    ]
    .map(|k| price(&w, k));
    for stance in [CombatStance::Aggressive, CombatStance::Defensive] {
        take(&mut w, stance);
        let same = [
            ActionKind::Melee,
            ActionKind::Shoot,
            ActionKind::Throw,
            ActionKind::Wait,
        ]
        .map(|k| price(&w, k));
        assert_eq!(same, base, "{stance}");
    }
    take(&mut w, CombatStance::Coward);
    let [melee, shoot, throw, wait] = [
        ActionKind::Melee,
        ActionKind::Shoot,
        ActionKind::Throw,
        ActionKind::Wait,
    ]
    .map(|k| price(&w, k));
    assert_eq!(melee, base[0] * 5 / 4);
    assert_eq!(shoot, base[1] * 5 / 4);
    assert_eq!(throw, base[2] * 5 / 4);
    assert_eq!(wait, base[3]);
    // Attacks drawing on a weapon skill are priced by `attack_price`, which applies it too.
    let blow = (AttackKind::Melee, None);
    assert_eq!(w.attack_price(ActorId(0), ActionKind::Melee, blow), melee);
    assert_eq!(
        w.price_of(ActorId(0), ActionKind::Reload),
        price(&world_on("#####\n#@r.#\n#####\n", ""), ActionKind::Reload),
        "reloading is not an attack"
    );
}

#[test]
fn a_cowards_flight_is_cheaper_and_quieter_only_away_from_adjacent_hostiles() {
    // The brute (hostile) stands east of the player; the rat in the corner is out of reach.
    let mut w = world_on("#######\n#.....#\n#..@k.#\n#.....#\n#######\n", "");
    let step_to = |w: &World, to: Pos| {
        let terrain = w.state.current().terrain.get(to).copied().expect("cell");
        let def = w.content.terrain(terrain);
        let fleeing = w.flees(ActorId(0), to);
        (fleeing, w.step_price(ActorId(0), def, fleeing))
    };
    let (west, north, south_west) = (Pos::new(2, 2), Pos::new(3, 1), Pos::new(2, 3));
    let (_, plain) = step_to(&w, west);
    // Neutral never flees, whatever stands next to it.
    assert_eq!(step_to(&w, west), (false, plain));
    take(&mut w, CombatStance::Coward);
    assert_eq!(step_to(&w, west), (true, plain * 4 / 5));
    assert_eq!(step_to(&w, south_west), (true, plain * 4 / 5));
    // Stepping to a cell still next to the brute is not a flight.
    assert_eq!(step_to(&w, north), (false, plain));
    // Nor is a step with no hostile beside the player.
    let mut alone = world_on("#######\n#.....#\n#..@..#\n#.....#\n#######\n", "");
    take(&mut alone, CombatStance::Coward);
    assert_eq!(step_to(&alone, west), (false, plain));
    // A defensive stance has no flight to speak of.
    take(&mut w, CombatStance::Defensive);
    assert_eq!(step_to(&w, west), (false, plain));
}

#[test]
fn a_cowards_flight_makes_half_the_noise() {
    let loudness = |stance, fleeing| {
        // Footsteps are silent in the test creatures: give the player some, and the brute ears.
        let creatures = CREATURES
            .replace("weight = 70000", "weight = 70000\nfootsteps = 100")
            .replace("base_hp = 20\n", "base_hp = 20\nhearing = 10\n");
        let mut files = files(
            "#######\n#.....#\n#..@k.#\n#.....#\n#######\n",
            "ambient = [255, 255, 255, 0]",
            "",
        );
        files.insert("data/creatures.toml", &creatures);
        let content = Arc::new(Content::load(&files).expect("test content is valid"));
        let mut w = World::new(content, "fight", None).expect("world");
        take(&mut w, stance);
        let to = Pos::new(2, 2);
        w.state.actors.get_mut(&ActorId(0)).expect("player").pos = to;
        w.state
            .actors
            .get_mut(&ActorId(1))
            .expect("brute")
            .heard
            .clear();
        w.make_step_noise(ActorId(0), to, fleeing);
        let heard = &w.state.actors[&ActorId(1)].heard;
        heard.first().map_or(0, |h| h.loudness)
    };
    let loud = loudness(CombatStance::Neutral, true);
    assert!(loud > 0);
    assert_eq!(loudness(CombatStance::Coward, false), loud, "not fleeing");
    let quiet = loudness(CombatStance::Coward, true);
    // Half as loud where it is made, so less where it is heard.
    assert!(quiet > 0 && quiet < loud, "{quiet} {loud}");
}

#[test]
fn taking_a_stance_takes_no_time_says_so_and_is_recorded_and_saved() {
    let mut w = world_on("######\n#@.kr#\n######\n", "hp = 1000");
    let (turn, time, energy) = (w.state.turn, w.state.time, w.state().player().energy);
    let outcome = run(&mut w, "stance:aggressive").remove(0);
    assert_eq!(
        (w.state.turn, w.state.time, w.state().player().energy),
        (turn, time, energy)
    );
    assert!(outcome.events.is_empty());
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You take an aggressive stance.")
    );
    for (text, said) in [
        ("stance:neutral", "You take a neutral stance."),
        ("stance:defensive", "You take a defensive stance."),
        ("stance:coward", "You take a cowardly stance."),
    ] {
        run(&mut w, text);
        assert_eq!(messages(&w).last().map(String::as_str), Some(said));
    }
    assert!("stance:brave".parse::<Action>().is_err());
    // Recorded in a replay, which verifies; and saved with the world.
    let content = w.content().clone();
    let mut fresh = World::new(content.clone(), "fight", None).expect("world");
    let mut replay = Replay::new(&fresh);
    for action in parse_actions("stance:aggressive wait stance:coward wait").expect("actions") {
        fresh.apply(&action);
        replay.push(&action, &fresh);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), fresh.state_hash());
    assert_eq!(verified.state().player().stance, CombatStance::Coward);
    let loaded = crate::save::decode(&crate::save::encode(&fresh), content).expect("loads");
    assert_eq!(loaded.state(), fresh.state());
    assert_eq!(loaded.perception().own_stance(), CombatStance::Coward);
    // The stance is part of the hash: the same actions without it differ.
    let mut other = World::new(fresh.content().clone(), "fight", None).expect("world");
    run(&mut other, "wait wait");
    assert_ne!(other.state_hash(), fresh.state_hash());
}

#[test]
fn an_aggressive_player_hits_more_and_a_coward_less() {
    // Over many blows at the same Defence, to-hit shows in the hit count.
    let hits = |stance| {
        let mut w = world_on(
            "#####\n#@t.#\n#####\n",
            "hp = 1000\nstats = { strength = 10 }",
        );
        take(&mut w, stance);
        let mut hit = 0;
        for _ in 0..150 {
            let outcome = run(&mut w, "move:E").remove(0);
            hit += outcome
                .events
                .iter()
                .filter(|e| matches!(e, Event::Attacked { strike: Some(s), .. } if s.is_hit()))
                .count();
        }
        hit
    };
    let (aggressive, neutral, coward) = (
        hits(CombatStance::Aggressive),
        hits(CombatStance::Neutral),
        hits(CombatStance::Coward),
    );
    assert!(
        aggressive > neutral && neutral > coward,
        "{aggressive} {neutral} {coward}"
    );
}
