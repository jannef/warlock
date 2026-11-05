//! Merchant's eye (spec §8 "Class abilities"; #816): the merchant's class active, used on a
//! stack the player carries (`use:merchants_eye:c`), identifies it as the scroll of identify
//! would, for the time of a reading, with nothing else spent.

use super::*;
use crate::ability::{Ability, Aim};
use crate::action::parse_actions;
use crate::content::{ActionKind, ClassActive, ContentFiles, Rarity};
use crate::magic::{Affix, Magic};

const EYE: ClassActive = ClassActive::MerchantsEye;

/// A lit hall, the player with two mana and a scroll of identify (a).
fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = 2\n\
         carry = [\"scroll_of_identify\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", Some(3)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

/// An unidentified magic ring, put at the end of the pack; its position.
fn ring(w: &mut World) -> usize {
    let id = w.content.object_id("silver_ring").expect("ring");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: "of_the_mule".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let pack = &mut w.player_mut().inventory;
    pack.push(Stack::with_magic(id, 1, magic));
    pack.len() - 1
}

#[test]
fn the_merchant_gains_it_at_level_twelve() {
    let w = world();
    let def = w.content.class("merchant").expect("merchant");
    let at = (def.abilities.iter()).find(|(_, t)| t == "merchants_eye");
    assert_eq!(at.map(|(level, _)| *level), Some(12));
    let talents = w.content.talents();
    let id = talents.id("merchants_eye").expect("the talent");
    assert!(talents.get(id).granted, "a class ability, never picked");
}

#[test]
fn without_it_the_eye_is_refused_with_no_time() {
    let mut w = world();
    let on = ring(&mut w);
    let before = w.state.time;
    let out = run(&mut w, &Action::UseOn(EYE, on).to_string());
    assert!(!out[0].took_turn);
    assert_eq!(w.state.time, before);
    assert_eq!(last(&w), "You have no such ability.");
    assert!(!w.state.player().inventory[on].is_identified());
}

#[test]
fn the_eye_identifies_a_stack_for_the_time_of_a_reading() {
    let mut w = world();
    w.player_mut().talents.take("merchants_eye", None);
    assert_eq!(Ability::Class(EYE).aim(&w.content), Aim::Item);
    let layer = crate::dump::parse_layers("abilities").expect("layer")[0];
    let rows = crate::dump::layer_rows(&w, layer);
    assert!(
        rows.contains(&"Merchant's eye (class): an item you carry".to_string()),
        "{rows:?}"
    );
    let on = ring(&mut w);
    assert_eq!(
        Ability::Class(EYE).action_on(&w.content, on),
        Some(Action::UseOn(EYE, on))
    );
    // Without a stack it asks for one, no time.
    let out = run(&mut w, "use:merchants_eye");
    assert!(!out[0].took_turn);
    assert_eq!(last(&w), "Use it on what?");
    // A stack not carried: refused.
    let out = run(&mut w, "use:merchants_eye:z");
    assert!(!out[0].took_turn);
    assert_eq!(last(&w), "You carry no such thing.");
    let (before, mana) = (w.state.time, w.state.player().mana);
    let out = run(
        &mut w,
        &format!("use:merchants_eye:{}", crate::action::position_text(on)),
    );
    assert!(out[0].took_turn);
    assert!(w.state.player().inventory[on].is_identified());
    assert!(
        last(&w).starts_with("This is a silver ring of the Mule"),
        "{}",
        last(&w)
    );
    assert_eq!(w.state.player().mana, mana, "no mana spent");
    assert_eq!(w.state.player().inventory[0].count, 1, "no scroll used");
    assert!(w.state.time > before, "it takes time");
    assert_eq!(EYE.cost(), ActionKind::Read, "the time of a reading");
    // Known now: refused, no time.
    let before = w.state.time;
    let out = run(&mut w, &Action::UseOn(EYE, on).to_string());
    assert!(!out[0].took_turn);
    assert_eq!(w.state.time, before);
    assert!(
        last(&w).starts_with("You already know what"),
        "{}",
        last(&w)
    );
    // Again and again, as often as there is something to learn: the scroll's type too.
    let out = run(&mut w, "use:merchants_eye:a");
    assert!(out[0].took_turn, "{}", last(&w));
    assert!(w.lore().knows(&w.state.player().inventory[0]));
}

#[test]
fn its_action_text_round_trips() {
    let action = Action::UseOn(EYE, 2);
    assert_eq!(action.to_string(), "use:merchants_eye:c");
    assert_eq!(
        parse_actions("use:merchants_eye:c").expect("parses"),
        [action]
    );
    assert!(parse_actions("use:merchants_eye:3,4").is_err());
}
