//! Caster level (spec §8 "Spells"; #394): the class's casting percent of the player's level,
//! what talents and magic items add, at least 1 with mana; a creature's at its level; and what
//! the dumps say of it.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor};
use crate::dump::{self, Layer};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human of `class` in the showcase, on `content`.
fn world_on(content: &Arc<Content>, class: &str) -> World {
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

/// The player's experience set to what takes them to `level` (nothing else of a level up: the
/// caster level is derived from the level alone).
fn at_level(w: &mut World, level: u8) {
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(w.content(), spec);
    w.state.experience = w.content().experience().threshold_for(level, factor);
    assert_eq!(w.player_level(), level);
}

#[test]
fn caster_level_is_the_class_casting_percent_of_the_level() {
    // (class, casting, level, caster level): rounded down, at least 1 with mana; the wizard
    // with Arcane training's +1 per 5 levels.
    for (class, level, expected) in [
        ("wizard", 1, 1),
        ("wizard", 30, 36),
        ("wizard", 60, 72),
        ("bard", 30, 22),
        ("paladin", 30, 15),
        ("warrior", 30, 7),
        ("warrior", 60, 15),
        ("thief", 3, 1),
    ] {
        let mut w = world_on(&BUILTIN, class);
        at_level(&mut w, level);
        w.player_mut().mana_base = 5;
        let player = w.state.player;
        assert_eq!(w.caster_level(player), expected, "{class} at level {level}");
    }
}

#[test]
fn without_mana_nobody_is_lifted_to_caster_level_1() {
    let mut w = world_on(&BUILTIN, "warrior");
    let player = w.state.player;
    w.player_mut().mana_base = -100;
    assert_eq!(w.state().player().max_mana(w.content()), 0);
    assert_eq!(
        w.caster_level(player),
        0,
        "a level-1 warrior's quarter is 0"
    );
    w.player_mut().mana_base = 3;
    assert_eq!(w.caster_level(player), 1, "with mana, at least 1");
    at_level(&mut w, 8);
    assert_eq!(w.caster_level(player), 2);
}

#[test]
fn talents_and_magic_items_add_to_caster_level() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/talents/zz_spell_test.toml",
        "[[talent]]\nid = \"zz_spell_test\"\nname = \"Test\"\ntree = \"scholar\"\n\
         description = \"x\"\neffects = [{ kind = \"caster_level\", value = 3 }]\n\
         systems = [\"mana\"]\n",
    );
    files.insert(
        "data/affixes/zz_spell_test.toml",
        "[[prefix]]\nid = \"zz_spell_test\"\nname = \"Testing\"\n\
         effect = \"caster_level\"\ncategory = \"caster_level\"\nbases = [\"ring\"]\n\
         tiers = [{ level = 1, min = 1, max = 2 }]\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = world_on(&content, "wizard");
    let player = w.state.player;
    at_level(&mut w, 10);
    assert_eq!(
        w.caster_level(player),
        12,
        "Arcane training's +1 per 5 levels"
    );
    w.player_mut().talents.take("zz_spell_test", None);
    assert_eq!(w.caster_level(player), 15, "the talent's +3");
    let magic = crate::magic::Magic {
        rarity: crate::magic::Rarity::Magic,
        level: 1,
        affixes: vec![crate::magic::Affix {
            id: "zz_spell_test".into(),
            tier: 1,
            value: 2,
        }],
        name: None,
        identified: true,
        carried: 0,
        unique: None,
    };
    let ring = content.object_id("gold_ring").expect("a ring");
    let at = w.state().player().inventory.len();
    (w.player_mut().inventory).push(Stack::with_magic(ring, 1, magic));
    let wear = format!("equip:left_ring:{at}");
    w.apply(&wear.parse().expect("action"));
    assert_eq!(w.caster_level(player), 17, "and the ring's +2");
    // A creature's is its type's level, all of it.
    let (id, level) = (w.state.actors.iter())
        .find(|(id, a)| **id != player && content.creature(a.creature).level > 0)
        .map(|(id, a)| (*id, content.creature(a.creature).level))
        .expect("a creature in the showcase");
    assert_eq!(w.caster_level(id), i32::from(level));
}

#[test]
fn the_dumps_say_the_caster_level() {
    let mut w = world_on(&BUILTIN, "wizard");
    at_level(&mut w, 12);
    let rows = dump::ascii(&w, &[Layer::Stats]);
    assert!(rows.contains("Caster level 14\n"), "{rows}");
    let json = dump::json(&w, &[], false);
    assert_eq!(json["caster_level"], 14);
}

/// The mana the player's base gains as they go from level `level - 1` to `level`.
fn mana_of_level(w: &mut World, level: u8) -> i32 {
    at_level(w, level - 1);
    let before = w.state().player().mana_base;
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(w.content(), spec);
    let need = w.content().experience().threshold_for(level, factor);
    w.grant_xp(need - w.player_xp());
    assert_eq!(w.player_level(), level);
    w.state().player().mana_base - before
}

#[test]
fn mana_grows_with_every_level_to_60() {
    // `mana_level` is 60 (tier 9's comfortable caster level), so a wizard's levels 31 and 60
    // add mana as level 2 does.
    let mut w = world_on(&BUILTIN, "wizard");
    let early = mana_of_level(&mut w, 2);
    assert!(early > 0, "a wizard's level brings mana");
    assert_eq!(mana_of_level(&mut w, 31), early);
    assert_eq!(mana_of_level(&mut w, 60), early);
}
