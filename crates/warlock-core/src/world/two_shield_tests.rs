//! Two-shield fighting (spec §8 "Fighting styles", #611): a shield fits either hand, a plain
//! attack with two is a bash that trains shield bashing and Two Shields, the other shield
//! follows up by the style's chance, the banes (to hit, slowness, spell failure) shrink with
//! the rank to nothing at 9, the style stays hidden until its first marks, and its talents.

use super::caster_tests::{kobold, reach, said, world_with};
use super::*;
use crate::action::{AttackKind, Event};
use crate::content::{Grip, ObjectId};
use crate::dump::{self, Layer};
use crate::equipment::{self, Equipment};
use crate::weapon_skills::WeaponSkills;

fn builtin() -> Arc<Content> {
    Arc::new(Content::builtin().expect("content"))
}

fn shield(w: &World) -> ObjectId {
    w.content()
        .object_id("small_shield")
        .expect("a small shield")
}

/// The player puts down what they hold and takes up two small shields, the first in the off
/// hand (where a shield goes first), the second in the main hand.
fn two_shields(w: &mut World) {
    let content = w.content().clone();
    let object = shield(w);
    let me = w.player_mut();
    me.equipment = Equipment::default();
    for _ in 0..2 {
        me.inventory.push(crate::object::Stack::new(object, 1));
        let item = me.inventory.len() - 1;
        let slot = equipment::free_slot(&content, &me.equipment, object).expect("a free hand");
        me.equipment
            .put(&equipment::slots_taken(&content, object, slot), item);
    }
}

/// The player's ranks are exactly `ranks` (and their defaults).
fn ranks(w: &mut World, ranks: &[(&str, u8)]) {
    let content = w.content().clone();
    let table = content.weapon_skills();
    let ranks = ranks.iter().map(|&(s, r)| (table.id(s).expect(s), r));
    w.player_mut().weapon_skills = WeaponSkills::from_ranks(table, ranks);
}

fn style(w: &World) -> crate::content::WeaponSkillId {
    w.content()
        .weapon_skills()
        .id("two_shields")
        .expect("the style")
}

/// A priest of level 3 (Blessing), two small shields in hand, no ranks.
fn priest() -> World {
    let mut w = world_with(builtin(), "priest");
    reach(&mut w, 3);
    two_shields(&mut w);
    ranks(&mut w, &[]);
    w
}

/// The player's bashes (attacks made with a shield) among `events`.
fn bashes(w: &World, events: &[Event]) -> usize {
    let object = shield(w);
    (events.iter())
        .filter(|e| {
            matches!(e, Event::Attacked { attacker, kind: AttackKind::Melee, with: Some(o), .. }
                if *attacker == w.player_id() && *o == object)
        })
        .count()
}

#[test]
fn a_shield_fits_either_hand_and_goes_to_the_off_hand_first() {
    let content = builtin();
    let object = content.object_id("small_shield").expect("a shield");
    let main = content.slot_id("main_hand").expect("main hand");
    let off = content.slot_id("off_hand").expect("off hand");
    assert!(content.fits(object, main) && content.fits(object, off));
    let empty = Equipment::default();
    assert_eq!(equipment::free_slot(&content, &empty, object), Some(off));
    let w = priest();
    let me = w.state.player();
    assert_eq!(me.grip(w.content()).map(|g| g.0), Some(Grip::TwoShields));
    assert_eq!(me.styles_in_use(w.content()), vec![style(&w)]);
}

#[test]
fn a_bump_with_two_shields_is_a_bash_and_trains_the_style_which_then_shows() {
    let mut w = priest();
    let table = w.content().weapon_skills().clone();
    let id = style(&w);
    assert!(!w.state.player().weapon_skills.shows(&table, id));
    let rows = dump::layer_rows(&w, Layer::WeaponSkills);
    assert!(
        !rows.iter().any(|r| r.starts_with("Two Shields")),
        "{rows:?}"
    );
    kobold(&mut w, 1, 10_000);
    let out = w.apply(&Action::Move(Dir::E));
    assert_eq!(bashes(&w, &out.events), 1, "{:?}", said(&w));
    let me = w.state.player();
    let bash = table
        .of_role(crate::content::Role::ShieldBash)
        .expect("shield bashing");
    assert!(
        me.weapon_skills.get(bash).marks > 0,
        "shield bashing trained"
    );
    assert!(me.weapon_skills.get(id).marks > 0, "the style trained");
    assert!(me.weapon_skills.shows(&table, id));
    let rows = dump::layer_rows(&w, Layer::WeaponSkills);
    assert!(
        rows.iter()
            .any(|r| r.starts_with("Two Shields 0") && r.ends_with(", in use")),
        "{rows:?}"
    );
}

#[test]
fn the_other_shield_follows_a_hit_by_the_styles_chance() {
    let mut most = 0;
    for (rank, follows) in [(0, false), (10, true)] {
        let mut w = priest();
        ranks(&mut w, &[("two_shields", rank), ("shield_bash", 10)]);
        kobold(&mut w, 1, 100_000);
        let mut twice = 0;
        for _ in 0..80 {
            // The ranks put back each time: the bashes train the style.
            ranks(&mut w, &[("two_shields", rank), ("shield_bash", 10)]);
            let out = w.apply(&Action::Move(Dir::E));
            let n = bashes(&w, &out.events);
            most = most.max(n);
            twice += usize::from(n == 2);
        }
        assert_eq!(twice > 0, follows, "rank {rank}: {twice} follow-ups");
    }
    assert_eq!(most, 2, "never more than both shields");
}

#[test]
fn the_banes_shrink_with_the_rank_to_nothing_at_nine() {
    let mut w = priest();
    let banes = |w: &World| {
        w.state
            .player()
            .shield_banes(w.content())
            .expect("two shields")
    };
    let raw = banes(&w);
    assert_eq!((raw.to_hit, raw.slow, raw.spell_failure), (-6, 50, 75));
    ranks(&mut w, &[("two_shields", 5)]);
    let mid = banes(&w);
    assert!(mid.to_hit > raw.to_hit && mid.slow < raw.slow && mid.spell_failure < 75);
    ranks(&mut w, &[("two_shields", 9)]);
    let top = banes(&w);
    assert_eq!((top.to_hit, top.slow, top.spell_failure), (0, 0, 0));
    // A shield and an empty hand: no banes, no style.
    let me = w.player_mut();
    let main = (me.equipment.items().into_iter()).map(|(s, _)| s).next();
    me.equipment
        .take_off(me.equipment.get(main.unwrap()).unwrap());
    assert_eq!(w.state.player().shield_banes(w.content()), None);
}

#[test]
fn the_slowness_makes_every_attack_dearer_until_the_rank_eases_it() {
    let mut w = priest();
    let price = |w: &World| {
        let me = w.state.player();
        let skills = me.attack_skills(w.content(), AttackKind::Melee, None);
        let with = me.blow_item(w.content());
        w.attack_price(w.player_id(), ActionKind::Melee, with, skills)
    };
    let slow = price(&w);
    ranks(&mut w, &[("two_shields", 9)]);
    let eased = price(&w);
    assert_eq!(slow, eased * 3 / 2, "50 % dearer untrained");
}

#[test]
fn spells_fail_by_the_banes_chance_costing_mana_and_time_until_rank_nine() {
    let count = |rank: u8| {
        let mut w = priest();
        ranks(&mut w, &[("two_shields", rank)]);
        let mut failed = 0;
        for _ in 0..200 {
            w.player_mut().mana = 40;
            let tally = |w: &World| {
                (w.messages().iter().rev().take(1))
                    .filter(|m| m.text == super::two_shields::FUMBLED)
                    .map(|m| (w.messages().len(), m.count))
                    .next()
            };
            let before = tally(&w);
            let mut events = Vec::new();
            let spent = w.cast_spell("blessing", None, &mut events);
            assert!(
                matches!(spent, Spent::Turn(_)),
                "a cast always takes its time"
            );
            // Said again and again, the line is counted on: "... (×2)".
            if tally(&w).is_some() && tally(&w) != before {
                failed += 1;
                assert!(w.state.player().mana < 40, "the mana is spent");
            }
        }
        failed
    };
    let untrained = count(0);
    assert!((110..=190).contains(&untrained), "about 75 %: {untrained}");
    assert_eq!(count(9), 0, "never from rank 9");
}

#[test]
fn a_spell_without_two_shields_draws_nothing_more() {
    let mut w = world_with(builtin(), "priest");
    reach(&mut w, 3);
    let rng = w.state.rng.clone();
    let mut events = Vec::new();
    w.cast_spell("blessing", None, &mut events);
    assert_eq!(w.state.rng, rng);
}

#[test]
fn the_styles_talents_and_the_stylists_choice_wait_for_its_first_marks() {
    let mut w = priest();
    let ours = ["anvil_and_hammer"];
    let listed = |w: &World| {
        let book = w.content().talents();
        (w.perception().talents().into_iter())
            .filter(|t| ours.contains(&book.get(t.id).id.as_str()))
            .map(|t| t.openings.len())
            .collect::<Vec<_>>()
    };
    assert!(listed(&w).is_empty(), "hidden until marks");
    let stylist = |w: &World| {
        let book = w.content().talents();
        let def = book.get(book.id("stylist").expect("Stylist"));
        let who = crate::talent::Candidate::player(
            w.content(),
            w.state.player(),
            w.state.character.as_ref(),
            w.player_level(),
        );
        crate::talent::may_take(w.content(), &who, def, Some("two_shields"))
    };
    assert!(matches!(
        stylist(&w),
        Err(crate::talent::Refusal::BadChoice(..))
    ));
    let id = style(&w);
    let content = w.content().clone();
    w.player_mut()
        .weapon_skills
        .train(content.weapon_skills(), id, 1);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    assert_eq!(listed(&w), vec![0], "listed, not yet takeable");
    assert!(!matches!(
        stylist(&w),
        Err(crate::talent::Refusal::BadChoice(..))
    ));
}

#[test]
fn a_creature_with_two_shields_bashes() {
    let mut w = priest();
    let kobold = kobold(&mut w, 1, 10_000);
    let content = w.content().clone();
    let object = shield(&w);
    let k = w.state.actors.get_mut(&kobold).expect("the kobold");
    k.equipment = Equipment::default();
    for _ in 0..2 {
        k.inventory.push(crate::object::Stack::new(object, 1));
        let item = k.inventory.len() - 1;
        let slot = equipment::free_slot(&content, &k.equipment, object).expect("a hand");
        k.equipment
            .put(&equipment::slots_taken(&content, object, slot), item);
    }
    let mut events = Vec::new();
    w.attack(kobold, w.player_id(), AttackKind::Melee, &mut events);
    let bash = (events.iter()).any(|e| {
        matches!(e, Event::Attacked { attacker, with: Some(o), .. } if *attacker == kobold && *o == object)
    });
    assert!(bash, "{events:?}");
}
