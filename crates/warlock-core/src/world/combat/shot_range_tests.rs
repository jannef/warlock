//! Shots by distance (spec §8 "Ranged combat"; #956; `World::flight_profile`, `data/ranged.toml`):
//! the to-hit a shot rolls with at each band and beside the shooter, for the player and a creature
//! alike, each creature on the line by its own distance, throws left as they are; and Parry against
//! a shot from beside its target. Exact, not statistical: with the same draws, a shot at a band
//! with a to-hit strikes as one at a clean distance with that to-hit plus the penalty.

use super::shoot::Flight;
use super::weapon_ability_tests::{at, give, ranks, world};
use crate::action::AttackKind;
use crate::combat::{Category, Profile, Strike};
use crate::rng::Rng;
use crate::world::*;

/// How many shots a comparison makes, each from a seed of its own.
const SHOTS: u64 = 300;
/// A distance within 50 % of the short range all these use.
const CLEAN: i32 = 3;
const RANGE: i32 = 10;

/// A corridor with the player at x = 1 and `other` (a glyph of the yard) `cells` east of them.
fn corridor(other: char, cells: i32) -> World {
    let gap = ".".repeat(cells as usize - 1);
    world(&format!(
        "##################\n#@{gap}{other}....\n##################\n"
    ))
}

/// A flight of `kind`, one arrow attacking with `to_hit`, reaching [`RANGE`] cells.
fn flight(w: &World, kind: AttackKind, to_hit: i32) -> Flight {
    let arrow = w.content().object_id("arrow").expect("arrows");
    let trap = w.content().trap_id("arrow").expect("the arrow trap");
    let crate::content::TrapEffect::Shot { attack, .. } = &w.content().trap(trap).effect else {
        panic!("the arrow trap shoots");
    };
    let mut profile = Profile::bare(attack);
    profile.to_hit = to_hit;
    Flight {
        kind,
        with: arrow,
        flies: arrow,
        profile,
        range: RANGE,
        lands: None,
        breaks: 0,
    }
}

/// The first strike of each of [`SHOTS`] shots of `kind` by `shooter` at cell `target`, with
/// `to_hit`, each from a fresh copy of `w` and a seed of its own.
fn strikes(
    w: &World,
    shooter: ActorId,
    target: Pos,
    kind: AttackKind,
    to_hit: i32,
) -> Vec<Option<Strike>> {
    (0..SHOTS)
        .map(|seed| {
            let mut w = w.clone();
            w.state.rng = Rng::stream(seed, "shot-range-test");
            let mut events = Vec::new();
            let shot = flight(&w, kind, to_hit);
            w.fly(shooter, target, shot, &mut events);
            events
                .iter()
                .find_map(|e| match e {
                    Event::Attacked { strike, .. } => Some(*strike),
                    _ => None,
                })
                .flatten()
        })
        .collect()
}

/// The strikes of an archer `cells` east of the player, shooting at them.
fn creature_shots(cells: i32, kind: AttackKind, to_hit: i32) -> Vec<Option<Strike>> {
    let w = corridor('a', cells);
    let archer = at(&w, 1 + cells, 1);
    let player = w.player_pos();
    strikes(&w, archer, player, kind, to_hit)
}

/// The strikes of the player's shots at a post `cells` east of them.
fn player_shots(cells: i32, to_hit: i32) -> Vec<Option<Strike>> {
    let w = corridor('p', cells);
    let player = w.player_id();
    strikes(
        &w,
        player,
        Pos::new(1 + cells, 1),
        AttackKind::Ranged,
        to_hit,
    )
}

/// The penalties of the game's data: the adjacent one, then the two bands'.
fn penalties() -> (i32, i32, i32) {
    let w = corridor('p', 1);
    let rules = w.content().ranged();
    let [near, far] = rules.bands[..] else {
        panic!("two bands");
    };
    (rules.adjacent, near.to_hit, far.to_hit)
}

#[test]
fn the_data_has_a_penalty_at_every_band() {
    let (adjacent, near, far) = penalties();
    assert!(adjacent < 0, "beside the shooter");
    assert!(near < 0, "beyond 50 %");
    assert!(far < near, "beyond 75 %, harsher");
}

#[test]
fn each_distance_rolls_its_penalty_for_the_player_and_a_creature_alike() {
    let (adjacent, near, far) = penalties();
    // Straight along a row, a range of 10 is beyond 50 % from 6 cells and beyond 75 % from 8.
    let cases = [
        (1, adjacent),
        (2, 0),
        (5, 0),
        (6, near),
        (7, near),
        (8, far),
        (10, far),
    ];
    // Unaided against a bare Defence, where each point tells.
    const TO_HIT: i32 = 0;
    for (cells, penalty) in cases {
        assert_eq!(
            creature_shots(cells, AttackKind::Ranged, TO_HIT),
            creature_shots(CLEAN, AttackKind::Ranged, TO_HIT + penalty),
            "a creature's shot at {cells} cells"
        );
        assert_eq!(
            player_shots(cells, TO_HIT),
            player_shots(CLEAN, TO_HIT + penalty),
            "the player's shot at {cells} cells"
        );
    }
    // And the comparison can tell the penalties apart.
    assert_ne!(player_shots(CLEAN, 0), player_shots(CLEAN, near));
    assert_ne!(player_shots(CLEAN, near), player_shots(CLEAN, far));
}

#[test]
fn adjacent_is_the_only_penalty_of_a_short_range_weapon() {
    // A range of 1: the neighbour lies beyond both bands, and takes the adjacent penalty alone.
    let (adjacent, ..) = penalties();
    let w = corridor('p', 1);
    let from = w.player_pos();
    let mut short = flight(&w, AttackKind::Ranged, 0);
    short.range = 1;
    let profile = w.flight_profile(&short, from, from.step(Dir::E));
    assert_eq!(profile.to_hit, adjacent);
}

#[test]
fn each_creature_on_the_line_is_judged_by_its_own_distance() {
    let (adjacent, _, far) = penalties();
    let w = world("##################\n#@p......p.......\n##################\n");
    let from = w.player_pos();
    let shot = flight(&w, AttackKind::Ranged, 0);
    let bystander = w.flight_profile(&shot, from, Pos::new(2, 1));
    let aimed = w.flight_profile(&shot, from, Pos::new(9, 1));
    assert_eq!((bystander.to_hit, aimed.to_hit), (adjacent, far));
    // The arrow aimed at the far one strikes the bystander first, as at that distance.
    let player = w.player_id();
    let on_the_line = strikes(&w, player, Pos::new(9, 1), AttackKind::Ranged, 0);
    assert_eq!(on_the_line, player_shots(CLEAN, adjacent));
}

#[test]
fn a_throw_takes_no_penalty_at_any_distance() {
    let thrown = creature_shots(CLEAN, AttackKind::Thrown, 0);
    for cells in [1, 6, 8] {
        assert_eq!(
            creature_shots(cells, AttackKind::Thrown, 0),
            thrown,
            "{cells} cells"
        );
    }
}

/// How many of an archer's shots from `cells` east are parried by the player with the test foil,
/// with `kind`, Duelist at `duelist`.
fn parried(cells: i32, kind: AttackKind, duelist: u8) -> usize {
    let mut w = corridor('a', cells);
    let player = w.player_id();
    give(&mut w, player, "wa_foil");
    ranks(&mut w, player, &[("long_blades", 1), ("duelist", duelist)]);
    let archer = at(&w, 1 + cells, 1);
    let at = w.player_pos();
    (strikes(&w, archer, at, kind, 0).iter())
        .filter(|s| {
            **s == Some(Strike::Missed {
                by: Some(Category::Parry),
            })
        })
        .count()
}

#[test]
fn parry_counts_against_a_shot_from_beside_and_with_measure_from_afar() {
    assert!(parried(1, AttackKind::Ranged, 9) > 0, "beside the shooter");
    assert_eq!(parried(2, AttackKind::Ranged, 9), 0, "two cells off");
    assert_eq!(parried(CLEAN, AttackKind::Ranged, 9), 0, "from afar");
    assert!(parried(1, AttackKind::Ranged, 10) > 0, "Measure, beside");
    assert!(parried(CLEAN, AttackKind::Ranged, 10) > 0, "Measure, afar");
}

#[test]
fn a_throw_from_beside_is_parried_only_with_measure() {
    assert_eq!(parried(1, AttackKind::Thrown, 9), 0);
    assert!(parried(1, AttackKind::Thrown, 10) > 0);
}
