//! Smoke (`world/magic/zone.rs`, Fume) and Flare (`only_sighted`): no eye sees into smoke or through
//! it, the player's and a creature's alike, one inside sees only the cells beside it, Gale tears
//! it away; a flash stuns those with eyes and passes the eyeless by.

use crate::content::ZoneKind;
use crate::stats::{PerStat, Stats};
use crate::test_support::run;
use crate::world::magic::caster_tests::{kobold_at, sp};
use crate::world::magic::spell_type_tests::{cast, cell, files, start};
use crate::world::*;

fn hall(seed: u64) -> World {
    let mut w = start(Arc::new(Content::load(&files()).expect("content")), seed);
    for spell in ["fume", "flare", "gale"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    w
}

/// A kobold `(dx, dy)` from the player that stands and takes it.
fn dummy(w: &mut World, offset: (i32, i32)) -> ActorId {
    let id = kobold_at(w, offset, 10_000);
    let actor = w.state.actors.get_mut(&id).expect("there");
    actor.stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    id
}

fn place(w: &mut World, id: ActorId, at: Pos) {
    w.state.actors.get_mut(&id).expect("there").pos = at;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn zone_kind(w: &World, p: Pos) -> Option<ZoneKind> {
    (w.state.current().zone_at(p, w.state.time)).map(|z| z.kind())
}

#[test]
fn smoke_stops_sight_both_ways_and_light_passes() {
    let mut w = hall(1);
    let me = w.state.player;
    place(&mut w, me, cell(2, 1));
    let kobold = dummy(&mut w, (8, 0));
    let sees = |w: &mut World, id: ActorId| w.nearest_hostile_in_sight(id).is_some();
    assert!(w.perception().is_visible(cell(10, 1)));
    assert!(sees(&mut w, kobold), "a lit hall");
    let lit = |w: &mut World| {
        let eye = w.state.eyes(&w.content);
        let World {
            state,
            content,
            lights,
            ..
        } = w;
        crate::sight::light(state, content, lights, cell(10, 1), &eye, true)
    };
    let before = lit(&mut w);

    run(&mut w, "zone:smoke@6,1,0");
    assert!(!w.perception().is_visible(cell(10, 1)), "behind the smoke");
    assert!(!sees(&mut w, kobold), "nor it the player");
    assert_eq!(lit(&mut w), before, "light passes");
    // The smoke itself is seen, though not what is in it.
    assert_eq!(w.perception().zone_at(cell(6, 1)), Some(ZoneKind::Smoke));
    assert!(!w.perception().is_visible(cell(6, 1)));
}

#[test]
fn one_in_smoke_is_hidden_but_beside_the_viewer_and_one_inside_sees_only_beside_it() {
    let mut w = hall(1);
    let me = w.state.player;
    place(&mut w, me, cell(2, 1));
    let kobold = dummy(&mut w, (4, 0));
    run(&mut w, "zone:smoke@6,1,0");
    // The kobold steps into the smoke: no one sees it from afar.
    place(&mut w, kobold, cell(6, 1));
    assert!(!w.perception.sees_actors_at(cell(6, 1)));
    assert!(w.nearest_hostile_in_sight(kobold).is_none(), "nor it out");
    // Beside it, the player sees it, and it the player.
    place(&mut w, me, cell(5, 1));
    assert!(w.perception.sees_actors_at(cell(6, 1)));
    assert!(w.nearest_hostile_in_sight(kobold).is_some());
    // Inside, the player sees the cells beside them and nothing farther.
    place(&mut w, kobold, cell(12, 3));
    place(&mut w, me, cell(6, 1));
    assert!(w.perception().is_visible(cell(7, 1)));
    assert!(w.perception().is_visible(cell(5, 2)));
    assert!(!w.perception().is_visible(cell(8, 1)));
    assert!(!w.perception().is_visible(cell(3, 1)));
}

#[test]
fn fume_lays_smoke_where_it_is_aimed_and_gale_tears_it_away() {
    let mut w = hall(1);
    let me = w.state.player;
    place(&mut w, me, cell(4, 1));
    let cast_at = w.state.time;
    cast(&mut w, "fume", cell(12, 3), None);
    for p in [cell(12, 3), cell(12, 1), cell(10, 3), cell(13, 5)] {
        assert_eq!(zone_kind(&w, p), Some(ZoneKind::Smoke), "{p:?}");
    }
    assert_eq!(zone_kind(&w, cell(15, 3)), None, "radius 2");
    assert_eq!(
        w.zones()[0].until(),
        cast_at + Duration::seconds(15 * 6),
        "15 turns"
    );
    // Gale along row 3 clears the cells of its line.
    place(&mut w, me, cell(9, 3));
    cast(&mut w, "gale", cell(15, 3), None);
    assert_eq!(zone_kind(&w, cell(12, 3)), None);
    assert_eq!(zone_kind(&w, cell(12, 2)), Some(ZoneKind::Smoke));
}

#[test]
fn a_flare_stuns_those_with_eyes_and_passes_the_eyeless_by() {
    let mut w = hall(1);
    let me = w.state.player;
    place(&mut w, me, cell(2, 1));
    dummy(&mut w, (8, 0));
    let worm = dummy(&mut w, (9, 0));
    let content = w.content().clone();
    (w.state.actors.get_mut(&worm).expect("there")).creature =
        content.creature_id("grave_worm").expect("worms");
    cast(&mut w, "flare", cell(10, 1), None);
    // Stunned ends at its next action: the words say who it fell on.
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.contains(&"The kobold is stunned.".to_string()),
        "{said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("worm")),
        "it has no eyes: {said:?}"
    );
}

#[test]
fn smoke_is_saved_and_a_run_with_it_replays_the_same() {
    let mut w = hall(3);
    run(&mut w, "zone:smoke@8,3,1 wait");
    let saved = serde_json::to_string(&w.state).expect("saves");
    assert!(saved.contains("\"smoke\""));
    let state: WorldState = serde_json::from_str(&saved).expect("loads");
    let restored = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(restored.state_hash(), w.state_hash());
    assert_eq!(zone_kind(&restored, cell(8, 3)), Some(ZoneKind::Smoke));
}
