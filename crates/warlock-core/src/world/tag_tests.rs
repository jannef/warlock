//! Kill quests after a creature tag (spec §8 "Quests", #421, #441): the player's kills of any
//! type carrying it count, on every level summed; one bled to death of the player's wound
//! counts, one killed by a trap or another creature does not; an actor's `faction_override`
//! changes nothing; the objective names the tag's types; `what` renames the noun.

use super::fight::Harm;
use super::*;
use crate::content::{ContentFiles, TrapId};

const QUESTS: &str = r#"
[[quest]]
id = "cull"
name = "Kobold cull"
giver = "shopkeeper"
type = "kill"
target = "tag:kobold"
count = 150
offer = "Kobolds."
accept = "Good."
reminder = "Kobolds."
done = "Thanks."

[[quest]]
id = "arena_cull"
name = "Arena cull"
giver = "shopkeeper"
type = "kill"
target = "tag:kobold"
count = 3
level = "arena"
offer = "Kobolds in the arena."
accept = "Good."
reminder = "Kobolds."
done = "Thanks."
"#;

/// The arena (kobolds, a kobold archer and shaman, giant rats), with both quests above active.
fn arena() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/quests/tag_test.toml", QUESTS);
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "arena", None).expect("run");
    w.state.quests = serde_json::from_str(
        r#"{"records": {"cull": {"stage": "active"}, "arena_cull": {"stage": "active"}}}"#,
    )
    .expect("records");
    w
}

/// The first actor of type `creature` on the level.
fn first(w: &World, creature: &str) -> ActorId {
    let c = w.content().creature_id(creature).expect("creature");
    (w.state.actors.values())
        .find(|a| a.creature == c)
        .expect("there is one")
        .id
}

/// Kills actor `id` outright by `harm`.
fn kill(w: &mut World, id: ActorId, harm: Harm) {
    assert!(w.hurt(id, 10_000, harm, &mut Vec::new()), "it died");
}

/// Quest `id`'s progress, as the log has it.
fn status(w: &World, id: &str) -> String {
    let quests = w.perception().quests();
    let quest = quests.iter().find(|q| q.id == id).expect("logged");
    quest.status.clone()
}

#[test]
fn every_member_counts_and_levels_sum() {
    let mut w = arena();
    let player = w.player_id();
    for creature in ["kobold", "kobold_archer", "kobold_shaman"] {
        let id = first(&w, creature);
        kill(&mut w, id, Harm::Actor(player));
    }
    // A rat is no kobold.
    let rat = first(&w, "rat");
    kill(&mut w, rat, Harm::Actor(player));
    assert_eq!(status(&w, "cull"), "3 / 150 killed.");
    // Kills on another level add to them; the quest that names the arena leaves them out.
    let kobold = w.content().creature_id("kobold").expect("kobold");
    w.tally_kill(kobold, "hub");
    w.tally_kill(kobold, "hub");
    assert_eq!(status(&w, "cull"), "5 / 150 killed.");
    assert_eq!(status(&w, "arena_cull"), "Return to the shopkeeper.");
}

#[test]
fn the_objective_names_the_tag_and_its_types() {
    let w = arena();
    let quests = w.perception().quests();
    let objective = |id: &str| {
        let q = quests.iter().find(|q| q.id == id).expect("logged");
        q.objective.clone()
    };
    assert_eq!(
        objective("cull"),
        "The shopkeeper wants you to kill 150 kobolds: kobolds, kobold archers, and kobold shamans."
    );
    assert_eq!(
        objective("arena_cull"),
        "The shopkeeper wants you to kill 3 kobolds in the Arena: kobolds, kobold archers, and kobold shamans."
    );
}

#[test]
fn a_member_bled_to_death_counts_and_is_said() {
    let mut w = arena();
    let (player, kobold) = (w.player_id(), first(&w, "kobold"));
    w.state.actors.get_mut(&kobold).expect("kobold").hp = 1;
    let bleeding = w.content().status_id("bleeding").expect("bleeding");
    w.afflict(kobold, bleeding, 5, Some(player), &mut Vec::new());
    for _ in 0..10 {
        if !w.state.actors.contains_key(&kobold) {
            break;
        }
        w.apply(&Action::Wait);
    }
    assert!(!w.state.actors.contains_key(&kobold), "it bled to death");
    assert_eq!(status(&w, "cull"), "1 / 150 killed.");
    let said: Vec<_> = w.messages().iter().map(|m| m.text.as_str()).collect();
    assert!(
        said.contains(&"Quest updated: 1 / 150 kobolds."),
        "{said:?}"
    );
}

#[test]
fn a_member_killed_by_a_trap_or_another_does_not_count() {
    let mut w = arena();
    let kobold = first(&w, "kobold");
    kill(&mut w, kobold, Harm::Trap(TrapId(0)));
    let (archer, rat) = (first(&w, "kobold_archer"), first(&w, "rat"));
    kill(&mut w, archer, Harm::Actor(rat));
    // A status no one dealt (a poison of its own, say).
    let (shaman, when) = (first(&w, "kobold_shaman"), w.state.time);
    kill(
        &mut w,
        shaman,
        Harm::Status {
            of: None,
            by: None,
            when,
        },
    );
    assert_eq!(status(&w, "cull"), "0 / 150 killed.");
    assert_eq!(w.quests().kills().count(), 0, "nothing tallied");
}

#[test]
fn a_faction_override_changes_no_tag() {
    let mut w = arena();
    let player = w.player_id();
    let factions = w.content().factions();
    let (merchants, kobolds) = (
        factions.id("merchants").expect("merchants"),
        factions.id("kobolds").expect("kobolds"),
    );
    // A kobold gone over to the merchants is a kobold still; a rat gone over to the kobolds
    // is no kobold.
    let (kobold, rat) = (first(&w, "kobold"), first(&w, "rat"));
    w.state
        .actors
        .get_mut(&kobold)
        .expect("kobold")
        .faction_override = Some(merchants);
    w.state.actors.get_mut(&rat).expect("rat").faction_override = Some(kobolds);
    kill(&mut w, kobold, Harm::Actor(player));
    kill(&mut w, rat, Harm::Actor(player));
    assert_eq!(status(&w, "cull"), "1 / 150 killed.");
}

#[test]
fn what_renames_the_noun() {
    let mut files = ContentFiles::builtin();
    let quests = QUESTS.replace("count = 150\n", "count = 150\nwhat = \"scalies\"\n");
    files.insert("data/quests/tag_test.toml", &quests);
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "arena", None).expect("run");
    w.state.quests =
        serde_json::from_str(r#"{"records": {"cull": {"stage": "active"}}}"#).expect("records");
    let quests = w.perception().quests();
    let quest = quests.iter().find(|q| q.id == "cull").expect("logged");
    assert_eq!(
        quest.objective,
        "The shopkeeper wants you to kill 150 scalies: kobolds, kobold archers, and kobold shamans."
    );
    // Bled to death of the player's wound, during an action: the progress is said.
    let (player, kobold) = (w.player_id(), first(&w, "kobold"));
    w.state.actors.get_mut(&kobold).expect("kobold").hp = 1;
    let bleeding = w.content().status_id("bleeding").expect("bleeding");
    w.afflict(kobold, bleeding, 5, Some(player), &mut Vec::new());
    for _ in 0..10 {
        if !w.state.actors.contains_key(&kobold) {
            break;
        }
        w.apply(&Action::Wait);
    }
    let said: Vec<_> = w.messages().iter().map(|m| m.text.as_str()).collect();
    assert!(
        said.contains(&"Quest updated: 1 / 150 scalies."),
        "{said:?}"
    );
}

#[test]
fn tag_targets_are_checked_at_load() {
    let load = |files: &ContentFiles| Content::load(files).map(|_| ()).map_err(|e| e.to_string());
    let first_quest = || {
        let first = QUESTS.split("\n[[quest]]").nth(1).expect("the first");
        format!("[[quest]]{first}")
    };
    let with_quest = |target: &str, count: u32| {
        let quest = first_quest()
            .replace("tag:kobold", target)
            .replace("count = 150", &format!("count = {count}"));
        let mut files = ContentFiles::builtin();
        files.insert("data/quests/tag_test.toml", &quest);
        load(&files)
    };
    assert_eq!(with_quest("tag:goblin", 999), Ok(()));
    assert_eq!(with_quest("tag:marsh-thing", 5), Ok(()));
    let error = with_quest("tag:elf", 5).expect_err("refused");
    assert!(
        error.contains("tag `elf`: no creature carries it"),
        "{error}"
    );
    let error = with_quest("tag:Goblin", 5).expect_err("refused");
    assert!(error.contains("lower-case words"), "{error}");
    let error = with_quest("tag:goblin", 1000).expect_err("refused");
    assert!(error.contains("`count` is 1 to 999"), "{error}");
    let error = with_quest("goblin", 100).expect_err("refused");
    assert!(error.contains("`count` is 1 to 99"), "{error}");
    // A tag only another campaign's creatures carry: the hermit is the showcase's, so an
    // Ascmearc quest may not ask for it.
    let mut files = ContentFiles::builtin();
    files.edit_table("creatures", |text| {
        text.replace(
            "id = \"hermit\"\n",
            "id = \"hermit\"\ntags = [\"hermit\"]\n",
        )
    });
    let quest = first_quest().replace("tag:kobold", "tag:hermit");
    files.insert("campaigns/ascmearc/data/quests/tag_test.toml", &quest);
    let error = load(&files).expect_err("refused");
    assert!(
        error.contains("tag `hermit`: only creatures of campaign `showcase` carry it"),
        "{error}"
    );
}

#[test]
fn bad_creature_tags_are_refused() {
    for (tags, says) in [
        ("[\"Goblin\"]", "lower-case words"),
        ("[\"goblin\", \"goblin\"]", "tag `goblin` twice"),
    ] {
        let mut files = ContentFiles::builtin();
        files.edit_table("creatures", |text| {
            text.replace("tags = [\"goblin\"]\n", &format!("tags = {tags}\n"))
        });
        let error = Content::load(&files).map(|_| ()).expect_err("refused");
        assert!(error.to_string().contains(says), "{tags}: {error}");
    }
}
