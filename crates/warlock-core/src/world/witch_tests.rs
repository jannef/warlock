//! Cackle (spec §8 "Class abilities"; #904; `world/witch.rs`): the witch's at 6, for the time of
//! a cast; every hex of hers on a creature she sees grows a degree (to its max) and starts
//! afresh; another's hex and one out of sight untouched; refused with no time when none of hers
//! is in sight; a voice 60 loud; no draws.

use super::class_active_tests::{last, reach, run, world};
use super::war_cry_tests::foe_at;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::content::{ClassActive, StatusId};
use crate::noise::{self, Act, SoundKind};

const CACKLE: ClassActive = ClassActive::Cackle;

/// A witch with Cackle, alone on the level.
fn witch() -> World {
    let mut w = world("witch");
    w.player_mut().talents.take("cackle", None);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    w
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

/// A kobold `dx` cells east of the player, seen once the player has looked; its id.
fn kobold(w: &mut World, dx: i32, dy: i32) -> ActorId {
    let me = w.player_pos();
    let id = foe_at(w, "kobold", Pos::new(me.x + dx, me.y + dy), 10);
    w.perceive(&mut Vec::new());
    id
}

/// Lays `degrees` of hex `id` on `on`, given by `by`, its degree `wear` seconds worn.
fn hex(w: &mut World, on: ActorId, id: &str, degrees: u16, by: Option<ActorId>, wear: u32) {
    let s = status(w, id);
    w.afflict(on, s, degrees, by, &mut Vec::new());
    let effect = (w
        .state
        .actors
        .get_mut(&on)
        .expect("there")
        .effects
        .iter_mut())
    .find(|e| e.status == s)
    .expect("hexed");
    effect.wear = wear;
}

/// The severity and wear of `id` on `on`.
fn on(w: &World, who: ActorId, id: &str) -> Option<(u16, u32)> {
    let s = status(w, id);
    (w.state.actors[&who].effects.get(s)).map(|e| (e.severity, e.wear))
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

fn cackle(w: &mut World) -> Outcome {
    w.apply(&Action::Use(CACKLE, None))
}

#[test]
fn the_witch_gains_it_at_six_and_on_load() {
    let mut w = world("witch");
    let def = w.content().class("witch").expect("witch");
    let at = (def.abilities.iter()).find(|(_, t)| t == "cackle");
    assert_eq!(at.map(|(level, _)| *level), Some(6));
    let id = w.content().talents().id("cackle").expect("the talent");
    assert!(w.content().talents().get(id).granted, "never picked");
    reach(&mut w, 5);
    assert!(!w.state.player().talents.has("cackle"), "not before 6");
    reach(&mut w, 6);
    assert!(said(&w, "You gain Cackle."), "{}", last(&w));
    // A save of a witch of 6 made before the table had it gains it on load.
    w.player_mut().talents.forget("cackle");
    let w = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert!(w.state.player().talents.has("cackle"));
    assert_eq!(
        (w.perception().abilities().into_iter())
            .find(|a| a.ability == Ability::Class(CACKLE))
            .map(|a| a.unavailable),
        Some(None),
        "listed, usable: the rules refuse"
    );
    let layer = crate::dump::parse_layers("abilities").expect("layer")[0];
    let rows = crate::dump::layer_rows(&w, layer);
    assert!(
        rows.contains(&"Cackle (class): nothing".to_string()),
        "{rows:?}"
    );
}

#[test]
fn the_hex_statuses_are_those_hex_spells_lay() {
    let w = witch();
    let hexes = w.content().hex_statuses();
    for id in ["ill_wished", "weakened", "sluggish", "drowsy", "webbed"] {
        assert!(hexes.contains(&status(&w, id)), "{id}");
    }
    assert!(!hexes.contains(&status(&w, "hasted")), "a buff is no hex");
    assert!(
        hexes.windows(2).all(|p| p[0] < p[1]),
        "in status order, once"
    );
}

#[test]
fn a_hex_grows_a_degree_and_one_at_its_max_starts_afresh() {
    let mut w = witch();
    let player = w.state.player;
    let orc = kobold(&mut w, 2, 0);
    hex(&mut w, orc, "ill_wished", 2, Some(player), 30);
    hex(&mut w, orc, "webbed", 6, Some(player), 4);
    let rng = w.state.rng.clone();
    let mut events = Vec::new();
    let spent = w.cackle(&mut events);
    assert!(matches!(spent, Spent::Turn(_)));
    assert_eq!(w.state.rng, rng, "no draws");
    assert_eq!(
        on(&w, orc, "ill_wished"),
        Some((3, 0)),
        "a degree more, fresh"
    );
    assert_eq!(on(&w, orc, "webbed"), Some((6, 0)), "at its max: fresh");
    assert!(said(&w, "You cackle."));
    assert_eq!(last(&w), "Your hexes hold.");
    // The time of a cast, and no mana.
    assert_eq!(
        spent,
        Spent::Turn(w.price_of(player, crate::content::ActionKind::Cast))
    );
    let mut w = witch();
    let orc = kobold(&mut w, 2, 0);
    hex(&mut w, orc, "weakened", 1, Some(player), 0);
    let mana = w.state.player().mana;
    assert!(cackle(&mut w).took_turn);
    assert!(w.state.player().mana >= mana, "no mana");
}

#[test]
fn another_s_hex_and_one_out_of_sight_are_untouched() {
    let mut w = witch();
    let player = w.state.player;
    let seen = kobold(&mut w, 2, 0);
    let other = kobold(&mut w, 2, 1);
    hex(&mut w, seen, "ill_wished", 1, Some(player), 10);
    hex(&mut w, seen, "weakened", 1, Some(other), 10);
    hex(&mut w, other, "sluggish", 1, None, 10);
    // One hexed by her that she does not see (far off, never perceived).
    let me = w.player_pos();
    let level = w.state.current().clone();
    let far = (-40..40)
        .flat_map(|y| (-40..40).map(move |x| Pos::new(me.x + x, me.y + y)))
        .filter(|&p| level.is_walkable(w.content(), p))
        .find(|&p| !w.perception.sees_actors_at(p))
        .expect("a cell out of sight");
    let hidden = foe_at(&mut w, "kobold", far, 10);
    hex(&mut w, hidden, "ill_wished", 1, Some(player), 10);
    w.perceive(&mut Vec::new());
    assert!(!w.perception.sees_actors_at(far));
    let out = cackle(&mut w);
    assert!(out.took_turn);
    assert_eq!(
        on(&w, seen, "ill_wished").map(|e| e.0),
        Some(2),
        "hers, seen"
    );
    // Worn on by the turn, never started afresh.
    let worn = |e: Option<(u16, u32)>| e.is_some_and(|(s, wear)| s == 1 && wear >= 10);
    assert!(worn(on(&w, seen, "weakened")), "another's");
    assert!(worn(on(&w, other, "sluggish")), "nobody's");
    assert_eq!(on(&w, hidden, "ill_wished").map(|e| e.0), Some(1), "unseen");
}

#[test]
fn refused_with_no_time_without_it_without_a_hex_or_silenced() {
    // Without the ability.
    let mut w = world("witch");
    let time = w.state.time;
    let out = cackle(&mut w);
    assert!(!out.took_turn);
    assert_eq!(w.state.time, time);
    assert_eq!(last(&w), "You have no such ability.");
    // With nobody in sight under a hex of hers: another's hex does not count.
    let mut w = witch();
    let foe = kobold(&mut w, 2, 0);
    hex(&mut w, foe, "ill_wished", 1, None, 0);
    let rng = w.state.rng.clone();
    let out = cackle(&mut w);
    assert!(!out.took_turn);
    assert_eq!(w.state.time, time);
    assert_eq!(w.state.rng, rng);
    assert_eq!(last(&w), "No one you see is under your hex.");
    assert!(
        !(w.state.actors[&foe].heard.iter()).any(|h| h.kind == SoundKind::Voice),
        "not a sound"
    );
    // Silenced: a status that blocks a cast blocks it.
    let player = w.state.player;
    hex(&mut w, foe, "weakened", 1, Some(player), 0);
    run(&mut w, "afflict:silenced");
    let silenced = status(&w, "silenced");
    assert_eq!(
        (w.perception().abilities().into_iter())
            .find(|a| a.ability == Ability::Class(CACKLE))
            .map(|a| a.unavailable),
        Some(Some(Unavailable::While(silenced)))
    );
    let out = cackle(&mut w);
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You cannot cast spells while silenced.");
}

#[test]
fn the_cackle_is_a_voice_sixty_loud_and_heard() {
    let mut w = witch();
    let player = w.state.player;
    let foe = kobold(&mut w, 2, 0);
    hex(&mut w, foe, "ill_wished", 1, Some(player), 0);
    let me = w.state.player();
    let burden = w.state.load(w.content(), me).penalty();
    let noise =
        noise::noise(w.content(), w.state.current(), me, burden, Act::Cackle).expect("a noise");
    assert_eq!(noise.kind, SoundKind::Voice);
    assert_eq!(noise.at, w.player_pos());
    assert_eq!(noise.loudness, 60, "louder than a voice, short of a howl");
    let at = w.player_pos();
    let _ = w.cackle(&mut Vec::new());
    assert!(
        (w.state.actors[&foe].heard.iter()).any(|h| h.kind == SoundKind::Voice && h.at == at),
        "heard where she stands"
    );
}

#[test]
fn its_action_text_round_trips() {
    let action = Action::Use(CACKLE, None);
    assert_eq!(action.to_string(), "use:cackle");
    assert_eq!(
        crate::action::parse_actions("use:cackle").expect("parses"),
        [action]
    );
}
