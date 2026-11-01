//! The talent that cares about the system affixes (the affixes design,
//! `docs/design/affixes-systems.md` §10; #634): Attuned counts the caster level a character's
//! magic items give half again, rounded up, in [`Actor::caster_level`]'s items term. No shipped
//! affix file has the effect yet, so the tests bring a fixture affix.

use super::caster_tests::{reach, world_with};
use super::*;
use crate::content::ContentFiles;
use crate::magic::{Affix, Magic, Rarity};
use crate::stats::{PerStat, Stats};

const AFFIXES: &str = r#"
[[suffix]]
id = "fixture_of_the_adept"
name = "of the Adept"
effect = "caster_level"
category = "caster_level"
bases = ["ring"]
tiers = [{ level = 1, min = 1, max = 3 }]
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_talent_fixture.toml", AFFIXES);
    Arc::new(Content::load(&files).expect("the fixture affixes load"))
}

/// A human of `class` at character level 6 (Attuned's), every score 14. The levels are reached
/// with every score 10, so the picks they bring (spent with the first talent the player may
/// take) are not Attuned.
fn world(class: &str) -> World {
    let mut w = world_with(content(), class);
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    reach(&mut w, 6);
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    assert_eq!(w.state.player().talents.rank_of("attuned", None), 0);
    w
}

/// The player wears an iron ring with the affix `id` at `value` on the hand `slot`.
fn ring(w: &mut World, slot: &str, id: &str, value: i32) {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = w.content().object_id("iron_ring").expect("object");
    let content = w.content().clone();
    let me = w.player_mut();
    me.inventory.push(Stack::with_magic(object, 1, magic));
    let slot = content.slot_id(slot).expect("slot");
    me.equipment.put(&[slot], me.inventory.len() - 1);
}

/// The player takes talent `id`, without a level to pick it at.
fn give(w: &mut World, id: &str) {
    w.player_mut().talents.take(id, None);
}

fn caster_level(w: &World) -> i32 {
    w.caster_level(w.state.player)
}

#[test]
fn attuned_counts_the_items_caster_level_half_again_rounded_up() {
    for (value, attuned) in [(1, 2), (2, 3), (3, 5)] {
        let mut w = world("wizard");
        let base = caster_level(&w);
        give(&mut w, "attuned");
        assert_eq!(caster_level(&w), base, "nothing worn: nothing more");
        ring(&mut w, "left_ring", "fixture_of_the_adept", value);
        assert_eq!(
            caster_level(&w),
            base + attuned,
            "+{value} counts {attuned}"
        );
    }
    // Without the talent the ring gives what it says.
    let mut w = world("wizard");
    let base = caster_level(&w);
    ring(&mut w, "left_ring", "fixture_of_the_adept", 3);
    assert_eq!(caster_level(&w), base + 3);
}

#[test]
fn attuned_lifts_the_items_sum_and_leaves_a_curse_as_it_is() {
    let mut w = world("wizard");
    let base = caster_level(&w);
    give(&mut w, "attuned");
    // Two rings: +1 and +2 are +3 together, counted +5.
    ring(&mut w, "left_ring", "fixture_of_the_adept", 1);
    ring(&mut w, "right_ring", "fixture_of_the_adept", 2);
    assert_eq!(caster_level(&w), base + 5);
    // A term below 0 (no shipped tier has one; a value set by hand) is not made worse.
    let mut w = world("wizard");
    let base = caster_level(&w);
    give(&mut w, "attuned");
    ring(&mut w, "left_ring", "fixture_of_the_adept", -2);
    assert_eq!(caster_level(&w), base - 2);
}

#[test]
fn attuned_on_one_without_mana_still_counts_and_nothing_breaks() {
    let mut w = world("warrior");
    w.player_mut().mana_base = 0;
    w.player_mut().mana = 0;
    let base = caster_level(&w);
    give(&mut w, "attuned");
    assert_eq!(caster_level(&w), base);
    ring(&mut w, "left_ring", "fixture_of_the_adept", 1);
    assert_eq!(caster_level(&w), base + 2);
}
