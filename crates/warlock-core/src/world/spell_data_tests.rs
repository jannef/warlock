//! The data-only spells (the spell types design §7, #998): Bleeding refusing the bloodless (Q7),
//! Flay's degrees on a damage spell's status, Renewal's quickened healing beside the statuses
//! that slow it, Bind and Unbinding.

use super::caster_tests::{kobold_at, sp};
use super::spell_type_tests::{cast, cell, files, put_back, start, status};
use super::*;
use crate::stats::{PerStat, Stat, Stats};

fn hall() -> World {
    let mut w = start(Arc::new(Content::load(&files()).expect("content")), 7);
    for spell in ["flay", "bind", "renewal", "unbinding", "barkskin"] {
        w.player_mut().spells.insert(sp(spell));
    }
    w
}

/// The kobold at `at` made a creature of type `creature`.
fn make(w: &mut World, target: ActorId, creature: &str) {
    let id = w.content().creature_id(creature).expect("a creature");
    w.state.actors.get_mut(&target).expect("there").creature = id;
}

fn degrees(w: &World, who: ActorId, id: &str) -> u16 {
    let s = status(w, id);
    (w.state.actors[&who].effects.get(s)).map_or(0, |e| e.severity)
}

#[test]
fn bleeding_is_refused_by_the_bloodless_and_taken_by_the_rest() {
    let mut w = hall();
    let kobold = kobold_at(&mut w, (4, 0), 1000);
    let skeleton = kobold_at(&mut w, (6, 0), 1000);
    make(&mut w, skeleton, "skeleton");
    assert!(
        w.content()
            .creature(w.content().creature_id("skeleton").unwrap())
            .blood
            .is_none()
    );
    let bleeding = status(&w, "bleeding");
    for id in [kobold, skeleton] {
        w.afflict(id, bleeding, 2, None, &mut Vec::new());
    }
    assert_eq!(degrees(&w, kobold, "bleeding"), 2);
    assert_eq!(
        degrees(&w, skeleton, "bleeding"),
        0,
        "no blood, no bleeding"
    );
    // Another status takes on the skeleton as ever: only a wound's needs blood.
    let pinned = status(&w, "pinned");
    w.afflict(skeleton, pinned, 2, None, &mut Vec::new());
    assert!(w.state.actors[&skeleton].effects.has(pinned));
}

#[test]
fn flay_bleeds_by_the_casters_presence_and_never_a_skeleton() {
    let at = cell(8, 3);
    let flayed = |presence: u8, creature: &str| {
        let mut w = hall();
        let mut stats = PerStat::splat(14);
        stats[Stat::Presence] = presence;
        w.player_mut().stats = Stats::new(stats, PerStat::splat(40));
        let target = kobold_at(&mut w, (6, 0), 1000);
        make(&mut w, target, creature);
        put_back(&mut w, target, at);
        let struck = cast(&mut w, "flay", at, Some(target));
        assert_eq!(struck.len(), 1, "an auto ray always strikes");
        degrees(&w, target, "bleeding")
    };
    assert_eq!(flayed(10, "kobold"), 2, "its base");
    assert_eq!(flayed(14, "kobold"), 3);
    assert_eq!(flayed(18, "kobold"), 4);
    assert_eq!(flayed(18, "skeleton"), 0, "bloodless");
}

#[test]
fn renewal_quickens_healing_and_a_slowing_status_still_slows_it() {
    let mut w = hall();
    let player = w.player_id();
    assert_eq!(w.healing_rate(player), 100);
    w.apply(&Action::Cast("renewal".into(), None));
    assert!(w.state.player().effects.has(status(&w, "renewing")));
    assert_eq!(w.healing_rate(player), 400, "four times the pace");
    // A buff that leaves the pace alone does not hold it back.
    w.apply(&Action::Cast("barkskin".into(), None));
    assert_eq!(w.healing_rate(player), 400);
    // Bleeding still stops it, and Renewal waits it out.
    let bleeding = status(&w, "bleeding");
    w.afflict(player, bleeding, 1, None, &mut Vec::new());
    assert_eq!(w.healing_rate(player), 0);
}

#[test]
fn bind_pins_a_foe_and_unbinding_ends_the_hexes_on_oneself() {
    let mut w = hall();
    let at = cell(8, 3);
    let target = kobold_at(&mut w, (6, 0), 1000);
    put_back(&mut w, target, at);
    cast(&mut w, "bind", at, Some(target));
    assert!(degrees(&w, target, "pinned") >= 2, "pinned for its actions");
    let player = w.player_id();
    for (id, n) in [("pinned", 2), ("frightened", 1), ("ill_wished", 1)] {
        let s = status(&w, id);
        w.afflict(player, s, n, None, &mut Vec::new());
    }
    w.apply(&Action::Cast("unbinding".into(), None));
    for id in ["pinned", "frightened", "ill_wished"] {
        assert_eq!(degrees(&w, player, id), 0, "{id} ended");
    }
}
