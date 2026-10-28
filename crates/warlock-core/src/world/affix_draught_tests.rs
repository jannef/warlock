//! The potion and wand affixes' hooks (docs/design/affixes-systems.md §4 and §6, #633), each
//! read where the matching talent's is: of the Alchemist's `potion_kept` in
//! [`Actor::potion_kept`] (beside Thrifty drinker's, which `take_draught` reads), of Wands'
//! `wand_level` in [`Actor::wand_caster_level`] (beside Wand mastery's). No shipped affix has
//! these effects yet, so the tests bring fixture affixes, as `world/affix_spell_tests.rs` does.

use super::potion::{KEPT, MAX_KEPT};
use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::magic::{Affix, Magic, Rarity};

/// Fixture affixes, one per effect, with ranges wide enough for every test.
const AFFIXES: &str = r#"
[[suffix]]
id = "fixture_of_the_alchemist"
name = "of the Alchemist"
effect = "potion_kept"
category = "potions"
bases = ["ring", "amulet", "gloves"]
tiers = [{ level = 1, min = 1, max = 50 }]

[[suffix]]
id = "fixture_of_wands"
name = "of Wands"
effect = "wand_level"
category = "wand_level"
bases = ["gloves", "ring"]
tiers = [{ level = 1, min = 1, max = 5 }]
"#;

/// Thrifty drinker at a sure chance, its file replaced.
const SURE_DRINKER: &str = "[[talent]]\nid = \"thrifty_drinker\"\nname = \"Thrifty drinker\"\n\
     tree = \"delver\"\ndescription = \"x\"\n\
     effects = [{ kind = \"potion_kept\", chance = 100 }]\nsystems = [\"potions\"]\n";

/// A lit hall, the player carrying thirty potions of antidote (a cure: no dice, so the only
/// draw a drink makes is the keep's), with the fixture affixes; Thrifty drinker sure when
/// `sure`.
fn world(sure: bool) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_draught_fixture.toml", AFFIXES);
    if sure {
        files.insert("data/talents/thrifty_drinker.toml", SURE_DRINKER);
    }
    files.insert(
        "levels/draught_hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/draught_hall.toml",
        "name = \"Hall\"\nstart_level = \"draught_hall\"\nseed = 7\nhp = 1000\n\
         carry = [\"potion_of_antidote*30\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("the fixture content loads"));
    World::new(content, "draught_hall", Some(7)).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

/// The player puts on one `object` with the affix `id` at `value` in `slot`, unidentified
/// (affixes work all the same).
fn wear(w: &mut World, slot: &str, object: &str, id: &str, value: i32) {
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
    let content = w.content().clone();
    let object = content.object_id(object).expect("object");
    let player = w.player_mut();
    player.inventory.push(Stack::with_magic(object, 1, magic));
    let item = player.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, player.inventory[item].object, slot);
    player.equipment.put(&taken, item);
}

fn kept(w: &World) -> u8 {
    w.state.player().potion_kept(w.content())
}

fn potions(w: &World) -> u32 {
    let antidote = w.content().object_id("potion_of_antidote").expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == antidote)
        .map(|s| s.count)
        .sum()
}

#[test]
fn of_the_alchemist_keeps_potions_by_its_chance_until_taken_off() {
    let mut w = world(false);
    assert_eq!(kept(&w), 0);
    let rng = w.state.rng.clone();
    run(&mut w, "eat:a");
    assert_eq!(w.state.rng, rng, "no chance, no draw");
    assert_eq!(potions(&w), 29);

    wear(
        &mut w,
        "left_ring",
        "iron_ring",
        "fixture_of_the_alchemist",
        40,
    );
    assert!(!w.state.player().inventory.last().unwrap().is_identified());
    assert_eq!(kept(&w), 40, "unidentified as it is");
    run(&mut w, "eat:a*20");
    let left = potions(&w);
    assert!(left > 9 && left < 29, "some kept, some drunk: {left} left");
    assert!(
        w.messages().iter().any(|m| m.display() == KEPT),
        "the kept line"
    );

    run(&mut w, "remove:left_ring");
    assert_eq!(kept(&w), 0, "taken off, the chance is gone");
    let rng = w.state.rng.clone();
    run(&mut w, "eat:a");
    assert_eq!(w.state.rng, rng);
    assert_eq!(potions(&w), left - 1);
}

#[test]
fn of_the_alchemist_adds_to_thrifty_drinker_and_never_past_the_cap() {
    let mut w = world(false);
    w.player_mut().talents.take("thrifty_drinker", None);
    assert_eq!(kept(&w), 25, "the talent alone");
    wear(
        &mut w,
        "hands",
        "leather_gloves",
        "fixture_of_the_alchemist",
        10,
    );
    assert_eq!(kept(&w), 35);
    wear(
        &mut w,
        "neck",
        "jade_amulet",
        "fixture_of_the_alchemist",
        30,
    );
    assert_eq!(kept(&w), MAX_KEPT, "25 + 40 held to the cap");
}

#[test]
fn a_talent_above_the_cap_keeps_its_own_chance() {
    let mut w = world(true);
    w.player_mut().talents.take("thrifty_drinker", None);
    wear(
        &mut w,
        "left_ring",
        "iron_ring",
        "fixture_of_the_alchemist",
        30,
    );
    assert_eq!(kept(&w), 100);
    run(&mut w, "eat:a*5");
    assert_eq!(potions(&w), 30, "every one kept");
}

#[test]
fn of_wands_raises_a_wands_level_beside_wand_mastery() {
    let mut w = world(false);
    let content = w.content().clone();
    let fireball = content.object_id("wand_of_fireball").expect("a wand");
    let wand = content.object(fireball).wand.expect("a wand");
    let level = |w: &World| w.state.player().wand_caster_level(&content, &wand);
    assert_eq!(level(&w), 12, "the wand's own");
    wear(&mut w, "left_ring", "iron_ring", "fixture_of_wands", 2);
    assert_eq!(level(&w), 14, "unidentified as it is");
    wear(&mut w, "hands", "leather_gloves", "fixture_of_wands", 1);
    assert_eq!(level(&w), 15, "two pieces add");
    w.player_mut().talents.take("wand_mastery", None);
    assert_eq!(level(&w), 17, "and Wand mastery's 2 with them");
    run(&mut w, "remove:left_ring");
    assert_eq!(level(&w), 15, "taken off, its levels are gone");
}
