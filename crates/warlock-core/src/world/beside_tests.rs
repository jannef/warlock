//! What a command or an ability aimed in a direction may act on beside the player (#1327,
//! `crate::beside`): the ways to its legal targets as the player knows them, and what the input
//! does with none, one, or several.

use super::class_active_tests::{run, world};
use super::intimidate_tests::foe_at;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::beside::{Beside, Target, Way};
use crate::content::Trap;

const TRAP: &str = "arrow";

fn ways(w: &World, what: Beside) -> Vec<Dir> {
    w.perception().ways(what).expect("aimed beside the player")
}

fn way(w: &World, what: Beside) -> Way {
    w.perception().way(what).expect("aimed beside the player")
}

/// Why the player cannot use `ability` now, as the menu has it; `None` while it is not listed.
fn own(w: &World, ability: Ability) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == ability)
        .map(|a| a.unavailable)
}

#[test]
fn doors_beside_the_player_are_what_open_close_listen_wedge_and_pick_lock_act_on() {
    // The showcase's thief at the start, trained in Listen and Woodcraft: no door beside.
    let mut w = world("thief");
    let content = w.content().clone();
    let trained = ["listen", "woodcraft"].map(|id| {
        let id = content.skill_id(id).expect("a skill");
        (
            id,
            crate::skills::Trained {
                value: 20,
                marks: 0,
            },
        )
    });
    let me = w.player_mut();
    me.skills = (me.skills.iter()).chain(trained).collect();
    w.perceive(&mut Vec::new());
    for what in [Beside::Open, Beside::Close, Beside::Use(Ability::PickLock)] {
        assert_eq!(ways(&w, what), [], "{what:?}");
        assert!(matches!(way(&w, what), Way::Refuse(_)), "{what:?}");
    }
    // Listen and Wedge are not listed away from a closed door (the owner); Pick lock is greyed.
    assert_eq!(own(&w, Ability::Listen), None);
    assert_eq!(own(&w, Ability::Wedge), None);
    let no_lock = Unavailable::NoTarget(Target::Lock);
    assert_eq!(own(&w, Ability::PickLock), Some(Some(no_lock)));
    assert_eq!(no_lock.reason(w.content()), "no lock beside you");
    assert_eq!(no_lock.id(w.content()), "no_target:lock");
    // Below the hall's door: one closed door, north.
    run(&mut w, "move:N*3");
    for what in [
        Beside::Open,
        Beside::Use(Ability::PickLock),
        Beside::Use(Ability::Listen),
        Beside::Use(Ability::Wedge),
        Beside::Use(Ability::Class(crate::content::ClassActive::Knock)),
    ] {
        assert_eq!(ways(&w, what), [Dir::N], "{what:?}");
        assert_eq!(way(&w, what), Way::Act(Dir::N), "{what:?}");
    }
    assert_eq!(ways(&w, Beside::Close), []);
    assert_eq!(own(&w, Ability::Listen), Some(None));
    assert_eq!(own(&w, Ability::Wedge), Some(None));
    assert_eq!(own(&w, Ability::PickLock), Some(None));
    // A door wedged already is no door to wedge, but one to listen at.
    let door = w.player_pos().step(Dir::N);
    w.state.current_mut().wedged.insert(door, 1);
    assert_eq!(ways(&w, Beside::Use(Ability::Wedge)), []);
    assert_eq!(ways(&w, Beside::Use(Ability::Listen)), [Dir::N]);
    assert_eq!(own(&w, Ability::Wedge), None);
    w.state.current_mut().wedged.clear();
    // Opened, it is a door to close, and none to open.
    run(&mut w, "open:N");
    assert_eq!(ways(&w, Beside::Close), [Dir::N]);
    assert_eq!(ways(&w, Beside::Open), []);
    assert_eq!(own(&w, Ability::Listen), None);
}

#[test]
fn a_door_with_someone_in_it_is_none_to_close() {
    let mut w = world("thief");
    run(&mut w, "move:N*3 open:N");
    let door = w.player_pos().step(Dir::N);
    foe_at(&mut w, "kobold", door, 10);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, Beside::Close), []);
}

#[test]
fn creatures_beside_the_player_are_what_chat_and_the_blows_act_on() {
    let mut w = world("thief");
    let here = w.player_pos();
    let none = [
        Beside::Chat,
        Beside::Use(Ability::ShieldBash),
        Beside::Use(Ability::Feint),
        Beside::Use(Ability::Steal),
        Beside::Use(Ability::Tame),
    ];
    for what in none {
        assert_eq!(ways(&w, what), [], "{what:?}");
    }
    // A kobold east: someone to talk to, a foe to bash and fool, no one to rob or tame.
    foe_at(&mut w, "kobold", here.step(Dir::E), 10);
    w.perceive(&mut Vec::new());
    assert_eq!(way(&w, Beside::Chat), Way::Act(Dir::E));
    assert_eq!(ways(&w, Beside::Use(Ability::ShieldBash)), [Dir::E]);
    assert_eq!(ways(&w, Beside::Use(Ability::Feint)), [Dir::E]);
    let palm = Ability::Class(crate::content::ClassActive::QuiveringPalm);
    assert_eq!(ways(&w, Beside::Use(palm)), [Dir::E]);
    assert_eq!(ways(&w, Beside::Use(Ability::Steal)), []);
    assert_eq!(ways(&w, Beside::Use(Ability::Tame)), []);
    // A second one west: the way is asked.
    foe_at(&mut w, "kobold", here.step(Dir::W), 10);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, Beside::Chat), [Dir::E, Dir::W]);
    assert_eq!(way(&w, Beside::Chat), Way::Ask);
    assert_eq!(way(&w, Beside::Use(Ability::ShieldBash)), Way::Ask);
    // A tameable beast south: one to tame (whatever it thinks of the player).
    foe_at(&mut w, "panther", here.step(Dir::S), 10);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, Beside::Use(Ability::Tame)), [Dir::S]);
}

#[test]
fn an_ally_is_someone_to_rob_and_no_beast_to_tame() {
    let mut w = world("thief");
    run(&mut w, "ally:panther");
    let ways_to = ways(&w, Beside::Use(Ability::Steal));
    assert_eq!(ways_to.len(), 1);
    assert_eq!(ways(&w, Beside::Chat), ways_to);
    assert_eq!(ways(&w, Beside::Use(Ability::Tame)), []);
    assert_eq!(ways(&w, Beside::Use(Ability::ShieldBash)), []);
}

#[test]
fn a_lunge_reaches_a_foe_two_paces_off() {
    let mut w = world("thief");
    let content = w.content().clone();
    let id = content.weapon_ability_id("lunge").expect("lunge");
    let skill = content.weapon_ability(id).skills[0];
    let lunge = Beside::Use(Ability::Weapon(skill, id));
    let here = w.player_pos();
    assert_eq!(ways(&w, lunge), []);
    foe_at(&mut w, "kobold", here.step(Dir::N).step(Dir::N), 10);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, lunge), [Dir::N]);
    // A shield bash reaches only beside.
    assert_eq!(ways(&w, Beside::Use(Ability::ShieldBash)), []);
}

#[test]
fn known_traps_are_what_disarm_acts_on() {
    let mut w = world("thief");
    let content = w.content().clone();
    let kind = content.trap_id(TRAP).expect("a trap");
    let at = w.player_pos().step(Dir::E);
    // A hidden trap counts for nothing.
    let hidden = Trap::new(kind, content.trap(kind), false, None);
    w.state.current_mut().traps.insert(at, hidden);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, Beside::Use(Ability::Disarm)), []);
    let known = Trap::new(kind, content.trap(kind), true, None);
    w.state.current_mut().traps.insert(at, known);
    w.perceive(&mut Vec::new());
    assert_eq!(ways(&w, Beside::Use(Ability::Disarm)), [Dir::E]);
    assert_eq!(own(&w, Ability::Disarm), Some(None));
}

#[test]
fn nothing_beside_is_refused_toward_a_known_wall() {
    // At the start the wall is far; below the hall's door, the walls beside it are either side of
    // the door: the first in compass order is north-east.
    let mut w = world("thief");
    assert_eq!(w.perception().blank_way(), Dir::N);
    run(&mut w, "move:N*3");
    assert_eq!(w.perception().blank_way(), Dir::NE);
    // The rules refuse the action there in their own words, taking no time.
    let turn = w.state.turn;
    w.apply(&Action::Steal(Dir::NE));
    assert_eq!(w.state.turn, turn);
}

#[test]
fn walls_that_break_are_what_dig_acts_on() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content, "quarry", None).expect("quarry");
    run(&mut w, "move:E pickup equip:tool:a");
    let content = w.content().clone();
    let level = w.state.current();
    let here = w.player_pos();
    let truth: Vec<Dir> = (Dir::ALL.into_iter())
        .filter(|&d| {
            let p = here.step(d);
            !level.is_walkable(&content, p) && level.breaks_at(&content, p)
        })
        .collect();
    assert_eq!(ways(&w, Beside::Dig), truth);
}

#[test]
fn kick_and_the_unaimed_are_not_beside() {
    let w = world("thief");
    let content = w.content().clone();
    assert_eq!(Beside::Use(Ability::Salvage).target(&content), None);
    assert_eq!(w.perception().ways(Beside::Use(Ability::Heel)), None);
}
