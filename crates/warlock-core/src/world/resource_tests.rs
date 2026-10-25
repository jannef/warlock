//! Alternative resources in the world (spec §8 "Resources"): granting, decay and regeneration
//! exact however time is cut, resting, ending at zero (and the bar going with it), several at
//! once, building from events, spending, abilities' costs, what the player is shown, saves,
//! and replays.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Rest, parse_actions};
use crate::character::CharacterSpec;
use crate::combat::Strike;
use crate::content::{Build, ClassActive, ContentFiles, ResourceId};
use crate::replay::Replay;
use crate::time::Duration;

// A lit hall without creatures; a den with a kobold east of the player.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#@......#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

const DEN: &str = r##"
name = "Den"
ambient = [255, 255, 255, 0]
map = '''
#####
#@k.#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

// Grit builds on everything a fight brings and never changes over time.
const GRIT: &str = "\n[[resource]]\nid = \"grit\"\nname = \"Grit\"\nshort = \"Grit\"\nmax = 1000\n\
                    start = 0\nbuilds = [{ on = \"kill\", amount = 50 }, { on = \"hit\", amount = 1 }, \
                    { on = \"critical\", amount = 10 }, { on = \"struck\", amount = 2 }]\n";

/// The built-in content with the hall, the den, a scenario of each, and Grit; Rage costs 6 Focus
/// when `costly`.
fn content(costly: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let resources = include_str!("../../../../assets/data/resources.toml");
    files.insert("data/resources.toml", &format!("{resources}{GRIT}"));
    files.insert("levels/hall.toml", HALL);
    files.insert("levels/den.toml", DEN);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    files.insert(
        "scenarios/den.toml",
        "name = \"Den\"\nstart_level = \"den\"\nhp = 1000\n",
    );
    files.remove("data/hunger.toml");
    if costly {
        let rage = include_str!("../../../../assets/data/talents/rage.toml");
        let costly = rage.replace(
            "systems = [",
            "resource = { id = \"focus\", cost = 6 }\nsystems = [",
        );
        files.insert("data/talents/rage.toml", &costly);
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(false), scenario, None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn id(w: &World, name: &str) -> ResourceId {
    w.content().resource_id(name).expect(name)
}

/// What the player holds of `name`, if they hold it.
fn held(w: &World, name: &str) -> Option<u16> {
    let r = id(w, name);
    (w.perception().own_resources().iter())
        .find(|o| o.resource == r)
        .map(|o| o.value)
}

fn set(w: &mut World, name: &str, value: u16) {
    let r = id(w, name);
    w.player_mut().resources.set(r, value);
}

fn last(w: &World) -> String {
    (w.messages().iter().map(|m| m.display()).next_back()).unwrap_or_default()
}

#[test]
fn granting_holds_it_at_its_start_and_takes_no_time() {
    let mut w = world("hall");
    assert!(
        w.perception().own_resources().is_empty(),
        "nobody starts with one"
    );
    let outcome = w.apply(&"resource:rage".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(held(&w, "rage"), Some(12));
    assert_eq!(last(&w), "You gain Rage: 12 (dev).");
    // Again: what it holds, if that is more than its start.
    set(&mut w, "rage", 17);
    run(&mut w, "resource:rage");
    assert_eq!(held(&w, "rage"), Some(17));
    set(&mut w, "rage", 3);
    run(&mut w, "resource:rage");
    assert_eq!(held(&w, "rage"), Some(12));
    // Refused for one there is not.
    let outcome = w.apply(&"resource:mirth".parse().expect("action"));
    assert!(!outcome.took_turn);
    assert_eq!(last(&w), "There is no resource `mirth`.");
}

#[test]
fn rage_decays_a_point_a_base_turn_and_ends_at_zero_with_its_bar() {
    let mut w = world("hall");
    run(&mut w, "resource:rage wait");
    assert_eq!(held(&w, "rage"), Some(11), "a wait is a base turn");
    run(&mut w, "wait*10");
    assert_eq!(held(&w, "rage"), Some(1));
    assert!(w.perception().own_resources().len() == 1);
    let outcome = w.apply(&Action::Wait);
    assert!(
        (outcome.events.iter()).any(|e| matches!(e, Event::ResourceEnded { .. })),
        "its end is an event"
    );
    assert_eq!(held(&w, "rage"), None, "it ended, and its bar went with it");
    assert!(w.state.player().resources.is_empty());
    assert!(
        w.messages()
            .iter()
            .any(|m| m.display() == "Your rage subsides."),
        "and the player was told"
    );
    // Granted again, it is back.
    run(&mut w, "resource:rage");
    assert_eq!(held(&w, "rage"), Some(12));
}

#[test]
fn it_makes_no_difference_how_time_is_cut_up() {
    let by = |actions: &str| {
        let mut w = world("hall");
        run(&mut w, "resource:rage resource:ki resource:focus");
        set(&mut w, "ki", 1);
        set(&mut w, "focus", 2);
        run(&mut w, actions);
        let state = w.state.player().resources.clone();
        (state, w.time())
    };
    // 45 seconds: by waits of 6 and a skip-free half turn's worth of rest are not comparable,
    // so cut 60 seconds by waits, and by two rests of 30 seconds each.
    let waits = by("wait*10");
    let rests = by("rest:30s rest:30s");
    assert_eq!(waits.1, rests.1, "the same time passed");
    let get =
        |pools: &crate::resource::Pools, w: &World, n: &str| pools.get(id(w, n)).map(|p| p.value);
    let w = world("hall");
    assert_eq!(get(&waits.0, &w, "rage"), Some(2));
    assert_eq!(
        get(&rests.0, &w, "rage"),
        Some(2),
        "rage decays while resting too"
    );
    // Ki: a point every 18 s, resting too, from 1: 60 s give 3, and the progress toward the
    // next is the same.
    assert_eq!(get(&waits.0, &w, "ki"), Some(4));
    assert_eq!(waits.0.get(id(&w, "ki")), rests.0.get(id(&w, "ki")));
    // Focus comes back a point a base turn, but not while resting.
    assert_eq!(
        get(&waits.0, &w, "focus"),
        Some(10),
        "full after 8 base turns"
    );
    assert_eq!(
        get(&rests.0, &w, "focus"),
        Some(2),
        "none back while resting"
    );
    // One long rest and many short waits end the rage at the same moment.
    let mut w = world("hall");
    run(&mut w, "resource:rage");
    w.apply(&Action::Rest(Rest::For(Duration::seconds(71))));
    assert_eq!(held(&w, "rage"), Some(1));
    w.apply(&Action::Rest(Rest::For(Duration::seconds(1))));
    assert_eq!(held(&w, "rage"), None, "gone at 72 seconds exactly");
}

#[test]
fn a_regenerating_resource_stays_at_its_maximum_and_never_ends() {
    let mut w = world("hall");
    run(&mut w, "resource:focus");
    assert_eq!(held(&w, "focus"), Some(10), "it starts full");
    run(&mut w, "wait*5");
    assert_eq!(held(&w, "focus"), Some(10));
    assert_eq!(
        w.state
            .player()
            .resources
            .get(id(&w, "focus"))
            .map(|p| p.progress),
        Some(0)
    );
    // Spent to 0 it sits there, held, and comes back.
    let player = w.state.player;
    let focus = id(&w, "focus");
    assert!(w.spend_resource(player, focus, 10, &mut Vec::new()));
    assert_eq!(held(&w, "focus"), Some(0), "held at 0");
    run(&mut w, "wait*3");
    assert_eq!(held(&w, "focus"), Some(3));
}

#[test]
fn spending_is_refused_when_short_and_ends_one_that_ends_at_zero() {
    let mut w = world("hall");
    let player = w.state.player;
    let (rage, focus) = (id(&w, "rage"), id(&w, "focus"));
    let mut events = Vec::new();
    assert!(!w.spend_resource(player, rage, 0, &mut events), "not held");
    run(&mut w, "resource:rage resource:focus");
    assert!(
        !w.spend_resource(player, rage, 13, &mut events),
        "too little"
    );
    assert_eq!(held(&w, "rage"), Some(12), "nothing spent");
    assert!(
        w.spend_resource(player, rage, 0, &mut events),
        "0 asks only that it is held"
    );
    assert!(w.spend_resource(player, focus, 4, &mut events));
    assert_eq!(held(&w, "focus"), Some(6));
    assert!(w.spend_resource(player, rage, 12, &mut events));
    assert_eq!(held(&w, "rage"), None, "spent to 0, it ends");
    assert!(matches!(
        events.as_slice(),
        [Event::ResourceEnded { resource, .. }] if *resource == rage
    ));
    // Ending one leaves the others.
    assert_eq!(held(&w, "focus"), Some(6));
}

#[test]
fn several_resources_are_held_at_once_in_resource_order() {
    let mut w = world("hall");
    run(&mut w, "resource:inspiration resource:rage resource:focus");
    let order: Vec<String> = (w.perception().own_resources().iter())
        .map(|r| w.content().resource(r.resource).id.clone())
        .collect();
    assert_eq!(
        order,
        ["rage", "focus", "inspiration"],
        "file order, not the order granted"
    );
    // Inspiration starts at 0 and stays held there: it does not end at zero.
    run(&mut w, "wait*40");
    assert_eq!(held(&w, "inspiration"), Some(0));
    assert_eq!(held(&w, "rage"), None);
    assert_eq!(held(&w, "focus"), Some(10));
}

#[test]
fn events_build_what_is_held_up_to_its_maximum() {
    let mut w = world("hall");
    let player = w.state.player;
    w.gain_resource(player, Build::Kill);
    assert!(
        w.state.player().resources.is_empty(),
        "nothing held, nothing built"
    );
    run(
        &mut w,
        "resource:inspiration resource:openings resource:rage",
    );
    w.gain_resource(player, Build::Kill);
    assert_eq!(
        held(&w, "inspiration"),
        Some(1),
        "a kill builds inspiration"
    );
    assert_eq!(held(&w, "openings"), Some(0), "but not openings");
    assert_eq!(held(&w, "rage"), Some(12), "nor rage, before Blood frenzy");
    w.gain_resource(player, Build::SneakAttack);
    assert_eq!(held(&w, "openings"), Some(1));
    for _ in 0..9 {
        w.gain_resource(player, Build::Kill);
    }
    assert_eq!(held(&w, "inspiration"), Some(5), "never above its maximum");
}

#[test]
fn a_fight_builds_hits_criticals_being_hit_and_the_kill() {
    let mut w = World::new(content(false), "den", Some(3)).expect("world");
    run(&mut w, "resource:grit");
    let (mut hits, mut criticals, mut struck) = (0u16, 0u16, 0u16);
    let player = w.state.player;
    for _ in 0..200 {
        if w.state.current().id != "den" || w.state.actors.len() == 1 {
            break;
        }
        let outcome = w.apply(&Action::Move(Dir::E));
        for e in &outcome.events {
            if let Event::Attacked {
                attacker,
                strike: Some(Strike::Hit { critical }),
                ..
            } = e
            {
                if *attacker == player {
                    hits += 1;
                    criticals += u16::from(*critical);
                } else {
                    struck += 1;
                }
            }
        }
    }
    assert_eq!(w.state.actors.len(), 1, "the kobold is dead");
    assert!(hits > 0);
    assert_eq!(
        held(&w, "grit"),
        Some(50 + hits + 10 * criticals + 2 * struck),
        "{hits} hits, {criticals} criticals, struck {struck} times, a kill"
    );
}

#[test]
fn a_class_active_that_costs_a_resource_is_unavailable_without_it_and_spends_it() {
    let spec: CharacterSpec = "human,male,barbarian".parse().expect("spec");
    let mut w = World::with_character(content(true), "hall", None, Some(&spec)).expect("world");
    let rage = Ability::Class(ClassActive::Rage);
    let focus = id(&w, "focus");
    let me = w.state.player();
    assert!(rage.had_by(me, w.content()));
    assert_eq!(
        rage.unavailable(me, w.content()),
        Some(Unavailable::Short(focus))
    );
    assert_eq!(
        Unavailable::Short(focus).reason(w.content()),
        "not enough focus"
    );
    assert_eq!(Unavailable::Short(focus).id(w.content()), "short:focus");
    let outcome = w.apply(&"use:rage".parse().expect("action"));
    assert!(!outcome.took_turn, "refused");
    assert_eq!(last(&w), "You cannot: not enough focus.");
    run(&mut w, "resource:focus");
    set(&mut w, "focus", 5);
    assert_eq!(
        rage.unavailable(w.state.player(), w.content()),
        Some(Unavailable::Short(focus)),
        "too little"
    );
    set(&mut w, "focus", 7);
    assert_eq!(rage.unavailable(w.state.player(), w.content()), None);
    assert!(w.apply(&"use:rage".parse().expect("action")).took_turn);
    // The rage began, and 6 focus went; a wait's time brought one back.
    let raging = w.content().status_id("raging").expect("raging");
    assert!(w.state.player().effects.has(raging));
    assert_eq!(held(&w, "focus"), Some(2));
}

#[test]
fn a_cost_must_name_a_resource_and_belong_to_a_class_active() {
    let mut files = ContentFiles::builtin();
    let rage = include_str!("../../../../assets/data/talents/rage.toml");
    let bad = rage.replace("systems = [", "resource = { id = \"mirth\" }\nsystems = [");
    files.insert("data/talents/rage.toml", &bad);
    let err = Content::load(&files)
        .expect_err("unknown resource")
        .to_string();
    assert!(err.contains("mirth"), "{err}");
    let mut files = ContentFiles::builtin();
    let rage = rage.replace(
        "systems = [",
        "resource = { id = \"focus\", cost = 11 }\nsystems = [",
    );
    files.insert("data/talents/rage.toml", &rage);
    let err = Content::load(&files).expect_err("too dear").to_string();
    assert!(err.contains("more"), "{err}");
    let mut files = ContentFiles::builtin();
    let hardy = include_str!("../../../../assets/data/talents/hardy.toml");
    let bad = hardy.replace("systems = [", "resource = { id = \"focus\" }\nsystems = [");
    files.insert("data/talents/hardy.toml", &bad);
    let err = Content::load(&files)
        .expect_err("not an active")
        .to_string();
    assert!(err.contains("class active"), "{err}");
}

#[test]
fn creatures_are_shown_none_and_the_dumps_show_the_players_own() {
    let mut w = world("hall");
    run(&mut w, "resource:rage resource:focus wait");
    let header = crate::dump::ascii(&w, &[]);
    assert!(
        header.contains(" mp 0/0 rage 11/20 focus 10/10 "),
        "{header}"
    );
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(
        json["resources"],
        serde_json::json!([
            { "id": "rage", "current": 11, "max": 20 },
            { "id": "focus", "current": 10, "max": 10 },
        ])
    );
    let actors = crate::dump::ascii(&w, &[crate::dump::Layer::Actors]);
    assert!(
        actors.contains("resources rage 11/20 (0), focus 10/10 (0)"),
        "{actors}"
    );
}

#[test]
fn resources_are_saved_and_hashed_and_replay() {
    let w = world("hall");
    let before = serde_json::to_string(w.state.player()).expect("json");
    assert!(
        !before.contains("resources"),
        "an actor without any saves as before"
    );
    let content = content(false);
    let mut w = World::new(content.clone(), "hall", None).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "resource:rage resource:focus wait*2 rest:30s wait*8";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(held(&w, "rage"), None, "it ran out");
    assert_eq!(held(&w, "focus"), Some(10));
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let mut other = World::new(content.clone(), "hall", None).expect("world");
    run(&mut other, "resource:rage wait wait");
    let state = other.state().clone();
    let restored = World::from_state(content.clone(), state).expect("restores");
    assert_eq!(restored.state_hash(), other.state_hash());
    assert_eq!(held(&restored, "rage"), Some(10));
    let mut plain = World::new(content.clone(), "hall", None).expect("world");
    run(&mut plain, "wait wait");
    assert_ne!(
        plain.state_hash(),
        other.state_hash(),
        "the pool is in the hash"
    );
    Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
}
