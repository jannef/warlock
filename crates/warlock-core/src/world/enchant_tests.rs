//! Enchanting's recipes (docs/design/enchanting.md §7, §8; `world/enchant.rs`): Kindle, Quicken,
//! Recast, Imbue, Ennoble, and Unbind made, failed, and warped; the colour rule and `Magic::check` on every result; the
//! reach and the item level; a whole stack of arrows at once; the wizard's steps, the plan, and
//! the rows' lacks; every refusal with no time; the hour and the mark; and a replay.

use super::enchant::{EnchantBand, crystal_object, enchant_band, enchant_out_for_test};
use super::*;
use crate::action::parse_actions;
use crate::content::{AffixKind, ContentFiles, Rarity, RecipeId};
use crate::magic::{Affix, Magic, Quality, capacity};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stats};
use crate::test_support::{last_text as last, run as apply, said_text as said, texts};
use std::sync::LazyLock;

const ROOM: &str = "\
#########
#.......#
#.@.....#
#.......#
#########
";

/// The built-in content with [`ROOM`]; a long sword and a red crystal lie east of the player.
/// A wizard is always trained in Enchanting, for the replay (which starts as the scenario does).
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.edit_table("classes", |text| {
        text.replace(
            "enchanting = { chance = 30,",
            "enchanting = { chance = 100,",
        )
    });
    files.insert(
        "levels/test_enchant.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             [[item]]\nat = [3, 2]\nobject = \"long_sword\"\n\
             [[item]]\nat = [3, 2]\nobject = \"crystal_red\"\n"
        ),
    );
    files.insert(
        "scenarios/test_enchant.toml",
        "name = \"T\"\nstart_level = \"test_enchant\"\n\
         character = { race = \"human\", gender = \"female\", class = \"commoner\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The room. The player is a human commoner carrying nothing, Enchanting 100 with no marks,
/// every score 10 (a modifier of 0), `seed` their fate.
fn world(seed: u64) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_enchant", Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    train(&mut w, 100);
    w
}

fn enchanting(w: &World) -> SkillId {
    w.content.skill_id("enchanting").expect("enchanting")
}

/// Enchanting at `value`, no marks.
fn train(w: &mut World, value: u16) {
    let id = enchanting(w);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

fn marks(w: &World) -> Option<u32> {
    let id = enchanting(w);
    w.state.player().skills.get(id).map(|t| t.marks)
}

/// `count` of `object`, at the end of the pack.
fn item(w: &mut World, object: &str, count: u32) {
    let id = w.content.object_id(object).expect(object);
    w.player_mut().inventory.push(Stack::new(id, count));
}

/// A crystal of `colour`, at the end of the pack.
fn crystal(w: &mut World, colour: &str) {
    let object = crystal_object(&w.content, colour);
    w.player_mut().inventory.push(Stack::one(object));
}

/// A long sword with `affixes` (ids at tier 1, the least value), magic or `rare`, identified or
/// not, at the end of the pack.
fn magic_sword(w: &mut World, affixes: &[&str], rare: bool, identified: bool) {
    let content = w.content.clone();
    let affixes = (affixes.iter())
        .map(|id| Affix {
            id: (*id).to_string(),
            tier: 1,
            value: content.affixes().get(id).expect(id).tiers[0].min,
        })
        .collect();
    let magic = Magic {
        rarity: if rare { Rarity::Rare } else { Rarity::Magic },
        level: 1,
        affixes,
        name: rare.then(|| "Grim Bite".to_string()),
        identified,
        carried: 0,
        unique: None,
    };
    let sword = content.object_id("long_sword").expect("sword");
    w.player_mut()
        .inventory
        .push(Stack::with_magic(sword, 1, magic));
}

/// The last stack of the pack made of `quality`.
fn worked_as(w: &mut World, quality: Quality) {
    let content = w.content.clone();
    let stack = w.player_mut().inventory.last_mut().expect("a stack");
    let worked = quality.worked(content.object(stack.object)).expect("arms");
    stack.set_worked(worked);
}

/// The kinds of `magic`'s affixes, in order.
fn kinds(w: &World, magic: &Magic) -> Vec<AffixKind> {
    (magic.affixes.iter())
        .map(|a| w.content.affixes().get(&a.id).expect("affix").kind)
        .collect()
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| w.lore().describe(s))
        .collect()
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

/// The families of colour `colour`.
fn families(w: &World, colour: &str) -> Vec<String> {
    let crystals = w.content.crystals().expect("crystals");
    crystals.colour(colour).expect(colour).families.clone()
}

/// The magic of pack stack `at`, checked as a save would be.
fn checked(w: &World, at: usize) -> Magic {
    let stack = &w.state.player().inventory[at];
    let magic = stack.magic().expect("magic").clone();
    let def = w.content.object(stack.object);
    magic.check(def, &w.content).expect("Magic::check holds");
    magic
}

fn category(w: &World, affix: &Affix) -> String {
    w.content
        .affixes()
        .get(&affix.id)
        .expect("affix")
        .category
        .clone()
}

#[test]
fn the_band_is_made_from_a_margin_of_0_and_a_natural_2_warps() {
    assert_eq!(enchant_band(12, -1), EnchantBand::Failed);
    assert_eq!(enchant_band(12, 0), EnchantBand::Made);
    assert_eq!(enchant_band(2, 40), EnchantBand::Warped);
    assert_eq!(enchant_band(24, -10), EnchantBand::Made);
}

#[test]
fn the_six_recipes_load_as_enchantings_profession() {
    let w = world(1);
    let recipes = w.content.recipes();
    let skill = enchanting(&w);
    let rows: Vec<(&str, u8, i32, u32, u16)> = (recipes.of(skill))
        .map(|(_, r)| (r.id.as_str(), r.crystals, r.difficulty, r.seconds, r.from))
        .collect();
    assert_eq!(
        rows,
        [
            ("ennoble", 1, 14, 5400, 40),
            ("imbue", 3, 15, 7200, 30),
            ("kindle", 1, 11, 3600, 0),
            ("quicken", 1, 13, 3600, 10),
            ("recast", 1, 12, 3600, 20),
            ("unbind", 0, 10, 1800, 0)
        ]
    );
    assert_eq!(
        recipes.description(skill),
        Some("Your work as an enchanter.")
    );
}

#[test]
fn kindle_makes_a_plain_item_magic_with_one_affix_of_the_crystals_colour() {
    for colour in ["red", "blue", "white", "black", "green", "gold"] {
        for seed in 1..12 {
            let mut w = world(seed);
            item(&mut w, "long_sword", 1);
            crystal(&mut w, colour);
            enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Made);
            let magic = checked(&w, 0);
            assert_eq!(magic.rarity, Rarity::Magic);
            assert_eq!(magic.affixes.len(), 1);
            assert!(magic.identified);
            // Enchanting 100: a reach of 11.
            assert_eq!(magic.level, 11);
            let families = families(&w, colour);
            assert!(
                families.contains(&category(&w, &magic.affixes[0])),
                "{colour}: {:?}",
                magic.affixes
            );
            assert_eq!(w.state.player().inventory.len(), 1, "the crystal is spent");
        }
    }
    let mut w = world(3);
    item(&mut w, "long_sword", 1);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Made);
    let name = pack(&w)[0].clone();
    assert!(
        said(
            &w,
            &format!("The long sword takes the red crystal's glow. It is now {name}.")
        ),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn a_failure_spends_the_crystal_and_a_natural_2_warps_the_item() {
    let mut w = world(1);
    item(&mut w, "long_sword", 1);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Failed);
    assert_eq!(pack(&w), ["a long sword"]);
    assert!(said(&w, "The crystal goes dark in your hands."));

    let mut w = world(1);
    crystal(&mut w, "red");
    item(&mut w, "long_sword", 1);
    enchant_out_for_test(&mut w, "kindle", &[Some(1), Some(0)], EnchantBand::Warped);
    assert_eq!(pack(&w), ["a warped long sword"], "{:?}", texts(&w));
    assert!(said(&w, "The long sword warps under your hands."));
    // Warped already, it falls apart, and the crystal ahead of it is spent all the same.
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Warped);
    assert!(pack(&w).is_empty(), "{:?}", pack(&w));
    assert!(
        said(&w, "The warped long sword warps and falls apart."),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn quicken_adds_the_kind_a_magic_item_lacks() {
    for seed in 1..12 {
        let mut w = world(seed);
        magic_sword(&mut w, &["sharp"], false, true);
        crystal(&mut w, "red");
        enchant_out_for_test(&mut w, "quicken", &[Some(0), Some(1)], EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.affixes.len(), 2);
        assert_eq!(magic.rarity, Rarity::Magic);
        assert_eq!(magic.affixes[0].id, "sharp");
        let added = w
            .content
            .affixes()
            .get(&magic.affixes[1].id)
            .expect("affix");
        assert_eq!(added.kind, AffixKind::Suffix);
        assert!(families(&w, "red").contains(&added.category));
        // The roll's level: the greater of the item's (1) and the reach (11).
        assert_eq!(magic.level, 11);
    }
}

#[test]
fn unbind_takes_every_affix_off_and_gives_nothing_back() {
    let mut w = world(1);
    magic_sword(&mut w, &["sharp", "of_the_bear"], true, true);
    enchant_out_for_test(&mut w, "unbind", &[Some(0)], EnchantBand::Made);
    assert_eq!(pack(&w), ["a long sword"]);
    assert!(w.state.player().inventory[0].magic().is_none());
    assert!(
        said(
            &w,
            "The magic goes out of Grim Bite. It is now a long sword."
        ),
        "{:?}",
        texts(&w)
    );

    let mut w = world(1);
    magic_sword(&mut w, &["sharp"], false, true);
    enchant_out_for_test(&mut w, "unbind", &[Some(0)], EnchantBand::Failed);
    assert!(w.state.player().inventory[0].magic().is_some());
    assert!(
        said(&w, "The Sharp long sword clings to its magic."),
        "{:?}",
        texts(&w)
    );
}

#[test]
fn a_stack_of_arrows_is_kindled_whole() {
    let mut w = world(2);
    item(&mut w, "arrow", 40);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Made);
    let stack = &w.state.player().inventory[0];
    assert_eq!(stack.count, 40);
    checked(&w, 0);
    assert_eq!(w.state.player().inventory.len(), 1);
}

#[test]
fn the_reach_grows_with_enchanting_and_a_deeper_item_keeps_its_level() {
    for (value, reach) in [(0, 1), (10, 2), (55, 6), (110, 12), (200, 12)] {
        let mut w = world(1);
        train(&mut w, value);
        item(&mut w, "long_sword", 1);
        crystal(&mut w, "red");
        enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Made);
        assert_eq!(checked(&w, 0).level, reach, "Enchanting {value}");
    }
    // Quicken rolls at a found item's own level when it is deeper than the reach.
    let mut w = world(1);
    train(&mut w, 10);
    magic_sword(&mut w, &["sharp"], false, true);
    if let Some(m) = w.player_mut().inventory[0].magic_mut() {
        m.level = 9;
    }
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "quicken", &[Some(0), Some(1)], EnchantBand::Made);
    assert_eq!(checked(&w, 0).level, 9);
}

#[test]
fn the_steps_plan_and_rows_say_what_the_work_takes() {
    let mut w = world(1);
    item(&mut w, "long_sword", 1); // a
    magic_sword(&mut w, &["sharp"], false, true); // b
    magic_sword(&mut w, &["sharp"], false, false); // c: unknown
    crystal(&mut w, "red"); // d
    item(&mut w, "iron_ring", 1); // e
    apply(&mut w, "equip:left_ring:e");
    let steps = w.perception().craft_steps(recipe(&w, "kindle"));
    let at = |n: usize| -> Vec<usize> { steps[n].options.iter().map(|o| o.at).collect() };
    assert_eq!(
        (steps.iter().map(|s| s.label.as_str())).collect::<Vec<_>>(),
        ["item", "crystal", "seal", "second seal", "third seal"]
    );
    assert_eq!(
        at(0),
        [0],
        "the plain sword; never the worn ring or a magic one"
    );
    assert_eq!(at(1), [3]);
    // Imbue's crystals are told apart: their order matters.
    let steps = w.perception().craft_steps(recipe(&w, "imbue"));
    assert_eq!(
        (steps.iter().map(|s| s.label.as_str())).collect::<Vec<_>>(),
        [
            "item",
            "first crystal",
            "second crystal",
            "third crystal",
            "seal",
            "second seal",
            "third seal"
        ]
    );
    let steps = w.perception().craft_steps(recipe(&w, "unbind"));
    assert_eq!(steps.len(), 1);
    assert_eq!(
        steps[0].options.iter().map(|o| o.at).collect::<Vec<_>>(),
        [1]
    );

    let plan = (w.perception())
        .craft_plan(recipe(&w, "kindle"), &[Some(0), Some(3)])
        .expect("a plan");
    assert_eq!(plan.verb, "Kindle");
    assert_eq!(plan.name, "the long sword");
    assert_eq!(plan.uses, "a red crystal");
    assert_eq!(plan.time, "about 1 hour");
    let plan = (w.perception())
        .craft_plan(recipe(&w, "unbind"), &[Some(1)])
        .expect("a plan");
    assert_eq!((plan.verb.as_str(), plan.uses.as_str()), ("Unbind", ""));
    assert_eq!(plan.time, "about 30 minutes");

    let rows = |w: &World| -> Vec<(String, String, Option<String>)> {
        let view = w.perception();
        (view.recipes(enchanting(w)).into_iter())
            .map(|r| (r.name, r.takes, r.unavailable.map(|l| l.id(&w.content))))
            .collect()
    };
    let row = |name: &str, takes: &str, lack: Option<&str>| {
        (name.to_string(), takes.to_string(), lack.map(String::from))
    };
    assert_eq!(
        rows(&w),
        [
            // The steel sword holds two: no rare can be made of it.
            row(
                "Ennoble",
                "a magic item, a crystal, about 1 hour and 30 minutes",
                Some("room")
            ),
            row(
                "Imbue",
                "a plain item, 3 crystals, about 2 hours",
                Some("room")
            ),
            row("Kindle", "a plain item, a crystal, about 1 hour", None),
            row("Quicken", "a magic item, a crystal, about 1 hour", None),
            row("Recast", "a magic item, a crystal, about 1 hour", None),
            row("Unbind", "a magic item, about 30 minutes", None),
        ]
    );
    // No crystal: the works that take one wait for it.
    super::outcome::remove_stacks(w.player_mut(), &[3]);
    let lacks: Vec<Option<String>> = rows(&w).into_iter().map(|r| r.2).collect();
    let some = |s: &str| Some(s.to_string());
    assert_eq!(
        lacks,
        [
            some("room"),
            some("room"),
            some("crystal"),
            some("crystal"),
            some("crystal"),
            None
        ]
    );
    // A Masterwork sword holds four, but two crystals are not Imbue's three.
    item(&mut w, "long_sword", 1);
    worked_as(&mut w, Quality::Masterwork);
    crystal(&mut w, "red");
    crystal(&mut w, "blue");
    assert_eq!(rows(&w)[1].2, some("crystal"));
    crystal(&mut w, "blue");
    assert_eq!(rows(&w)[1].2, None);
    let n = w.state.player().inventory.len();
    let picks = [Some(n - 4), Some(n - 2), Some(n - 3), Some(n - 1)];
    let plan = (w.perception())
        .craft_plan(recipe(&w, "imbue"), &picks)
        .expect("a plan");
    assert_eq!(
        (plan.name.as_str(), plan.uses.as_str(), plan.time.as_str()),
        (
            "the Masterwork long sword",
            "a blue crystal, a red crystal, and a blue crystal",
            "about 2 hours"
        )
    );
    // A sword of two affixes has no room; nothing plain, nothing to kindle.
    let mut w = world(1);
    magic_sword(&mut w, &["sharp", "of_the_bear"], false, true);
    let lacks: Vec<Option<String>> = rows(&w).into_iter().map(|r| r.2).collect();
    assert_eq!(
        lacks,
        [
            some("room"),
            some("nothing"),
            some("nothing"),
            some("room"),
            some("crystal"),
            None
        ]
    );
    train(&mut w, 5);
    let lacks: Vec<Option<String>> = rows(&w).into_iter().map(|r| r.2).collect();
    assert_eq!(lacks[3], Some("skill".into()));
}

#[test]
fn every_refusal_takes_no_time() {
    let mut w = world(1);
    item(&mut w, "long_sword", 1); // a
    crystal(&mut w, "red"); // b
    magic_sword(&mut w, &["sharp"], false, false); // c: unknown
    magic_sword(&mut w, &["sharp", "of_the_bear"], false, true); // d: full
    magic_sword(&mut w, &["sharp"], true, true); // e: rare
    item(&mut w, "iron_ring", 1); // f
    apply(&mut w, "equip:left_ring:f");
    let ruby = w.content.object_id("heart_crystal").expect("heart-crystal");
    w.player_mut().inventory.push(Stack::one(ruby)); // g
    item(&mut w, "long_sword", 1); // h
    worked_as(&mut w, Quality::Masterwork);
    let before = w.state.time;
    for (picks, line) in [
        ("craft:kindle", "You must choose the item."),
        ("craft:kindle:a", "You must choose the crystal."),
        (
            "craft:kindle:a,b,-,-,-,b",
            "That is more than the work takes.",
        ),
        ("craft:kindle:a,g", "That is no crystal."),
        ("craft:kindle:b,b", "No magic can be bound in that."),
        (
            "craft:kindle:c,b",
            "You must know what is bound in it first.",
        ),
        ("craft:kindle:f,b", "Take it off first."),
        ("craft:kindle:d,b", "There is magic in it already."),
        ("craft:kindle:z,b", "You carry no such thing."),
        ("craft:quicken:a,b", "There is no magic in it."),
        ("craft:quicken:d,b", "It holds all it can."),
        ("craft:unbind:a", "There is no magic in it."),
        ("craft:unbind:d,b", "That is more than the work takes."),
        ("craft:recast:a,b", "There is no magic in it."),
        (
            "craft:recast:e,b",
            "It is rare: unbind it, then imbue it anew.",
        ),
        ("craft:imbue:a,b", "You must choose the crystals."),
        ("craft:imbue:a,b,b,b", "It would not hold so much."),
        ("craft:imbue:d,b,b,b", "There is magic in it already."),
        ("craft:ennoble:a,b", "There is no magic in it."),
        ("craft:ennoble:d,b", "It would not hold so much."),
        (
            "craft:ennoble:e,b",
            "It is rare already: only a heart-crystal's Quicken adds to it.",
        ),
        // A Masterwork sword holds four; one red crystal is not three.
        ("craft:imbue:h,b,b,b", "You do not carry so many crystals."),
        ("craft:imbue:h,b,g,g", "That is no crystal."),
    ] {
        apply(&mut w, picks);
        assert_eq!(last(&w), line, "{picks}");
    }
    train(&mut w, 5);
    apply(&mut w, "craft:quicken:c,b");
    assert_eq!(last(&w), "That is beyond your skill.");
    w.player_mut().skills = Default::default();
    apply(&mut w, "craft:kindle:a,b");
    assert_eq!(last(&w), "You are not trained in Enchanting.");
    assert_eq!(w.state.time, before);
}

#[test]
fn a_colour_with_nothing_left_to_bind_is_refused() {
    // Silver holds magic badly: a silver ring holds one affix. A ring of every family of a
    // colour that may roll on it has none left; simpler, a pick-axe takes no suffix, so a
    // Quicken on a pick-axe with a prefix has nothing to draw.
    let mut w = world(1);
    let content = w.content.clone();
    let pick = content.object_id("pick_axe").expect("pick-axe");
    let def = content.object(pick);
    let prefix = (content.affixes().all().iter())
        .find(|a| {
            a.kind == AffixKind::Prefix
                && def.kind.is_some_and(|k| crate::magic::may_roll(a, k, def))
        })
        .expect("a prefix for tools");
    let suffix = (content.affixes().all().iter()).any(|a| {
        a.kind == AffixKind::Suffix && def.kind.is_some_and(|k| crate::magic::may_roll(a, k, def))
    });
    if suffix {
        return; // tools have suffixes now: nothing to show here
    }
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: prefix.id.clone(),
            tier: 1,
            value: prefix.tiers[0].min,
        }],
        name: None,
        identified: true,
        carried: 0,
        unique: None,
    };
    w.player_mut()
        .inventory
        .push(Stack::with_magic(pick, 1, magic));
    crystal(&mut w, "red");
    let before = w.state.time;
    apply(&mut w, "craft:quicken:a,b");
    assert_eq!(last(&w), "No more of that nature will take.");
    assert_eq!(w.state.time, before);
}

#[test]
fn the_work_takes_its_hour_and_marks_enchanting() {
    let (mut made, mut failed) = (false, false);
    for seed in 1..40 {
        let mut w = world(seed);
        train(&mut w, 0);
        item(&mut w, "long_sword", 1);
        crystal(&mut w, "red");
        let start = w.state.time;
        apply(&mut w, "craft:kindle:a,b");
        assert!(said(&w, "You take the long sword in your lap and begin."));
        assert_eq!(w.state.time.since(start).as_seconds(), 3600);
        assert_eq!(marks(&w), Some(1));
        assert_eq!(w.state.player().inventory.len(), 1, "{:?}", texts(&w));
        match w.state.player().inventory[0].magic() {
            Some(_) => {
                checked(&w, 0);
                made = true;
            }
            None => failed = true,
        }
    }
    assert!(made && failed);
}

/// Quick hands (#1425) halves an enchanting's hour, as the plan's time says.
#[test]
fn quick_hands_halves_the_hour() {
    let mut w = world(1);
    w.player_mut().talents.take("quick_hands", None);
    item(&mut w, "long_sword", 1);
    crystal(&mut w, "red");
    let id = recipe(&w, "kindle");
    let content = w.content.clone();
    let plan = super::enchant::plan(
        &content,
        w.lore(),
        w.state.player(),
        content.recipe(id),
        &[Some(0), Some(1)],
    )
    .expect("a plan");
    assert_eq!(plan.time, "about 30 minutes");
    let start = w.state.time;
    apply(&mut w, "craft:kindle:a,b");
    assert_eq!(w.state.time.since(start).as_seconds(), 1800);
}

#[test]
fn an_enchanting_replays() {
    // A wizard, trained in Enchanting from birth here: the untrained make nothing (#1387).
    let wizard: crate::character::CharacterSpec = "human,female,wizard".parse().expect("spec");
    let mut w = World::with_character(FIXTURE.clone(), "test_enchant", Some(8), Some(&wizard))
        .expect("world");
    assert!(w.state.player().skills.get(enchanting(&w)).is_some());
    let mut replay = Replay::new(&w);
    let text = "move:E pickup";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let pack = &w.state.player().inventory;
    let sword = (pack.iter())
        .position(|s| w.lore().title(s) == "long sword")
        .expect("the sword");
    let stone = (pack.iter())
        .position(|s| w.lore().title(s) == "red crystal")
        .expect("the crystal");
    let text = format!("craft:kindle:{sword},{stone} wait");
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        said(&w, "You take the long sword in your lap and begin."),
        "{:?}",
        texts(&w)
    );
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn recast_rolls_as_many_affixes_of_the_same_kinds_anew() {
    for seed in 1..12 {
        let mut w = world(seed);
        magic_sword(&mut w, &["sharp", "of_the_bear"], false, true);
        crystal(&mut w, "blue");
        enchant_out_for_test(&mut w, "recast", &[Some(0), Some(1)], EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.rarity, Rarity::Magic);
        assert_eq!(magic.name, None);
        assert_eq!(kinds(&w, &magic), [AffixKind::Prefix, AffixKind::Suffix]);
        let blue = families(&w, "blue");
        for affix in &magic.affixes {
            assert!(blue.contains(&category(&w, affix)), "{:?}", magic.affixes);
        }
        assert_eq!(magic.level, 11);
        assert_eq!(w.state.player().inventory.len(), 1, "the crystal is spent");
    }
    // One suffix stays one suffix.
    let mut w = world(2);
    magic_sword(&mut w, &["of_the_bear"], false, true);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "recast", &[Some(0), Some(1)], EnchantBand::Made);
    let magic = checked(&w, 0);
    assert_eq!(kinds(&w, &magic), [AffixKind::Suffix]);
    // Failed: the old affixes stay, the crystal is spent.
    let mut w = world(2);
    magic_sword(&mut w, &["sharp", "of_the_bear"], false, true);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "recast", &[Some(0), Some(1)], EnchantBand::Failed);
    let ids: Vec<String> = checked(&w, 0).affixes.into_iter().map(|a| a.id).collect();
    assert_eq!(ids, ["sharp", "of_the_bear"]);
    assert_eq!(w.state.player().inventory.len(), 1);
}

#[test]
fn imbue_makes_a_plain_item_rare_within_its_capacity() {
    for seed in 1..16 {
        // A Marvellous steel sword holds three: always three affixes.
        let mut w = world(seed);
        item(&mut w, "long_sword", 1);
        worked_as(&mut w, Quality::Marvellous);
        assert_eq!(capacity(&w.state.player().inventory[0], &w.content), 3);
        crystal(&mut w, "red");
        crystal(&mut w, "blue");
        crystal(&mut w, "white");
        let picks = [Some(0), Some(1), Some(2), Some(3)];
        enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.rarity, Rarity::Rare);
        assert!(magic.identified);
        assert_eq!(magic.affixes.len(), 3, "{:?}", magic.affixes);
        assert_eq!(magic.level, 11);
        // Prefixes before suffixes, each affix of one crystal's colour.
        let k = kinds(&w, &magic);
        assert!(k.windows(2).all(|p| p[0] <= p[1]), "{k:?}");
        let colours = ["red", "blue", "white"].map(|c| families(&w, c));
        for a in &magic.affixes {
            assert!(colours.iter().any(|f| f.contains(&category(&w, a))));
        }
        assert_eq!(
            w.state.player().inventory.len(),
            1,
            "the crystals are spent"
        );
        let name = magic.name.clone().expect("a name");
        let line =
            format!("The Marvellous long sword drinks the three crystals and wakes as {name}.");
        assert!(said(&w, &line), "{:?}", texts(&w));
    }
    // Three red crystals from one stack: every affix red; a Masterwork sword holds four.
    let mut seen = Vec::new();
    for seed in 1..24 {
        let mut w = world(seed);
        item(&mut w, "long_sword", 1);
        worked_as(&mut w, Quality::Masterwork);
        crystal(&mut w, "red");
        w.player_mut().inventory[1].count = 4;
        let picks = [Some(0), Some(1), Some(1), Some(1)];
        enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        let red = families(&w, "red");
        assert!(magic.affixes.iter().all(|a| red.contains(&category(&w, a))));
        assert!((3..=4).contains(&magic.affixes.len()));
        seen.push(magic.affixes.len());
        assert_eq!(
            w.state.player().inventory[1].count,
            1,
            "three of four spent"
        );
    }
    assert!(seen.contains(&4), "{seen:?}");
}

#[test]
fn imbues_crystals_each_give_an_affix_of_their_colour() {
    // A green crystal first and two black ones: one affix at least is green's, and all are of
    // the two colours.
    for seed in 1..16 {
        let mut w = world(seed);
        item(&mut w, "long_sword", 1);
        worked_as(&mut w, Quality::Marvellous);
        crystal(&mut w, "green");
        crystal(&mut w, "black");
        crystal(&mut w, "black");
        let picks = [Some(0), Some(1), Some(2), Some(3)];
        enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        let green = families(&w, "green");
        let black = families(&w, "black");
        let of = |f: &[String], a: &Affix| f.contains(&category(&w, a));
        assert!(magic.affixes.iter().any(|a| of(&green, a)));
        assert!((magic.affixes.iter()).all(|a| of(&green, a) || of(&black, a)));
        assert!((magic.affixes.iter()).filter(|a| of(&black, a)).count() >= 2);
    }
}

#[test]
fn a_failed_imbue_spends_all_three_crystals() {
    let mut w = world(1);
    crystal(&mut w, "red"); // a
    item(&mut w, "long_sword", 1); // b
    worked_as(&mut w, Quality::Masterwork);
    crystal(&mut w, "blue"); // c
    w.player_mut().inventory[2].count = 2;
    let picks = [Some(1), Some(0), Some(2), Some(2)];
    enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Failed);
    assert!(said(&w, "The crystals go dark in your hands."));
    assert_eq!(pack(&w), ["a Masterwork long sword"]);
    // A natural 2: it warps, and the crystals around it are spent all the same.
    let red = crystal_object(&w.content, "red");
    w.player_mut().inventory.insert(0, Stack::one(red));
    w.player_mut().inventory.push(Stack::new(red, 2));
    let picks = [Some(1), Some(0), Some(2), Some(2)];
    enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Warped);
    assert_eq!(pack(&w), ["a warped long sword"], "{:?}", texts(&w));
}

#[test]
fn ennoble_makes_a_magic_item_rare_with_one_affix_more_and_a_name() {
    let mut drew = Vec::new();
    for seed in 1..16 {
        let mut w = world(seed);
        magic_sword(&mut w, &["sharp"], false, true);
        worked_as(&mut w, Quality::Marvellous);
        crystal(&mut w, "red");
        let thing = format!("The {}", w.lore().title(&w.state.player().inventory[0]));
        enchant_out_for_test(&mut w, "ennoble", &[Some(0), Some(1)], EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.rarity, Rarity::Rare);
        assert_eq!(magic.affixes.len(), 2);
        let k = kinds(&w, &magic);
        assert!(k.windows(2).all(|p| p[0] <= p[1]), "{k:?}");
        let added = (magic.affixes.iter())
            .find(|a| a.id != "sharp")
            .expect("added");
        assert!(families(&w, "red").contains(&category(&w, added)));
        drew.push(k);
        let name = magic.name.clone().expect("a name");
        let line = format!("{thing} takes the red crystal's glow and wakes as {name}.");
        assert!(said(&w, &line), "{:?}", texts(&w));
    }
    // Both kinds have room on a magic item: either may be drawn.
    assert!(drew.iter().any(|k| k[1] == AffixKind::Suffix), "{drew:?}");
    assert!(drew.iter().any(|k| k[1] == AffixKind::Prefix), "{drew:?}");
    // A Fine sword holds two, too few for a rare; a Marvellous one of two affixes takes a third.
    let mut w = world(1);
    magic_sword(&mut w, &["sharp", "of_the_bear"], false, true);
    worked_as(&mut w, Quality::Fine);
    crystal(&mut w, "red");
    let before = w.state.time;
    apply(&mut w, "craft:ennoble:a,b");
    assert_eq!(last(&w), "It would not hold so much.");
    assert_eq!(w.state.time, before);
    let content = w.content.clone();
    let stack = &mut w.player_mut().inventory[0];
    let worked = Quality::Marvellous.worked(content.object(stack.object));
    stack.set_worked(worked.expect("arms"));
    apply(&mut w, "craft:ennoble:a,b");
    assert_eq!(w.state.time.since(before).as_seconds(), 5400);
}

/// A warp on a stack of arrows warps one of them, set apart, and the rest are as they were
/// (#1487); a proof item that withstands a warp says so.
#[test]
fn a_warp_on_a_stack_warps_one_and_a_proof_item_says_it_withstood() {
    let mut w = world(2);
    item(&mut w, "arrow", 40);
    crystal(&mut w, "red");
    enchant_out_for_test(&mut w, "kindle", &[Some(0), Some(1)], EnchantBand::Warped);
    let damaged = |s: &Stack| s.worked().is_some_and(|w| w.damaged);
    let pack = &w.state.player().inventory;
    assert_eq!(pack.len(), 2, "{:?}", texts(&w));
    assert_eq!((pack[0].count, damaged(&pack[0])), (39, false));
    assert_eq!((pack[1].count, damaged(&pack[1])), (1, true));
    assert!(
        said(&w, "The arrow warps under your hands."),
        "{:?}",
        texts(&w)
    );

    let mut w = world(1);
    magic_sword(&mut w, &["sharp"], false, true);
    let content = w.content.clone();
    let adamantium = content.material_id("adamantium").expect("adamantium");
    let sword = w.player_mut().inventory.last_mut().expect("the sword");
    *sword = sword
        .clone()
        .with_parts(vec![crate::item_parts::ItemPart::material(
            &content, None, adamantium,
        )]);
    enchant_out_for_test(&mut w, "unbind", &[Some(0)], EnchantBand::Warped);
    assert!(!damaged(&w.state.player().inventory[0]));
    assert_eq!(last(&w), "The work goes wrong, but no harm is done.");
}

/// Bread ahead of the sword rots away during the hour (#1564): the work finds the sword and the
/// crystal where they went and kindles the sword, never the crystal or the dagger after them.
#[test]
fn a_stack_gone_ahead_of_the_work_never_turns_it_on_the_next() {
    for seed in 1..20 {
        let mut w = world(seed);
        let loaf = super::work_on::rotting_loaf(&w);
        w.player_mut().inventory.push(loaf);
        item(&mut w, "long_sword", 1);
        crystal(&mut w, "red");
        item(&mut w, "dagger", 1);
        apply(&mut w, "craft:kindle:b,c");
        assert!(said(&w, "Your loaf of bread has rotted away."));
        let pack = &w.state.player().inventory;
        let name = |s: &Stack| w.content.object(s.object).name.clone();
        assert_eq!(name(&pack[0]), "long sword", "{:?}", texts(&w));
        let dagger = pack.last().expect("the dagger");
        assert_eq!((name(dagger).as_str(), dagger.magic()), ("dagger", None));
        if pack[0].magic().is_some() {
            assert_eq!(pack.len(), 2, "the crystal spent");
            return;
        }
    }
    panic!("some fate kindles the sword");
}
