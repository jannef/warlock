//! The craftsperson re-enabled (#1419; the craftsperson design §4, §5, §7.1, §7.2): offered at
//! creation, its two trades as bonus picks first, Time served's +6 on each, Eye for the make's
//! lines in an item's description, and a scenario's own character's named picks.

use super::*;
use crate::character::{CharacterSpec, Gender};
use crate::content::TalentId;
use crate::dice::Dice;
use crate::object::Stack;
use crate::rng::Rng;
use crate::world::pick_text;
use std::sync::LazyLock;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(crate::background::storyless()));

fn draft(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::draft(CONTENT.clone(), "showcase", Some(4), &spec).expect("a draft")
}

/// A kitted run of the showcase as `character` with `talents` named first, the creation picks
/// left filled with the first talents offered outside the Heritage tree.
fn start(character: &str, talents: &[&str], seed: u64) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    let d = World::draft(CONTENT.clone(), "showcase", Some(seed), &spec).expect("a draft");
    let book = CONTENT.talents();
    let heritage = book.tree_id("heritage").expect("a tree");
    let mut named = picks(talents);
    loop {
        let (left, offers) = d.creation_offers(&named).expect("the picks hold");
        if left == 0 {
            break;
        }
        let (id, ways) = (offers.iter())
            .find(|(id, _)| book.get(*id).tree != heritage)
            .expect("a talent to pick");
        named.push(pick_text(&book.get(*id).id, ways[0].as_deref()));
    }
    let spec = spec.with_talents(named);
    World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("starts")
}

fn tree_of(id: TalentId) -> String {
    let book = CONTENT.talents();
    book.tree(book.get(id).tree).id.clone()
}

fn picks(list: &[&str]) -> Vec<String> {
    list.iter().map(|p| p.to_string()).collect()
}

#[test]
fn the_craftsperson_is_offered_and_picks_two_trades_first() {
    let offered = CONTENT.classes_offered("human", Gender::Male);
    assert!(offered.iter().any(|c| c.id == "craftsperson"));
    let d = draft("human,male,craftsperson");
    // Two trades, then everyone's two and Versatile's one.
    let (left, offers) = d.creation_offers(&[]).expect("offers");
    assert_eq!((left, d.creation_bonus_left(&[])), (5, 2));
    assert!(!offers.is_empty());
    assert!(offers.iter().all(|(id, _)| tree_of(*id) == "crafts"));
    let two = picks(&["smith", "wright"]);
    let (left, offers) = d.creation_offers(&two).expect("offers");
    assert_eq!((left, d.creation_bonus_left(&two)), (3, 0));
    let book = CONTENT.talents();
    let (ordinary, _) = (offers.iter())
        .find(|(id, _)| !["crafts", "heritage"].contains(&tree_of(*id).as_str()))
        .expect("an ordinary talent");
    // An ordinary talent named first spends an ordinary pick; the trades stay due.
    let first = picks(&[book.get(*ordinary).id.as_str()]);
    let (left, _) = d.creation_offers(&first).expect("offers");
    assert_eq!((left, d.creation_bonus_left(&first)), (4, 2));
    // The Random picks draw two trades first.
    let random = d.random_creation();
    assert_eq!(random.len(), 5, "{random:?}");
    let trees: Vec<String> = (random.iter())
        .map(|p| tree_of(book.id(p.split(':').next().unwrap()).expect("a talent")))
        .collect();
    assert_eq!(trees[..2], ["crafts", "crafts"], "{random:?}");
    // No other class has the bonus picks.
    assert_eq!(draft("human,male,warrior").creation_bonus_left(&[]), 0);
}

#[test]
fn a_bonus_pick_takes_only_a_trade_and_the_run_records_every_pick() {
    let world = start("dwarf,female,craftsperson", &["wright", "gemcutter"], 2);
    let talents = &world.state().player().talents;
    for t in ["wright", "gemcutter", "time_served", "eye_for_the_make"] {
        assert!(talents.has(t), "{t}");
    }
    let taken = &world
        .state()
        .character
        .as_ref()
        .expect("a character")
        .spec
        .talents;
    assert_eq!(taken.len(), 4, "two trades and everyone's two: {taken:?}");
    assert_eq!(world.due(), None);
    // A dwarf is born to Metalcraft: Smith is closed to them.
    let spec: CharacterSpec = "dwarf,female,craftsperson".parse().expect("a character");
    let smith = spec.with_talents(picks(&["smith", "gemcutter"]));
    let e = World::start_run(
        CONTENT.clone(),
        "showcase",
        Some(2),
        Some(&smith),
        None,
        true,
    )
    .expect_err("a dwarf smith");
    assert!(e.to_string().contains("dwarf"), "{e}");
}

#[test]
fn time_served_adds_six_to_each_trade() {
    let world = start("elf,female,craftsperson", &["brewster", "pitman"], 7);
    let dice: Dice = "3d6".parse().expect("dice");
    for skill in ["alchemy", "mining"] {
        let stream = format!("character/trade/elf,female,craftsperson/{skill}");
        let rolled = dice.roll(&mut Rng::stream(7, &stream)) + 6;
        let id = CONTENT.skill_id(skill).expect("a skill");
        let value = world.state().player().skills.value(id).expect("trained");
        assert_eq!(i32::from(value), rolled, "{skill}");
    }
}

#[test]
fn eye_for_the_make_reads_an_items_make() {
    let mut world = start("human,male,craftsperson", &["smith", "wright"], 1);
    let sword = CONTENT.object_id("long_sword").expect("a long sword");
    let ration = CONTENT.object_id("ration").expect("a ration");
    world.player_mut().inventory.push(Stack::one(sword));
    let texts = |world: &World, object| -> Vec<String> {
        let view = world.perception();
        let stack = (view.inventory().iter())
            .find(|s| s.object == object)
            .expect("carried");
        (view.describe_item(stack).into_iter())
            .map(|l| l.text)
            .collect()
    };
    let lines = texts(&world, sword);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Taken apart, it would give")),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"It will take three more improvements at the anvil.".to_string()),
        "{lines:?}"
    );
    let lines = texts(&world, ration);
    assert!(
        lines.contains(&"Nothing in it is worth saving.".to_string()),
        "{lines:?}"
    );
    // Without the talent, none of it.
    world.player_mut().talents = Default::default();
    let lines = texts(&world, sword);
    assert!(
        !lines.iter().any(|l| l.starts_with("Taken apart")),
        "{lines:?}"
    );
}

#[test]
fn a_scenarios_own_character_takes_its_named_picks() {
    // The forge's smith, without the kit: Smith and Wright, nothing else picked.
    let world = World::new(CONTENT.clone(), "forge", None).expect("the forge");
    let talents = &world.state().player().talents;
    assert!(talents.has("smith") && talents.has("wright"));
    assert_eq!(world.due(), None);
    let metalcraft = CONTENT.skill_id("metalcraft").expect("metalcraft");
    assert!(world.state().player().skills.value(metalcraft).is_some());
    // The quarry's dwarf: Wright and Gemcutter; Mining and Metalcraft his race's.
    let world = World::new(CONTENT.clone(), "quarry", None).expect("the quarry");
    let talents = &world.state().player().talents;
    assert!(talents.has("wright") && talents.has("gemcutter"));
    // With the kit, the Random picks fill the rest after them.
    let world = World::start_run(CONTENT.clone(), "forge", None, None, None, true).expect("kit");
    let taken = &world
        .state()
        .character
        .as_ref()
        .expect("a character")
        .spec
        .talents;
    assert_eq!(taken[..2], ["smith", "wright"]);
    assert_eq!(taken.len(), 5, "{taken:?}");
    assert_eq!(world.due(), None);
}
