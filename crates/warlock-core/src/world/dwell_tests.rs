//! Statuses a special room gives everyone inside (spec §7 "Special rooms"; `World::dwell`): the
//! draining room's Ebbing, on the player and on creatures, in a rest, over a whole level, eased
//! after leaving, and Anchored mind.

use super::*;
use crate::action::Rest;
use crate::content::{ContentFiles, StatusId};
use crate::time::Duration;

/// A hall, 25 wide: the west part plain, the east part (x 14 on, walls included) a draining
/// room unless the level's header says otherwise. A peaceful kobold stands at `k` inside it,
/// another at `j` just outside.
const HALL: &str = "\
#########################
#.......................#
#.......................#
#...@........j....k.....#
#.......................#
#.......................#
#########################
";

/// The built-in content with [`HALL`], its east part a draining room (`room`), the level's
/// header `header`; everyone's pool of mana 20 and full.
fn world(room: bool, header: &str) -> World {
    let mut files = ContentFiles::builtin();
    let room = if room {
        "[[room]]\neffect = \"draining\"\narea = [14, 0, 24, 6]\n"
    } else {
        ""
    };
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\n{header}map = '''\n{HALL}'''\n{room}\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n\
             \"j\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_room", None).expect("world");
    let ids: Vec<ActorId> = w.state.actors.keys().copied().collect();
    for id in ids {
        let actor = w.state.actors.get_mut(&id).expect("actor");
        actor.mana_base = 20;
        let max = actor.max_mana(&w.content);
        w.state.actors.get_mut(&id).expect("actor").mana = max;
    }
    w
}

fn ebbing(w: &World) -> StatusId {
    w.content.status_id("ebbing").expect("ebbing")
}

/// The kobold standing in column `x`.
fn at(w: &World, x: i32) -> &Actor {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature && a.pos.x == x)
        .expect("a kobold")
}

fn wait(w: &mut World, turns: u32) {
    for _ in 0..turns {
        w.apply(&Action::Wait);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

#[test]
fn ebbing_is_a_status_of_mana_alone() {
    let w = world(true, "");
    assert!(w.content.status(ebbing(&w)).touches_only_mana());
    let poisoned = w.content.status_id("poisoned").expect("poisoned");
    assert!(!w.content.status(poisoned).touches_only_mana());
    let draining = w.content.room_effects().id("draining").expect("draining");
    let def = w.content.room_effect(draining);
    assert_eq!(def.status, Some(ebbing(&w)));
    assert_eq!(def.level_chance, 5);
}

#[test]
fn a_draining_room_ebbs_the_player_s_mana_while_inside_and_eases_after() {
    let mut w = world(true, "");
    let full = w.state.player().mana;
    assert!(full >= 10, "a pool to drain: {full}");
    wait(&mut w, 20);
    assert_eq!(w.state.player().mana, full, "outside, nothing drains");
    assert!(!w.state.player().effects.has(ebbing(&w)));
    w.player_mut().pos = Pos::new(16, 3);
    let start = w.state.time;
    wait(&mut w, 1);
    assert!(w.state.player().effects.has(ebbing(&w)), "inside: Ebbing");
    assert!(said(&w, "Your power ebbs away."));
    wait(&mut w, 20);
    // A point every 20 seconds, none coming back.
    let seconds = w.state.time.since(start).as_seconds();
    let lost = full - w.state.player().mana;
    let expected = i32::try_from(seconds / 20).expect("small");
    assert!(
        (expected - 1..=expected).contains(&lost),
        "lost {lost} in {seconds} s"
    );
    // Out of the room it eases within half a minute, and then mana comes back.
    w.player_mut().pos = Pos::new(4, 3);
    let left = w.state.player().mana;
    wait(&mut w, 8);
    assert!(!w.state.player().effects.has(ebbing(&w)), "eased away");
    assert!(said(&w, "Your power stops ebbing."));
    let after = w.state.player().mana;
    assert!(after <= left, "nothing came back meanwhile");
    wait(&mut w, 100);
    assert!(w.state.player().mana > after, "mana comes back outside");
}

#[test]
fn everyone_inside_ebbs_creatures_too_but_not_those_without_mana() {
    let mut w = world(true, "");
    let (inside, outside) = (at(&w, 18).id, at(&w, 13).id);
    let full = w.state.actors[&inside].mana;
    wait(&mut w, 30);
    let (k, j) = (&w.state.actors[&inside], &w.state.actors[&outside]);
    assert!(k.effects.has(ebbing(&w)), "the kobold inside ebbs");
    assert!(k.mana < full, "and its mana drains: {} of {full}", k.mana);
    assert!(!j.effects.has(ebbing(&w)), "the one outside does not");
    assert_eq!(j.mana, full);
    // One with no mana never holds a status of mana alone.
    let mut w = world(true, "");
    let id = at(&w, 18).id;
    let kobold = w.state.actors.get_mut(&id).expect("kobold");
    kobold.mana_base = 0;
    kobold.mana = 0;
    assert_eq!(w.state.actors[&id].max_mana(&w.content), 0, "no pool");
    wait(&mut w, 5);
    assert!(!w.state.actors[&id].effects.has(ebbing(&w)));
}

#[test]
fn a_rest_inside_ebbs_all_along() {
    let mut w = world(true, "");
    w.player_mut().pos = Pos::new(16, 3);
    let full = w.state.player().mana;
    w.apply(&Action::Rest(Rest::For(Duration::minutes(5))));
    assert!(w.state.player().effects.has(ebbing(&w)));
    // Five minutes: fifteen points, or all there was.
    let lost = full - w.state.player().mana;
    assert!(lost >= full.min(14), "lost {lost} of {full}");
}

#[test]
fn a_drained_level_ebbs_everywhere() {
    let mut w = world(false, "effect = \"draining\"\n");
    assert!(
        said(&w, "Something here drinks at your power."),
        "the arrival"
    );
    wait(&mut w, 3);
    assert!(w.state.player().effects.has(ebbing(&w)));
    assert!(at(&w, 13).effects.has(ebbing(&w)), "a kobold anywhere");
}

#[test]
fn anchored_mind_is_never_taken_by_ebbing() {
    let mut w = world(true, "");
    w.player_mut().talents.take("anchored_mind", None);
    w.player_mut().pos = Pos::new(16, 3);
    let full = w.state.player().mana;
    wait(&mut w, 30);
    assert!(!w.state.player().effects.has(ebbing(&w)));
    assert_eq!(w.state.player().mana, full, "nothing drains");
    // The kobold beside them ebbs as ever.
    assert!(at(&w, 18).effects.has(ebbing(&w)));
}

#[test]
fn ebbing_saves_and_hashes_alike() {
    let mut w = world(true, "");
    w.player_mut().pos = Pos::new(16, 3);
    wait(&mut w, 10);
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(back.state_hash(), w.state_hash());
    assert!(back.state.player().effects.has(ebbing(&w)));
}
