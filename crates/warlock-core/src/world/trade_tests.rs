//! The Crafts trades (the craftsperson design §3, §4): a craftsperson's creation picks alone,
//! never one the race teaches everyone; each trains its skill, hands down its tools, and makes
//! the skill's level-up roll a step up.

use super::*;
use crate::character::CharacterSpec;
use crate::object::Stack;
use std::sync::LazyLock;

// Without background stories: what the trades alone hand down.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(crate::background::storyless()));

const TRADES: [&str; 7] = [
    "smith",
    "wright",
    "gemcutter",
    "brewster",
    "pitman",
    "wortcunner",
    "spellwright",
];

/// A kitted run of the showcase as `character` with `talents` named first, the creation picks
/// left filled with the first talents offered outside the Heritage and Crafts trees.
fn start(character: &str, talents: &[&str], seed: u64) -> Result<World, WorldError> {
    let spec: CharacterSpec = character.parse().expect("a character");
    let draft = World::draft(CONTENT.clone(), "showcase", Some(seed), &spec)?;
    let book = CONTENT.talents();
    let skip = ["heritage", "crafts"].map(|t| book.tree_id(t).expect("a tree"));
    let mut picks: Vec<String> = talents.iter().map(|t| t.to_string()).collect();
    loop {
        let (left, offers) = draft
            .creation_offers(&picks)
            .map_err(WorldError::Character)?;
        if left == 0 {
            break;
        }
        let (id, ways) = (offers.iter())
            .find(|(id, _)| !skip.contains(&book.get(*id).tree))
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

fn skill(world: &World, id: &str) -> Option<u16> {
    let id = CONTENT.skill_id(id).expect("a skill");
    world.state().player().skills.value(id)
}

/// How many of `object` the player carries.
fn count(world: &World, object: &str) -> u32 {
    let id = CONTENT.object_id(object).expect("an object");
    (world.state().player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// The trade's skill and the tools it hands down, by object and count.
fn trade_of(trade: &str) -> (&'static str, &'static [(&'static str, u32)]) {
    match trade {
        "smith" => ("metalcraft", &[("piece", 3)]),
        "wright" => ("woodcraft", &[("piece", 5)]),
        "gemcutter" => ("jewelcraft", &[("gem", 1)]),
        "brewster" => ("alchemy", &[("moonwort", 2), ("glowcap", 2)]),
        "pitman" => ("mining", &[("pick_axe", 1)]),
        "wortcunner" => ("herbalism", &[("comfrey", 2), ("vervain", 1)]),
        "spellwright" => ("enchanting", &[("scroll_of_identify", 1)]),
        _ => unreachable!(),
    }
}

#[test]
fn every_trade_trains_its_skill_and_hands_down_its_tools() {
    // A halfling would be refused Wortcunner: a human may take every trade.
    let plain = start("human,female,craftsperson", &[], 3).expect("starts");
    for trade in TRADES {
        let world = start("human,female,craftsperson", &[trade], 3).expect("starts");
        assert!(world.state().player().talents.has(trade), "{trade}");
        let (id, tools) = trade_of(trade);
        let before = skill(&plain, id).unwrap_or(0);
        let after = skill(&world, id).expect("trained");
        assert!(
            (3..=18).contains(&(after - before)),
            "{trade}: {before} -> {after}"
        );
        for &(object, n) in tools {
            assert_eq!(
                count(&world, object),
                count(&plain, object) + n,
                "{trade}: {object}"
            );
        }
    }
    // The pick-axe is readied in the Tool slot when the kit left it free (a half-orc's: no
    // lantern), else carried (a human's lantern stays).
    let axe = CONTENT.object_id("pick_axe").expect("a pick-axe");
    let tool = CONTENT.slot_id("tool").expect("the Tool slot");
    for (character, readied) in [
        ("half_orc,female,craftsperson", true),
        ("human,female,craftsperson", false),
    ] {
        let world = start(character, &["pitman"], 3).expect("starts");
        let player = world.state().player();
        let at = (player.inventory.iter()).position(|s| s.object == axe);
        assert!(at.is_some(), "{character}");
        assert_eq!(player.equipment.get(tool) == at, readied, "{character}");
    }
    // The gem is uncut; the scroll is known by its name.
    let world = start("human,female,craftsperson", &["gemcutter"], 3).expect("starts");
    let gems = CONTENT.gems().expect("gems");
    let gem: &Stack = (world.state().player().inventory.iter())
        .find(|s| s.object == gems.object)
        .expect("a gem");
    let (_, cutting) = gem.gem(&CONTENT).expect("a loose gem");
    assert!(gems.cut(cutting.cut).rough);
    let world = start("human,female,craftsperson", &["spellwright"], 3).expect("starts");
    let scroll = CONTENT.object_id("scroll_of_identify").expect("a scroll");
    assert!(world.state().known_scrolls.contains(&scroll));
}

#[test]
fn a_race_is_never_offered_a_trade_its_people_all_have() {
    let refused = |character: &str, trade: &str| {
        let e = start(character, &[trade], 1).expect_err(trade);
        e.to_string()
    };
    let name = |trade: &str| {
        let book = CONTENT.talents();
        book.get(book.id(trade).expect("a trade")).name.clone()
    };
    for (character, closed, words) in [
        (
            "dwarf,male,craftsperson",
            &["smith", "pitman"][..],
            "not to be a dwarf or a duergar",
        ),
        (
            "gnome,male,craftsperson",
            &["gemcutter"],
            "not to be a gnome",
        ),
        (
            "kobold,female,craftsperson",
            &["pitman"],
            "not to be a dwarf, a kobold, or a duergar",
        ),
        (
            "halfling,female,craftsperson",
            &["wortcunner"],
            "not to be a halfling",
        ),
    ] {
        let spec: CharacterSpec = character.parse().expect("a character");
        let draft = World::draft(CONTENT.clone(), "showcase", Some(1), &spec).expect("a draft");
        let (_, offers) = draft.creation_offers(&[]).expect("offers");
        let book = CONTENT.talents();
        let offered = |id: &str| offers.iter().any(|(t, _)| book.get(*t).id == id);
        for trade in TRADES {
            assert_eq!(
                offered(trade),
                !closed.contains(&trade),
                "{character}: {trade}"
            );
        }
        for trade in closed {
            let e = refused(character, trade);
            if trade == &"smith" || closed.len() == 1 {
                let want = format!("You cannot take {}: you need {words}.", name(trade));
                assert!(e.contains(&want), "{character}: {e}");
            }
        }
    }
    // A dwarf craftsperson takes the other five.
    for trade in [
        "wright",
        "gemcutter",
        "brewster",
        "wortcunner",
        "spellwright",
    ] {
        start("dwarf,male,craftsperson", &[trade], 1).expect(trade);
    }
}

#[test]
fn a_trade_is_a_craftspersons_creation_pick_alone() {
    // Never another class's.
    let e = start("human,male,warrior", &["smith"], 1).expect_err("a warrior");
    assert!(e.to_string().contains("to be a craftsperson"), "{e}");
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let draft = World::draft(CONTENT.clone(), "showcase", Some(1), &spec).expect("a draft");
    let (_, offers) = draft.creation_offers(&[]).expect("offers");
    let crafts = CONTENT.talents().tree_id("crafts").expect("crafts");
    assert!(
        offers
            .iter()
            .all(|(t, _)| CONTENT.talents().get(*t).tree != crafts)
    );
    // Never a level's pick.
    let mut world = start("human,female,craftsperson", &[], 1).expect("starts");
    for action in crate::action::parse_actions("xp:2431 talent:brewster").expect("actions") {
        world.apply(&action);
    }
    let said = &world.messages().iter().last().expect("a refusal").text;
    assert_eq!(
        said,
        "You cannot take Brewster: you need to be a new character."
    );
    assert!(!world.state().player().talents.has("brewster"));
    // A third trade is an ordinary creation pick (a human has three).
    let world = start(
        "human,female,craftsperson",
        &["brewster", "wright", "spellwright"],
        1,
    )
    .expect("three trades");
    let talents = &world.state().player().talents;
    assert!(
        ["brewster", "wright", "spellwright"]
            .iter()
            .all(|t| talents.has(t))
    );
}

#[test]
fn a_trade_rolls_a_step_up_at_a_level() {
    let world = start("human,female,craftsperson", &["brewster", "smith"], 5).expect("starts");
    let view = world.perception();
    let alchemy = CONTENT.skill_id("alchemy").expect("alchemy");
    let roll = view.skill_roll(alchemy).expect("trained");
    assert!(roll.known && roll.trade, "{roll:?}");
    assert_eq!(
        roll.die,
        CONTENT.ladder().die(CONTENT.ladder().step(roll.marks) + 1)
    );
    // The class is known for Metalcraft already: one step, the class's.
    let metalcraft = CONTENT.skill_id("metalcraft").expect("metalcraft");
    let roll = view.skill_roll(metalcraft).expect("trained");
    assert!(roll.known && !roll.trade, "{roll:?}");
    assert_eq!(
        roll.die,
        CONTENT.ladder().die(CONTENT.ladder().step(roll.marks) + 1)
    );
    // Without the trade, no step.
    let plain = start("human,female,craftsperson", &["wortcunner"], 5).expect("starts");
    let herbalism = CONTENT.skill_id("herbalism").expect("herbalism");
    assert!(
        plain
            .perception()
            .skill_roll(herbalism)
            .expect("trained")
            .trade
    );
    assert!(
        plain
            .perception()
            .skill_roll(alchemy)
            .is_none_or(|r| !r.known)
    );
}

#[test]
fn a_trade_moves_no_other_roll() {
    // Two trades of one pick each: everything but their skills and tools is the same.
    let brewster = start("elf,female,craftsperson", &["brewster"], 9).expect("starts");
    let wortcunner = start("elf,female,craftsperson", &["wortcunner"], 9).expect("starts");
    let tools = ["moonwort", "glowcap", "comfrey", "vervain"].map(|o| CONTENT.object_id(o));
    let kit = |w: &World| -> Vec<Stack> {
        (w.state().player().inventory.iter())
            .filter(|s| !tools.contains(&Some(s.object)))
            .cloned()
            .collect()
    };
    assert_eq!(kit(&brewster), kit(&wortcunner));
    let others = |w: &World| -> Vec<(crate::skills::SkillId, u16)> {
        (w.state().player().skills.iter())
            .filter(|(id, _)| !["alchemy", "herbalism"].contains(&CONTENT.skill(*id).id.as_str()))
            .map(|(id, t)| (id, t.value))
            .collect()
    };
    assert_eq!(others(&brewster), others(&wortcunner));
    assert_eq!(
        brewster.state().player().stats,
        wortcunner.state().player().stats
    );
    // The same character and seed learn the same.
    let again = start("elf,female,craftsperson", &["brewster"], 9).expect("starts");
    assert_eq!(again.state_hash(), brewster.state_hash());
}
