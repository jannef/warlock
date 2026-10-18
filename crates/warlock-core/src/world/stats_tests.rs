//! Stats (spec §8 "Stats"): every actor starts with its scores and potentials, they are saved and
//! hashed, and the player perceives only their own scores.

use super::*;
use crate::save;
use crate::stats::{Stat, Stats};
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn showcase() -> World {
    World::new(CONTENT.clone(), "showcase", None).expect("world")
}

fn scores(stats: &Stats) -> Vec<u8> {
    Stat::ALL.iter().map(|&s| stats.score(s)).collect()
}

fn potentials(stats: &Stats) -> Vec<u8> {
    Stat::ALL.iter().map(|&s| stats.potential(s)).collect()
}

#[test]
fn the_player_starts_with_the_scenarios_stats() {
    let w = showcase();
    let stats = &w.state.player().stats;
    assert_eq!(scores(stats), [12, 14, 11, 13, 9, 10]);
    assert_eq!(potentials(stats), [15, 17, 14, 16, 13, 12]);
    assert_eq!(stats.modifier(Stat::Dexterity), 2);
    assert_eq!(stats.modifier(Stat::Willpower), -1);
    // A scenario that gives none: average in everything.
    let depths = World::new(CONTENT.clone(), "depths", None).expect("world");
    assert_eq!(depths.state.player().stats, Stats::default());
}

#[test]
fn creatures_start_with_their_types_stats_on_every_level() {
    let mut w = showcase();
    let check = |w: &World| {
        let creatures: Vec<&Actor> = (w.state.actors.values())
            .filter(|a| a.kind == ActorKind::Creature)
            .collect();
        assert!(!creatures.is_empty());
        for actor in creatures {
            assert_eq!(actor.stats, w.content.creature(actor.creature).stats);
        }
    };
    check(&w);
    assert!(
        (w.state.actors.values()).any(|a| a.stats != Stats::default()),
        "the showcase's creatures are not all average"
    );
    // A level generated on its first visit spawns its creatures the same way.
    w.travel("depths:1", None).expect("travel");
    check(&w);
}

#[test]
fn stats_are_saved_and_hashed() {
    let mut w = showcase();
    w.apply(&Action::Wait);
    let restored = save::decode(&save::encode(&w), CONTENT.clone()).expect("decode");
    assert_eq!(restored.state.player().stats, w.state.player().stats);
    assert_eq!(restored.state_hash(), w.state_hash());
    // A different potential, hidden as it is, is a different state.
    let mut other = w.state.as_ref().clone();
    let player = other.player;
    let actor = other.actors.get_mut(&player).expect("player");
    let mut potential = BTreeMap::from([(Stat::Presence, 13)]);
    for stat in Stat::ALL {
        potential.entry(stat).or_insert(actor.stats.potential(stat));
    }
    let scores = Stat::ALL.map(|s| (s, actor.stats.score(s))).into();
    actor.stats = Stats::from_data(&scores, &potential).expect("valid");
    assert_ne!(other.hash(), w.state.hash());
}

#[test]
fn the_player_perceives_their_own_scores_only() {
    let w = showcase();
    let view = w.perception();
    let own = view.own_scores();
    assert_eq!(
        own.iter().collect::<Vec<_>>(),
        Stat::ALL.map(|s| (s, w.state.player().stats.score(s)))
    );
    // What perception hands out is a score per stat id: no potentials, no modifiers.
    let json = serde_json::to_value(own).expect("serializes");
    let expected = serde_json::json!({
        "strength": 12, "dexterity": 14, "toughness": 11, "intelligence": 13, "willpower": 9,
        "presence": 10,
    });
    assert_eq!(json, expected);
    // Seen creatures carry no stats.
    for seen in view.actors() {
        let json = serde_json::to_value(seen).expect("serializes");
        let keys: Vec<&str> = (json.as_object().expect("an object").keys())
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["creature", "id", "kind", "pos"]);
    }
}
