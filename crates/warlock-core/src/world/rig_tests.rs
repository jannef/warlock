//! Rigging traps (spec §8 "Traps": "Rigging traps"; #1423, the craftsperson design §7.6): the
//! craftsperson's Trapwright rigs a tripwire, stakes, or a pitfall beside them; the refusals,
//! the wall rule, the check and its natural 2, the kill credited, the sidestep of one's own
//! trap, the menu, saves and replays.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::content::{ClassActive, ContentFiles, Trap};
use crate::object::Stack;
use crate::replay::Replay;
use crate::rng::Rng;

/// A lit hall on the built-in content, the player at `@` as a human craftsperson with
/// Trapwright, every score 10, born under no sign (no gift of theirs saves them); `r` a rat.
fn world(map: &str) -> World {
    world_with(map, true)
}

/// [`world`], with or without Trapwright.
fn world_with(map: &str, trapwright: bool) -> World {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    let legend = r#"
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"+" = { terrain = "door_closed" }
"~" = { terrain = "shallow_water" }
"#;
    files.insert(
        "levels/rig_hall.toml",
        &format!("name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n{legend}"),
    );
    files.insert(
        "scenarios/rig_hall.toml",
        "name = \"Hall\"\nstart_level = \"rig_hall\"\nseed = 7\ncharacter = { race = \"human\", \
         gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let w = World::new(content.clone(), "rig_hall", None).expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    if trapwright {
        w.player_mut().talents.take("trapwright", None);
    }
    w.player_mut().stats = crate::stats::Stats::new(
        crate::stats::PerStat::splat(10),
        crate::stats::PerStat::splat(40),
    );
    w
}

const HALL: &str = "#######\n#.....#\n#.@...#\n#.....#\n#######\n";

/// East of the player in [`HALL`].
const EAST: Pos = Pos::new(3, 2);

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn texts(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn last(world: &World) -> String {
    texts(world).last().cloned().unwrap_or_default()
}

fn carry(world: &mut World, object: &str, count: u32) {
    let id = world.content.object_id(object).expect(object);
    let content = world.content.clone();
    crate::object::add_to_inventory(world.state.inventory_mut(), Stack::new(id, count), &content);
}

/// A billet of wood and a knife.
fn billet(world: &mut World) {
    carry(world, "knife", 1);
    let wood = world.content.material_id("wood").expect("wood");
    let stack = crate::piece::make(&world.content, wood, 1).expect("a billet");
    world.player_mut().inventory.push(stack);
}

fn count_of(world: &World, object: &str) -> u32 {
    let id = world.content.object_id(object).expect(object);
    (world.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// The world after `action`, its gameplay stream set to the first whose rig comes out with
/// `said` among what was said.
fn fated(world: &World, action: &str, said: &str) -> World {
    for seed in 0..2_000 {
        let mut w = world.clone();
        w.state.rng = Rng::gameplay(seed);
        run(&mut w, action);
        if texts(&w).iter().any(|t| t.contains(said)) {
            return w;
        }
    }
    panic!("no stream rigs `{action}` with `{said}`");
}

#[test]
fn a_tripwire_takes_up_to_three_arrows_and_is_known_and_rigged() {
    let mut w = world(HALL);
    carry(&mut w, "arrow", 5);
    let start = w.state.time;
    let w = fated(&w, "rig:tripwire:E", "You rig a tripwire across the way.");
    let trap = w.state.current().traps[&EAST];
    assert!(trap.known && trap.rigged);
    assert_eq!(trap.charges, Some(3));
    assert_eq!(w.content.trap(trap.kind).id, "rigged_arrows");
    assert_eq!(count_of(&w, "arrow"), 2);
    assert!(w.state.time.since(start) >= crate::time::Duration::seconds(300));
    assert_eq!(w.perception().known_traps(), vec![(EAST, trap.kind)]);
    assert!(w.perception().rigged(EAST));
    let truth = crate::dump::layer_rows(&w, crate::dump::Layer::Truth).join("\n");
    assert!(
        truth.contains("trap rigged_arrows at (3, 2): 3 charges (rigged)"),
        "{truth}"
    );
}

#[test]
fn quick_hands_rigs_in_half_the_time() {
    let took = |quick: bool| {
        let mut w = world(HALL);
        if quick {
            w.player_mut().talents.take("quick_hands", None);
        }
        carry(&mut w, "arrow", 3);
        let listed = w.perception().rigs()[0].time;
        let start = w.state.time;
        let w = fated(&w, "rig:tripwire:E", "You rig a tripwire");
        (listed, w.state.time.since(start))
    };
    let (plain_listed, plain) = took(false);
    let (quick_listed, quick) = took(true);
    assert_eq!(quick_listed.as_seconds() * 2, plain_listed.as_seconds());
    assert_eq!(quick.as_seconds() * 2, plain.as_seconds());
}

#[test]
fn two_arrows_give_two_charges() {
    let mut w = world(HALL);
    carry(&mut w, "arrow", 2);
    let w = fated(&w, "rig:tripwire:E", "You rig a tripwire");
    assert_eq!(w.state.current().traps[&EAST].charges, Some(2));
    assert_eq!(count_of(&w, "arrow"), 0);
}

#[test]
fn stakes_take_a_billet_and_a_knife_and_a_pitfall_a_digging_tool() {
    let mut w = world(HALL);
    billet(&mut w);
    let w2 = fated(&w, "rig:stakes:E", "You set sharpened stakes");
    let trap = w2.state.current().traps[&EAST];
    assert_eq!(w2.content.trap(trap.kind).id, "rigged_stakes");
    assert_eq!(trap.charges, Some(2));
    assert!(
        w2.state
            .player()
            .inventory
            .iter()
            .all(|s| { w2.content.object(s.object).id != crate::piece::ID })
    );
    // A pitfall: the pick-axe ready in the Tool slot, kept.
    let mut w = world(HALL);
    carry(&mut w, "pick_axe", 1);
    let pick = (w.state.player().inventory.iter())
        .position(|s| w.content.object(s.object).id == "pick_axe")
        .expect("the pick");
    run(&mut w, &format!("equip:tool:{pick}"));
    let w = fated(&w, "rig:pitfall:W", "You dig a pit");
    let west = Pos::new(1, 2);
    let trap = w.state.current().traps[&west];
    assert_eq!(w.content.trap(trap.kind).id, "rigged_pit");
    assert_eq!(trap.charges, Some(3));
    assert_eq!(count_of(&w, "pick_axe"), 1);
}

#[test]
fn a_failed_check_rigs_nothing_and_keeps_the_arrows() {
    let mut w = world(HALL);
    carry(&mut w, "arrow", 3);
    let w = fated(&w, "rig:tripwire:E", "The rig will not hold.");
    assert!(!w.state.current().traps.contains_key(&EAST));
    assert_eq!(count_of(&w, "arrow"), 3);
}

#[test]
fn a_natural_two_springs_it_on_the_rigger_and_leaves_it_a_charge_less() {
    let mut w = world(HALL);
    billet(&mut w);
    let max = w.state.player().max_hp(&w.content);
    w.player_mut().hp = max;
    let w = fated(&w, "rig:stakes:E", "A stake springs loose as you set it!");
    let trap = w.state.current().traps[&EAST];
    assert_eq!(trap.charges, Some(1));
    assert!(trap.rigged && trap.known);
    assert!(
        texts(&w)
            .iter()
            .any(|t| t.contains("Sharpened stakes spring up!"))
    );
}

#[test]
fn refusals_take_no_time() {
    let refused = |map: &str, setup: &dyn Fn(&mut World), action: &str, why: &str| {
        let mut w = world(map);
        setup(&mut w);
        let before = w.state.time;
        run(&mut w, action);
        assert_eq!(w.state.time, before, "{action}");
        assert!(last(&w).contains(why), "{action}: {}", last(&w));
    };
    let arrows = |w: &mut World| carry(w, "arrow", 3);
    refused(HALL, &arrows, "rig:tripwire", "Which way?");
    refused(HALL, &arrows, "rig:snare:E", "There is no rig `snare`.");
    refused(
        HALL,
        &|_| {},
        "rig:tripwire:E",
        "You have nothing to rig it with.",
    );
    refused(
        HALL,
        &|_| {},
        "rig:pitfall:E",
        "You have no digging tool ready.",
    );
    // A wall, a door, water, someone seen there.
    refused(
        "#####\n#@#.#\n#####\n",
        &arrows,
        "rig:tripwire:E",
        "no room",
    );
    refused(
        "#####\n#@+.#\n#####\n",
        &arrows,
        "rig:tripwire:E",
        "a way through",
    );
    refused("#####\n#@~.#\n#####\n", &arrows, "rig:tripwire:E", "water");
    refused(
        "#####\n#@r.#\n#####\n",
        &arrows,
        "rig:tripwire:E",
        "in the way",
    );
    // A tripwire with no wall within two cells in a straight line.
    let open = "#########\n#.......#\n#.......#\n#.......#\n#...@...#\n#.......#\n#.......#\n\
                #.......#\n#########\n";
    refused(open, &arrows, "rig:tripwire:E", "no wall near");
    // A known trap there already.
    let mut w = world(HALL);
    carry(&mut w, "arrow", 6);
    let mut w = fated(&w, "rig:tripwire:E", "You rig a tripwire");
    let before = w.state.time;
    run(&mut w, "rig:tripwire:E");
    assert_eq!(w.state.time, before);
    assert_eq!(last(&w), "There is a trap there already.");
}

#[test]
fn without_the_talent_or_the_rigs_it_is_refused() {
    let mut w = world_with(HALL, false);
    carry(&mut w, "arrow", 3);
    run(&mut w, "rig:tripwire:E");
    assert_eq!(last(&w), "You have no such ability.");
    assert!(w.perception().rigs().is_empty());
    // Without `rigs.toml`: not usable yet.
    let mut files = ContentFiles::builtin();
    files.remove("data/rigs.toml");
    let content = Arc::new(Content::load(&files).expect("content"));
    let me = World::new(content.clone(), "showcase", None).expect("world");
    let mut actor = me.state.player().clone();
    actor.talents.take("trapwright", None);
    assert_eq!(
        Ability::Class(ClassActive::Rig).unavailable(&actor, &content),
        Some(Unavailable::NotYet)
    );
}

#[test]
fn the_menu_and_the_rig_list_say_what_is_lacking() {
    let mut w = world(HALL);
    let view = w.perception();
    let own = (view.abilities().into_iter())
        .find(|a| a.ability == Ability::Class(ClassActive::Rig))
        .expect("Trapwright in the menu");
    assert_eq!(own.unavailable, None);
    let rows = view.rigs();
    let lacking: Vec<_> = rows.iter().map(|r| (r.id.as_str(), r.lacking)).collect();
    assert_eq!(
        lacking,
        [
            ("tripwire", Some("nothing to rig it with")),
            ("stakes", Some("no knife")),
            ("pitfall", Some("no digging tool ready")),
        ]
    );
    assert_eq!(rows[0].takes, "up to 3 arrows");
    assert_eq!(rows[1].takes, "a wood billet and a knife");
    assert_eq!(rows[2].takes, "a digging tool ready");
    carry(&mut w, "arrow", 1);
    assert_eq!(w.perception().rigs()[0].lacking, None);
}

#[test]
fn a_rigged_traps_kill_is_the_players_and_a_plain_ones_is_not() {
    for rigged in [true, false] {
        let mut w = world("#######\n#.@.r.#\n#######\n");
        let rat = (w.state.actors.values())
            .find(|a| a.id != w.state.player)
            .map(|a| a.id)
            .expect("the rat");
        let at = Pos::new(4, 1);
        let kind = w.content.trap_id("rigged_stakes").expect("stakes");
        w.state.current_mut().traps.insert(
            at,
            Trap {
                kind,
                known: true,
                charges: Some(2),
                dir: None,
                rigged,
            },
        );
        let before = w.state.experience;
        // Spring it until the rat is dead (its blows may miss).
        for seed in 0..200 {
            if !w.state.actors.contains_key(&rat) {
                break;
            }
            if let Some(a) = w.state.actors.get_mut(&rat) {
                a.hp = 1;
            }
            w.state.rng = Rng::gameplay(seed);
            w.state.current_mut().traps.entry(at).or_insert(Trap {
                kind,
                known: true,
                charges: Some(2),
                dir: None,
                rigged,
            });
            w.spring_trap(at, rat, &mut Vec::new());
        }
        assert!(!w.state.actors.contains_key(&rat), "the rat dies");
        assert_eq!(w.state.experience > before, rigged, "rigged {rigged}");
    }
}

#[test]
fn ones_own_trap_springs_on_the_rigger_as_a_trap() {
    let mut w = world(HALL);
    let kind = w.content.trap_id("rigged_stakes").expect("stakes");
    let here = w.player_pos();
    w.state.current_mut().traps.insert(
        here,
        Trap {
            kind,
            known: true,
            charges: Some(2),
            dir: None,
            rigged: true,
        },
    );
    let player = w.state.player;
    w.player_mut().hp = 1;
    let before = w.state.experience;
    for seed in 0..200 {
        if w.death().is_some() {
            break;
        }
        w.state.rng = Rng::gameplay(seed);
        w.state.current_mut().traps.entry(here).or_insert(Trap {
            kind,
            known: true,
            charges: Some(2),
            dir: None,
            rigged: true,
        });
        w.spring_trap(here, player, &mut Vec::new());
    }
    let death = w.death().expect("the rigger dies of it");
    assert_eq!(death.cause, Some(crate::world::Cause::Trap(kind)));
    assert_eq!(w.state.experience, before);
}

#[test]
fn rigging_is_deterministic_and_saves() {
    let mut w = world(HALL);
    carry(&mut w, "arrow", 4);
    let content = w.content().clone();
    let mut again = w.clone();
    let actions = "rig:tripwire:E wait rig:tripwire:W";
    run(&mut w, actions);
    run(&mut again, actions);
    assert_eq!(again.state_hash(), w.state_hash());
    let saved = crate::save::decode(&crate::save::encode(&w), content).expect("loads");
    assert_eq!(saved.state(), w.state());
}

/// A replay of a craftsperson of the showcase raised to Trapwright, rigging, replays to the same
/// hash.
#[test]
fn rigging_replays() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: crate::character::CharacterSpec =
        "human,male,craftsperson".parse().expect("a character");
    let mut w =
        World::with_character(content.clone(), "showcase", Some(3), Some(&spec)).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "xp:3000000 levelup:auto rig:pitfall:N rig:stakes:S use:trapwright";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}
