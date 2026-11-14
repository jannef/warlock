//! The fey courts (spec §7 "Factions", #452): each fey joins the Seelie or the Unseelie court
//! when it is made, by its type's weights, from its own stream; the pick is saved; a level's
//! `faction` wins; a Seelie fey the player attacks turns wrathful.

use super::*;
use crate::content::{ContentFiles, Stance};
use std::collections::BTreeMap;

/// A lit glade with the player (`@`) and a row of `p` pixies, `R` redcaps, `D` dryads, and `P`
/// a pixie its level places among the peaceful folk.
fn world(map: &str, seed: u64) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/glade.toml",
        &format!(
            "name = \"Glade\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"p\" = {{ terrain = \"floor\", creature = \"pixie\" }}\n\
             \"R\" = {{ terrain = \"floor\", creature = \"redcap\" }}\n\
             \"D\" = {{ terrain = \"floor\", creature = \"dryad\" }}\n\
             \"P\" = {{ terrain = \"floor\", creature = \"pixie\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/glade.toml",
        "name = \"Glade\"\nstart_level = \"glade\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "glade", Some(seed)).expect("world")
}

const CROWD: &str = "\
@.......................................
pppppppppppppppppppppppppppppppppppppppp
RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR
DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
P.......................................
";

/// Every creature's faction id by actor, in actor order.
fn courts(w: &World) -> BTreeMap<ActorId, String> {
    (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature)
        .map(|a| {
            let f = a.faction(&w.content).expect("a faction");
            (a.id, w.content.factions().def(f).id.clone())
        })
        .collect()
}

fn count(w: &World, creature: &str, court: &str) -> usize {
    let creature = w.content.creature_id(creature).expect(creature);
    (w.state.actors.values())
        .filter(|a| a.kind == ActorKind::Creature && a.creature == creature)
        .filter(|a| a.faction_override.is_some())
        .filter(|a| w.content.factions().def(a.faction(&w.content).unwrap()).id == court)
        .count()
}

#[test]
fn each_fey_joins_a_court_by_its_types_weights() {
    let w = world(CROWD, 7);
    for (_, court) in courts(&w).iter().filter(|(_, c)| *c != "peaceful") {
        assert!(court == "seelie" || court == "unseelie", "{court}");
    }
    // Forty of each: the even pixies split, the redcaps lean Unseelie, the dryads Seelie.
    let pixies = count(&w, "pixie", "seelie");
    assert!((8..=32).contains(&pixies), "{pixies} Seelie pixies of 40");
    assert!(count(&w, "redcap", "unseelie") >= 28);
    assert!(count(&w, "dryad", "seelie") >= 28);
    // A level's own faction wins over the draw.
    assert_eq!(count(&w, "pixie", "peaceful"), 1);
}

#[test]
fn the_draw_follows_the_weights_over_many() {
    let w = world("@\n", 1);
    let content = &w.content;
    let redcap = content.creature_id("redcap").expect("redcap");
    let pixie = content.creature_id("pixie").expect("pixie");
    let unseelie = content.factions().id("unseelie").expect("unseelie");
    let share = |creature, seed| {
        let n = 4000;
        let hits = (0..n)
            .filter(|&i| super::court::join(content, seed, ActorId(i), creature) == Some(unseelie))
            .count();
        hits as f64 / f64::from(n) * 100.0
    };
    for seed in [1, 2, 3] {
        let r = share(redcap, seed);
        assert!((87.0..=93.0).contains(&r), "redcaps {r} % Unseelie");
        let p = share(pixie, seed);
        assert!((46.0..=54.0).contains(&p), "pixies {p} % Unseelie");
    }
    // A type of one faction draws nothing.
    let rat = content.creature_id("rat").expect("rat");
    assert_eq!(super::court::join(content, 1, ActorId(5), rat), None);
}

#[test]
fn the_pick_is_the_seeds_and_is_saved() {
    let (a, b) = (world(CROWD, 7), world(CROWD, 7));
    assert_eq!(courts(&a), courts(&b), "the same seed, the same courts");
    assert_eq!(a.state_hash(), b.state_hash());
    let other = world(CROWD, 8);
    assert_ne!(courts(&a), courts(&other), "another seed, other courts");
    let loaded = crate::save::decode(&crate::save::encode(&a), a.content.clone()).expect("loads");
    assert_eq!(courts(&loaded), courts(&a));
    assert_eq!(loaded.state_hash(), a.state_hash());
}

#[test]
fn look_mode_tells_the_courts_apart_and_a_struck_seelie_turns() {
    let mut w = world(CROWD, 7);
    let player = w.state.player;
    let by_court = |w: &World, court: &str| {
        (w.state.actors.values())
            .find(|a| {
                a.kind == ActorKind::Creature
                    && w.content.factions().def(a.faction(&w.content).unwrap()).id == court
            })
            .map(|a| a.id)
            .expect(court)
    };
    let (seelie, unseelie) = (by_court(&w, "seelie"), by_court(&w, "unseelie"));
    assert_eq!(w.stance(player, seelie), Stance::Neutral);
    assert_eq!(w.stance(player, unseelie), Stance::Hostile);
    assert_eq!(w.stance(seelie, unseelie), Stance::Hostile);
    let view = w.perception();
    let seen = |id| (view.actors().iter()).find(|s| s.id == id).expect("seen");
    assert_eq!(view.stance(seen(seelie)), Stance::Neutral);
    assert_eq!(view.stance(seen(unseelie)), Stance::Hostile);
    // Struck, a Seelie fey turns wrathful: hostile to the player, still at peace with its court.
    w.provoke_court(seelie);
    let wrathful = w.content.factions().id("wrathful_fey");
    assert_eq!(w.state.actors[&seelie].faction_override, wrathful);
    assert_eq!(w.stance(player, seelie), Stance::Hostile);
    let other_seelie = by_court(&w, "seelie");
    assert_eq!(w.stance(seelie, other_seelie), Stance::Neutral);
    // An Unseelie fey has nowhere to turn.
    w.provoke_court(unseelie);
    assert_eq!(
        w.state.actors[&unseelie].faction_override,
        w.content.factions().id("unseelie")
    );
}
