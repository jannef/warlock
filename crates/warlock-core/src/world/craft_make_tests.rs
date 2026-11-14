//! Making a generic recipe (docs/dev/crafting.md "Making a recipe"; #1335): the grades by
//! margin and the extremes, the odds the wiki states, each grade's `Worked`, the failure's half
//! of the stock back, Temper refusing a crafted Masterwork, an item with no aspect always plain,
//! the wizard's steps and the dialog's plan, the refusals with no time, the hour's work end to
//! end with its mark, and an interruption that spends nothing.

use super::craft_make::make_out_for_test;
use super::*;
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::run as apply;

/// A forge with its anvil at (5, 2), the player beside it at (4, 3), and at `k` a kobold, when
/// asked for.
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`] and a ring of metal stock (no aspect) to forge; the kobold
/// only with `kobold`. The player (a craftsperson) carries nothing and has Metalcraft 50.
fn world(kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = match kobold {
        true => "{ terrain = \"floor\", creature = \"kobold\" }",
        false => "\"floor\"",
    };
    files.insert(
        "levels/test_forge.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"forge\"\narea = [0, 0, 10, 6]\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_forge.toml",
        "name = \"T\"\nstart_level = \"test_forge\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    files.insert(
        "data/recipes/metalcraft/ring.toml",
        "[[recipe]]\nid = \"ring\"\nresult = { object = \"iron_ring\" }\ndifficulty = 5\n\
         seconds = 600\nstation = \"anvil\"\n[recipe.parts]\nmain = \"stock\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_forge", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    train(&mut w, 50);
    w
}

fn metalcraft(w: &World) -> SkillId {
    w.content.skill_id("metalcraft").expect("metalcraft")
}

/// Metalcraft at `value`, no marks, the other skills as they were.
fn train(w: &mut World, value: u16) {
    let id = metalcraft(w);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

/// `count` ingots of `material` at the end of the pack.
fn ingots(w: &mut World, material: &str, count: u32) {
    let m = w.content.material_id(material).expect(material);
    let stack = crate::piece::make(&w.content, m, count).expect("ingots");
    w.player_mut().inventory.push(stack);
}

/// A loose `gem` of the plain size, cut `cut`, at the end of the pack.
fn gem(w: &mut World, gem: &str, cut: &str) {
    let content = w.content.clone();
    let gems = content.gems().expect("gems");
    let mut cutting = gems.plain();
    cutting.cut = gems.cut_id(cut).expect(cut);
    let m = content.material_id(gem).expect(gem);
    let part = crate::item_parts::ItemPart::gem(&content, None, m, cutting);
    let stack = Stack::one(gems.object).with_parts(vec![part]);
    w.player_mut().inventory.push(stack);
}

fn said(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

/// The pack, by name and count.
fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| w.lore().describe(s))
        .collect()
}

fn last(w: &World) -> &Stack {
    w.state
        .player()
        .inventory
        .last()
        .expect("something carried")
}

#[test]
fn the_margin_and_the_extremes_set_the_grade() {
    let settings = Content::builtin()
        .expect("content")
        .recipes()
        .settings
        .clone();
    let grade = |natural, margin| grade(settings.as_ref(), natural, margin);
    assert_eq!(grade(12, -1), Grade::Failed);
    assert_eq!(grade(12, 0), Grade::Plain);
    assert_eq!(grade(12, 5), Grade::Plain);
    assert_eq!(grade(12, 6), Grade::Fine);
    assert_eq!(grade(12, 11), Grade::Fine);
    assert_eq!(grade(12, 12), Grade::Marvellous);
    assert_eq!(grade(12, 19), Grade::Marvellous);
    assert_eq!(grade(12, 20), Grade::Masterwork);
    // A natural 24 makes at least Fine; a natural 2 fails however good the smith.
    assert_eq!(grade(24, -5), Grade::Fine);
    assert_eq!(grade(24, 14), Grade::Marvellous);
    assert_eq!(grade(2, 30), Grade::Failed);
}

#[test]
fn the_odds_are_those_the_design_and_the_wiki_state() {
    // The long sword, 13, steel: at a bonus of 4, 8, 12, 16, failed / plain / Fine / Marvellous
    // / Masterwork in percent (docs/design/recipes.md §4).
    let settings = Content::builtin()
        .expect("content")
        .recipes()
        .settings
        .clone();
    for (bonus, odds) in [
        (4, [19, 42, 31, 7, 0]),
        (8, [4, 27, 44, 25, 0]),
        (12, [1, 10, 35, 47, 7]),
        (16, [1, 0, 19, 56, 25]),
    ] {
        let mut counts = [0u32; 5];
        for a in 1..=12 {
            for b in 1..=12 {
                let natural = a + b;
                let g = grade(settings.as_ref(), natural, natural + bonus - 13);
                counts[g as usize] += 1;
            }
        }
        for (got, want) in counts.iter().zip(odds) {
            let percent = f64::from(*got) * 100.0 / 144.0;
            assert!(
                (percent - f64::from(want)).abs() <= 1.0,
                "+{bonus}: {counts:?}"
            );
        }
    }
}

#[test]
fn each_grade_is_a_worked_item_with_attempts_equal_to_its_improvements() {
    for (margin, improvements, name, words) in [
        (5, 0, "a short sword", "You have made a short sword."),
        (6, 1, "a Fine short sword", "This is a Fine short sword."),
        (
            12,
            2,
            "a Marvellous short sword",
            "This is a Marvellous short sword!",
        ),
        (
            20,
            3,
            "a Masterwork short sword",
            "You have made a Masterwork short sword!",
        ),
    ] {
        let mut w = world(false);
        ingots(&mut w, "steel", 2);
        make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, margin);
        assert_eq!(said(&w), words);
        assert_eq!(pack(&w), [name], "the ingots spent");
        let made = last(&w);
        assert!(made.is_identified());
        match improvements {
            0 => assert_eq!(made.worked(), None),
            n => {
                let worked = made.worked().expect("worked");
                assert_eq!(worked.improved.len(), n);
                assert_eq!(worked.attempts, n as u8);
            }
        }
    }
}

#[test]
fn a_crafted_fine_item_is_a_once_tempered_one_and_a_masterwork_cannot_be_tempered() {
    let mut w = world(false);
    ingots(&mut w, "steel", 4);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, 6);
    let fine = last(&w).worked().expect("worked").attempts;
    assert_eq!(w.content.temper().against(fine), 13 + 2);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 12, 25);
    let at = w.state.player().inventory.len() - 1;
    assert!(!super::temper::workable(&w.content, last(&w)));
    let start = w.state.time;
    w.apply(&Action::Temper { item: at });
    assert_eq!(said(&w), "It can be made no finer.");
    assert_eq!(w.state.time, start, "no time");
}

#[test]
fn a_failure_makes_nothing_and_gives_half_the_stock_back_and_the_gem_whole() {
    let mut w = world(false);
    train(&mut w, 60);
    ingots(&mut w, "steel", 7);
    gem(&mut w, "ruby", "fair");
    make_out_for_test(&mut w, "great_sword", &[Some(0), Some(1)], 12, -1);
    assert_eq!(
        said(&w),
        "The blade comes out brittle and you break it up for the metal."
    );
    // Seven ingots' worth: half is three and a half, four by the piece's rounding.
    assert_eq!(pack(&w), ["4 steel ingots", "a ruby"]);
    // A natural 2 fails as well.
    let mut w = world(false);
    ingots(&mut w, "steel", 2);
    make_out_for_test(&mut w, "short_sword", &[Some(0), None], 2, 30);
    assert_eq!(pack(&w), ["a steel ingot"]);
}

#[test]
fn an_item_with_no_aspect_comes_out_plain_whatever_the_margin() {
    let mut w = world(false);
    ingots(&mut w, "steel", 1);
    make_out_for_test(&mut w, "ring", &[Some(0)], 24, 30);
    assert_eq!(said(&w), "You have made a steel ring.");
    assert_eq!(last(&w).worked(), None);
}

#[test]
fn the_wizard_steps_offer_what_fits_and_the_plan_names_the_item() {
    let mut w = world(false);
    ingots(&mut w, "steel", 1);
    ingots(&mut w, "mithril", 3);
    gem(&mut w, "ruby", "fair");
    gem(&mut w, "emerald", "uncut");
    let wood = w.content.material_id("hardwood").expect("hardwood");
    let billet = crate::piece::make(&w.content, wood, 5).expect("billets");
    w.player_mut().inventory.push(billet);
    let view = w.perception();
    let steps = view.craft_steps(recipe(&w, "long_sword"));
    #[allow(clippy::type_complexity)]
    let shown: Vec<(String, bool, Vec<(usize, u32, bool)>)> = (steps.iter())
        .map(|s| {
            let options = s
                .options
                .iter()
                .map(|o| (o.at, o.takes, o.enough))
                .collect();
            (s.label.clone(), s.optional, options)
        })
        .collect();
    // The blade: the steel (too few) and the mithril, never the wood; the pommel: the cut ruby,
    // never the uncut emerald.
    assert_eq!(
        shown,
        [
            (
                "blade".to_string(),
                false,
                vec![(0, 3, false), (1, 3, true)]
            ),
            ("pommel".to_string(), true, vec![(2, 1, true)]),
        ]
    );
    let plan = view
        .craft_plan(recipe(&w, "long_sword"), &[Some(1), Some(2)])
        .expect("a plan");
    assert_eq!(plan.verb, "Forge");
    assert_eq!(plan.name, "a ruby-pommelled mithril long sword");
    assert_eq!(plan.uses, "3 mithril ingots and the ruby");
    assert_eq!(plan.station.as_deref(), Some("anvil"));
    assert_eq!(plan.time, "about 1 hour");
    // The rules' refusals, as the dialog would hear them.
    let refused = |picks: &[Option<usize>]| view.craft_plan(recipe(&w, "long_sword"), picks);
    assert_eq!(
        refused(&[Some(0)]).err().as_deref(),
        Some("You have too few of those.")
    );
    assert_eq!(
        refused(&[Some(4)]).err().as_deref(),
        Some("That will not do for the blade.")
    );
    assert_eq!(
        refused(&[Some(1), Some(3)]).err().as_deref(),
        Some("Only a cut gem can be set.")
    );
    assert_eq!(
        refused(&[]).err().as_deref(),
        Some("You must choose the blade.")
    );
    assert_eq!(
        refused(&[Some(9)]).err().as_deref(),
        Some("You carry no such thing.")
    );
}

#[test]
fn forging_takes_its_time_spends_the_stock_at_the_end_and_may_mark_metalcraft() {
    let mut w = world(false);
    ingots(&mut w, "steel", 3);
    gem(&mut w, "ruby", "fair");
    let start = w.state.time;
    apply(&mut w, "craft:short_sword:a,b");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    let names = pack(&w);
    assert_eq!(names[0], "a steel ingot");
    assert!(
        names[1].ends_with("ruby-pommelled short sword"),
        "{names:?}"
    );
    assert!(last(&w).is_identified());
    let marks = w
        .state
        .player()
        .skills
        .get(metalcraft(&w))
        .expect("trained")
        .marks;
    assert_eq!(marks, 1, "one mark, at a hundred percent");
}

#[test]
fn a_craft_is_refused_with_no_time_beyond_skill_away_from_the_anvil_or_without_the_stock() {
    let mut w = world(false);
    train(&mut w, 30);
    ingots(&mut w, "steel", 7);
    let start = w.state.time;
    for (text, words) in [
        ("craft:great_sword:a", "That is beyond your skill."),
        ("craft:dagger", "You must choose the metal."),
        ("craft:dagger:a,a", "Only a cut gem can be set."),
        ("craft:knife:a,a", "That is more than the work takes."),
    ] {
        apply(&mut w, text);
        assert_eq!(said(&w), words, "{text}");
        assert_eq!(w.state.time, start, "{text}");
    }
    apply(&mut w, "move:S");
    let start = w.state.time;
    apply(&mut w, "craft:dagger:a");
    assert_eq!(said(&w), "There is no anvil here.");
    assert_eq!(w.state.time, start);
}

#[test]
fn an_interrupted_work_spends_only_time() {
    let mut w = world(true);
    ingots(&mut w, "steel", 2);
    let start = w.state.time;
    apply(&mut w, "craft:short_sword:a");
    assert!(said(&w).ends_with("put down your work."), "{}", said(&w));
    assert!(w.state.time.since(start).as_seconds() < 3600);
    assert_eq!(pack(&w), ["2 steel ingots"]);
}
