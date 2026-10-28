//! The two talents that care about the system affixes (the affixes design,
//! `docs/design/affixes-systems.md` §10; #634): Attuned counts the caster level a character's
//! magic items give half again, rounded up, in [`Actor::caster_level`]'s items term; Bellows
//! doubles the stamina its gear adds to its maximum, in [`Actor::resource_max`]'s items term. No
//! shipped affix file has these effects yet, so the tests bring fixture affixes.

use super::caster_tests::{reach, world_with};
use super::*;
use crate::content::{ContentFiles, ResourceId};
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

[[prefix]]
id = "fixture_tireless"
name = "Tireless"
effect = "resource_max"
resource = "stamina"
category = "stamina"
bases = ["ring"]
tiers = [{ level = 1, min = 1, max = 5 }]

[[suffix]]
id = "fixture_of_fury"
name = "of Fury"
effect = "resource_max"
resource = "rage"
category = "rage"
bases = ["ring"]
tiers = [{ level = 1, min = 2, max = 8 }]
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_talent_fixture.toml", AFFIXES);
    Arc::new(Content::load(&files).expect("the fixture affixes load"))
}

/// A human of `class` at character level 6 (Attuned's), every score 14. The levels are reached
/// with every score 10, so the picks they bring (spent with the first talent the player may
/// take) are neither Attuned nor Bellows.
fn world(class: &str) -> World {
    let mut w = world_with(content(), class);
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    reach(&mut w, 6);
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    for id in ["attuned", "bellows"] {
        assert_eq!(w.state.player().talents.rank_of(id, None), 0, "{id}");
    }
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

fn pool(w: &World, id: &str) -> ResourceId {
    w.content().resource_id(id).expect(id)
}

fn max(w: &World, id: &str) -> u16 {
    let level = w.player_level();
    (w.state.player()).resource_max(w.content(), pool(w, id), level)
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

#[test]
fn bellows_doubles_the_stamina_gear_gives_and_nothing_else() {
    let mut w = world("warrior");
    let before = max(&w, "stamina");
    give(&mut w, "bellows");
    assert_eq!(max(&w, "stamina"), before, "nothing worn: nothing more");
    ring(&mut w, "left_ring", "fixture_tireless", 3);
    assert_eq!(max(&w, "stamina"), before + 6);
    // Without the talent the ring gives what it says.
    let mut w = world("warrior");
    ring(&mut w, "left_ring", "fixture_tireless", 3);
    assert_eq!(max(&w, "stamina"), before + 3);
    // Another pool's gear is not doubled.
    let mut w = world("barbarian");
    let rage = max(&w, "rage");
    give(&mut w, "bellows");
    ring(&mut w, "left_ring", "fixture_of_fury", 4);
    assert_eq!(max(&w, "rage"), rage + 4);
}
