//! The Spellcraft talents and the casters' reworked ones (#400; the magic design §10):
//! Overreach on the overreach surcharge, School focus on its tag's cost and reach, Arcane
//! training on the wizard's caster level, and the cap that holds whatever talents a caster
//! has, spell scrolls read by Empower and Archmage included.

use super::caster_tests::{kobold, reach, sp, world_with};
use super::*;
use crate::content::Choice;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// What the player's `spell` costs them now, and how far it reaches.
fn cost_and_range(w: &World, spell: &str) -> (i32, Option<i32>) {
    let content = w.content().clone();
    let def = content.spell(sp(spell));
    let level = w.caster_level(w.state.player);
    let me = w.state.player();
    (
        me.spell_cost(&content, def, level),
        me.spell_reach(&content, def).range,
    )
}

#[test]
fn overreach_cuts_only_the_surcharge() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    let rules = BUILTIN.spell_rules().expect("the tier table");
    let level = w.caster_level(w.state.player);
    assert_eq!(rules.tier_of(level), 0, "a level-1 wizard is at tier 0");
    let (fireball, frost) = (
        cost_and_range(&w, "fireball").0,
        cost_and_range(&w, "ray_of_frost").0,
    );
    w.player_mut().talents.take("overreach", None);
    let (cut, same) = (
        cost_and_range(&w, "fireball").0,
        cost_and_range(&w, "ray_of_frost").0,
    );
    assert_eq!(
        same, frost,
        "a spell at the caster's tier costs what it did"
    );
    let def = BUILTIN.spell(sp("fireball"));
    // Three tiers over: × (1 + 3 × 0.5) before, × (1 + 3 × 0.335) with a third off.
    assert_eq!(fireball, rules.mana(def, level, 0));
    assert_eq!(cut, rules.mana_with(def, level, 0, -33));
    assert!(
        cut < fireball && cut > rules.mana(def, 60, 0),
        "{cut} against {fireball}"
    );
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
    assert_eq!(cold.0, frost.0, "a 1-mana cantrip stays at 1");
    // Evocation: Magic missile too, never past its own cap (range 10).
    w.player_mut()
        .talents
        .take("school_focus", Some("evocation"));
    let (cost, range) = cost_and_range(&w, "magic_missile");
    assert!(cost < missile.0, "{cost} against {}", missile.0);
    assert_eq!(range, missile.1.map(|r| (r + 1).min(10)));
    assert_eq!(
        cost_and_range(&w, "ray_of_frost").1,
        frost.1.map(|r| (r + 2).min(8)),
        "two foci on one spell add up, to its cap"
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
fn arcane_training_grows_the_wizards_caster_level() {
    for (class, level, expected) in [
        ("wizard", 4, 4),
        ("wizard", 5, 6),
        ("wizard", 12, 14),
        ("wizard", 30, 36),
        ("priest", 12, 12),
    ] {
        let mut w = world_with(BUILTIN.clone(), class);
        reach(&mut w, level);
        assert_eq!(
            w.caster_level(w.state.player),
            expected,
            "{class} at level {level}"
        );
    }
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
fn the_cap_holds_with_every_talent_combination() {
    let mut w = world_with(BUILTIN.clone(), "wizard");
    reach(&mut w, 60);
    w.player_mut().mana_base = 500;
    for spell in ["fireball", "magic_missile", "ray_of_frost"] {
        w.player_mut().spells.insert(sp(spell));
    }
    let target = kobold(&mut w, 3, 10_000);
    let rules = BUILTIN.spell_rules().expect("the tier table");
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
            let cap = rules.cap_of(BUILTIN.spell(sp(spell)));
            let mut most = 0;
            for _ in 0..12 {
                let hurt = cast_hurt(&mut w, spell, target, 3);
                assert!(hurt <= cap, "{spell} with {mask:05b}: {hurt} over {cap}");
                most = most.max(hurt);
            }
            assert!(most > 0, "{spell} with {mask:05b} was cast");
        }
    }
}

#[test]
fn empowered_scrolls_stop_at_the_cap_of_the_spell_they_teach() {
    // The scroll of lightning rolls 4d6 of itself (24 at most, over Lightning bolt's 19): its
    // own roll stays, but an archmage's +2 a die lifts it no farther.
    let mut w = world_with(BUILTIN.clone(), "wizard");
    reach(&mut w, 60);
    assert!(w.state.player().talent_mods(w.content()).spell_per_die > 0);
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
        assert!(hurt <= 24, "{hurt}");
        most = most.max(hurt);
    }
    assert_eq!(most, 24, "the scroll's own most");
}
