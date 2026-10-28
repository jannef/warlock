//! The heritage talents (#697): only at character creation; Family wealth's gold and gems,
//! Father's sword's heirloom, each from a stream of its own.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{GOLD, Rarity, TalentEffect};
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// A kitted run of the showcase as `character` with `talents` named first, the creation picks
/// left filled with the first talents offered outside the Heritage tree.
fn start(character: &str, talents: &[&str], seed: u64) -> Result<World, WorldError> {
    let spec: CharacterSpec = character.parse().expect("a character");
    let draft = World::draft(CONTENT.clone(), "showcase", Some(seed), &spec)?;
    let book = CONTENT.talents();
    let heritage = book.tree_id("heritage").expect("heritage");
    let mut picks: Vec<String> = talents.iter().map(|t| t.to_string()).collect();
    loop {
        let (left, offers) = draft
            .creation_offers(&picks)
            .map_err(WorldError::Character)?;
        if left == 0 {
            break;
        }
        let (id, ways) = (offers.iter())
            .find(|(id, _)| book.get(*id).tree != heritage)
            .expect("a talent to pick");
        let id = &book.get(*id).id;
        picks.push(crate::world::pick_text(id, ways[0].as_deref()));
    }
    World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(seed),
        Some(&spec.with_talents(picks)),
        None,
        true,
    )
}

fn gold(world: &World) -> u32 {
    let coin = CONTENT.object_id(GOLD).expect("gold");
    (world.state().player().inventory.iter())
        .filter(|s| s.object == coin)
        .map(|s| s.count)
        .sum()
}

fn gems(world: &World) -> usize {
    let gem = CONTENT.gems().expect("gems").object;
    (world.state().player().inventory.iter())
        .filter(|s| s.object == gem)
        .count()
}

/// The heirloom's table: its weapon by race and its fallback.
fn heirloom_weapon(race: &str) -> String {
    let book = CONTENT.talents();
    let def = book.get(book.id("fathers_sword").expect("fathers_sword"));
    for effect in &def.effects {
        if let TalentEffect::Heirloom {
            weapons, fallback, ..
        } = effect
        {
            return weapons.get(race).unwrap_or(fallback).clone();
        }
    }
    panic!("no heirloom")
}

#[test]
fn the_heritage_talents_are_only_for_creation() {
    let world =
        start("human,male,warrior", &["family_wealth", "fathers_sword"], 1).expect("starts");
    let talents = &world.state().player().talents;
    assert!(talents.has("family_wealth", 1) && talents.has("fathers_sword", 1));
    // Father's sword needs Family wealth.
    let err = start("human,male,warrior", &["fathers_sword"], 1).expect_err("needs Family wealth");
    assert!(err.to_string().contains("Family wealth"), "{err}");
    // A level's pick never offers them, and refuses them in words.
    let mut world = start("human,male,warrior", &[], 1).expect("starts");
    for action in crate::action::parse_actions("xp:2431").expect("actions") {
        world.apply(&action);
    }
    let dues = &world.state().progress.due;
    assert!(
        dues.iter().any(|d| d.talents > 0),
        "a pick at level 3: {dues:?}"
    );
    let book = CONTENT.talents();
    let offers = crate::talent::offers(&CONTENT, &world.candidate());
    let heritage = book.tree_id("heritage").expect("heritage");
    assert!(offers.iter().all(|(id, _)| book.get(*id).tree != heritage));
    for action in crate::action::parse_actions("talent:family_wealth").expect("actions") {
        world.apply(&action);
    }
    let said = &world.messages().iter().last().expect("a refusal").text;
    assert_eq!(
        said,
        "You cannot take Family wealth: you need to be a new character."
    );
    assert!(!world.state().player().talents.has("family_wealth", 1));
}

#[test]
fn family_wealth_brings_gold_and_a_few_gems() {
    // A warrior's 25 gold, four times over; a monk's 5, at least 60 more; an aristocrat's
    // 305 at most 150 more.
    for (character, before, after) in [
        ("human,male,warrior", 25, 100),
        ("human,male,monk", 5, 65),
        ("human,male,aristocrat", 305, 455),
    ] {
        let plain = start(character, &[], 4).expect("starts");
        let rich = start(character, &["family_wealth"], 4).expect("starts");
        assert_eq!(gold(&plain), before, "{character}");
        assert_eq!(gold(&rich), after, "{character}");
        assert!(
            (2..=3).contains(&(gems(&rich) - gems(&plain))),
            "{character}"
        );
    }
}

#[test]
fn fathers_sword_is_the_races_weapon_magic_and_known() {
    for race in &CONTENT.races().list {
        for seed in 0..4 {
            let character = format!("{},female,thief", race.id);
            let world = start(&character, &["family_wealth", "fathers_sword"], seed);
            let world = world.unwrap_or_else(|e| panic!("{character}: {e}"));
            let player = world.state().player();
            let weapon = CONTENT
                .object_id(&heirloom_weapon(&race.id))
                .expect("a weapon");
            let (at, stack) = (player.inventory.iter().enumerate())
                .rfind(|(_, s)| s.object == weapon && s.magic().is_some())
                .unwrap_or_else(|| panic!("{character}: no heirloom"));
            let magic = stack.magic().expect("magic");
            assert!(magic.identified, "{character}: known from the start");
            assert!(magic.rarity >= Rarity::Magic);
            let affixes = magic.affixes.len();
            assert!((2..=4).contains(&affixes), "{character}: {affixes} affixes");
            assert_eq!(magic.rarity == Rarity::Rare, affixes > 2, "{character}");
            // Readied in its own slot if that was free, else carried.
            let own = CONTENT.object(weapon).slot.as_ref();
            let (slot, _) = (CONTENT.slot_ids())
                .find(|(_, s)| own == Some(&s.fits))
                .expect("its slot");
            // Readied there unless the kit had something there first.
            let held = player.equipment.get(slot);
            assert!(held.is_some(), "{character}: its slot was free");
            if held != Some(at) {
                assert!(held < Some(at), "{character}: the kit's own stays");
            }
        }
    }
}

#[test]
fn the_heritage_moves_no_other_roll() {
    let plain = start("elf,female,ranger", &[], 9).expect("starts");
    let rich = start("elf,female,ranger", &["family_wealth", "fathers_sword"], 9).expect("starts");
    let coin = CONTENT.object_id(GOLD).expect("gold");
    let kit = |w: &World| -> Vec<crate::object::Stack> {
        (w.state().player().inventory.iter())
            .filter(|s| s.object != coin)
            .cloned()
            .collect()
    };
    let (plain_kit, rich_kit) = (kit(&plain), kit(&rich));
    assert_eq!(
        plain_kit[..],
        rich_kit[..plain_kit.len()],
        "the kit is the same"
    );
    assert_eq!(
        plain.character().map(|c| (c.sign.clone(), c.born)),
        rich.character().map(|c| (c.sign.clone(), c.born))
    );
    // The same seed and character inherit the same.
    let again = start("elf,female,ranger", &["family_wealth", "fathers_sword"], 9).expect("starts");
    assert_eq!(
        again.state().player().inventory,
        rich.state().player().inventory
    );
    assert_eq!(again.state_hash(), rich.state_hash());
}
