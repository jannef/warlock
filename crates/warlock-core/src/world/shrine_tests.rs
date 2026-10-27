//! Shrines of a side (spec §7 "Special rooms", design §9.3): a hidden side, good or evil; one of
//! its side heals twice as fast inside, one of the other side is Unwelcome (half the healing, a
//! point off to-hit) until half a minute after leaving, a neutral one is ignored; creatures too,
//! by their own side; the first entry's line says which.

use super::*;
use crate::action::parse_actions;
use crate::alignment::{Alignment, Score};
use crate::content::{ContentFiles, Regard, Shrine, StatusId};

/// A hall, 25 wide: the west part plain, the east part (x 14 on, walls included) a shrine of
/// a side with its stone at `S`. A creature may stand at `k` inside it.
const HALL: &str = "\
#########################
#.......................#
#.......................#
#...@...............k...#
#..................S....#
#.......................#
#########################
";

/// The built-in content with [`HALL`], its shrine of `side`; at `k` a peaceful `creature` of
/// side `nature` (none when `creature` is empty).
fn world(side: &str, creature: &str, nature: &str) -> World {
    let mut files = ContentFiles::builtin();
    let k = if creature.is_empty() {
        "\"k\" = \"floor\"\n".to_string()
    } else {
        format!(
            "\"k\" = {{ terrain = \"floor\", creature = \"{creature}\", faction = \"peaceful\", \
             alignment = \"{nature}\" }}\n"
        )
    };
    files.insert(
        "levels/test_room.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n\
             [[room]]\neffect = \"shrine\"\narea = [14, 0, 24, 6]\nside = \"{side}\"\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"S\" = \"shrine_stone\"\n{k}"
        ),
    );
    files.insert(
        "scenarios/test_room.toml",
        "name = \"T\"\nstart_level = \"test_room\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_room", None).expect("world")
}

/// [`world`] with no creature, the player of alignment score `score`.
fn with_player(side: &str, score: i32) -> World {
    let mut w = world(side, "", "neutral");
    w.player_mut().alignment = Some(Score::new(score));
    w
}

fn unwelcome(w: &World) -> StatusId {
    w.content.status_id("unwelcome").expect("unwelcome")
}

fn creature(w: &World) -> &Actor {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("a creature")
}

fn run(w: &mut World, actions: &str) {
    for a in parse_actions(actions).expect("actions") {
        w.apply(&a);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// Steps the player from the hall into the shrine.
fn walk_in(w: &mut World) {
    run(w, "move:E*11");
    assert_eq!(w.state.player().pos, Pos::new(15, 3), "inside");
}

#[test]
fn the_shrine_is_data_a_side_and_a_status() {
    let w = with_player("good", 0);
    let id = w.content.room_effects().id("shrine").expect("shrine");
    let def = w.content.room_effect(id);
    let shrine = def.shrine.as_ref().expect("a shrine");
    assert_eq!(shrine.regen, 200);
    assert_eq!(shrine.status, unwelcome(&w));
    assert_eq!(def.level_chance, 0, "never a whole level");
    assert_eq!(def.feature, w.content.terrain_id("shrine_stone"));
    let status = w.content.status(unwelcome(&w));
    assert_eq!((status.regen, status.eases), (50, Some(30)));
    assert_eq!(status.combat.to_hit, -1);
    for (shrine, them, regard) in [
        (Alignment::Good, Alignment::Good, Regard::Welcome),
        (Alignment::Evil, Alignment::Evil, Regard::Welcome),
        (Alignment::Good, Alignment::Evil, Regard::Unwelcome),
        (Alignment::Evil, Alignment::Good, Regard::Unwelcome),
        (Alignment::Good, Alignment::Neutral, Regard::Ignored),
        (Alignment::Evil, Alignment::Neutral, Regard::Ignored),
    ] {
        assert_eq!(Shrine::regard(shrine, them), regard, "{shrine:?} {them:?}");
    }
    // Its stone carries the side's light.
    let stone = Pos::new(19, 4);
    let lights = &w.state.current().lights;
    assert_eq!(lights.get(&stone).copied(), def.side_light(Alignment::Good));
}

#[test]
fn one_of_the_other_side_is_unwelcome_until_half_a_minute_after_leaving() {
    // An evil player in a good shrine.
    let mut w = with_player("good", -500);
    walk_in(&mut w);
    assert!(said(&w, "Something here wants you gone."));
    assert!(!said(&w, "An old shrine. It ignores you."));
    assert!(w.state.player().effects.has(unwelcome(&w)));
    assert!(said(&w, "You feel unwelcome."));
    // Healing at half the pace inside.
    let player = w.state.player;
    let inside = w.healing_rate(player);
    run(&mut w, "move:W*3");
    assert!(w.state.player().effects.has(unwelcome(&w)), "eases slowly");
    run(&mut w, "wait*6");
    assert!(!w.state.player().effects.has(unwelcome(&w)), "eased away");
    assert!(said(&w, "You no longer feel unwelcome."));
    assert_eq!(
        inside * 2,
        w.healing_rate(player),
        "half the healing inside"
    );
    // And the same in an evil shrine for a good player.
    let mut w = with_player("evil", 500);
    walk_in(&mut w);
    assert!(w.state.player().effects.has(unwelcome(&w)));
}

#[test]
fn one_of_its_side_heals_twice_as_fast_and_a_neutral_one_is_ignored() {
    for (side, score) in [("good", 500), ("evil", -500)] {
        let mut w = with_player(side, score);
        let player = w.state.player;
        let outside = w.healing_rate(player);
        walk_in(&mut w);
        assert!(said(&w, "You feel welcome here."), "{side}");
        assert!(!w.state.player().effects.has(unwelcome(&w)));
        assert_eq!(w.healing_rate(player), outside * 2, "{side}");
    }
    let mut w = with_player("good", 0);
    let player = w.state.player;
    let outside = w.healing_rate(player);
    walk_in(&mut w);
    assert!(said(&w, "An old shrine. It ignores you."));
    assert!(!w.state.player().effects.has(unwelcome(&w)));
    assert_eq!(w.healing_rate(player), outside, "neutral: nothing");
}

#[test]
fn creatures_inside_are_treated_by_their_own_side() {
    let mut w = world("good", "kobold", "evil");
    assert!(
        creature(&w).effects.has(unwelcome(&w)),
        "an evil kobold standing in a good shrine as the run starts"
    );
    run(&mut w, "wait");
    assert!(creature(&w).effects.has(unwelcome(&w)));
    let rate = |w: &World| w.healing_rate(creature(w).id);
    let mut good = world("good", "kobold", "good");
    let mut neutral = world("good", "kobold", "neutral");
    run(&mut good, "wait");
    run(&mut neutral, "wait");
    assert!(!creature(&good).effects.has(unwelcome(&good)));
    assert!(!creature(&neutral).effects.has(unwelcome(&neutral)));
    assert_eq!(
        rate(&good),
        2 * rate(&neutral),
        "its side heals twice as fast"
    );
    assert_eq!(rate(&w) * 2, rate(&neutral), "the other side half as fast");
}

#[test]
fn a_level_files_shrine_needs_a_good_or_evil_side() {
    for side in ["neutral", ""] {
        let mut files = ContentFiles::builtin();
        let side = if side.is_empty() {
            String::new()
        } else {
            format!("side = \"{side}\"\n")
        };
        files.insert(
            "levels/test_room.toml",
            &format!(
                "name = \"Test\"\nmap = '''\n{HALL}'''\n\
                 [[room]]\neffect = \"shrine\"\narea = [14, 0, 24, 6]\n{side}\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
                 \"S\" = \"shrine_stone\"\n\"k\" = \"floor\"\n"
            ),
        );
        let e = Content::load(&files).unwrap_err();
        assert!(e.to_string().contains("good or evil"), "{e}");
    }
}

#[test]
fn unwelcome_saves_and_replays() {
    let mut w = with_player("good", -500);
    walk_in(&mut w);
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert!(restored.state.player().effects.has(unwelcome(&w)));
    assert_eq!(restored.state_hash(), w.state_hash());
}
