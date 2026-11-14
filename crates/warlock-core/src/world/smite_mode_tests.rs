//! The modes of smiting (`world/smite.rs`; spec §8 "Class abilities"; the paladin design P2,
//! #911): Branding smite and Wrathful smite, their riders, their cost, and their refusals.

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, world};
use super::*;
use crate::ability::Ability;
use crate::action::{Action, parse_actions};
use crate::content::{ClassActive, ResourceId};
use crate::replay::Replay;
use crate::stats::Stat;
use crate::talent::SmiteRider;

/// A blow that hit, as a rider asks for one.
const A_HIT: crate::combat::Blow = crate::combat::Blow {
    strike: crate::combat::Strike::Hit { critical: false },
    damage: 1,
    kind: None,
};

fn zeal(w: &World) -> ResourceId {
    w.content().resource_id("zeal").expect("zeal")
}

fn held(w: &World) -> Option<u16> {
    w.state.player().resources.get(zeal(w)).map(|p| p.value)
}

fn set(w: &mut World, value: u16) {
    let id = zeal(w);
    w.player_mut().resources.set(id, value);
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("a status")
}

fn rider(w: &World, active: ClassActive) -> SmiteRider {
    let mods = w.state.player().talent_mods(w.content());
    mods.faith.smite_mode(active).expect("the mode")
}

fn said_since(w: &World, from: usize) -> Vec<String> {
    (w.messages().iter().skip(from))
        .map(|m| m.display())
        .collect()
}

#[test]
fn the_paladin_gains_branding_at_12_and_wrathful_at_32() {
    let mut w = world("paladin");
    let has = |w: &World, a| (w.state.player().talent_mods(w.content())).has_active(a);
    reach(&mut w, 11);
    assert!(!has(&w, ClassActive::BrandingSmite));
    reach(&mut w, 12);
    assert!(has(&w, ClassActive::BrandingSmite));
    assert!(!has(&w, ClassActive::WrathfulSmite));
    let rows: Vec<_> = (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .collect();
    assert!(
        rows.contains(&Ability::Class(ClassActive::BrandingSmite)),
        "a menu row"
    );
    reach(&mut w, 32);
    assert!(has(&w, ClassActive::WrathfulSmite));
    // Action text names each by its id.
    for (text, active) in [
        ("use:branding_smite:E", ClassActive::BrandingSmite),
        ("use:wrathful_smite:SW", ClassActive::WrathfulSmite),
    ] {
        let action = &parse_actions(text).expect("parses")[0];
        assert!(
            matches!(action, Action::Use(a, Some(_)) if *a == active),
            "{text}"
        );
        assert_eq!(action.to_string(), text);
    }
}

#[test]
fn a_branding_smite_brands_only_a_foe_it_wounds_and_the_foe_sheds_light() {
    let mut w = world("paladin");
    reach(&mut w, 12);
    at_normal_speed(&mut w);
    let kobold = beside(&mut w, "kobold", 500);
    let branded = status(&w, "branded");
    let mut brands = 0;
    for _ in 0..30 {
        w.state
            .actors
            .get_mut(&kobold)
            .expect("kobold")
            .effects
            .remove(branded);
        set(&mut w, 3);
        let from = w.messages().len();
        run(&mut w, "use:branding_smite:E");
        let said = said_since(&w, from);
        assert_eq!(said.first().map(String::as_str), Some("You smite!"));
        // Three spent; a hit, a blow taken, and a minute's trickle may each build one back.
        assert!(held(&w) <= Some(3), "{:?}", held(&w));
        let missed = said.iter().any(|l| l.starts_with("You miss"));
        let took = w.state.actors[&kobold].effects.has(branded);
        assert!(!(missed && took), "a miss brands nothing: {said:?}");
        assert_eq!(
            took,
            said.iter().any(|l| l == "Holy light clings to the kobold.")
        );
        if took {
            brands += 1;
            // It sheds the item glow where it stands.
            let at = w.state.actors[&kobold].pos;
            let lights: Vec<_> = (w.perception().light_sources().iter())
                .filter(|s| s.pos() == Some(at))
                .filter_map(|s| s.preset())
                .map(|l| w.content().light(l).id.clone())
                .collect();
            assert!(lights.iter().any(|l| l == "item_glow"), "{lights:?}");
        }
    }
    assert!(brands > 0, "some blow landed");
}

#[test]
fn the_brand_lasts_five_minutes() {
    let mut w = world("paladin");
    reach(&mut w, 12);
    at_normal_speed(&mut w);
    let kobold = beside(&mut w, "kobold", 500);
    let branded = status(&w, "branded");
    let brand = rider(&w, ClassActive::BrandingSmite);
    let mut events = Vec::new();
    w.smite_rider(kobold, brand, A_HIT, &mut events);
    assert!(w.state.actors[&kobold].effects.has(branded));
    // The kobold fights on: the paladin outlasts it.
    w.player_mut().hp = 100_000;
    run(&mut w, "wait*40");
    assert!(
        w.state.actors[&kobold].effects.has(branded),
        "four minutes on"
    );
    run(&mut w, "wait*20");
    assert!(!w.state.actors[&kobold].effects.has(branded), "gone by six");
}

#[test]
fn a_wrathful_smite_frightens_unless_the_will_holds() {
    let mut w = world("paladin");
    reach(&mut w, 32);
    let player = w.state.player;
    let kobold = beside(&mut w, "kobold", 500);
    let frightened = status(&w, "frightened");
    let fear = rider(&w, ClassActive::WrathfulSmite);
    // The kobold in sight, so the player is told how it took it.
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let SmiteRider::Fear {
        check, difficulty, ..
    } = fear
    else {
        panic!("a fear rider");
    };
    assert_eq!((check, difficulty), (Stat::Willpower, 13));
    let (mut quailed, mut held_firm) = (0, 0);
    for _ in 0..80 {
        w.state
            .actors
            .get_mut(&kobold)
            .expect("kobold")
            .effects
            .remove(frightened);
        let mut events = Vec::new();
        w.smite_rider(kobold, fear, A_HIT, &mut events);
        // The same line again in the same turn is counted on the last, not logged anew.
        let said = last(&w);
        match w.state.actors[&kobold].effects.get(frightened) {
            Some(e) => {
                quailed += 1;
                assert!((1..=3).contains(&e.severity), "2d3, at most Frightened's 3");
                assert_eq!(e.by, Some(player), "it flees the paladin");
                assert!(said.starts_with("The kobold quails!"), "{said}");
            }
            None => {
                held_firm += 1;
                assert!(said.starts_with("The kobold's will holds."), "{said}");
            }
        }
    }
    assert!(
        quailed > 0 && held_firm > 0,
        "{quailed} quailed, {held_firm} held"
    );
}

#[test]
fn the_modes_spend_their_zeal_only_when_answered() {
    let mut w = world("paladin");
    reach(&mut w, 32);
    at_normal_speed(&mut w);
    // Nobody there: refused, free.
    set(&mut w, 4);
    let turn = w.state.turn;
    run(&mut w, "use:wrathful_smite:E");
    assert_eq!(last(&w), "There is nobody there to strike.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(4)));
    beside(&mut w, "kobold", 500);
    // Short of zeal: refused, free.
    set(&mut w, 3);
    run(&mut w, "use:wrathful_smite:E");
    assert_eq!(last(&w), "Your zeal is not enough.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(3)));
    // Neutral: none answers, a cast's time, nothing spent.
    w.player_mut().alignment = Some(crate::alignment::Score::new(0));
    set(&mut w, 4);
    let from = w.messages().len();
    run(&mut w, "use:wrathful_smite:E");
    let said = said_since(&w, from);
    assert_eq!(said[0], "You call for holy wrath, but none answers.");
    assert_eq!(w.state.turn, turn + 1);
    assert!(held(&w) >= Some(4), "nothing spent");
    // Good again: four spent (a blow given or taken and a minute's trickle may each build one
    // back, never all four).
    w.player_mut().alignment = Some(crate::alignment::Score::new(550));
    set(&mut w, 4);
    run(&mut w, "use:wrathful_smite:E");
    assert!(held(&w) <= Some(3), "{:?}", held(&w));
}

#[test]
fn a_replay_of_the_modes_verifies() {
    let content = std::sync::Arc::new(crate::content::Content::builtin().expect("content"));
    let spec: crate::character::CharacterSpec = "human,male,paladin".parse().expect("spec");
    let mut w = World::start_run(content.clone(), "arena", Some(7), Some(&spec), None, true)
        .expect("world");
    let mut replay = Replay::new(&w);
    let actions = "xp:3000000 levelup:auto use:branding_smite:E use:wrathful_smite:E wait*3";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.player().talents.has("wrathful_smite"));
    Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
}
