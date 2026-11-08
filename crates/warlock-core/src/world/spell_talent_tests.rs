//! The Spellcraft talents and the casters' reworked ones (#400; the magic design §10, the spell
//! scaling design §7.2): Overreach on the study check, School focus on its tag's cost and
//! reach, Arcane training's ten points of Decipher script, every talent combination casting with
//! nothing holding its roll (decision 16), and the six spell hooks by their shipped users:
//! Empower's and Archmage's rungs, Keen mind's dice, Sure hand's reroll, Brimming's die and
//! cost, Scroll adept's scroll rung.

use super::caster_tests::{kobold, reach, sp, world_with};
use super::*;
use crate::content::Choice;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(super::caster_tests::quiet()));

/// What the player's `spell` costs them now, and how far it reaches.
fn cost_and_range(w: &World, spell: &str) -> (i32, Option<i32>) {
    let content = w.content().clone();
    let def = content.spell(sp(spell));
    let me = w.state.player();
    (
        me.spell_cost(&content, def),
        me.spell_reach(&content, def).range,
    )
}

#[test]
fn overreach_counts_a_third_less_in_the_study_and_never_touches_mana() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    // At Decipher script's floor: learning 1, whatever this run's dice gave.
    let decipher = BUILTIN.skill_id("decipher_script").expect("skill");
    w.player_mut().skills = crate::skills::Skills::default();
    w.player_mut().skills.add(decipher, 0);
    let odds = |w: &World| {
        let character = w.state.character.as_ref();
        study_odds(w.state.player(), &BUILTIN, character, sp("fireball")).expect("odds")
    };
    let (fireball, before) = (cost_and_range(&w, "fireball").0, odds(&w));
    w.player_mut().talents.take("overreach", None);
    let (cost, after) = (cost_and_range(&w, "fireball").0, odds(&w));
    // A known spell costs its level's mana whoever casts it: no surcharge to cut.
    assert_eq!((fireball, cost), (26, 26));
    // Fireball, level 11, ten levels over: ten points harder, six with a third off (rounded
    // down); the overreach itself (the mishap's band) stays ten.
    assert_eq!((before.overreach, after.overreach), (10, 10));
    let (difficulty, cut) = (
        before.check.expect("a check").1,
        after.check.expect("a check").1,
    );
    assert_eq!(difficulty - cut, 4, "{difficulty} against {cut}");
}

#[test]
fn school_focus_cheapens_and_lengthens_only_its_own_spells() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    reach(&mut w, 3);
    let (missile, frost, mend) = (
        cost_and_range(&w, "magic_missile"),
        cost_and_range(&w, "ray_of_frost"),
        cost_and_range(&w, "mend"),
    );
    // Cold: Ray of frost (evocation, cold), not Magic missile (force).
    w.player_mut().talents.take("school_focus", Some("cold"));
    let cold = cost_and_range(&w, "ray_of_frost");
    assert_eq!(cost_and_range(&w, "magic_missile"), missile);
    assert_eq!(cold.1, frost.1.map(|r| r + 1), "a step farther");
    assert_eq!(
        (frost.0, cold.0),
        (4, 3),
        "a tenth off the floor's 4, rounded down"
    );
    // Evocation: Magic missile too, a cell past its ladder (only the engine's reach holds it).
    w.player_mut()
        .talents
        .take("school_focus", Some("evocation"));
    let (cost, range) = cost_and_range(&w, "magic_missile");
    assert!(cost < missile.0, "{cost} against {}", missile.0);
    assert_eq!(range, missile.1.map(|r| r + 1));
    assert_eq!(
        cost_and_range(&w, "ray_of_frost").1,
        frost.1.map(|r| r + 2),
        "two foci on one spell add up"
    );
    assert_eq!(cost_and_range(&w, "mend"), mend, "restoration is untouched");
}

#[test]
fn school_focus_is_chosen_among_schools_and_elements() {
    let options = Choice::School.options(&BUILTIN);
    for id in ["evocation", "restoration", "fire", "force", "necrotic"] {
        assert!(options.iter().any(|o| o == id), "{id} in {options:?}");
        assert!(Choice::School.name(&BUILTIN, id).is_some(), "{id}");
    }
    assert_eq!(Choice::School.name(&BUILTIN, "slashing"), None);
    assert_eq!(Choice::School.name(&BUILTIN, "goblin"), None);
}

#[test]
fn arcane_training_reads_decipher_script_ten_points_deeper_for_the_trained_only() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    assert!(w.state.player().talents.holds("arcane_training", None));
    let decipher = w.content.skill_id("decipher_script").expect("skill");
    let learning = |w: &World| w.state.player().learning(&w.content);
    for (raw, reach) in [(50, 10), (90, 30), (140, 70)] {
        let me = w.player_mut();
        let by = raw - i32::from(me.skills.value(decipher).unwrap_or(0));
        me.skills.add(decipher, by);
        // The curve reads 10 points more: 60 gives 10, 100 gives 30, 150 gives 70.
        assert_eq!(learning(&w), Some(reach), "Decipher script {raw} + 10");
    }
    // A bonus alone never trains the skill.
    w.player_mut().skills = crate::skills::Skills::default();
    assert_eq!(learning(&w), None);
}

/// The player casts `spell` at a kobold `dx` cells east, its hit points and their mana full
/// first; the hit points the kobold lost.
fn cast_hurt(w: &mut World, spell: &str, target: ActorId, dx: i32) -> i32 {
    let here = w.player_pos();
    w.player_mut().mana = 500;
    let kobold = w.state.actors.get_mut(&target).expect("the kobold");
    kobold.pos = Pos::new(here.x + dx, here.y);
    kobold.hp = 10_000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let at = Pos::new(here.x + dx, here.y);
    w.apply(&Action::Cast(spell.into(), Some(at)));
    10_000 - w.state.actors[&target].hp
}

#[test]
fn every_talent_combination_casts_with_nothing_holding_it() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    reach(&mut w, 60);
    w.player_mut().mana_base = 500;
    for spell in ["fireball", "magic_missile", "ray_of_frost"] {
        w.player_mut().spells.insert(sp(spell), 1);
    }
    let target = kobold(&mut w, 3, 10_000);
    let optional: [(&str, Option<&str>); 5] = [
        ("empower", None),
        ("archmage", None),
        ("scroll_adept", None),
        ("overreach", None),
        ("school_focus", Some("evocation")),
    ];
    for mask in 0..(1u32 << optional.len()) {
        for (i, (id, _)) in optional.iter().enumerate() {
            w.player_mut().talents.forget(id);
            if mask & (1 << i) != 0 {
                w.player_mut().talents.take(id, optional[i].1);
            }
        }
        for spell in ["fireball", "magic_missile", "ray_of_frost"] {
            let mut most = 0;
            for _ in 0..12 {
                let hurt = cast_hurt(&mut w, spell, target, 3);
                most = most.max(hurt);
            }
            assert!(most > 0, "{spell} with {mask:05b} was cast");
        }
    }
}

/// The player's own roll of `spell` now, as text.
fn roll(w: &World, spell: &str) -> String {
    let def = BUILTIN.spell(sp(spell));
    let roll = w.state.player().spell_roll(&BUILTIN, def, CastBy::Own);
    roll.map(|r| r.to_string()).unwrap_or_default()
}

/// A wizard of level 1 with every score 10 (no rung by stats), holding `talents`.
fn plain_wizard(talents: &[(&str, Option<&str>)]) -> World {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    super::cast_tests::scores(&mut w, &[]);
    for (id, choice) in talents {
        w.player_mut().talents.take(id, *choice);
    }
    w
}

#[test]
fn empower_and_archmage_climb_rungs_to_the_ladders_last_then_the_other_hooks_apply() {
    let base = plain_wizard(&[]);
    assert_eq!(
        roll(&base, "fireball"),
        "4d6",
        "the base at Intelligence 10"
    );
    let empowered = plain_wizard(&[("empower", None)]);
    assert_eq!(roll(&empowered, "fireball"), "6d6", "a rung up");
    assert_eq!(
        roll(&empowered, "ray_of_frost"),
        "2d8",
        "a cantrip's one rung"
    );
    let both = plain_wizard(&[("empower", None), ("archmage", None)]);
    assert_eq!(roll(&both, "fireball"), "10d6", "three rungs");
    assert_eq!(
        roll(&both, "ray_of_frost"),
        "2d8",
        "never past the last rung"
    );
    // Past the last rung, the other hooks still apply: Keen mind's bigger dice.
    let keen = plain_wizard(&[
        ("empower", None),
        ("archmage", None),
        ("keen_mind", Some("cold")),
    ]);
    assert_eq!(roll(&keen, "ray_of_frost"), "2d9");
    // And the mana: Archmage half, Empower a tenth off.
    let def = BUILTIN.spell(sp("fireball"));
    assert_eq!(base.state.player().spell_cost(&BUILTIN, def), 26);
    assert_eq!(both.state.player().spell_cost(&BUILTIN, def), 10);
}

#[test]
fn keen_mind_grows_its_schools_dice_only() {
    let w = plain_wizard(&[("keen_mind", Some("evocation"))]);
    assert_eq!(roll(&w, "fireball"), "4d7");
    assert_eq!(roll(&w, "magic_missile"), "2d5");
    assert_eq!(roll(&w, "mend"), "1d3", "restoration is untouched");
    let w = plain_wizard(&[("keen_mind", Some("fire"))]);
    assert_eq!(roll(&w, "fireball"), "4d7", "an element: fire");
    assert_eq!(
        roll(&w, "lightning_bolt"),
        "4d6",
        "electricity is untouched"
    );
}

#[test]
fn brimming_rolls_a_die_more_and_costs_a_tenth_more() {
    let base = plain_wizard(&[]);
    let w = plain_wizard(&[("brimming", None)]);
    assert_eq!(roll(&w, "fireball"), "5d6");
    assert_eq!(roll(&w, "cure_wounds"), "3d6", "healing too");
    assert_eq!(roll(&w, "scorching_ray"), "3d6");
    let def = BUILTIN.spell(sp("fireball"));
    let (before, after) = (
        base.state.player().spell_cost(&BUILTIN, def),
        w.state.player().spell_cost(&BUILTIN, def),
    );
    assert_eq!((before, after), (26, 28), "a tenth more, rounded down");
}

#[test]
fn sure_hand_sets_its_hook() {
    let w = plain_wizard(&[("sure_hand", None)]);
    let me = w.state.player();
    let frost = me.spell_mods(&BUILTIN, BUILTIN.spell(sp("ray_of_frost")));
    let fire = me.spell_mods(&BUILTIN, BUILTIN.spell(sp("fireball")));
    assert_eq!((frost.reroll, fire.reroll), (1, 1), "every spell's 1s");
}

#[test]
fn scroll_adept_reads_a_spell_scroll_a_rung_higher() {
    // The scroll of lightning casts at rung 2 (8d6, 48 at most); Scroll adept, the wizard's at
    // level 1, takes it a rung higher (10d6), whatever the reader's Intelligence.
    let mut w = world_with(BUILTIN.clone(), "wizard");
    super::cast_tests::scores(&mut w, &[]);
    let def = BUILTIN.spell(sp("lightning_bolt"));
    let at = |w: &World| {
        let me = w.state.player();
        let by = CastBy::Scroll(2);
        me.spell_roll(&BUILTIN, def, by).map(|r| r.to_string())
    };
    assert_eq!(at(&w).as_deref(), Some("10d6"));
    w.player_mut().talents.forget("scroll_adept");
    assert_eq!(at(&w).as_deref(), Some("8d6"), "the scroll's own rung");
    w.player_mut().talents.take("scroll_adept", None);
    // Read for real: past the scroll's own 48.
    let target = kobold(&mut w, 2, 10_000);
    let scroll = BUILTIN
        .object_id("scroll_of_lightning")
        .expect("the scroll");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(scroll, 200));
    let item = w.state.player().inventory.len() - 1;
    let mut most = 0;
    for _ in 0..150 {
        let here = w.player_pos();
        let kobold = w.state.actors.get_mut(&target).expect("the kobold");
        kobold.pos = Pos::new(here.x + 2, here.y);
        kobold.hp = 10_000;
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let item = item.min(w.state.player().inventory.len() - 1);
        let at = Pos::new(here.x + 2, here.y);
        w.apply(&Action::Read {
            item,
            target: Some(crate::action::ReadTarget::Cell(at)),
        });
        let hurt = 10_000 - w.state.actors[&target].hp;
        most = most.max(hurt);
    }
    assert!(most > 48, "past the scroll's own rung: {most}");
}
