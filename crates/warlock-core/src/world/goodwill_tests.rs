//! Cēaptūn's goodwill (#1247; `goodwill.rs`): every quest its people give that the player
//! completes makes their services 5% cheaper, after every other discount, up to free; the player
//! is told as it rises; quests from elsewhere do not count; it is derived from the saved records.

use super::goodwill::discounted;
use super::*;
use crate::content::ContentFiles;
use crate::test_support::run as act;
use crate::test_support::texts as said;
use std::sync::LazyLock;

const HILD: Pos = Pos::new(27, 25);
const LEOFRIC: Pos = Pos::new(60, 26);
const WULFRUN: Pos = Pos::new(72, 26);

/// Built-in content with nobody hungering (a disease takes days to show).
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn town() -> World {
    World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn")
}

/// Puts the player west of `npc`, so they talk to the east.
fn beside(world: &mut World, npc: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = npc.step(Dir::W);
    world.perceive(&mut Vec::new());
}

fn gold(world: &World) -> u32 {
    let coin = world.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&world.state().player().inventory, coin)
}

/// The price of service `id` of whoever stands on `at`.
fn price(world: &World, at: Pos, id: &str) -> Option<u32> {
    let who = (world.state.actor_at("ceaptun", at))
        .expect("someone there")
        .id;
    (world.topics(who).into_iter())
        .find(|t| t.id == id)
        .and_then(|t| t.price)
}

/// Sets the quest records: each quest done `n` times.
fn done(world: &mut World, done: &[(&str, u32)]) {
    let records: Vec<String> = (done.iter())
        .map(|(q, n)| format!(r#""{q}": {{"stage": "done", "completed": {n}}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    world.state.quests = serde_json::from_str(&json).expect("quests");
}

#[test]
fn the_discount_is_rounded_up_and_free_only_at_a_hundred_percent() {
    assert_eq!(discounted(60, 0), 60);
    assert_eq!(discounted(60, 5), 57);
    assert_eq!(discounted(40, 15), 34);
    assert_eq!(discounted(1, 95), 1, "a coin's last part is still paid");
    assert_eq!(discounted(100, 100), 0);
    assert_eq!(discounted(100, 150), 0);
}

#[test]
fn a_quest_done_for_the_reeve_makes_the_town_cheaper_and_says_so() {
    let mut w = town();
    assert_eq!(price(&w, HILD, "cure"), Some(60));
    assert_eq!(price(&w, WULFRUN, "identify"), Some(100));
    w.state.quests =
        serde_json::from_str(r#"{"kills": {"cealdhlaw:1": {"rat": 5}}}"#).expect("quests");
    beside(&mut w, LEOFRIC);
    act(&mut w, "ask:E:quest-barrow_rats ask:E:quest-barrow_rats");
    let told = said(&w);
    let complete = (told.iter())
        .position(|m| m.starts_with("Quest complete:"))
        .expect("the quest is done");
    assert_eq!(
        told[complete + 1],
        "Cēaptūn's folk think the better of you."
    );
    let town = &CONTENT.towns()[0];
    assert_eq!(w.goodwill(town), 1);
    assert_eq!(price(&w, HILD, "cure"), Some(57));
    assert_eq!(price(&w, HILD, "mend"), Some(38));
    assert_eq!(price(&w, WULFRUN, "identify"), Some(95));
    // Kept: derived from the quest records the save holds.
    let loaded = crate::save::decode(&crate::save::encode(&w), CONTENT.clone()).expect("loads");
    assert_eq!(price(&loaded, HILD, "cure"), Some(57));
}

#[test]
fn every_completion_counts_but_quests_from_elsewhere_do_not() {
    let mut w = town();
    // Gammer Wynflæd's is Holtwīc's, not the town's.
    done(&mut w, &[("taken_under_the_hill", 1)]);
    assert_eq!(price(&w, HILD, "cure"), Some(60));
    // The Gilde's count, and a repeated quest counts each time.
    done(
        &mut w,
        &[("barrow_rats", 3), ("guild_quarry", 1), ("grave_goods", 1)],
    );
    assert_eq!(w.goodwill(&CONTENT.towns()[0]), 5);
    assert_eq!(price(&w, HILD, "cure"), Some(45));
    assert_eq!(price(&w, WULFRUN, "identify"), Some(75));
}

#[test]
fn twenty_quests_make_the_town_s_services_free() {
    let mut w = town();
    done(&mut w, &[("barrow_rats", 25)]);
    assert_eq!(price(&w, HILD, "cure"), Some(0), "the cap is everything");
    let before = gold(&w);
    beside(&mut w, HILD);
    act(&mut w, "infect:the_shakes rest:24h*3");
    assert!(w.state().player().diseases.shows(), "the shakes show");
    act(&mut w, "ask:E:cure");
    assert!(w.state().player().diseases.iter().next().is_none(), "cured");
    assert_eq!(gold(&w), before, "nothing paid");
    assert!(!said(&w).iter().any(|m| m.starts_with("You pay")));
}
