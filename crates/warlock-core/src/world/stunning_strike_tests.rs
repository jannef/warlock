//! The ascetic's Stunning strike and the Stunned status (spec §8 "Class abilities", "Surprise
//! and sneak attacks"; the ki tricks design §6, #848): the trick comes at level 12, is refused
//! (no time, no ki) before the spend, spends a point of ki on a bare-handed blow, and a wounding
//! hit leaves the foe Stunned: flat-footed by the status's `flat_footed` field (as Surprised
//! is), no free blows, sneak dice land, and its next action is slowed and ends it.

use super::class_active_tests::{at_normal_speed, beside, last, reach, run, suffers, world};
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::{Category, Defence, Target};
use crate::content::{ClassActive, StatusId};
use crate::equipment::{free_slot, slots_taken};
use crate::object::Stack;
use crate::replay::Replay;

const STRIKE: Ability = Ability::Class(ClassActive::StunningStrike);

fn ki(w: &World) -> Option<u16> {
    let ki = w.content().resource_id("ki").expect("ki");
    (w.state.player().resources.get(ki)).map(|p| p.value)
}

fn set_ki(w: &mut World, value: u16) {
    let ki = w.content().resource_id("ki").expect("ki");
    w.player_mut().resources.set(ki, value);
}

fn own(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == STRIKE)
        .map(|a| a.unavailable)
}

fn stunned(w: &World) -> StatusId {
    w.content().status_id("stunned").expect("stunned")
}

/// An ascetic with nothing in hand at `level`.
fn ascetic(level: u8) -> World {
    let mut w = world("ascetic");
    w.player_mut().equipment = Default::default();
    reach(&mut w, level);
    at_normal_speed(&mut w);
    w
}

/// Strikes east, ki topped up each time, until the kobold there is Stunned; the messages of
/// the strike that stunned it.
fn strike_until_stunned(w: &mut World, kobold: ActorId) -> Vec<String> {
    for _ in 0..40 {
        set_ki(w, 5);
        let before = w.messages().len();
        run(w, "use:stunning_strike:E");
        if suffers(w, kobold, "stunned") {
            return (w.messages().iter().skip(before))
                .map(|m| m.display())
                .collect();
        }
    }
    panic!("forty strikes and never a wounding hit");
}

#[test]
fn stunning_strike_comes_at_level_twelve_in_place_of_deflect_missiles() {
    let w = ascetic(11);
    assert_eq!(own(&w), None, "not before 12");
    let w = ascetic(12);
    // Had, with no foe beside to strike (#1327).
    let none = Unavailable::NoTarget(crate::beside::Target::Foe);
    assert_eq!(own(&w), Some(Some(none)));
    let talents = &w.state.player().talents;
    assert!(talents.has("stunning_strike"));
    // Deflect missiles left the table (a Finesse pick now); a bonus pick may still take it.
    assert!(w.content().classes().iter().all(|c| {
        c.id != "ascetic"
            || c.abilities
                .iter()
                .all(|(_, talent)| talent != "deflect_missiles")
    }));
}

#[test]
fn a_strike_is_refused_before_the_ki_is_spent() {
    let mut w = ascetic(12);
    let turn = w.state.turn;
    // Nobody there.
    set_ki(&mut w, 3);
    run(&mut w, "use:stunning_strike:E");
    assert_eq!(last(&w), "There is nobody there to strike.");
    assert_eq!((w.state.turn, ki(&w)), (turn, Some(3)));
    // No ki: greyed in the menu, and refused.
    let kobold = beside(&mut w, "kobold", 500);
    set_ki(&mut w, 0);
    let ki_id = w.content().resource_id("ki").expect("ki");
    assert_eq!(own(&w), Some(Some(Unavailable::Short(ki_id))));
    run(&mut w, "use:stunning_strike:E");
    assert_eq!(last(&w), "Your ki is spent.");
    assert_eq!((w.state.turn, ki(&w)), (turn, Some(0)));
    // A weapon in hand.
    set_ki(&mut w, 3);
    let content = w.content().clone();
    let dagger = content.object_id("dagger").expect("a dagger");
    let me = w.player_mut();
    me.inventory.push(Stack::new(dagger, 1));
    let item = me.inventory.len() - 1;
    let slot = free_slot(&content, &me.equipment, dagger).expect("a free slot");
    me.equipment.put(&slots_taken(&content, dagger, slot), item);
    run(&mut w, "use:stunning_strike:E");
    assert_eq!(last(&w), "Your hands are not free.");
    assert_eq!((w.state.turn, ki(&w)), (turn, Some(3)));
    assert!(!suffers(&w, kobold, "stunned"));
}

#[test]
fn a_wounding_strike_stuns_for_a_point_of_ki() {
    let mut w = ascetic(12);
    let kobold = beside(&mut w, "kobold", 500);
    let turn = w.state.turn;
    let said = strike_until_stunned(&mut w, kobold);
    assert!(w.state.turn > turn, "a blow's time");
    assert!(
        ki(&w).is_some_and(|k| k <= 4),
        "a point spent: {:?}",
        ki(&w)
    );
    assert!(said.iter().any(|s| s == "You stun the kobold."), "{said:?}");
    let hp = w.state.actors[&kobold].hp;
    assert!(hp < 500, "it wounded");
    let effect = w.state.actors[&kobold].effects.get(stunned(&w)).copied();
    assert_eq!(effect.map(|e| e.severity), Some(1));
}

#[test]
fn stunned_is_flat_footed_by_the_field_as_surprised_is() {
    let mut w = world("thief");
    let content = w.content().clone();
    for id in ["stunned", "surprised"] {
        let def = content.status(content.status_id(id).expect(id));
        assert!(def.flat_footed, "{id}");
        assert!(def.per_action && def.cost == 150, "{id}");
    }
    let player = w.state.player;
    let kobold = beside(&mut w, "kobold", 500);
    w.state
        .actors
        .get_mut(&kobold)
        .expect("kobold")
        .free_blows
        .riposte = true;
    let struck = Target::bare(Defence::from_bonuses([
        (Category::Dexterity, 3),
        (Category::Dodge, 4),
        (Category::Armour, 2),
    ]));
    let total = |t: Target| t.defence.total();
    assert!(!w.off_guard(kobold));
    // 10, and Dexterity 3, Dodge 4, Armour 2.
    assert_eq!(total(w.flat_footed(kobold, false, struck.clone())), 19);
    assert!(w.free_blows_of(&w.state.actors[&kobold]).riposte);
    assert!(w.sneak_dice(player).is_some(), "a thief has sneak dice");
    assert!(!w.sneaks(player, kobold, false, false));
    // Stunned: Dexterity and Dodge count nothing, no free blows, and the sneak dice land.
    w.afflict(kobold, stunned(&w), 1, Some(player), &mut Vec::new());
    assert!(w.off_guard(kobold));
    assert_eq!(
        total(w.flat_footed(kobold, false, struck)),
        12,
        "the armour only"
    );
    assert!(!w.free_blows_of(&w.state.actors[&kobold]).riposte);
    assert!(w.sneaks(player, kobold, false, false));
    assert_eq!(w.status_cost(kobold, stunned(&w)), 150);
}

#[test]
fn the_stunned_foes_next_action_ends_it() {
    let mut w = ascetic(12);
    let kobold = beside(&mut w, "kobold", 500);
    strike_until_stunned(&mut w, kobold);
    for _ in 0..6 {
        if !suffers(&w, kobold, "stunned") {
            return;
        }
        run(&mut w, "wait");
    }
    panic!("still stunned after six waits");
}

#[test]
fn a_stunning_strike_replays_the_same() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,ascetic".parse().expect("a character");
    let mut w = World::start_run(content, "arena", None, Some(&spec), None, true).expect("world");
    let text = "xp:100000 levelup:auto use:stunning_strike:E move:E*3 use:stunning_strike:E wait";
    let mut replay = Replay::new(&w);
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
