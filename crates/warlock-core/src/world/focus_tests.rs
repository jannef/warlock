//! The ranger's Focus (spec §8 "Resources"; the progression design §4.18, R2, #250): Rapid shot
//! grants it for good, a shot spends 2 of it for a tenth less time, and Volley costs 6 of it
//! instead of a cooldown.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ClassActive, Factor, ResourceId, percent_of};
use crate::replay::Replay;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human ranger in the showcase (seed 7), with their kit, at `level` with every choice spent.
fn ranger(level: u8) -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let factor = Factor::of(w.content(), Some(&spec));
    let need = w.content().experience().threshold_for(level, factor);
    let more = need - w.player_xp();
    run(&mut w, &format!("xp:{more} levelup:auto"));
    assert_eq!(w.player_level(), level);
    assert_eq!(w.due(), None, "every choice spent");
    // The picks may have taken Steady hand: these tests are of focus's own pace.
    w.player_mut().talents.forget("steady_hand");
    super::class_active_tests::at_normal_speed(&mut w);
    w
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn focus(w: &World) -> ResourceId {
    w.content().resource_id("focus").expect("focus")
}

fn held(w: &World) -> Option<u16> {
    (w.state.player().resources.get(focus(w))).map(|p| p.value)
}

fn set(w: &mut World, value: u16) {
    let focus = focus(w);
    w.player_mut().resources.set(focus, value);
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// What a plain shot costs the player now.
fn shot_price(w: &World) -> i64 {
    let (launcher, _) = w.state.player().launcher(w.content()).expect("a bow");
    w.shot_price(w.state.player, launcher)
}

fn energy(w: &World) -> i64 {
    w.state.player().energy
}

/// The player's energy after paying `price` from `energy` (and the turns that refill it).
fn after(energy: i64, price: i64) -> i64 {
    (energy - price).rem_euclid(1000)
}

/// A cell three east of the player, in a bow's range.
fn target(w: &World) -> Pos {
    w.player_pos().step(Dir::E).step(Dir::E).step(Dir::E)
}

#[test]
fn rapid_shot_grants_focus_full_and_for_good() {
    let w = ranger(11);
    assert_eq!(held(&w), None, "no focus before Rapid shot");
    let mut w = ranger(12);
    assert_eq!(held(&w), Some(10), "Rapid shot: Focus, full");
    // Spent to 0 it stays, and comes back with time.
    set(&mut w, 0);
    run(&mut w, "wait*2");
    assert_eq!(held(&w), Some(2));
    // A character saved holding the talent without its focus gets it back as the run is restored.
    let mut state = w.state().clone();
    let id = focus(&w);
    state
        .actors
        .get_mut(&state.player)
        .expect("player")
        .resources
        .remove(id);
    let w = World::from_state(BUILTIN.clone(), state).expect("restores");
    assert_eq!(held(&w), Some(10));
}

#[test]
fn a_shot_spends_two_focus_for_a_tenth_less_time() {
    let mut w = ranger(12);
    let at = target(&w);
    let (price, start) = (shot_price(&w), energy(&w));
    w.apply(&Action::Fire(at));
    assert_eq!(energy(&w), after(start, percent_of(price, 90)), "quicker");
    assert!(
        held(&w).is_some_and(|v| (8..10).contains(&v)),
        "{:?}",
        held(&w)
    );
    // With less than 2 the shot is an ordinary one, and spends nothing.
    set(&mut w, 1);
    let (price, start) = (shot_price(&w), energy(&w));
    w.apply(&Action::Fire(at));
    assert_eq!(energy(&w), after(start, price), "an ordinary shot");
    assert!(held(&w).is_some_and(|v| v >= 1));
    // Without Rapid shot nothing changes either.
    let mut w = ranger(11);
    let (price, start) = (shot_price(&w), energy(&w));
    w.apply(&Action::Fire(at));
    assert_eq!(energy(&w), after(start, price));
}

#[test]
fn a_volley_costs_six_focus_and_its_shots_are_not_rapid() {
    let mut w = ranger(50);
    let volley = Ability::Class(ClassActive::Volley);
    let at = target(&w);
    // Short: refused, no time, nothing spent.
    set(&mut w, 5);
    let short = (w.perception().abilities().into_iter()).find(|a| a.ability == volley);
    assert_eq!(
        short.map(|a| a.unavailable),
        Some(Some(Unavailable::Short(focus(&w))))
    );
    let turn = w.state.turn;
    w.apply(&Action::UseAt(ClassActive::Volley, at));
    assert_eq!(last(&w), "You have too little focus.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(5)));
    // Out of reach: refused before anything is spent.
    set(&mut w, 10);
    w.apply(&Action::UseAt(
        ClassActive::Volley,
        Pos::new(at.x + 60, at.y),
    ));
    assert_eq!((w.state.turn, held(&w)), (turn, Some(10)));
    // Loosed: 6 spent, priced as one ordinary shot (Rapid shot does not touch it).
    let (price, start) = (shot_price(&w), energy(&w));
    w.apply(&Action::UseAt(ClassActive::Volley, at));
    assert_eq!(w.state.turn, turn + 1);
    assert_eq!(energy(&w), after(start, price));
    assert!(
        held(&w).is_some_and(|v| (4..6).contains(&v)),
        "{:?}",
        held(&w)
    );
}

#[test]
fn shots_and_volleys_on_focus_replay() {
    let w = ranger(50);
    let at = target(&w);
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let text = format!(
        "xp:3000000 levelup:auto fire:{x},{y} use:volley:{x},{y} use:volley:{x},{y} wait",
        x = at.x,
        y = at.y
    );
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
