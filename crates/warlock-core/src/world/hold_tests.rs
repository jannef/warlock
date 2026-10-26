//! Holds (spec §8 "Status effects"; #431): a bite's hold ties the held to its holder, ends as the
//! holder dies, leaves, or is held or slowed, lasts a few turns at most, and the held player's
//! step is a struggle to break free.

use super::*;
use crate::action::parse_actions;
use crate::content::{Affliction, ContentFiles, StatusId};
use crate::geom::{Dir, Pos};
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};

/// A creature whose bite always holds, with a grip of 40 (a natural 24 alone breaks it), and one
/// whose bite holds with the status's own grip.
const GRABBERS: &str = r##"
[[creature]]
id = "zz_grabber"
level = 1
xp = 15
name = "grabber"
glyph = "g"
fg = "#8a6a9a"
faction = "vermin"
weight = 20000
stats = { strength = 10, dexterity = 4, toughness = 30, intelligence = 1, willpower = 12, presence = 4 }

[creature.attack]
damage = "1d2"
name = "claw"
verb = "claws"
on_hit = [{ status = "held", chance = 100, difficulty = 40 }]

[[creature]]
id = "zz_clutcher"
level = 1
xp = 15
name = "clutcher"
glyph = "g"
fg = "#6a8a9a"
faction = "vermin"
weight = 20000
stats = { strength = 10, dexterity = 4, toughness = 30, intelligence = 1, willpower = 12, presence = 4 }

[creature.attack]
damage = "1d2"
name = "claw"
verb = "claws"
on_hit = [{ status = "held", chance = 100 }]
"##;

/// The player beside a grabber, room to step west.
const BESIDE: &str = "#########\n#...@g..#\n#.......#\n#########\n";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/zz_grabbers.toml", GRABBERS);
    files.insert(
        "levels/holds.toml",
        &format!(
            "name = \"Holds\"\nambient = [255, 255, 255, 0]\nmap = '''\n{BESIDE}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"g\" = {{ terrain = \"floor\", creature = \"zz_grabber\" }}\n"
        ),
    );
    files.insert(
        "scenarios/holds.toml",
        "name = \"Holds\"\nstart_level = \"holds\"\n",
    );
    files
}

fn fresh() -> World {
    let content = Content::load(&files()).expect("test content is valid");
    World::new(Arc::new(content), "holds", Some(3)).expect("world")
}

fn world() -> World {
    let mut w = fresh();
    // Hardy enough to take the claws for as long as a test runs.
    let me = w.player_mut();
    me.hp_base = 500;
    me.hp = 500;
    w
}

fn held(w: &World) -> StatusId {
    w.content().status_id("held").expect("held")
}

fn grabber(w: &World) -> ActorId {
    let kind = w.content().creature_id("zz_grabber").expect("grabber");
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .map(|a| a.id)
        .expect("a grabber")
}

/// The player's hold, if any: its degrees (turns left) and holder.
fn hold(w: &World) -> Option<(u16, Option<ActorId>)> {
    let status = held(w);
    (w.state.player().effects.get(status)).map(|e| (e.severity, e.by))
}

/// Holds the player fast by `by`, as a bite does.
fn grab(w: &mut World, by: Option<ActorId>) -> Vec<Event> {
    let status = held(w);
    let player = w.state.player;
    let mut events = Vec::new();
    w.afflict(player, status, 1, by, &mut events);
    events
}

fn messages(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn ends(events: &[Event], status: StatusId) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::Status { status: s, began: false, .. } if *s == status))
}

#[test]
fn a_bite_holds_with_its_grip_for_a_few_turns_tied_to_the_biter() {
    let w = world();
    let content = w.content();
    let def = content.status(held(&w));
    let turns = def.hold.expect("held is a hold").turns;
    assert_eq!((turns.min(), turns.max()), (3, 12));
    let claw = |id: &str| {
        let kind = content.creature_id(id).expect(id);
        content
            .creature(kind)
            .attack
            .as_ref()
            .expect("a claw")
            .on_hit[0]
            .what
    };
    let status = held(&w);
    assert_eq!(
        claw("zz_grabber"),
        Affliction::Hold {
            status,
            difficulty: Some(40)
        }
    );
    assert_eq!(
        claw("zz_clutcher"),
        Affliction::Hold {
            status,
            difficulty: None
        }
    );
    // A hold's degrees are the turns it lasts, drawn.
    let mut w = world();
    let g = grabber(&w);
    grab(&mut w, Some(g));
    let (turns, by) = hold(&w).expect("held");
    assert!((3..=12).contains(&turns), "{turns}");
    assert_eq!(by, Some(g));
    // Its claw holds: the first that lands.
    let mut w = world();
    let g = grabber(&w);
    for _ in 0..10 {
        if hold(&w).is_some() {
            break;
        }
        w.apply(&Action::Wait);
    }
    assert_eq!(hold(&w).map(|(_, by)| by), Some(Some(g)));
}

#[test]
fn the_hold_ends_as_the_holder_dies_leaves_or_is_held_or_slowed() {
    // Dies.
    let mut w = world();
    let status = held(&w);
    let g = grabber(&w);
    grab(&mut w, Some(g));
    let mut events = Vec::new();
    w.loosen_holds(&mut events);
    assert!(hold(&w).is_some(), "it holds while beside");
    w.state.actors.remove(&g);
    w.loosen_holds(&mut events);
    assert!(hold(&w).is_none() && ends(&events, status));
    // Leaves the player's side (knocked away).
    let mut w = world();
    let g = grabber(&w);
    grab(&mut w, Some(g));
    w.state.actors.get_mut(&g).expect("grabber").pos = Pos::new(7, 2);
    w.loosen_holds(&mut Vec::new());
    assert!(hold(&w).is_none());
    // Slowed (sluggish) or held itself.
    for hex in ["sluggish", "held"] {
        let mut w = world();
        let g = grabber(&w);
        grab(&mut w, Some(g));
        let hex = w.content().status_id(hex).expect(hex);
        w.afflict(g, hex, 1, None, &mut Vec::new());
        w.loosen_holds(&mut Vec::new());
        assert!(hold(&w).is_none(), "{hex:?}");
    }
    // The turn's end asks too: a holder gone during it lets go, said as the hold's end.
    let mut w = world();
    let g = grabber(&w);
    grab(&mut w, Some(g));
    w.state.actors.remove(&g);
    w.apply(&Action::Wait);
    assert!(hold(&w).is_none());
    assert!(messages(&w).iter().any(|m| m == "You are free to move."));
}

#[test]
fn no_hold_outlasts_its_turns() {
    let mut w = world();
    let g = grabber(&w);
    // Out of reach of its own claws, but held by nothing (the dev hold): only time frees.
    w.state.actors.remove(&g);
    grab(&mut w, None);
    let (turns, _) = hold(&w).expect("held");
    for _ in 1..turns {
        w.apply(&Action::Wait);
    }
    assert!(hold(&w).is_some(), "still held before its last turn");
    w.apply(&Action::Wait);
    assert!(hold(&w).is_none(), "free after {turns} turns");
}

#[test]
fn a_step_while_held_is_a_struggle_that_costs_the_turn_or_breaks_free() {
    let mut w = world();
    let g = grabber(&w);
    let start = w.player_pos();
    // One struggle, from where it began with the grabber beside, held anew: whether it broke
    // free, checking what it cost and what was said.
    let struggle = |w: &mut World, holder: &str| {
        w.player_mut().pos = start;
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        w.state.actors.get_mut(&g).expect("grabber").pos = start.step(Dir::E);
        let status = held(w);
        w.player_mut().effects.remove(status);
        grab(w, Some(g));
        let turn = w.state.turn;
        let outcome = w.apply(&Action::Move(Dir::W));
        // What was said since (a repeat is logged again at its latest turn).
        let said: Vec<String> = (w.messages().iter())
            .filter(|m| m.turn >= turn)
            .map(|m| m.text.clone())
            .collect();
        if w.player_pos() == start {
            assert!(outcome.took_turn && w.state.turn == turn + 1, "{said:?}");
            assert!(hold(w).is_some());
            let fast = format!("You struggle, but the {holder} holds you fast.");
            assert!(said.contains(&fast), "{said:?}");
            false
        } else {
            assert_eq!(w.player_pos(), start.step(Dir::W));
            let free = format!("You break free of the {holder}.");
            assert!(said.contains(&free), "{said:?}");
            true
        }
    };
    // A grip of 40: only a natural 24 (1 in 144) breaks it.
    let freed = (0..20).filter(|_| struggle(&mut w, "grabber")).count();
    assert!(freed <= 1, "freed {freed}");
    // The clutcher's bite names no grip: the hold's own (13), which a strong arm breaks but
    // for a natural 2.
    let clutcher = w.content().creature_id("zz_clutcher").expect("clutcher");
    w.state.actors.get_mut(&g).expect("grabber").creature = clutcher;
    w.player_mut().stats = Stats::new(PerStat::splat(40), PerStat::splat(40));
    let freed = (0..20).filter(|_| struggle(&mut w, "clutcher")).count();
    assert!(freed >= 19, "freed {freed}");
}

#[test]
fn an_attack_while_held_is_no_struggle() {
    let mut w = world();
    let g = grabber(&w);
    grab(&mut w, Some(g));
    let before = w.player_pos();
    w.apply(&Action::Move(Dir::E));
    assert_eq!(w.player_pos(), before);
    assert!(!messages(&w).iter().any(|m| m.contains("struggle")));
}

#[test]
fn holds_replay_and_the_holder_survives_a_save() {
    let mut w = fresh();
    let content = w.content().clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait*4 move:W*6 wait*3").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    // The holder is the effect's cause, saved with it.
    let mut w = world();
    let g = grabber(&w);
    grab(&mut w, Some(g));
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    let status = held(&loaded);
    let effect = loaded.state.player().effects.get(status).copied();
    assert_eq!(effect.and_then(|e| e.by), Some(g));
}
