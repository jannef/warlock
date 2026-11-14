//! Make do and mend (#1422; the craftsperson design §7.5): the craftsperson's class active at
//! level 18 mends a damaged thing carried, anywhere, in ten minutes, by a check that never
//! destroys anything.

use super::*;
use crate::ability::{Ability, Aim, Unavailable};
use crate::action::{parse_actions, position_text};
use crate::content::{ActionKind, ClassActive, ContentFiles, ItemHarm};
use crate::item_list::ItemList;
use crate::magic::Worked;
use crate::talent::CraftEffect;
use crate::test_support::builtin;
use crate::test_support::last;
use crate::test_support::run;

const MEND: ClassActive = ClassActive::MakeDo;

/// A lit hall, the player with a long sword (a) and a wool cloak (b), with Make do and mend at
/// `difficulty` (the talent's own file replaced, so a check is sure either way but for the
/// natural roll's ends).
fn world(difficulty: i32) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\n\
         carry = [\"long_sword\", \"wool_cloak\"]\n",
    );
    files.insert(
        "data/talents/make_do_and_mend.toml",
        &format!(
            "[[talent]]\nid = \"make_do_and_mend\"\nname = \"Make do and mend\"\n\
             tree = \"class\"\ngranted = true\ndescription = \"Mends.\"\n\
             effects = [{{ kind = \"make_do\", seconds = 600, difficulty = {difficulty} }}]\n\
             systems = [\"skills\"]\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", Some(3)).expect("world");
    w.player_mut().talents.take("make_do_and_mend", None);
    w
}

/// The stack at `at` damaged, after `attempts` attempts at the anvil.
fn damage(w: &mut World, at: usize, attempts: u8) {
    let mut worked = Worked::default();
    for _ in 0..attempts {
        worked.attempted();
    }
    worked.damage(ItemHarm::Rust);
    w.player_mut().inventory[at].set_worked(worked);
}

fn worked(w: &World, at: usize) -> Worked {
    (w.state.player().inventory[at].worked().cloned()).unwrap_or_default()
}

/// Whether the ability's row is greyed, and why.
fn row(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().iter())
        .find(|own| own.ability == Ability::Class(MEND))
        .map(|own| own.unavailable)
}

#[test]
fn the_craftsperson_gains_it_at_level_eighteen() {
    let content = builtin();
    let def = content.class("craftsperson").expect("craftsperson");
    let at = (def.abilities.iter()).find(|(_, t)| t == "make_do_and_mend");
    assert_eq!(at.map(|(level, _)| *level), Some(18));
    let talents = content.talents();
    let id = talents.id("make_do_and_mend").expect("the talent");
    assert!(talents.get(id).granted, "a class ability, never picked");
    let effects = &talents.get(id).effects;
    assert!(
        matches!(
            effects[0],
            crate::content::TalentEffect::Craft(CraftEffect::MakeDo {
                seconds: 600,
                difficulty: 10
            })
        ),
        "{effects:?}"
    );
}

#[test]
fn its_action_text_round_trips() {
    let action = Action::UseOn(MEND, 2);
    assert_eq!(action.to_string(), "use:make_do_and_mend:c");
    assert_eq!(
        parse_actions("use:make_do_and_mend:c").expect("parses"),
        [action]
    );
    assert_eq!(MEND.cost(), ActionKind::Craft);
}

#[test]
fn without_it_a_mend_is_refused_with_no_time() {
    let mut w = world(-100);
    w.player_mut().talents = Default::default();
    damage(&mut w, 0, 0);
    let before = w.state.time;
    let out = run(&mut w, "use:make_do_and_mend:a");
    assert!(!out[0].took_turn);
    assert_eq!(w.state.time, before);
    assert_eq!(last(&w), "You have no such ability.");
    assert!(worked(&w, 0).damaged);
}

#[test]
fn the_row_is_greyed_with_nothing_to_mend() {
    let mut w = world(-100);
    assert_eq!(
        Ability::Class(MEND).aim(&w.content),
        Aim::Item(ItemList::MakeDo)
    );
    assert_eq!(row(&w), Some(Some(Unavailable::NothingToMend)));
    assert!(w.perception().item_choices(ItemList::MakeDo).is_empty());
    damage(&mut w, 0, 0);
    assert_eq!(row(&w), Some(None));
    // The list offers the damaged sword; Salvage the sword too, not the cloth cloak.
    assert_eq!(w.perception().item_choices(ItemList::MakeDo), [0]);
    assert_eq!(w.perception().item_choices(ItemList::Salvage), [0]);
    let layer = crate::dump::parse_layers("abilities").expect("layer")[0];
    let rows = crate::dump::layer_rows(&w, layer);
    assert!(
        rows.contains(&"Make do and mend (class): an item you carry".to_string()),
        "{rows:?}"
    );
    // Wielded, it is not one to mend, nor to take apart.
    run(&mut w, "equip:main_hand:a");
    assert_eq!(row(&w), Some(Some(Unavailable::NothingToMend)));
    assert!(w.perception().item_choices(ItemList::MakeDo).is_empty());
    assert!(w.perception().item_choices(ItemList::Salvage).is_empty());
}

#[test]
fn what_cannot_be_mended_is_refused_with_no_time() {
    let mut w = world(-100);
    let refused = |w: &mut World, text: &str, said: &str| {
        let before = w.state.time;
        let out = run(w, text);
        assert!(!out[0].took_turn, "{text}");
        assert_eq!(w.state.time, before, "{text}");
        assert_eq!(last(w), said, "{text}");
    };
    refused(&mut w, "use:make_do_and_mend", "Use it on what?");
    refused(&mut w, "use:make_do_and_mend:z", "You carry no such thing.");
    refused(
        &mut w,
        "use:make_do_and_mend:a",
        "There is nothing wrong with it.",
    );
    // Wool is cloth: no profession works it.
    damage(&mut w, 1, 0);
    refused(
        &mut w,
        "use:make_do_and_mend:b",
        "You know no craft that would mend it.",
    );
    damage(&mut w, 0, 0);
    run(&mut w, "equip:main_hand:a");
    refused(&mut w, "use:make_do_and_mend:a", "Take it off first.");
}

#[test]
fn a_mend_made_takes_ten_minutes_and_keeps_the_attempts() {
    let mut w = world(-100);
    damage(&mut w, 0, 2);
    let before = w.state.time;
    let out = run(&mut w, &Action::UseOn(MEND, 0).to_string());
    assert!(out[0].took_turn);
    assert_eq!(w.state.time.since(before), Duration::seconds(600));
    let state = worked(&w, 0);
    assert!(!state.damaged, "{}", last(&w));
    assert_eq!(state.attempts, 2, "the attempts stay as they were");
    assert!(state.improved.is_empty());
    assert_eq!(
        last(&w),
        "You make good the damage: it is a long sword again."
    );
}

#[test]
fn a_mend_failed_destroys_nothing_and_may_be_tried_again() {
    let mut w = world(100);
    damage(&mut w, 0, 1);
    let count = w.state.player().inventory.len();
    let out = run(
        &mut w,
        &format!("use:make_do_and_mend:{}", position_text(0)),
    );
    assert!(out[0].took_turn);
    assert_eq!(
        last(&w),
        "You cannot make it good here; it will need an anvil."
    );
    assert_eq!(w.state.player().inventory.len(), count, "nothing destroyed");
    let state = worked(&w, 0);
    assert!(state.damaged);
    assert_eq!(state.attempts, 1);
    // Again: refused never, only the time spent.
    let out = run(&mut w, "use:make_do_and_mend:a");
    assert!(out[0].took_turn);
    assert!(worked(&w, 0).damaged);
}

#[test]
fn eye_for_the_make_names_your_own_hands_with_it() {
    let mut w = world(-100);
    w.player_mut().talents.take("eye_for_the_make", None);
    damage(&mut w, 0, 0);
    let lines = |w: &World| -> Vec<String> {
        let view = w.perception();
        (view.describe_item(&view.inventory()[0]).into_iter())
            .map(|l| l.text)
            .collect()
    };
    let with = "It is damaged: work at the anvil would make it good, or your own hands.";
    assert!(lines(&w).contains(&with.to_string()), "{:?}", lines(&w));
    w.player_mut().talents = Default::default();
    w.player_mut().talents.take("eye_for_the_make", None);
    let without = "It is damaged: work at the anvil would make it good.";
    assert!(lines(&w).contains(&without.to_string()), "{:?}", lines(&w));
}

#[test]
fn a_mend_replays_the_same() {
    let make = || {
        let mut w = world(10);
        damage(&mut w, 0, 0);
        run(&mut w, "use:make_do_and_mend:a wait");
        w.state_hash()
    };
    assert_eq!(make(), make());
}
