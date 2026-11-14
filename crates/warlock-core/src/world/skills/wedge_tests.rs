//! Wedging a door shut (spec §8 "Doors"; #1027): a turn's work for those trained in Woodcraft;
//! a creature that opens doors must kick the door in instead; the player opens it as usual; the
//! wedge is saved, hashed, and replays.

use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stats};
use crate::test_support::last_text as last_message;
use std::sync::LazyLock;

/// A hall split by iron bars and a door; the hobgoblin sees the player through the bars, and its way round is the door beside the player.
const HALL: &str = "\
#########
#...#...#
#k..=@..#
#...+...#
#########
";

const DOOR: Pos = Pos::new(4, 3);

static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_wedge.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"hobgoblin\" }}\n             \"=\" = {{ terrain = \"statue\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_wedge.toml",
        "name = \"T\"\nstart_level = \"test_wedge\"\n\
         character = { race = \"human\", gender = \"female\", class = \"ranger\" }\n",
    );
    files.insert(
        "scenarios/test_wedge_plain.toml",
        "name = \"T\"\nstart_level = \"test_wedge\"\n\
         character = { race = \"human\", gender = \"female\", class = \"commoner\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The hall, the player a tough human with Woodcraft `woodcraft` (none: untrained).
fn world(woodcraft: Option<u16>) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_wedge_plain", Some(7)).expect("world");
    let id: SkillId = w.content.skill_id("woodcraft").expect("Woodcraft");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    let rest: Vec<_> = me.skills.iter().filter(|(s, _)| *s != id).collect();
    me.skills = rest
        .into_iter()
        .chain(woodcraft.map(|value| (id, Trained { value, marks: 0 })))
        .collect();
    w
}

fn terrain(w: &World) -> String {
    let (level, _) = w.ground_truth();
    w.content().terrain(level.terrain[DOOR]).id.clone()
}

fn wedged(w: &World) -> Option<u8> {
    w.state().current().wedged.get(&DOOR).copied()
}

#[test]
fn wedging_is_woodcrafts_and_refused_without_it() {
    let mut w = world(None);
    assert!(!Ability::Wedge.had_by(w.state().player(), w.content()));
    let out = w.apply(&Action::Wedge(Dir::SW));
    assert!(!out.took_turn);
    assert_eq!(wedged(&w), None);
    let mut w = world(Some(50));
    assert!(Ability::Wedge.had_by(w.state().player(), w.content()));
    assert!(
        w.perception()
            .abilities()
            .iter()
            .any(|a| a.ability == Ability::Wedge)
    );
    // No door that way, and a door only once.
    assert!(!w.apply(&Action::Wedge(Dir::E)).took_turn);
    assert!(w.apply(&Action::Wedge(Dir::SW)).took_turn);
    assert_eq!(last_message(&w), "You drive a wedge under the door.");
    assert!(wedged(&w).is_some());
    assert!(!w.apply(&Action::Wedge(Dir::SW)).took_turn);
    assert_eq!(last_message(&w), "That door is wedged already.");
}

#[test]
fn woodcraft_adds_to_the_wedge() {
    for (value, points) in [(0, 0), (100, 8), (200, 12)] {
        let mut w = world(Some(value));
        w.apply(&Action::Wedge(Dir::SW));
        assert_eq!(wedged(&w), Some(points), "Woodcraft {value}");
    }
}

#[test]
fn a_door_opening_creature_opens_a_plain_door_but_must_kick_a_wedged_one() {
    // Control: the hobgoblin opens an unwedged door.
    let mut w = world(Some(50));
    let opened = (0..40).map(|_| w.apply(&Action::Wait)).any(|o| {
        o.events
            .iter()
            .any(|e| matches!(e, Event::DoorOpened { .. }))
    });
    assert!(opened, "the hobgoblin opens a plain door");
    // Wedged: it never opens it; it kicks until the door breaks.
    let mut w = world(Some(50));
    w.apply(&Action::Wedge(Dir::SW));
    let (mut kicks, mut broke) = (0, false);
    for _ in 0..600 {
        let out = w.apply(&Action::Wait);
        for e in &out.events {
            assert!(!matches!(e, Event::DoorOpened { .. }), "{e:?}");
            kicks += usize::from(matches!(e, Event::DoorKicked { .. }));
            broke |= matches!(e, Event::DoorBroken { .. });
        }
        if broke {
            break;
        }
    }
    assert!(kicks > 0 || broke, "the hobgoblin kicks at the wedge");
    assert!(broke, "and breaks it in the end");
    assert_eq!(terrain(&w), "door_broken");
    assert_eq!(wedged(&w), None);
}

#[test]
fn the_player_opens_a_wedged_door_as_usual() {
    let mut w = world(Some(50));
    w.apply(&Action::Wedge(Dir::SW));
    let out = w.apply(&Action::Open(Dir::SW));
    assert!(out.took_turn);
    assert_eq!(terrain(&w), "door_open");
    assert_eq!(wedged(&w), None);
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text == "You pull the wedge free and open the door.")
    );
}

#[test]
fn a_wedge_makes_the_door_sturdier_against_the_players_kick() {
    // Strength 10 against sturdiness 30 plus 12: the average kick is a third likely to hold
    // less than it did; the wedge is gone once the door breaks.
    let mut w = world(Some(200));
    w.apply(&Action::Wedge(Dir::SW));
    assert_eq!(wedged(&w), Some(12));
    for _ in 0..400 {
        w.apply(&Action::Kick(Dir::SW));
        if terrain(&w) == "door_broken" {
            break;
        }
    }
    assert_eq!(terrain(&w), "door_broken");
    assert_eq!(wedged(&w), None);
}

#[test]
fn a_wedge_is_saved_and_counts_in_the_state_hash() {
    let mut w = world(Some(100));
    let before = w.state_hash();
    w.apply(&Action::Wedge(Dir::SW));
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state().current().wedged.get(&DOOR), Some(&8));
    assert_eq!(loaded.state_hash(), w.state_hash());
    // The same turn without the wedge hashes differently.
    let mut plain = world(Some(100));
    plain.apply(&Action::Wait);
    assert_ne!(plain.state_hash(), w.state_hash());
    assert_ne!(before, w.state_hash());
}

#[test]
fn wedging_and_kicking_replay_exactly() {
    let content = FIXTURE.clone();
    let mut w = World::new(content.clone(), "test_wedge", Some(7)).expect("world");
    assert!(Ability::Wedge.had_by(w.state().player(), w.content()));
    let mut replay = Replay::new(&w);
    for action in parse_actions("wedge:E wait*30 kick:E").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
