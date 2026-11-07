//! Whittling (`world/temper.rs`, `Craft::Whittle`; #1028): Temper's rules for wood, with a knife,
//! anywhere. Wood accepted and metal refused, the knife asked for and its reason, ten minutes
//! and the Woodcraft mark, an interruption, and the one table Temper and Whittle share, row by
//! row, by seed.

use super::temper::{workable, workable_wood};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::content::ContentFiles;
use crate::magic::{Aspect, Worked};
use crate::object::Stack;
use crate::rng::Rng;

/// A plain room, the player at (4, 3), and at `k` a kobold, when asked for. No anvil.
const ROOM: &str = "\
###########
#.........#
#.........#
#...@k....#
#.........#
###########
";

/// The built-in content with [`ROOM`]; the kobold only with `kobold`. The player carries only
/// what a test gives them.
fn world(kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    files.insert(
        "levels/test_whittle.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_whittle.toml",
        "name = \"T\"\nstart_level = \"test_whittle\"\n\
         character = { race = \"human\", gender = \"female\", class = \"ranger\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_whittle", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    w
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content.object_id(id).expect(id)
}

/// One of `id` in the pack's first place.
fn give(w: &mut World, id: &str) {
    let stack = Stack::one(object(w, id));
    w.player_mut().inventory.insert(0, stack);
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// The state of the stack in the pack's first place, if there is one worked.
fn worked_after(w: &World) -> Option<Worked> {
    (w.state.player().inventory.first()).and_then(|s| s.worked().cloned())
}

/// Whether an attempt of `action` on stack 0, on fate's draw `n`, came out made.
fn made_on(w: &World, action: &Action, n: u64) -> bool {
    let mut w = w.clone();
    w.state.rng = Rng::stream(n, "whittle_tests");
    w.apply(action);
    match worked_after(&w) {
        None => false,
        Some(after) => !after.damaged,
    }
}

#[test]
fn wood_is_accepted_and_metal_refused() {
    let mut w = world(false);
    give(&mut w, "knife");
    give(&mut w, "long_sword");
    give(&mut w, "turnip");
    give(&mut w, "short_bow");
    let before = w.state.time;
    w.apply(&Action::Whittle { item: 1 });
    assert!(said(&w, "Only a weapon of wood can be whittled."));
    w.apply(&Action::Whittle { item: 2 });
    assert!(said(&w, "Only a weapon of wood can be whittled."));
    w.apply(&Action::Whittle { item: 9 });
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.time, before, "refused, no time");
    let content = w.content.clone();
    let wood = |id: &str| workable_wood(&content, &Stack::one(content.object_id(id).expect(id)));
    let metal = |id: &str| workable(&content, &Stack::one(content.object_id(id).expect(id)));
    assert!(wood("short_bow") && wood("quarterstaff") && wood("club"));
    assert!(!wood("long_sword") && !wood("chain_mail") && !wood("turnip"));
    assert!(
        !metal("short_bow") && !metal("club"),
        "the anvil keeps to metal"
    );
    assert!(metal("long_sword"));
}

#[test]
fn a_spirit_weapon_is_refused() {
    let mut w = world(false);
    give(&mut w, "knife");
    let bow = object(&w, "short_bow");
    w.player_mut().inventory.insert(0, Stack::spirit(bow));
    let before = w.state.time;
    w.apply(&Action::Whittle { item: 0 });
    assert!(said(&w, "Only a weapon of wood can be whittled."));
    assert_eq!(w.state.time, before, "refused, no time");
}

#[test]
fn without_a_knife_it_is_refused_with_its_reason() {
    let mut w = world(false);
    give(&mut w, "short_bow");
    let before = w.state.time;
    w.apply(&Action::Whittle { item: 0 });
    assert!(said(&w, "You have no knife."));
    assert_eq!(w.state.time, before, "refused, no time");
    let content = w.content.clone();
    let reason = |w: &World| {
        let me = w.state.player();
        Ability::Whittle.unavailable(me, &content)
    };
    assert_eq!(reason(&w), Some(Unavailable::NoKnife));
    assert_eq!(Unavailable::NoKnife.reason(&content), "you have no knife");
    give(&mut w, "knife");
    assert_eq!(reason(&w), None);
}

#[test]
fn everyone_has_it_beside_fletch() {
    let w = world(false);
    let every = Ability::every(&w.content);
    let at = every
        .iter()
        .position(|&a| a == Ability::Fletch)
        .expect("fletch");
    assert_eq!(every[at + 1], Ability::Whittle);
    assert!(Ability::Whittle.had_by(w.state.player(), &w.content));
    assert_eq!(
        Ability::Whittle.aim(&w.content),
        crate::ability::Aim::Item,
        "it asks which"
    );
    assert_eq!(
        Ability::Whittle.action_on(&w.content, 3),
        Some(Action::Whittle { item: 3 })
    );
}

#[test]
fn an_attempt_takes_ten_minutes_names_the_item_and_marks_woodcraft() {
    let mut w = world(false);
    give(&mut w, "knife");
    give(&mut w, "short_bow");
    let woodcraft = w.content.skill_id("woodcraft").expect("woodcraft");
    let marks = |w: &World| {
        let skills = &w.state.player().skills;
        skills.get(woodcraft).map_or(0, |t| t.marks)
    };
    // Trained in it, so that a mark is earned.
    {
        let skills = &mut w.player_mut().skills;
        let mut all: Vec<_> = skills.iter().filter(|(s, _)| *s != woodcraft).collect();
        all.push((
            woodcraft,
            crate::skills::Trained {
                value: 10,
                marks: 0,
            },
        ));
        *skills = all.into_iter().collect();
    }
    let action = Action::Whittle { item: 0 };
    let made = (0..500)
        .find(|&n| made_on(&w, &action, n))
        .expect("some draw is made");
    w.state.rng = Rng::stream(made, "whittle_tests");
    let start = w.state.time;
    w.apply(&action);
    assert!(said(
        &w,
        "You set to whittling the short bow with your knife."
    ));
    assert!(said(&w, "The wood takes the knife well."));
    assert_eq!(w.state.time.since(start).as_seconds(), 600);
    assert_eq!(marks(&w), 1, "every attempt marks Woodcraft");
    let after = worked_after(&w).expect("worked");
    assert_eq!(
        (after.improved.len(), after.attempts, after.damaged),
        (1, 1, false)
    );
    assert!(
        after.improved[0] != Aspect::Defence,
        "a bow has attack aspects only"
    );
    let name = w.lore().title(&w.state.player().inventory[0]);
    assert!(name.contains("Fine short bow"), "{name}");
}

#[test]
fn the_table_is_the_anvils_row_by_row() {
    // Made once, three times at most; failed, damaged and stripped; damaged, mended or gone.
    let mut w = world(false);
    give(&mut w, "knife");
    give(&mut w, "club");
    let action = Action::Whittle { item: 0 };
    let find = |w: &World, made: bool| {
        (0..500)
            .find(|&n| {
                let mut v = w.clone();
                v.state.rng = Rng::stream(n, "whittle_tests");
                v.apply(&action);
                let after = worked_after(&v);
                let damaged = after.as_ref().is_some_and(|a| a.damaged);
                made == (after.is_some() && !damaged)
            })
            .expect("a draw")
    };
    let step = |w: &mut World, made: bool| {
        w.state.rng = Rng::stream(find(w, made), "whittle_tests");
        w.apply(&action);
    };
    step(&mut w, true);
    step(&mut w, true);
    step(&mut w, true);
    assert_eq!(worked_after(&w).expect("worked").improved.len(), 3);
    let before = w.state.time;
    w.apply(&action);
    assert!(said(&w, "It can be made no finer."));
    assert_eq!(w.state.time, before, "refused, no time");
    // A failure damages it and takes all three.
    let mut failed = w.clone();
    failed.state.rng = Rng::stream(0, "whittle_tests");
    let mut damage = None;
    for n in 0..500 {
        let mut v = failed.clone();
        v.state.rng = Rng::stream(n, "whittle_tests");
        // Three improvements are refused: take one away to try again.
        let mut s = v.state.player().inventory[0]
            .worked()
            .cloned()
            .expect("worked");
        s.improved.truncate(2);
        v.player_mut().inventory[0].set_worked(s);
        v.apply(&action);
        if worked_after(&v).is_some_and(|a| a.damaged) {
            damage = Some(v);
            break;
        }
    }
    let mut damaged = damage.expect("a failed draw");
    let state = worked_after(&damaged).expect("worked");
    assert!(state.damaged && state.improved.is_empty() && state.attempts == 4);
    assert!(said(&damaged, "The knife slips and splits the wood."));
    // A damaged item is mended on a made attempt.
    step(&mut damaged, true);
    assert!(said(&damaged, "You pare the damage away."));
    let state = worked_after(&damaged).expect("worked");
    assert!(!state.damaged && state.attempts == 5);
    // And destroyed on a failed one.
    let mut again = damaged.clone();
    again.player_mut().inventory[0].set_worked(Worked {
        improved: Vec::new(),
        attempts: 5,
        damaged: true,
        by: None,
    });
    let gone = (0..500).find(|&n| {
        let mut v = again.clone();
        v.state.rng = Rng::stream(n, "whittle_tests");
        v.apply(&action);
        v.state.player().inventory.len() == again.state.player().inventory.len() - 1
    });
    assert!(gone.is_some(), "a failure destroys a damaged item");
}

#[test]
fn something_coming_into_view_stops_it() {
    let mut w = world(true);
    give(&mut w, "knife");
    give(&mut w, "short_bow");
    let start = w.state.time;
    w.apply(&Action::Whittle { item: 0 });
    assert!(said(&w, "You put down the knife."));
    assert!(w.state.time.since(start).as_seconds() < 600);
    assert_eq!(worked_after(&w), None, "no attempt counted");
}

#[test]
fn the_action_text_round_trips() {
    let action: Action = "whittle:b".parse().expect("parses");
    assert_eq!(action, Action::Whittle { item: 1 });
    assert_eq!(action.to_string(), "whittle:b");
}
