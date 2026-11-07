//! A Beast-friend's quests (#1189): Leofric sends one after the barrow's skeletal rats instead of
//! the living ones (the fallback variant), the talent read as the quest is given; and no rolled
//! kill quest of any campaign draws a creature type the talent calms, or a tag one carries.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Goal};
use std::sync::LazyLock;

const LEOFRIC: Pos = Pos::new(60, 26);

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

/// A run in Cēaptūn as a human of `class`, west of Leofric.
fn town(class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    let mut w = World::start_run(CONTENT.clone(), "ceaptun", Some(5), Some(&spec), None, true)
        .expect("a run in Cēaptūn");
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = LEOFRIC.step(Dir::W);
    w.perceive(&mut Vec::new());
    w
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn def<'a>(world: &'a World, id: &str) -> &'a crate::content::QuestDef {
    let content = world.content();
    content.quest(content.quest_id(id).expect("a quest"))
}

fn rats_variant(world: &World) -> Option<String> {
    (world.quests().record("barrow_rats")).and_then(|r| r.roll.clone()?.variant)
}

/// Whether kill target `target` is a creature type of an animal faction, or a tag one carries.
fn animal(content: &Content, campaign: Option<&str>, target: &str) -> bool {
    let is = |c: crate::content::CreatureId| {
        (content.creature(c).faction).is_some_and(|f| content.factions().def(f).animal)
    };
    match target.strip_prefix("tag:") {
        Some(tag) => content.tagged_in(tag, campaign).any(is),
        None => is(content.creature_id(target).expect("a type")),
    }
}

#[test]
fn leofric_sends_a_druid_after_the_dead_rats() {
    let mut w = town("druid");
    act(&mut w, "ask:E:quest-barrow_rats");
    assert_eq!(rats_variant(&w).as_deref(), Some("skeletal"));
    let said: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        said.iter()
            .any(|m| m.contains("keep faith with the beasts")),
        "{said:?}"
    );
    let given = (w.perception().quests().into_iter())
        .find(|q| q.id == "barrow_rats")
        .expect("in the log");
    assert!(
        given.objective.contains("skeletal rats"),
        "{}",
        given.objective
    );
}

#[test]
fn leofric_sends_anyone_else_after_the_rats() {
    let mut w = town("warrior");
    act(&mut w, "ask:E:quest-barrow_rats");
    assert_eq!(rats_variant(&w).as_deref(), Some("rats"));
    let roll = w
        .quests()
        .record("barrow_rats")
        .and_then(|r| r.roll.clone());
    assert_eq!(roll.expect("drawn").target, "rat");
}

#[test]
fn the_talent_not_the_class_decides_and_only_as_the_quest_is_given() {
    // A warrior who is a Beast-friend is sent after the dead rats too.
    let mut w = town("warrior");
    w.player_mut().talents.take("beast_friend", None);
    act(&mut w, "ask:E:quest-barrow_rats");
    assert_eq!(rats_variant(&w).as_deref(), Some("skeletal"));
    // The talent taken after the giving changes nothing.
    let mut w = town("warrior");
    act(&mut w, "ask:E:quest-barrow_rats");
    w.player_mut().talents.take("beast_friend", None);
    assert_eq!(rats_variant(&w).as_deref(), Some("rats"));
    let json = serde_json::to_string(&w.state).expect("serializes");
    let back: WorldState = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.quests, w.state.quests, "the roll is saved");
}

#[test]
fn rats_killed_before_the_giving_still_count() {
    // The variants are authored: every kill counts, as before the alternative (a pick counts
    // from the giving only).
    let mut w = town("warrior");
    w.state.quests =
        serde_json::from_str(r#"{"kills": {"cealdhlaw:1": {"rat": 5}}}"#).expect("quests");
    act(&mut w, "ask:E:quest-barrow_rats ask:E:quest-barrow_rats");
    assert_eq!(
        w.quests().record("barrow_rats").map(|r| r.completed),
        Some(1)
    );
}

#[test]
fn no_rolled_kill_quest_draws_what_a_beast_friend_calms() {
    let mut druid = town("druid");
    let warrior = town("warrior");
    let content = CONTENT.clone();
    let mut kills = 0;
    for q in content.quests() {
        if !q.is_rolled() || !matches!(q.goal, Goal::Kill { .. }) {
            continue;
        }
        kills += 1;
        let campaign = q.campaign.as_deref();
        for c in druid.quest_pool(q) {
            assert!(
                !animal(&content, campaign, &c.target),
                "{}: {}",
                q.id,
                c.target
            );
        }
        // Many draws, never one.
        for seed in 0..200 {
            druid.state.seed = seed;
            if let Some(roll) = druid.roll_quest(q) {
                assert!(!animal(&content, campaign, &roll.target), "{}", q.id);
            }
        }
    }
    assert!(kills >= 3, "the guild's two and Leofric's rats at least");
    // The filter matters: anyone else's pools hold the rats and the vermin.
    let has = |w: &World, q: &str, t: &str| w.quest_pool(def(w, q)).iter().any(|c| c.target == t);
    assert!(has(&warrior, "guild_score", "tag:vermin"));
    assert!(!has(&druid, "guild_score", "tag:vermin"));
    assert!(has(&druid, "guild_score", "tag:undead"));
    let quarry = |w: &World| w.quest_pool(def(w, "guild_quarry")).len();
    assert!(quarry(&druid) < quarry(&warrior));
    // The pool size the `progress` dump shows is the druid's.
    assert!(
        (druid.quest_pool(def(&druid, "barrow_rats")).iter()).all(|c| c.target == "skeletal_rat")
    );
}
