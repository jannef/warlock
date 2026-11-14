//! Gammer Wynflæd's quest, "Taken under the hill" (#1141; `campaigns/ascmearc/data/quests/
//! wynflaed.toml`): the Ascmearc's first escort. Asked, she marks Ælfhām on the map and Ælfrun
//! is made on the moss bed at its bottom (`escort_waits.rs`), or there as the floor is first made;
//! spoken with, the girl follows in the coward stance, and home in Holtwīc she says her thanks
//! and the wise-woman pays. Made once only; the load checks of `waits`; a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::{CombatStance, ContentFiles, Goal, QuestKind};
use crate::replay::Replay;
use crate::test_support::run as act;
use crate::test_support::texts as said;
use std::sync::LazyLock;

const WYNFLAED: Pos = Pos::new(54, 13);
/// The fairy mound over Ælfhām, on the overworld.
const MOUND: Pos = Pos::new(25, 13);
const QUEST: &str = "taken_under_the_hill";
const BOTTOM: &str = "aelfham:3";

/// Built-in content with nobody hungering, and nobody met on the way.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.remove_encounter_tables();
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn village() -> World {
    World::new(CONTENT.clone(), "holtwic", None).expect("a run in Holtwīc")
}

fn heard(world: &World, start: &str) -> bool {
    said(world).iter().any(|m| m.starts_with(start))
}

fn put_player(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at;
    world.perceive(&mut Vec::new());
}

fn stage(world: &World) -> Option<Stage> {
    world.state.quests.record(QUEST).map(|r| r.stage)
}

/// Every Ælfrun in the run.
fn aelfruns(world: &World) -> Vec<&Actor> {
    (world.state.actors.values())
        .filter(|a| a.npc.as_deref() == Some("aelfrun"))
        .collect()
}

/// The moss bed's cell on the bottom floor.
fn moss_bed(world: &World) -> Pos {
    let level = &world.state.levels[BOTTOM];
    (level.places.iter())
        .find(|(_, p)| p.name == "the moss bed")
        .map(|(&at, _)| at)
        .expect("the moss bed")
}

/// Beside Gammer Wynflæd (west of her), asked about her quest.
fn given() -> World {
    let mut w = village();
    put_player(&mut w, WYNFLAED.step(Dir::W));
    act(&mut w, &format!("ask:E:quest-{QUEST}"));
    w
}

#[test]
fn the_quest_is_an_escort_home_to_holtwic() {
    let q = CONTENT.quest_id(QUEST).expect("the quest");
    let def = CONTENT.quest(q);
    assert_eq!(def.kind(), QuestKind::Escort);
    assert_eq!(def.campaign.as_deref(), Some("ascmearc"));
    let Goal::Escort { level, waits, .. } = &def.goal else {
        panic!("an escort");
    };
    assert_eq!(level, "holtwic");
    let waits = waits.as_ref().expect("placed by the quest");
    assert_eq!(
        (waits.level.as_str(), waits.at.as_str()),
        (BOTTOM, "the moss bed")
    );
}

#[test]
fn asked_she_marks_the_mound_and_the_girl_waits_at_the_bottom() {
    let mut w = given();
    assert_eq!(stage(&w), Some(Stage::Active));
    assert!(heard(&w, "Gammer Wynflæd says: \"It's Ælfrun, my dear"));
    assert!(w.state.revealed.contains("aelfham"), "the mound on the map");
    let log = &w.perception().quests()[0];
    assert_eq!(log.name, "Taken under the hill");
    assert!(aelfruns(&w).is_empty(), "the floor is not made yet");
    w.travel(BOTTOM, None).expect("down");
    let girl = aelfruns(&w);
    assert_eq!(girl.len(), 1);
    assert_eq!(girl[0].level, BOTTOM);
    assert_eq!(girl[0].pos, moss_bed(&w), "on the moss bed");
    assert_eq!(CONTENT.creature(girl[0].creature).id, "cottar");
    assert!(girl[0].ally.is_none());
    // Back up and down again: still one.
    w.travel("holtwic", None).expect("up");
    w.travel(BOTTOM, None).expect("down again");
    assert_eq!(aelfruns(&w).len(), 1);
}

#[test]
fn a_bottom_made_before_the_asking_gets_her_as_it_is_given() {
    let mut w = village();
    w.travel(BOTTOM, None).expect("down");
    assert!(aelfruns(&w).is_empty(), "no quest, no girl");
    w.travel("holtwic", None).expect("back");
    put_player(&mut w, WYNFLAED.step(Dir::W));
    act(&mut w, &format!("ask:E:quest-{QUEST}"));
    let girl = aelfruns(&w);
    assert_eq!(girl.len(), 1);
    assert_eq!(
        (girl[0].level.as_str(), girl[0].pos),
        (BOTTOM, moss_bed(&w))
    );
}

#[test]
fn spoken_with_she_follows_and_home_the_wise_woman_pays() {
    let mut w = given();
    w.travel(BOTTOM, None).expect("down");
    let bed = moss_bed(&w);
    let beside = (Dir::ALL.into_iter())
        .find(|&d| {
            let p = bed.step(d);
            w.state.levels[BOTTOM].is_walkable(&w.content, p)
                && w.state.actor_at(BOTTOM, p).is_none()
        })
        .expect("room beside the bed");
    put_player(&mut w, bed.step(beside));
    act(&mut w, &format!("chat:{}", beside.opposite()));
    assert!(heard(&w, "Ælfrun says: \"You're real?"));
    let girl = aelfruns(&w)[0];
    assert_eq!(girl.escorting(), Some(QUEST));
    assert_eq!(girl.stance, CombatStance::Coward);
    let gold = w.content.object_id("gold_coin").expect("gold");
    let before = super::shop::count_of(&w.state.player().inventory, gold);
    // Out by the mound, she in the retinue, and into the village.
    w.travel("ascmearc", Some(MOUND)).expect("out");
    assert_eq!(stage(&w), Some(Stage::Active));
    assert!(aelfruns(&w)[0].in_retinue(), "along on the map");
    w.travel("holtwic", Some(WYNFLAED.step(Dir::W)))
        .expect("home");
    assert_eq!(stage(&w), Some(Stage::Done));
    let girl = aelfruns(&w)[0];
    assert_eq!(girl.level, "holtwic", "she came home");
    assert!(girl.ally.is_none(), "no longer following");
    assert!(heard(&w, "Ælfrun says: \"Holtwīc!"));
    assert!(heard(&w, "Gammer Wynflæd says: \"Bless you"));
    assert!(heard(&w, "Quest complete: Taken under the hill."));
    assert!(heard(&w, "Gammer Wynflæd gives you"));
    let after = super::shop::count_of(&w.state.player().inventory, gold);
    assert!(after >= before + 60, "{before} -> {after}");
}

#[test]
fn waits_is_checked_at_load() {
    let bad = |from: &str, to: &str| {
        let mut files = ContentFiles::builtin();
        files.edit_table("quests", |t| match t.contains(QUEST) {
            true => t.replace(from, to),
            false => t.to_string(),
        });
        Content::load(&files).expect_err("refused").to_string()
    };
    let at = "at = \"the moss bed\"";
    assert!(bad(at, "at = \"the hay loft\"").contains("0 places named"));
    let level = "level = \"aelfham:3\"";
    assert!(bad(level, "level = \"aelfham:2\"").contains("0 places named"));
    assert!(bad(level, "level = \"aelfham:9\"").contains("unknown level"));
    let creature = "creature = \"cottar\"";
    assert!(bad(creature, "creature = \"cotter\"").contains("unknown creature"));
}

#[test]
fn the_chain_replays_exactly() {
    let mut w = village();
    let mut replay = Replay::new(&w);
    // Walk to her (the village replay's way), ask, and go on talking.
    let actions = format!(
        "move:N*8 move:E*11 move:NE move:N*3 move:E*11 ask:SE:quest-{QUEST} \
         ask:SE:quest-{QUEST} chat:SE wait*3"
    );
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(stage(&w), Some(Stage::Active));
    assert!(heard(
        &w,
        "Gammer Wynflæd says: \"Ælfrun's under the hill still"
    ));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
