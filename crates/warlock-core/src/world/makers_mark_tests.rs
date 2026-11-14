//! Maker's mark (#1426; the craftsperson design §7.10): the craftsperson's at level 60, its one
//! read in `world/craft_make.rs::make_out` (arms, armour, and jewellery made marked, whatever
//! the grade; a kindless result never), and the mark's reads: the worth (`Worked::value`), the
//! description's line, the capacity, and the patron's bonus. Without the talent nothing is
//! marked.

use super::craft_make::make_out_for_test;
use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor};
use crate::magic::worked::Quality;
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::CONTENT;
use crate::test_support::run as act;
use std::sync::LazyLock;

/// A forge with its anvil at (5, 2), the player beside it at (4, 3).
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
###########
";

/// The built-in content with [`ROOM`], a forge, and a dwarf craftsperson named Brakka.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_mark.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 5]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n"
        ),
    );
    files.insert(
        "scenarios/test_mark.toml",
        "name = \"T\"\nstart_level = \"test_mark\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\", \
         name = \"Brakka\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The forge, the player carrying nothing, trained in Metalcraft and Jewelcraft at 50, holding
/// Maker's mark when `marks`.
fn world(marks: bool) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_mark", None).expect("world");
    let me = w.player_mut();
    me.inventory.clear();
    me.equipment = Default::default();
    for id in ["metalcraft", "jewelcraft"] {
        let s: SkillId = w.content.skill_id(id).expect(id);
        let skills = &mut w.player_mut().skills;
        let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(k, _)| *k != s).collect();
        all.push((
            s,
            Trained {
                value: 50,
                marks: 0,
            },
        ));
        *skills = all.into_iter().collect();
    }
    if marks {
        w.player_mut().talents.take("makers_mark", None);
    }
    w
}

/// `count` ingots of `metal` into the pack.
fn ingots(w: &mut World, metal: &str, count: u32) {
    let id = w.content.material_id(metal).expect(metal);
    let stack = crate::piece::make(&w.content, id, count).expect("ingots");
    w.player_mut().inventory.push(stack);
}

/// The last thing in the pack.
fn made(w: &World) -> &Stack {
    w.state.player().inventory.last().expect("made")
}

fn marked(stack: &Stack) -> bool {
    stack.worked().is_some_and(|w| w.marked())
}

#[test]
fn the_craftsperson_takes_makers_mark_at_level_sixty() {
    let spec: CharacterSpec = "human,male,craftsperson".parse().expect("a character");
    let content = CONTENT.clone();
    let mut w =
        World::with_character(content.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    let factor = Factor::of(&content, w.state.character.as_ref().map(|c| &c.spec));
    let held = |w: &World| w.state.player().talents.holds("makers_mark", None);
    for level in [59, 60] {
        assert!(!held(&w), "not before level 60");
        let more = content.experience().threshold_for(level, factor) - w.player_xp();
        act(&mut w, &format!("xp:{more}"));
        while w.due().is_some() {
            act(&mut w, "levelup:auto");
        }
    }
    assert_eq!(w.player_level(), 60);
    assert!(held(&w), "at level 60");
    assert_eq!(content.talents().mark_percent(), 50);
}

#[test]
fn a_masters_sword_bears_the_mark_and_is_worth_half_again() {
    for natural in [12, 24] {
        let mut w = world(true);
        ingots(&mut w, "steel", 2);
        // A plain sword (margin 0) and a Masterwork (a natural 24, margin 30) alike.
        let margin = if natural == 24 { 30 } else { 0 };
        make_out_for_test(&mut w, "short_sword", &[Some(0), None], natural, margin);
        let sword = made(&w).clone();
        assert!(marked(&sword), "marked at natural {natural}");
        let mut plain = sword.clone();
        let mut unmarked = sword.worked().cloned().expect("worked");
        unmarked.marked = false;
        plain.set_worked(unmarked);
        assert!(!marked(&plain));
        assert_eq!(
            sword.value(&w.content),
            plain.value(&w.content) * 3 / 2,
            "half again"
        );
        // Its name is unchanged; its description says whose mark it is.
        assert_eq!(w.lore().describe(&sword), w.lore().describe(&plain));
        let lines: Vec<String> = (w.perception().describe_item(&sword).into_iter())
            .map(|l| l.text)
            .collect();
        assert!(
            lines.iter().any(|l| l == "It bears Brakka's mark."),
            "{lines:?}"
        );
        let nameless = crate::magic::describe::lines(&sword, &w.content);
        assert!(
            (nameless.iter()).any(|l| l.text == "It bears its maker's mark."),
            "{nameless:?}"
        );
        // One enchantment more.
        let room = |s: &Stack| crate::magic::capacity(s, &w.content);
        assert_eq!(room(&sword), room(&plain) + 1);
        // The dev dump marks it.
        assert!(sword.describe_truth(&w.content).contains("(marked)"));
    }
}

#[test]
fn a_ring_is_marked_and_a_lantern_never() {
    let mut w = world(true);
    ingots(&mut w, "iron", 2);
    make_out_for_test(&mut w, "iron_ring", &[Some(0), None], 12, 0);
    assert!(marked(made(&w)), "jewellery bears the mark");
    let mut w = world(true);
    ingots(&mut w, "iron", 4);
    make_out_for_test(&mut w, "lantern", &[Some(0)], 12, 0);
    assert_eq!(w.content.object(made(&w).object).id, "lantern");
    assert!(!marked(made(&w)), "a kindless thing never");
}

#[test]
fn without_the_talent_nothing_is_marked() {
    let mut w = world(false);
    ingots(&mut w, "steel", 2);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, 0);
    assert!(!marked(made(&w)));
    assert!(made(&w).worked().is_none(), "a plain sword carries nothing");
}

#[test]
fn the_mark_is_saved_and_survives_damage_and_repair() {
    let mut w = world(true);
    ingots(&mut w, "steel", 2);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, 0);
    let sword = made(&w).clone();
    let text = serde_json::to_string(&sword).expect("saved");
    assert!(text.contains("\"marked\":true"), "{text}");
    let back: Stack = serde_json::from_str(&text).expect("loaded");
    assert_eq!(back, sword);
    let mut worked = sword.worked().cloned().expect("worked");
    worked.damage(crate::content::ItemHarm::Rust);
    assert!(worked.marked());
    worked.repair();
    assert!(worked.marked());
}

/// A level-60 craftsperson in Cēaptūn beside their patron, Metalcraft at 40 at least.
fn at_the_patron() -> World {
    let spec: CharacterSpec = "human,male,craftsperson".parse().expect("a character");
    let mut w = World::start_run(CONTENT.clone(), "ceaptun", None, Some(&spec), None, true)
        .expect("a run in Cēaptūn");
    let factor = Factor::of(&CONTENT, w.state.character.as_ref().map(|c| &c.spec));
    let more = CONTENT.experience().threshold_for(60, factor) - w.player_xp();
    act(&mut w, &format!("xp:{more}"));
    while w.due().is_some() {
        act(&mut w, "levelup:auto");
    }
    assert!(w.state.player().talents.has("makers_mark"));
    let metalcraft = CONTENT.skill_id("metalcraft").expect("metalcraft");
    w.player_mut().skills.add(metalcraft, 40);
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = Pos::new(49, 23);
    w.perceive(&mut Vec::new());
    w
}

fn gold(w: &World) -> u64 {
    let coin = CONTENT.object_id("gold_coin").expect("gold");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == coin)
        .map(|s| u64::from(s.count))
        .sum()
}

#[test]
fn the_patron_pays_half_again_for_a_marked_piece() {
    let mut w = at_the_patron();
    act(&mut w, "chat:E");
    let open = w.state.patron.open.as_ref().expect("a commission open");
    let recipe = CONTENT.recipe(CONTENT.recipes().id(&open.recipe).expect("a recipe"));
    let object = recipe.result.as_ref().expect("a result").object;
    let mut piece = Stack::new(object, 1);
    let mut worked = Quality::Fine
        .worked(CONTENT.object(object))
        .expect("graded");
    piece.set_worked(worked.clone());
    let unmarked = piece.value(&CONTENT);
    worked.mark();
    piece.set_worked(worked);
    let worth = piece.value(&CONTENT);
    assert_eq!(worth, unmarked * 3 / 2, "the mark is worth half again");
    let content = w.content.clone();
    crate::object::add_to_inventory(w.state.inventory_mut(), piece, &content);
    let coins = gold(&w);
    act(&mut w, "chat:E");
    assert!(w.state.patron.open.is_none(), "taken");
    // Half again the unmarked piece's pay, the mark paid for once (#1489): not 2.25 times.
    assert_eq!(gold(&w), coins + unmarked * 3 / 2);
}
