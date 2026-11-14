//! The edge rule (spec §7 "Leaving an outdoor area", #803): walking off the edge of an outdoor
//! area with an overworld cell is refused while a hostile creature is within `edge_reach` (3,
//! `data/encounters.toml`) cells of the player, seen or not; refused, it takes no time and names
//! nobody. The fixture is the forest scenario's generated local area, the player moved to its
//! west edge and a kobold placed where each case needs it.

use super::caster_tests::kobold_at;
use super::edge_leave::EDGE_LEAVE_REFUSED;
use super::quick_leave::QUICK_LEAVE_REFUSED;
use super::*;
use crate::action::parse_actions;
use crate::content::Stance;
use crate::test_support::builtin as content;
use crate::test_support::last_text as last;

/// A run in the forest's local area, the player on a free, walkable cell of its west edge.
fn at_the_edge() -> World {
    let mut w = World::new(content(), "forest", None).expect("the forest");
    let level = w.state.current();
    assert!(level.exit.is_some(), "an edge exit");
    let (content, id) = (w.content().clone(), level.id.clone());
    let free = (0..level.terrain.height())
        .map(|y| Pos::new(0, y))
        .find(|&p| {
            level
                .terrain
                .get(p)
                .is_some_and(|&t| content.terrain(t).walkable)
                && w.state.actor_at(&id, p).is_none()
                && w.state.actor_at(&id, Pos::new(p.x + 3, p.y)).is_none()
                && w.state.actor_at(&id, Pos::new(p.x + 4, p.y)).is_none()
        })
        .expect("a free cell on the west edge");
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("the player").pos = free;
    w.perceive(&mut Vec::new());
    let near = (w.state.actors.values())
        .filter(|a| a.id != player && a.level == id && a.pos.chebyshev(free) <= 4)
        .count();
    assert_eq!(near, 0, "nobody near the edge cell yet");
    w
}

fn act(world: &mut World, actions: &str) -> Vec<bool> {
    let actions = parse_actions(actions).expect("actions");
    actions.iter().map(|a| world.apply(a).took_turn).collect()
}

#[test]
fn a_hostile_creature_within_three_holds_the_player_and_one_farther_does_not() {
    let mut w = at_the_edge();
    assert_eq!(w.content().encounters().edge_reach, 3);
    let area = w.state.current_level.clone();
    let exit = w.state.current().exit.clone().expect("an edge exit");
    let kobold = kobold_at(&mut w, (3, 0), 10);
    assert_eq!(w.stance(w.state.player, kobold), Stance::Hostile);
    let (before, here) = (w.state.time, w.player_pos());
    assert_eq!(act(&mut w, "move:W"), [false]);
    assert_eq!(w.state.current_level, area);
    assert_eq!(w.player_pos(), here);
    assert_eq!(w.state.time, before, "no time");
    assert_eq!(last(&w), EDGE_LEAVE_REFUSED);
    // Four away: the way is clear, and the player walks off to the overworld cell.
    let at = w.state.actors[&kobold].pos;
    w.state.actors.get_mut(&kobold).expect("the kobold").pos = Pos::new(at.x + 1, at.y);
    assert_eq!(act(&mut w, "move:W"), [true]);
    assert_eq!(w.state.current_level, exit.to);
    assert_eq!(w.player_pos(), exit.pos);
}

#[test]
fn the_hold_counts_unseen_creatures_and_diagonals() {
    let mut w = at_the_edge();
    // Three rows up and two across: three away, whether the player sees it or not.
    let kobold = kobold_at(&mut w, (2, -3), 10);
    if w.state.actors[&kobold].pos.y < 0 {
        w.state.actors.get_mut(&kobold).expect("the kobold").pos.y += 6;
    }
    assert_eq!(
        w.state.actors[&kobold].pos.chebyshev(w.player_pos()),
        3,
        "three away"
    );
    assert_eq!(act(&mut w, "move:W"), [false]);
    assert_eq!(last(&w), EDGE_LEAVE_REFUSED);
}

#[test]
fn a_peaceful_creature_at_the_players_side_does_not_hold_them() {
    let mut w = at_the_edge();
    let kobold = kobold_at(&mut w, (1, 0), 10);
    let player = w.state.player;
    let peaceful = w
        .content()
        .factions()
        .id("peaceful")
        .expect("peaceful folk");
    // Peaceful toward the player: the rule asks only after hostile creatures.
    w.state
        .actors
        .get_mut(&kobold)
        .expect("the kobold")
        .faction_override = Some(peaceful);
    assert_ne!(w.stance(player, kobold), Stance::Hostile);
    assert_eq!(act(&mut w, "move:W"), [true]);
    assert_ne!(w.state.current().kind, crate::content::LevelKind::Outdoor);
}

#[test]
fn the_quick_leave_keeps_its_own_refusal() {
    let mut w = at_the_edge();
    kobold_at(&mut w, (3, 0), 10);
    assert_eq!(act(&mut w, "ascend"), [false]);
    assert_eq!(last(&w), QUICK_LEAVE_REFUSED);
}
