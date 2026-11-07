//! Prestige classes and the calling (spec §8 "Prestige classes"; #876): a class's cap holds the
//! level, a calling opens to the right class at the right level, `World::call` gives the class
//! and exactly the experience the first prestige level costs, and every level past the cap is
//! the prestige class's: its hit points, rolls, bonus picks, and class abilities. On a fixture
//! prestige class, the Warden, called from a bard capped at 12 (the shipped bard ends at 30;
//! a short class keeps these tests small).

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor, Table};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::sync::LazyLock;

/// The bard's cap in this fixture.
const CAP: u8 = 12;

const WARDEN: &str = r##"
[[prestige]]
id = "warden"
name = "Warden"
description = "One who swore the road's oath and was answered."
levels = 5
hp = 7
rolls = 3
known_for = ["athletics"]
bonus_talents = { levels = [2], trees = ["guard"] }
abilities = [
    { level = 1, talent = "fencing" },
    { level = 2, talent = "riposte" },
]
"##;

const CALLING: &str = r##"
[[calling]]
class = "warden"
from = ["bard"]
level = 12
message = "The oath is in you now, as a blade is in its sheath. You are a warden."
"##;

/// The built-in content with the class tables stripped, the bard capped at 12, and the Warden
/// and its calling in the showcase.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    crate::content::strip_class_tables(&mut files);
    files.edit_table("classes", |text| match text.contains("id = \"bard\"") {
        true => text.replace("max_level = 30", &format!("max_level = {CAP}")),
        false => text.to_string(),
    });
    files.insert("data/prestige/warden.toml", WARDEN);
    // The test world's own door into the Hunter, from the bard too, makes way for the Warden's
    // (one door a class), and the hermit's topic calls the Warden instead.
    files.remove("campaigns/showcase/data/callings/hunter.toml");
    files.edit_table("dialogue", |t| {
        t.replace(
            "hunter.\"\ncalls = \"hunter\"",
            "hunter.\"\ncalls = \"warden\"",
        )
    });
    files.insert("campaigns/showcase/data/callings/warden.toml", CALLING);
    Arc::new(Content::load(&files).expect("fixture content"))
});

/// A human of `class` in the showcase.
fn world(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(FIXTURE.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn threshold(world: &World, level: u8) -> u32 {
    let spec = world.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(world.content(), spec);
    world.content().experience().threshold_for(level, factor)
}

/// Grants the XP that takes the player to `level`'s threshold (the level itself follows only
/// as far as the cap lets it), spending every choice due.
fn reach(world: &mut World, level: u8) {
    settle(world);
    let more = threshold(world, level) - world.player_xp();
    run(world, &format!("xp:{more}"));
    settle(world);
}

/// Spends every choice due (while one is, every other action is refused).
fn settle(world: &mut World) {
    while world.due().is_some() {
        run(world, "levelup:auto");
    }
}

fn holds(world: &World, id: &str) -> bool {
    world.state.player().talents.holds(id, None)
}

fn stat_row(world: &World, start: &str) -> String {
    (dump::layer_rows(world, Layer::Stats).into_iter())
        .find(|r| r.starts_with(start))
        .unwrap_or_default()
}

#[test]
fn a_class_cap_holds_the_level_and_banks_the_experience() {
    let mut w = world("bard");
    assert_eq!(w.level_cap(), CAP);
    assert_eq!(FIXTURE.class("bard").expect("bard").max_level, CAP);
    assert_eq!(
        FIXTURE.class("thief").expect("thief").max_level,
        60,
        "the default"
    );
    reach(&mut w, 15);
    assert_eq!(w.player_level(), CAP, "stopped at the cap");
    assert_eq!(w.player_xp(), threshold(&w, 15), "the bank is kept");
    let xp = w.perception().own_experience();
    assert_eq!((xp.level, xp.next), (CAP, None), "no next level at the cap");
    assert_eq!(
        stat_row(&w, "Class"),
        "Class Bard 12",
        "full: the cap shown"
    );
    // The dev progress dump says where the player stands with the calling.
    let rows = dump::layer_rows(&w, Layer::Progress);
    assert!(
        rows.contains(&"calling warden: open".to_string()),
        "{rows:?}"
    );
}

#[test]
fn a_calling_opens_to_its_class_at_its_level() {
    let mut w = world("bard");
    assert!(!w.calling_open("warden"), "level 1");
    let calling = FIXTURE
        .calling("showcase", "warden")
        .expect("the fixture's");
    assert_eq!(w.calling_status(calling), CallingStatus::LevelNotMet);
    let rows = dump::layer_rows(&w, Layer::Progress);
    assert!(
        rows.contains(&"calling warden: level 12 not met".to_string()),
        "{rows:?}"
    );
    reach(&mut w, CAP);
    assert!(w.calling_open("warden"));
    assert!(!w.calling_open("hunter"), "no such calling");
    // A thief is none it is from.
    let mut t = world("thief");
    reach(&mut t, CAP);
    assert!(!t.calling_open("warden"));
    assert_eq!(t.calling_status(calling), CallingStatus::NotFrom);
    let rows = dump::layer_rows(&t, Layer::Progress);
    assert!(
        rows.contains(&"calling warden: not for a thief".to_string()),
        "{rows:?}"
    );
    run(&mut t, "call:warden");
    assert!(
        said(&t)
            .last()
            .expect("said")
            .contains("No calling of this world opens")
    );
    assert!(t.character().expect("character").prestige.is_none());
}

#[test]
fn the_calling_gives_the_class_and_exactly_the_next_level() {
    let mut w = world("bard");
    reach(&mut w, CAP);
    let hp_before = w.state.player().max_hp(w.content());
    let toughness =
        (w.state.player().raised_stats(w.content())).modifier(crate::stats::Stat::Toughness);
    run(&mut w, "call:warden");
    let c = w.character().expect("character");
    assert_eq!(c.prestige.as_deref(), Some("warden"));
    // Exactly the next level's threshold, and the level with it: Warden 1 at total 13.
    assert_eq!(w.player_xp(), threshold(&w, CAP + 1));
    assert_eq!(w.player_level(), CAP + 1);
    assert_eq!(w.level_cap(), CAP + 5);
    let (table, own) = c.class_of(w.content(), CAP + 1).expect("a class");
    assert!(matches!(table, Table::Prestige(p) if p.id == "warden") && own == 1);
    let (table, own) = c.class_of(w.content(), CAP).expect("a class");
    assert!(matches!(table, Table::Class(k) if k.id == "bard") && own == CAP);
    // The level's hit points are the Warden's (7 ÷ 2 + Toughness), its ability given, the
    // message said, then the level's news.
    assert_eq!(
        w.state.player().max_hp(w.content()),
        hp_before + (3 + toughness).max(1)
    );
    assert!(holds(&w, "fencing") && !holds(&w, "riposte"));
    let spoken = said(&w);
    let at = |text: &str| {
        spoken
            .iter()
            .position(|m| m == text)
            .unwrap_or_else(|| panic!("{text}: {spoken:?}"))
    };
    assert!(
        at("The oath is in you now, as a blade is in its sheath. You are a warden.")
            < at("Welcome to level 13.")
    );
    assert!(at("Welcome to level 13.") < at("You gain Fencing."));
    // Its skill rolls are the Warden's 3 (the bard's 4 before).
    let due = w.due().expect("level 13's choices");
    let count = crate::world::roll_count(w.content(), w.state.player(), w.character(), CAP + 1);
    assert_eq!((count.class, due.rolls), (3, count.total()));
    let count = crate::world::roll_count(w.content(), w.state.player(), w.character(), CAP);
    assert_eq!(count.class, 4, "the bard's at a bard level");
    // The character screen's row and the dumps.
    assert_eq!(stat_row(&w, "Class"), "Class Bard 12, Warden 1");
    assert_eq!(c.class_phrase(w.content()), "a bard and warden");
    let rows = dump::layer_rows(&w, Layer::Progress);
    assert!(
        rows.contains(&"called warden at 12".to_string()),
        "{rows:?}"
    );
    let talents = dump::layer_rows(&w, Layer::Talents);
    assert!(
        talents.contains(&"Fencing (class)".to_string()),
        "{talents:?}"
    );
    let json = dump::json(&w, &[], true);
    assert_eq!(json["character"]["prestige"], "warden");
    assert_eq!(
        json["character"]["classes"],
        serde_json::json!([{ "class": "bard", "level": 12 }, { "class": "warden", "level": 1 }])
    );
    assert_eq!(json["callings"][0]["status"], "called");
    // The XP row shows the next level again.
    let xp = w.perception().own_experience();
    assert_eq!(xp.next, Some(threshold(&w, CAP + 2)));
    // Called once: the calling is shut, and the dev action refuses.
    assert!(!w.calling_open("warden"));
    settle(&mut w);
    run(&mut w, "call:warden");
    assert!(said(&w).last().expect("said").contains("already"));
}

#[test]
fn a_full_bank_is_spent_at_the_calling_and_nothing_is_given() {
    let mut w = world("bard");
    reach(&mut w, 15);
    let banked = w.player_xp();
    run(&mut w, "call:warden");
    assert_eq!(w.player_xp(), banked, "nothing given: the bank covers it");
    assert_eq!(w.player_level(), 15, "every level the bank buys, at once");
    // Level 14 is Warden 2: Riposte, and a bonus pick from Guard.
    assert!(holds(&w, "fencing") && holds(&w, "riposte"));
    let dues = w.state.progress.due.clone();
    let at_14 = dues
        .iter()
        .find(|d| d.level == 14)
        .expect("level 14's choices");
    assert_eq!(at_14.bonus, 1);
    let guard = FIXTURE.talents().tree_id("guard").expect("guard");
    assert_eq!(w.bonus_trees_at(14), Some(vec![guard]));
    assert_eq!(
        w.bonus_trees_at(CAP),
        w.bonus_trees_at(1),
        "a bard level's trees are the bard's"
    );
    assert_eq!(stat_row(&w, "Class"), "Class Bard 12, Warden 3");
    // The cap is the last Warden level: XP past it banks again.
    reach(&mut w, 20);
    assert_eq!(w.player_level(), CAP + 5);
    assert_eq!(stat_row(&w, "Class"), "Class Bard 12, Warden 5");
}

#[test]
fn the_dev_call_comes_whatever_the_level_and_refuses_what_is_not_there() {
    let mut w = world("bard");
    run(&mut w, "call:ranger");
    assert!(said(&w).last().expect("said").contains("no prestige class"));
    // The Hunter is a class, but this fixture's world opens no door into it.
    run(&mut w, "call:hunter");
    assert!(
        said(&w)
            .last()
            .expect("said")
            .contains("No calling of this world")
    );
    assert!(w.character().expect("character").prestige.is_none());
    // At level 1 the calling is not open, but the dev action makes it: Warden 1 at once.
    run(&mut w, "call:warden");
    assert_eq!(w.player_level(), CAP + 1);
    assert_eq!(w.player_xp(), threshold(&w, CAP + 1));
    assert!(holds(&w, "fencing"));
}

#[test]
fn a_lost_level_lowers_the_prestige_level_and_comes_back() {
    let mut w = world("bard");
    reach(&mut w, CAP);
    run(&mut w, "call:warden");
    while w.due().is_some() {
        run(&mut w, "levelup:auto");
    }
    let hp_at_13 = w.state.player().max_hp(w.content());
    assert!(w.lose_level());
    assert_eq!(w.player_level(), CAP);
    assert_eq!(
        stat_row(&w, "Class"),
        "Class Bard 12, Warden",
        "called, no Warden level now"
    );
    assert!(holds(&w, "fencing"), "nothing is taken away");
    // Gained again: the Warden's hit points come back, no choices do.
    reach(&mut w, CAP + 1);
    assert_eq!(w.state.player().max_hp(w.content()), hp_at_13);
    assert!(w.due().is_none());
}

#[test]
fn a_restore_keeps_the_class_and_grants_both_tables() {
    let mut w = world("bard");
    reach(&mut w, 15);
    run(&mut w, "call:warden");
    while w.due().is_some() {
        run(&mut w, "levelup:auto");
    }
    let state = w.state.as_ref().clone();
    let json = serde_json::to_string(&state).expect("json");
    assert!(json.contains("\"prestige\":\"warden\""), "saved");
    let back = World::from_state(FIXTURE.clone(), state).expect("restore");
    assert_eq!(back.state_hash(), w.state_hash());
    assert_eq!(back.player_level(), 15);
    assert!(holds(&back, "fencing") && holds(&back, "riposte"));
    // A character never called saves no prestige.
    let plain = world("bard");
    let json = serde_json::to_string(plain.state.as_ref()).expect("json");
    assert!(!json.contains("prestige"), "left out while none");
}

#[test]
fn the_calling_replays_exactly() {
    let mut w = world("bard");
    let more = threshold(&w, 15);
    let actions = parse_actions(&format!(
        "xp:{more} levelup:auto*6 call:warden levelup:auto*4 wait"
    ))
    .expect("actions");
    let mut replay = Replay::new(&w);
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert_eq!(w.player_level(), 15);
    assert!(holds(&w, "riposte"));
    let back = replay.run(FIXTURE.clone()).expect("replays");
    assert_eq!(back.state_hash(), w.state_hash());
}
