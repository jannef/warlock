//! A letter for the deep (#1435; `campaigns/ascmearc/data/quests/wulfrun.toml`): once Fire in
//! the deep is done, Factor Wulfrun gives a sealed letter for the deep gnomes; handed to Headman
//! Peran in Gimmwīc it is done, and he pays with a cut stone and gives the moot's answer, which
//! Wulfrun takes back in Cēaptūn. Dēopgang is never hidden, so neither quest reveals anything.

use super::*;
use crate::content::ContentFiles;
use crate::test_support::run as act;
use std::sync::LazyLock;

const WULFRUN: Pos = Pos::new(72, 26);

/// Built-in content with nobody hungering, and nobody met on the way.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    files.remove_encounter_tables();
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// The player's topics with whoever stands at `at` on the current level.
fn topics(world: &World, at: Pos) -> Vec<String> {
    let level = world.state.current_level.clone();
    let id = (world.state.actor_at(&level, at))
        .expect("someone there")
        .id;
    (world.topics(id).into_iter()).map(|t| t.id).collect()
}

/// Puts the player west of `at`, so they talk to the east.
fn beside(world: &mut World, at: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = at.step(Dir::W);
    world.perceive(&mut Vec::new());
}

fn carried(world: &World, object: &str) -> u32 {
    let object = world.content().object_id(object).expect("an object");
    super::shop::count_of(&world.state().player().inventory, object)
}

fn stage(world: &World, quest: &str) -> Option<Stage> {
    world.state.quests.record(quest).map(|r| r.stage)
}

/// Marks `done` quests done, as if handed in.
fn done(world: &mut World, done: &[&str]) {
    let records: Vec<String> = (done.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    world.state.quests = serde_json::from_str(&json).expect("quests");
}

/// Headman Peran's cell in Gimmwīc.
fn peran(world: &World) -> Pos {
    (world.state.actors.values())
        .find(|a| a.npc.as_deref() == Some("peran"))
        .map(|a| a.pos)
        .expect("Headman Peran")
}

#[test]
fn the_letter_goes_down_to_peran_and_his_answer_comes_back_up() {
    let mut w = World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn");
    beside(&mut w, WULFRUN);
    // Not before the drake is dead.
    done(&mut w, &["barrow_heart", "lost_shaft"]);
    assert!(!topics(&w, WULFRUN).contains(&"quest-gnome_letter".into()));
    done(&mut w, &["barrow_heart", "lost_shaft", "fire_deep"]);
    assert!(topics(&w, WULFRUN).contains(&"quest-gnome_letter".into()));

    // Asked, she hands over the letter; nothing is marked, Dēopgang is never hidden.
    let revealed = w.state.revealed.clone();
    act(&mut w, "ask:E:quest-gnome_letter");
    assert_eq!(stage(&w, "gnome_letter"), Some(Stage::Active));
    assert_eq!(carried(&w, "sealed_letter"), 1);
    assert_eq!(w.state.revealed, revealed);
    assert!(
        (w.content().hidden_location("ascmearc", "deopgang")).is_none(),
        "Dēopgang is open to anyone"
    );

    // Down in Gimmwīc, Peran takes it and pays with a cut stone.
    w.travel("gimmwic", None).expect("the deep gnomes' town");
    let headman = peran(&w);
    // A human without a light: the lamp over his seat shows him (#1435).
    beside(&mut w, headman);
    assert!(w.perception().is_visible(headman), "Peran is seen");
    let gems_before = carried(&w, "gem");
    assert!(topics(&w, headman).contains(&"quest-gnome_letter".into()));
    act(&mut w, "ask:E:quest-gnome_letter");
    assert_eq!(stage(&w, "gnome_letter"), Some(Stage::Done));
    assert_eq!(carried(&w, "sealed_letter"), 0);
    assert_eq!(carried(&w, "gem"), gems_before + 1);
    assert!(said(&w).iter().any(|m| m.contains("for the walk down")));

    // Then he has the moot's answer to send back up (once the level the reward brought is spent).
    assert!(topics(&w, headman).contains(&"quest-gnome_reply".into()));
    act(&mut w, "levelup:auto ask:E:quest-gnome_reply");
    assert_eq!(stage(&w, "gnome_reply"), Some(Stage::Active));
    assert_eq!(carried(&w, "sealed_letter"), 1);

    // Wulfrun takes it, and pays.
    w.travel("ceaptun", None).expect("back in Cēaptūn");
    beside(&mut w, WULFRUN);
    let gold = carried(&w, "gold_coin");
    act(&mut w, "ask:E:quest-gnome_reply");
    assert_eq!(stage(&w, "gnome_reply"), Some(Stage::Done));
    assert_eq!(carried(&w, "sealed_letter"), 0);
    assert_eq!(carried(&w, "gold_coin"), gold + 300);
    assert!(said(&w).iter().any(|m| m.contains("every word of it")));
}
