//! Burden in the world (spec §8 "Burden"): carrying limits from Strength, Toughness, and size;
//! every action slowed ×1.25 Burdened and ×1.5 Strained, overworld journeys included, exactly over
//! any run; overloaded actors cannot move but do everything else; Dexterity capped in Defence;
//! louder steps; dwarves shrugging off Burdened; messages when the burden changes; creatures by
//! the same rules; nothing new saved.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::ContentFiles;
use crate::noise::Act;
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
id = "packed_earth"
name = "packed earth"
glyph = ","
fg = "#806040"
move_cost = 90

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

[[terrain]]
id = "plains"
name = "plains"
glyph = "."
fg = "#a0a060"
travel_minutes = 45

[[terrain]]
id = "road"
name = "road"
glyph = "="
fg = "#a08060"
travel_minutes = 20

[[terrain]]
id = "tower"
name = "tower"
glyph = "T"
fg = "#c0c0c0"
stairs = "down"
travel_minutes = 30
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "wildlife"
name = "wildlife"
"##;

// The walker is small: it carries three quarters of what the player does with the same scores.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000
footsteps = 60
hearing = 10

[[creature]]
id = "walker"
name = "walker"
glyph = "w"
fg = "#806040"
weight = 20000
size = "small"
faction = "wildlife"
footsteps = 60
hearing = 10
stats = { strength = 10, toughness = 10 }
"##;

const OBJECTS: &str = r##"
[[object]]
id = "anvil"
name = "anvil"
glyph = "&"
fg = "#808080"
material = "iron"
weight = 10000
value = 0

[[object]]
id = "ingot"
name = "ingot"
glyph = "="
fg = "#808080"
material = "iron"
weight = 1000
value = 0
stackable = true

[[object]]
id = "mail"
name = "mail shirt"
glyph = "["
fg = "#c0c0c0"
material = "iron"
weight = 9000
value = 0
slot = "body"
armour = 5
max_dex = 2
"##;

const SLOTS: &str = r##"
[[slot]]
id = "body"
name = "Body"
action = "body_armor"
mark = "being worn"
"##;

const SIZES: &str = r##"
[[size]]
id = "small"
name = "Small"
carry = 75

[[size]]
id = "medium"
name = "Medium"
carry = 100
"##;

// With a female character (Dexterity +1) and the commoner: a human St 10 To 10, a dwarf St 14 To
// 16 (Dexterity 18), a halfling St 8 To 12.
const RACES: &str = r##"
base_potential = 15

[[race]]
id = "human"
name = "Human"
description = "Everywhere."
vision = "normal"
age = [20, 30]

[[race]]
id = "dwarf"
name = "Dwarf"
description = "Stout."
vision = "normal"
age = [40, 50]
stats = { strength = 4, dexterity = 7, toughness = 6 }
ignores_burdened = true

[[race]]
id = "halfling"
name = "Halfling"
description = "Small."
vision = "normal"
age = [20, 30]
size = "small"
stats = { strength = -2, toughness = 2 }
"##;

const CLASSES: &str = r##"
[[class]]
id = "commoner"
name = "Commoner"
description = "Everyday folk."
"##;

const LEGEND: &str = r##"
[legend]
" " = "void"
"." = "floor"
"#" = "wall"
"," = "packed_earth"
"##;

// Five anvils and five ingots on the player's cell, a row of packed earth (a step 90 % of a
// normal one), and stairs down.
const YARD: &str = r##"
name = "Yard"
map = '''
##########
#@.......#
#,,,,,,,,#
#>.......#
##########
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["anvil", "anvil", "anvil", "anvil", "anvil", "ingot*5"] }
">" = { terrain = "stairs_down", to = "cellar" }
"##;

const CELLAR: &str = r##"
name = "Cellar"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "yard" }
"##;

// The road east to a tower, plains around.
const WILDS: &str = r##"
name = "Wilds"
kind = "overworld"
map = '''
.......
.@==T..
.......
'''
[legend]
"." = "plains"
"=" = "road"
"@" = { terrain = "road", spawn = "player" }
"T" = { terrain = "tower", to = "keep" }
"##;

const KEEP: &str = r##"
name = "Keep"
map = '''
#####
#<..#
#####
'''
[legend]
"<" = { terrain = "stairs_up", to = "wilds" }
"##;

// The walker paces its own row two cells below the player.
const HALL: &str = r##"
name = "Hall"
map = '''
###########
#@........#
#.........#
#w........#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"w" = { terrain = "floor", creature = "walker", route = "E8 W8" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/objects.toml", OBJECTS),
        ("data/slots.toml", SLOTS),
        ("data/sizes.toml", SIZES),
        ("data/races.toml", RACES),
        ("data/classes.toml", CLASSES),
        (
            "data/materials.toml",
            "[[material]]\nid = \"iron\"\nname = \"iron\"\n",
        ),
        ("data/legend.toml", LEGEND),
        // Body armor takes 18 s to strap on at a normal pace.
        ("data/actions.toml", "body_armor = 3000\n"),
        ("levels/yard.toml", YARD),
        ("levels/cellar.toml", CELLAR),
        ("levels/wilds.toml", WILDS),
        ("levels/keep.toml", KEEP),
        ("levels/hall.toml", HALL),
    ] {
        files.insert(path, text);
    }
    // St 10 To 10: a limit of 45 kg (burdened over 15 kg, strained over 30 kg); Dexterity 18 (+4).
    for name in ["yard", "wilds", "hall"] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"{name}\"\nstart_level = \"{name}\"\nhp = 1000\n\
                 stats = {{ strength = 10, toughness = 10, dexterity = 18 }}\n"
            ),
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

fn as_character(scenario: &str, spec: &str) -> World {
    let spec: CharacterSpec = spec.parse().expect("a character");
    World::with_character(content(), scenario, None, Some(&spec)).expect("world")
}

/// Actor `id` carries `kg` kilograms of ingots and nothing else.
fn carry(w: &mut World, id: ActorId, kg: u32) {
    let ingot = w.content().object_id("ingot").expect("ingot");
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.inventory = if kg == 0 {
        Vec::new()
    } else {
        vec![Stack::new(ingot, kg)]
    };
    actor.equipment = Default::default();
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| w.apply(a))
        .collect()
}

fn total(outs: &[Outcome]) -> u64 {
    outs.iter().map(|o| o.elapsed.as_seconds()).sum()
}

/// Seconds `actions` took, each having to take a turn.
fn took(w: &mut World, actions: &str) -> u64 {
    let outs = run(w, actions);
    for (out, action) in outs.iter().zip(actions.split_whitespace()) {
        assert!(out.took_turn, "{action}: {:?}", last_message(w));
    }
    total(&outs)
}

fn last_message(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map_or(String::new(), |m| m.text.clone())
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn burden(w: &World) -> Burden {
    w.perception().own_load().burden()
}

#[test]
fn limits_follow_strength_toughness_and_size() {
    let w = world("yard");
    let load = w.perception().own_load();
    assert_eq!((load.carried, load.limit), (0, 45_000), "St 10 To 10");
    assert_eq!(load.thresholds().map(|(_, g)| g), [15_000, 30_000, 45_000]);
    assert!(!load.ignores_burdened);
    for (spec, limit, ignores) in [
        ("human,female,commoner", 45_000, false),
        ("dwarf,female,commoner", 66_000, true),
        ("halfling,female,commoner", 31_500, false),
    ] {
        let w = as_character("yard", spec);
        let load = w.perception().own_load();
        assert_eq!(
            (load.limit, load.ignores_burdened),
            (limit, ignores),
            "{spec}"
        );
        assert_eq!(w.actor_load(w.player_id()), Some(load));
    }
    // A creature by its type's size: the small walker, St 10 To 10, carries 33.75 kg.
    let w = world("hall");
    let walker = w.actor_load(ActorId(1)).expect("the walker");
    assert_eq!(walker.limit, 33_750);
    assert_eq!(walker.burden(), Burden::Unburdened);
}

#[test]
fn the_player_is_told_when_their_burden_changes_and_only_then() {
    let mut w = world("yard");
    let burdens = |w: &mut World, actions: &str| -> Vec<(Burden, String)> {
        run(w, actions);
        vec![(burden(w), last_message(w))]
    };
    // Ten kilograms: still unburdened, nothing said of it.
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Unburdened, "You pick up an anvil.".into())]
    );
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Burdened, "You are burdened.".into())]
    );
    // Thirty kilograms is still burdened: no news.
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Burdened, "You pick up an anvil.".into())]
    );
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Strained, "You strain under your load.".into())]
    );
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Overloaded, "You carry too much to move.".into())]
    );
    // Picking up is always allowed, even overloaded.
    assert_eq!(
        burdens(&mut w, "pickup:a"),
        [(Burden::Overloaded, "You pick up 5 ingots.".into())]
    );
    assert_eq!(
        burdens(&mut w, "drop:ab"),
        [(Burden::Strained, "You strain under your load.".into())]
    );
    assert_eq!(
        burdens(&mut w, "drop:a"),
        [(Burden::Burdened, "You are burdened.".into())]
    );
    assert_eq!(
        burdens(&mut w, "drop"),
        [(Burden::Unburdened, "You are no longer burdened.".into())]
    );
    // The news comes right after what changed it, and each once.
    let said = texts(&w);
    let count = |text: &str| said.iter().filter(|t| *t == text).count();
    assert_eq!(count("You are burdened."), 2);
    assert_eq!(count("You strain under your load."), 2);
    assert_eq!(count("You are no longer burdened."), 1);
    let at = said.iter().position(|t| t == "You strain under your load.");
    assert_eq!(said[at.expect("said") - 1], "You pick up an anvil.");
}

#[test]
fn burden_slows_every_action_exactly() {
    let mut w = world("yard");
    let player = w.player_id();
    assert_eq!(
        took(&mut w, "wait move:E move:W"),
        18,
        "unburdened: 6 s each"
    );
    // Burdened (20 kg): 7.5 s each, so eight waits take exactly a minute and no energy is left.
    carry(&mut w, player, 20);
    let outs = run(&mut w, "wait*8");
    let each: Vec<u64> = outs.iter().map(|o| o.elapsed.as_seconds()).collect();
    assert_eq!(each, [8, 7, 8, 7, 8, 7, 8, 7]);
    assert_eq!(w.state().player().energy, 0, "no drift");
    assert_eq!(took(&mut w, "move:E*4"), 30, "steps");
    // Packed earth, 90 %: 6.75 s a step.
    assert_eq!(took(&mut w, "move:SW move:E*3"), 27);
    // The stairs, and back.
    run(&mut w, "move:W*5 move:SW");
    assert_eq!(took(&mut w, "descend ascend"), 15);
    // Strained (40 kg): 9 s each, any action.
    carry(&mut w, player, 40);
    assert_eq!(took(&mut w, "wait*4 move:E move:W descend"), 7 * 9);
    // Putting on body armor (3000: 18 s at normal speed) takes 27 s strained (31 kg and the
    // mail's 9).
    carry(&mut w, player, 31);
    let mail = w.content().object_id("mail").expect("mail");
    w.player_mut().inventory.push(Stack::one(mail));
    assert_eq!(burden(&w), Burden::Strained);
    assert_eq!(took(&mut w, "equip:body:b"), 27);
    // A rest still ends exactly at its goal.
    let start = w.time();
    run(&mut w, "rest:1h");
    assert_eq!(w.time().since(start), Duration::hours(1));
}

#[test]
fn overworld_journeys_take_longer_under_a_load() {
    let minutes = |w: &mut World, actions: &str| took(w, actions) / 60;
    let mut w = world("wilds");
    let player = w.player_id();
    // The road: 20 minutes a step unburdened, 25 burdened, 30 strained.
    assert_eq!(minutes(&mut w, "move:E"), 20);
    carry(&mut w, player, 20);
    assert_eq!(minutes(&mut w, "move:W"), 25);
    carry(&mut w, player, 40);
    assert_eq!(minutes(&mut w, "move:E"), 30);
    // Plains, 45 minutes: 67.5 strained, so two take exactly 135 minutes.
    assert_eq!(took(&mut w, "move:N move:E"), 135 * 60);
    // Waiting and entering a location take their 6 s slowed too.
    assert_eq!(took(&mut w, "wait"), 9);
    run(&mut w, "move:SE");
    assert_eq!(took(&mut w, "descend"), 9);
    assert_eq!(w.state().current_level, "keep");
}

#[test]
fn the_overloaded_cannot_move_but_do_everything_else() {
    let mut w = world("yard");
    let player = w.player_id();
    carry(&mut w, player, 46);
    assert_eq!(burden(&w), Burden::Overloaded);
    let (turn, time, at) = (w.turn(), w.time(), w.player_pos());
    for action in ["move:E", "move:S"] {
        let out = run(&mut w, action).remove(0);
        assert!(!out.took_turn, "{action}");
        assert_eq!(last_message(&w), "You carry too much to move.");
    }
    assert_eq!((w.turn(), w.time(), w.player_pos()), (turn, time, at));
    // Waiting, picking up, and dropping work, as slowly as strained.
    assert_eq!(took(&mut w, "wait pickup:a"), 18);
    assert_eq!(took(&mut w, "drop:b"), 9, "priced at the load as it begins");
    assert_eq!(
        burden(&w),
        Burden::Overloaded,
        "the anvil dropped, the ingots kept"
    );
    assert_eq!(took(&mut w, "drop:a"), 9);
    assert_eq!(burden(&w), Burden::Unburdened);
    assert_eq!(took(&mut w, "move:E"), 6, "free to move again");
    // Neither stairs nor the overworld let them go.
    run(&mut w, "move:W move:S*2");
    carry(&mut w, player, 46);
    let out = run(&mut w, "descend").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last_message(&w), "You carry too much to move.");
    let mut w = world("wilds");
    run(&mut w, "move:E*3");
    carry(&mut w, player, 46);
    for action in ["move:W", "descend"] {
        assert!(!run(&mut w, action).remove(0).took_turn, "{action}");
        assert_eq!(last_message(&w), "You carry too much to move.");
    }
}

#[test]
fn burden_caps_dexterity_in_defence_and_the_lower_cap_wins() {
    let mut w = world("yard");
    let player = w.player_id();
    let dv = |w: &World| w.perception().own_defence();
    // Dexterity 18: +4 unburdened, then at most +3, +1, and 0.
    let mut seen = Vec::new();
    for kg in [0, 16, 31, 46] {
        carry(&mut w, player, kg);
        seen.push(dv(&w));
    }
    assert_eq!(seen, [14, 13, 11, 10]);
    // Mail caps Dexterity at +2 on its own: it wins over Burdened's +3, Strained's +1 over it.
    let mail = w.content().object_id("mail").expect("mail");
    let body = w.content().slot_id("body").expect("body");
    let mut seen = Vec::new();
    for kg in [0, 7, 22, 37] {
        carry(&mut w, player, kg);
        let me = w.player_mut();
        me.inventory.push(Stack::one(mail));
        let item = me.inventory.len() - 1;
        me.equipment.put(&[body], item);
        seen.push((burden(&w), dv(&w)));
    }
    assert_eq!(
        seen,
        [
            (Burden::Unburdened, 17),
            (Burden::Burdened, 17),
            (Burden::Strained, 16),
            (Burden::Overloaded, 15)
        ]
    );
    // The rules see the same Defence.
    assert_eq!(w.defence_of(player).total(), 15);
}

#[test]
fn steps_under_a_load_are_louder() {
    let mut w = world("hall");
    let (player, walker) = (w.player_id(), ActorId(1));
    // The walker stands two cells below the player: a step loses 20 on the way.
    let heard_by = |w: &mut World, listener: ActorId, maker: ActorId| {
        w.state
            .actors
            .get_mut(&listener)
            .expect("listener")
            .heard
            .clear();
        let at = w.state.actors[&maker].pos;
        w.make_noise(maker, Act::Step(at));
        w.state.actors[&listener]
            .heard
            .iter()
            .map(|h| h.loudness)
            .collect::<Vec<_>>()
    };
    let mut loudness = Vec::new();
    for kg in [0, 16, 31] {
        carry(&mut w, player, kg);
        loudness.extend(heard_by(&mut w, walker, player));
    }
    assert_eq!(loudness, [40, 55, 70], "60, 75, and 90 where made");
    // The walker by the same rules: strained with 23 kg (small, 33.75 kg at most).
    carry(&mut w, walker, 23);
    assert_eq!(heard_by(&mut w, player, walker), [70]);
}

#[test]
fn dwarves_shrug_off_burdened_but_not_strained() {
    let mut w = as_character("yard", "dwarf,female,commoner");
    let player = w.player_id();
    // St 14 To 16: burdened over 22 kg, strained over 44 kg. Dexterity 18: +4.
    let dv = |w: &World| w.perception().own_defence();
    assert_eq!(dv(&w), 14);
    carry(&mut w, player, 30);
    let load = w.perception().own_load();
    assert_eq!(
        (load.burden(), load.penalty()),
        (Burden::Burdened, Burden::Unburdened)
    );
    assert_eq!(took(&mut w, "wait move:E move:W"), 18, "not slowed");
    assert_eq!(dv(&w), 14, "Dexterity not capped");
    carry(&mut w, player, 50);
    assert_eq!(burden(&w), Burden::Strained);
    assert_eq!(took(&mut w, "wait move:E"), 18, "slowed like anyone");
    assert_eq!(dv(&w), 11);
    // Their steps: as quiet as ever while burdened, louder strained.
    let mut w = as_character("hall", "dwarf,female,commoner");
    let mut loudness = Vec::new();
    for kg in [30, 50] {
        carry(&mut w, player, kg);
        w.state
            .actors
            .get_mut(&ActorId(1))
            .expect("walker")
            .heard
            .clear();
        let at = w.player_pos();
        w.make_noise(player, Act::Step(at));
        loudness.extend(w.state.actors[&ActorId(1)].heard.iter().map(|h| h.loudness));
    }
    assert_eq!(loudness, [40, 70]);
    // Anyone else is slowed burdened.
    let mut w = as_character("yard", "human,female,commoner");
    carry(&mut w, player, 20);
    assert_eq!(took(&mut w, "wait*2"), 15);
}

/// How often the walker moved in `outs`.
fn walker_moves(outs: &[Outcome]) -> usize {
    (outs.iter().flat_map(|o| o.events.iter()))
        .filter(|e| matches!(e, Event::ActorMoved { actor, .. } if *actor == ActorId(1)))
        .count()
}

#[test]
fn creatures_carry_by_the_same_rules() {
    // Ten turns of waiting, a minute: the walker steps every 6 s unburdened, every 7.5 s
    // burdened (12 kg of its 33.75), every 9 s strained (23 kg), and not at all overloaded.
    let mut moves = Vec::new();
    for kg in [0, 12, 23, 34] {
        let mut w = world("hall");
        carry(&mut w, ActorId(1), kg);
        moves.push(walker_moves(&run(&mut w, "wait*10")));
    }
    assert_eq!(moves, [10, 8, 7, 0]);
}

#[test]
fn burden_is_derived_never_saved_and_replays() {
    let content = content();
    let mut fresh = World::new(content.clone(), "yard", None).expect("world");
    let mut replay = Replay::new(&fresh);
    for action in parse_actions("pickup:a pickup:a wait move:E drop:a wait").expect("parse") {
        fresh.apply(&action);
        replay.push(&action, &fresh);
    }
    assert_eq!(burden(&fresh), Burden::Unburdened);
    let (played, divergence) = (Replay::from_json(&replay.to_json()))
        .expect("replay parses")
        .play(content.clone())
        .expect("plays");
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), fresh.state_hash());
    // A burdened run saves and loads as it was: its burden follows from what it carries.
    let mut w = world("yard");
    run(&mut w, "pickup:a pickup:a");
    assert_eq!(burden(&w), Burden::Burdened);
    let mut loaded = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(loaded.perception().own_load(), w.perception().own_load());
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(took(&mut loaded, "wait*2"), took(&mut w, "wait*2"));
    assert_eq!(loaded.state_hash(), w.state_hash());
}
