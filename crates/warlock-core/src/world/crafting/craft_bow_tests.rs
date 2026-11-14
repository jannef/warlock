//! The bows, crossbows, the sling, the lute, and the pipe (docs/dev/crafting.md "Stock inputs";
//! #1352): the stock each takes by the base's weight, the stock inputs (a bow's sinew, a
//! crossbow's ingot of any metal) as wizard steps, spent and never a part of what is made, their
//! greyings and refusals, a wooden composite bow named from its parts, and a lute that comes out
//! plain whatever the margin.

use super::craft::RecipeLack;
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::{builtin, last_text as said, run as apply};
use crate::world::*;

/// The built-in content on a bare floor. The player (a craftsperson) carries nothing and has
/// Woodcraft at 200.
fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_bench.toml",
        "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n\
         #######\n#.....#\n#..@..#\n#.....#\n#######\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/test_bench.toml",
        "name = \"T\"\nstart_level = \"test_bench\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_bench", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    let woodcraft = skill(&w, "woodcraft");
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != woodcraft)
        .collect();
    all.push((
        woodcraft,
        Trained {
            value: 200,
            marks: 0,
        },
    ));
    w.player_mut().skills = all.into_iter().collect();
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// `count` pieces of `material` at the end of the pack.
fn stock(w: &mut World, material: &str, count: u32) {
    let m = w.content.material_id(material).expect(material);
    let stack = crate::piece::make(&w.content, m, count).expect("stock");
    w.player_mut().inventory.push(stack);
}

/// One `object` at the end of the pack.
fn carry(w: &mut World, object: &str) {
    let id = w.content.object_id(object).expect(object);
    w.player_mut().inventory.push(Stack::one(id));
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

/// The row of recipe `id`: what it takes and why it is greyed, if it is.
fn row(w: &World, id: &str) -> (String, Option<RecipeLack>) {
    let def = w.content.recipe(recipe(w, id));
    let rows = w.perception().recipes(def.skill);
    let row = rows.iter().find(|r| r.id == recipe(w, id)).expect("listed");
    (row.takes.clone(), row.unavailable)
}

/// Each step's label and the positions it offers.
fn shown(w: &World, id: &str) -> Vec<(String, Vec<usize>)> {
    (w.perception().craft_steps(recipe(w, id)).iter())
        .map(|s| (s.label.clone(), s.options.iter().map(|o| o.at).collect()))
        .collect()
}

#[test]
fn the_stock_follows_the_bases_weight_as_the_design_counts_it() {
    for (id, material, count) in [
        ("short_bow", "wood", 4),
        ("long_bow", "wood", 7),
        ("composite_bow", "horn", 5),
        ("composite_bow", "wood", 5),
        ("crossbow", "wood", 16),
        ("heavy_crossbow", "wood", 23),
        ("sling", "leather", 1),
        ("lute", "wood", 8),
        ("pipe", "wood", 1),
    ] {
        let mut w = world();
        stock(&mut w, material, 40);
        let steps = w.perception().craft_steps(recipe(&w, id));
        assert_eq!(steps[0].options[0].takes, count, "{id} of {material}");
    }
}

#[test]
fn the_rows_tell_the_stock_inputs() {
    let w = world();
    assert_eq!(
        row(&w, "short_bow").0,
        "4 wood billets or 4 horn shards, a length of sinew, a knife, about 1 hour"
    );
    assert_eq!(
        row(&w, "composite_bow").0,
        "5 wood billets or 5 horn shards, 2 lengths of sinew, a knife, about 1 hour and 30 minutes"
    );
    assert_eq!(
        row(&w, "crossbow").0,
        "16 wood billets, an ingot of any metal, a knife, about 1 hour and 30 minutes"
    );
}

#[test]
fn the_wizard_offers_the_sinew_and_every_metal_ingot_after_the_stock() {
    let mut w = world();
    stock(&mut w, "wood", 20);
    stock(&mut w, "sinew", 3);
    stock(&mut w, "iron", 1);
    stock(&mut w, "bronze", 2);
    assert_eq!(
        shown(&w, "short_bow"),
        [("material".to_string(), vec![0]), ("sinew".into(), vec![1])]
    );
    // The lock and trigger: either ingot, the wizard asks which.
    assert_eq!(
        shown(&w, "crossbow"),
        [("wood".to_string(), vec![0]), ("metal".into(), vec![2, 3])]
    );
    // The composite bow's two lengths: a stack of one is shown, too few.
    let mut w = world();
    stock(&mut w, "horn", 5);
    stock(&mut w, "sinew", 1);
    let steps = w.perception().craft_steps(recipe(&w, "composite_bow"));
    assert_eq!(steps[1].options[0].takes, 2);
    assert!(!steps[1].options[0].enough);
}

#[test]
fn without_sinew_or_metal_the_rows_grey_and_the_work_is_refused_with_no_time() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "wood", 20);
    let (_, lack) = row(&w, "short_bow");
    assert_eq!(lack.expect("greyed").reason(&w.content), "no sinew to hand");
    let (_, lack) = row(&w, "crossbow");
    assert_eq!(lack.expect("greyed").reason(&w.content), "no metal to hand");
    let start = w.state.time;
    apply(&mut w, "craft:short_bow:b");
    assert_eq!(said(&w), "You must choose the sinew.");
    apply(&mut w, "craft:crossbow:b,b");
    assert_eq!(said(&w), "That will not do for the metal.");
    assert_eq!(w.state.time, start);
    // One length will not string a composite bow.
    stock(&mut w, "sinew", 1);
    let (_, lack) = row(&w, "composite_bow");
    assert_eq!(lack.expect("greyed").reason(&w.content), "no sinew to hand");
    apply(&mut w, "craft:composite_bow:b,c");
    assert_eq!(said(&w), "You have too few of those.");
    assert_eq!(w.state.time, start);
    // The sling and the pipe take no input.
    assert_eq!(row(&w, "pipe").1, None);
}

#[test]
fn a_long_bow_is_made_of_the_wood_its_sinew_spent_and_no_part_of_it() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "wood", 7);
    stock(&mut w, "sinew", 2);
    let plan = (w.perception())
        .craft_plan(recipe(&w, "long_bow"), &[Some(1), Some(2)])
        .expect("a plan");
    assert_eq!(plan.verb, "Make");
    assert_eq!(plan.uses, "7 wood billets and a length of sinew");
    let start = w.state.time;
    apply(&mut w, "craft:long_bow:b,c");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    let content = w.content.clone();
    let pack = &w.state.player().inventory;
    let bow = (pack.iter())
        .find(|s| s.object == content.object_id("long_bow").expect("long_bow"))
        .expect("a long bow made");
    let sinew = content.material_id("sinew").expect("sinew");
    assert!(bow.parts(&content).iter().all(|p| p.material != sinew));
    // The knife, the bow, and the one length of sinew left.
    assert_eq!(pack.len(), 3);
    assert_eq!(pack[1].count, 1);
}

#[test]
fn a_crossbow_spends_the_ingot_chosen() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "wood", 16);
    stock(&mut w, "iron", 1);
    stock(&mut w, "bronze", 1);
    apply(&mut w, "craft:crossbow:b,d");
    let content = w.content.clone();
    let pack = &w.state.player().inventory;
    let crossbow = content.object_id("crossbow").expect("crossbow");
    assert!(pack.iter().any(|s| s.object == crossbow), "{}", said(&w));
    let iron = content.material_id("iron");
    let left: Vec<_> = (pack.iter())
        .filter_map(|s| super::craft::stack_material(&content, s))
        .collect();
    assert_eq!(left, iron.into_iter().collect::<Vec<_>>());
}

#[test]
fn a_composite_bow_of_wood_is_a_variant_named_from_its_parts() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "horn", 5);
    stock(&mut w, "wood", 5);
    stock(&mut w, "sinew", 2);
    let name = |w: &World, at: usize| {
        (w.perception())
            .craft_plan(recipe(w, "composite_bow"), &[Some(at), Some(3)])
            .expect("a plan")
            .name
    };
    assert_eq!(name(&w, 1), "a horn composite bow");
    assert_eq!(name(&w, 2), "a composite bow");
}

#[test]
fn a_lute_comes_out_plain_whatever_the_margin() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "wood", 8);
    stock(&mut w, "sinew", 1);
    super::craft_make::make_out_for_test(&mut w, "lute", &[Some(1), Some(2)], 12, 20);
    assert_eq!(said(&w), "You have made a lute.");
    let lute = w.content.object_id("lute").expect("lute");
    let made = (w.state.player().inventory.iter())
        .find(|s| s.object == lute)
        .expect("a lute");
    assert_eq!(made.worked(), None);
    // The bow at the same margin is a Masterwork.
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "wood", 4);
    stock(&mut w, "sinew", 1);
    super::craft_make::make_out_for_test(&mut w, "short_bow", &[Some(1), Some(2)], 12, 20);
    assert!(
        said(&w).starts_with("You have made a Masterwork"),
        "{}",
        said(&w)
    );
}

#[test]
fn a_short_bow_made_at_the_forge_replays() {
    // The forge's craftsperson picks up the stock (wood `c` and sinew `d`, joining his trade's) and the
    // bench's dagger.
    let content = builtin();
    let mut w = World::new(content.clone(), "forge", None).expect("world");
    let mut replay = crate::replay::Replay::new(&w);
    let actions = "move:E pickup move:S move:S pickup craft:short_bow:c,d";
    for action in crate::action::parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let bow = content.object_id("short_bow").expect("short_bow");
    assert!(
        (w.state.player().inventory.iter()).any(|s| s.object == bow),
        "{}",
        said(&w)
    );
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
