//! Kill quests after a tribe (spec §8 "Quests", #421): the player's kills of any member type
//! count, on every level summed; a member bled to death of the player's wound counts, one
//! killed by a trap or another creature does not; an actor's `faction_override` changes nothing;
//! the objective names the members.

use super::fight::Harm;
use super::*;
use crate::content::{ContentFiles, TrapId};

const QUESTS: &str = r#"
[[quest]]
id = "cull"
name = "Kobold cull"
giver = "shopkeeper"
type = "kill"
target = "tribe:kobolds"
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
target = "tribe:kobolds"
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
    files.insert("data/quests/tribe_test.toml", QUESTS);
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
fn the_objective_names_the_tribe_and_its_members() {
    let w = arena();
    let quests = w.perception().quests();
    let objective = |id: &str| {
        let q = quests.iter().find(|q| q.id == id).expect("logged");
        q.objective.clone()
    };
    assert_eq!(
        objective("cull"),
        "Kill 150 kobolds: kobolds, kobold archers, and kobold shamans."
    );
    assert_eq!(
        objective("arena_cull"),
        "Kill 3 kobolds in the Arena: kobolds, kobold archers, and kobold shamans."
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
fn a_faction_override_changes_no_membership() {
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
fn tribe_targets_are_checked_at_load() {
    let load = |files: &ContentFiles| Content::load(files).map(|_| ()).map_err(|e| e.to_string());
    let with_quest = |target: &str, count: u32| {
        let first = QUESTS.split("\n[[quest]]").nth(1).expect("the first");
        let quest = format!("[[quest]]{first}")
            .replace("tribe:kobolds", target)
            .replace("count = 150", &format!("count = {count}"));
        let mut files = ContentFiles::builtin();
        files.insert("data/quests/tribe_test.toml", &quest);
        load(&files)
    };
    assert_eq!(with_quest("tribe:goblins", 999), Ok(()));
    let error = with_quest("tribe:elves", 5).expect_err("refused");
    assert!(error.contains("unknown tribe `elves`"), "{error}");
    let error = with_quest("tribe:goblins", 1000).expect_err("refused");
    assert!(error.contains("`count` is 1 to 999"), "{error}");
    let error = with_quest("goblin", 100).expect_err("refused");
    assert!(error.contains("`count` is 1 to 99"), "{error}");
    // A shared tribe names shared creature types only.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/tribes/hags.toml",
        "[[tribe]]\nid = \"hags\"\nname = \"hag\"\nplural = \"hags\"\ncreatures = [\"bog_hag\"]\n",
    );
    let error = load(&files).expect_err("refused");
    assert!(
        error.contains("a creature of campaign `ascmearc`"),
        "{error}"
    );
}
