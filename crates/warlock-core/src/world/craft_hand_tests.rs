//! The hand-made and metal melee weapons (docs/dev/crafting.md "Routes and the recipe's
//! profession"; #1351): the stock each takes by the base's weight, a recipe's profession being its
//! directory's (the stat, the skill, the mark), a part of another profession's stock (a spear's
//! wooden haft), the knife, skill, and anvil greyings, the spear's steps, and the made items.

use super::craft::RecipeLack;
use super::*;
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::run as apply;

/// A forge with its anvil at (5, 2) and the player beside it at (4, 3).
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@.....#
#.........#
###########
";

/// The built-in content with [`ROOM`]. The player (a craftsperson) carries nothing and has
/// Metalcraft and Woodcraft at 50.
fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_forge.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"anvil\"\n"
        ),
    );
    files.insert(
        "scenarios/test_forge.toml",
        "name = \"T\"\nstart_level = \"test_forge\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_forge", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    train(&mut w, "metalcraft", 50);
    train(&mut w, "woodcraft", 50);
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// `id` at `value`, no marks, the other skills as they were.
fn train(w: &mut World, id: &str, value: u16) {
    let id = skill(w, id);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> u32 {
    (w.state.player().skills.get(skill(w, id))).map_or(0, |t| t.marks)
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

fn said(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

/// What the first step of `id` takes of the first stack carried.
fn takes(w: &World, id: &str) -> u32 {
    let steps = w.perception().craft_steps(recipe(w, id));
    steps[0].options[0].takes
}

/// The row of recipe `id` among its profession's, with its reason if greyed.
fn row_lack(w: &World, id: &str) -> Option<RecipeLack> {
    let def = w.content.recipe(recipe(w, id));
    let rows = w.perception().recipes(def.skill);
    let row = rows.iter().find(|r| r.id == recipe(w, id)).expect("listed");
    row.unavailable
}

#[test]
fn the_stock_follows_the_bases_weight_as_the_design_counts_it() {
    for (id, material, count) in [
        ("knife", "steel", 1),
        // A bronze blade takes as many bronze ingots as a steel one steel (P5's rule).
        ("long_sword", "bronze", 3),
        ("great_sword", "steel", 7),
        ("hand_axe", "iron", 3),
        ("battle_axe", "iron", 5),
        ("great_axe", "iron", 10),
        ("mace", "iron", 5),
        ("morning_star", "iron", 6),
        ("war_hammer", "iron", 5),
        ("maul", "iron", 12),
        ("trident", "iron", 4),
        ("glaive", "iron", 10),
        ("halberd", "iron", 11),
        ("stiletto", "steel", 1),
        ("spear", "steel", 1),
        ("javelin", "iron", 1),
        ("knife_by_hand", "flint", 1),
        ("knife_by_hand", "bone", 1),
        ("dagger_by_hand", "obsidian", 1),
        ("hand_axe_by_hand", "flint", 3),
        ("club", "wood", 7),
        ("club", "bone", 7),
        ("quarterstaff", "wood", 10),
        ("quarterstaff", "hardwood", 10),
        ("ironwood_staff", "ironwood", 7),
        ("spear_by_hand", "bone", 1),
        ("javelin_by_hand", "flint", 1),
    ] {
        let mut w = world();
        stock(&mut w, material, 20);
        assert_eq!(takes(&w, id), count, "{id} of {material}");
    }
    // The haft: seven billets for a spear, four for a javelin, whatever the head.
    for (id, count) in [
        ("spear", 7),
        ("spear_by_hand", 7),
        ("javelin", 4),
        ("javelin_by_hand", 4),
    ] {
        let mut w = world();
        stock(&mut w, "wood", 20);
        let steps = w.perception().craft_steps(recipe(&w, id));
        assert_eq!(steps[1].label, "haft");
        assert_eq!(steps[1].options[0].takes, count, "{id}");
    }
}

#[test]
fn the_spear_steps_are_the_head_then_the_haft_of_another_professions_stock() {
    let mut w = world();
    stock(&mut w, "iron", 2);
    stock(&mut w, "wood", 7);
    stock(&mut w, "flint", 1);
    let steps = w.perception().craft_steps(recipe(&w, "spear"));
    let shown: Vec<(&str, Vec<usize>)> = (steps.iter())
        .map(|s| (s.label.as_str(), s.options.iter().map(|o| o.at).collect()))
        .collect();
    // The smith's head takes the iron, never the flint; the haft the wood, Woodcraft's stock on
    // a Metalcraft recipe; then the inlay, none carried.
    assert_eq!(
        shown,
        [("head", vec![0]), ("haft", vec![1]), ("inlay", vec![])]
    );
    let steps = w.perception().craft_steps(recipe(&w, "spear_by_hand"));
    let shown: Vec<(&str, Vec<usize>)> = (steps.iter())
        .map(|s| (s.label.as_str(), s.options.iter().map(|o| o.at).collect()))
        .collect();
    assert_eq!(shown, [("head", vec![2]), ("haft", vec![1])]);
}

#[test]
fn a_metal_spear_needs_billets_greyed_and_refused_with_no_time_without_them() {
    let mut w = world();
    stock(&mut w, "iron", 2);
    let lack = row_lack(&w, "spear").expect("greyed");
    assert_eq!(lack.reason(&w.content), "no wood to hand");
    let start = w.state.time;
    apply(&mut w, "craft:spear:a");
    assert_eq!(said(&w), "You must choose the haft.");
    assert_eq!(w.state.time, start);
    // The iron offered for the haft will not do.
    apply(&mut w, "craft:spear:a,a");
    assert_eq!(said(&w), "That will not do for the haft.");
    assert_eq!(w.state.time, start);
}

#[test]
fn hand_recipes_grey_without_a_knife_and_beyond_skill_and_the_forge_away_from_the_anvil() {
    let mut w = world();
    stock(&mut w, "flint", 3);
    stock(&mut w, "iron", 3);
    assert_eq!(row_lack(&w, "knife_by_hand"), Some(RecipeLack::Knife));
    let start = w.state.time;
    apply(&mut w, "craft:knife_by_hand:a");
    assert_eq!(said(&w), "You have no knife.");
    assert_eq!(w.state.time, start);
    carry(&mut w, "knife");
    assert_eq!(row_lack(&w, "knife_by_hand"), None);
    // Ironwood is listed from Woodcraft 30.
    train(&mut w, "woodcraft", 20);
    assert_eq!(row_lack(&w, "ironwood_staff"), Some(RecipeLack::Skill));
    // Away from the anvil the smith's axe is greyed; the knife in hand, the flint axe is not.
    apply(&mut w, "move:S move:S");
    assert!(matches!(
        row_lack(&w, "hand_axe"),
        Some(RecipeLack::Station(_))
    ));
    assert_eq!(row_lack(&w, "hand_axe_by_hand"), None);
}

#[test]
fn a_recipes_profession_is_its_directorys_for_the_stat_the_skill_and_the_mark() {
    let mut w = world();
    let content = w.content.clone();
    let me = w.state.player();
    let stats = me.effective_stats(&content);
    let woodcraft = content.recipe(recipe(&w, "knife_by_hand"));
    let metalcraft = content.recipe(recipe(&w, "knife"));
    let flint = content.material_id("flint");
    // A flint knife by Woodcraft's Dexterity, a steel one by Metalcraft's Strength; both at 50
    // give the same craft points.
    let craft = me.skill_effect_of(&content, woodcraft.skill, crate::skills::Effect::Craft);
    assert_eq!(
        w.craft_modifier(woodcraft, flint),
        stats.modifier(crate::stats::Stat::Dexterity) + craft
    );
    assert_eq!(
        w.craft_modifier(metalcraft, content.material_id("steel")),
        stats.modifier(crate::stats::Stat::Strength) + craft
    );
    // Made: the flint knife marks Woodcraft alone, the verb the woodworker's.
    stock(&mut w, "flint", 1);
    carry(&mut w, "knife");
    let plan = (w.perception())
        .craft_plan(recipe(&w, "knife_by_hand"), &[Some(0)])
        .expect("a plan");
    assert_eq!(plan.verb, "Make");
    assert_eq!(plan.name, "a flint knife");
    assert_eq!(plan.station, None);
    apply(&mut w, "craft:knife_by_hand:a");
    assert_eq!(marks(&w, "woodcraft"), 1);
    assert_eq!(marks(&w, "metalcraft"), 0);
    // The steel knife marks Metalcraft.
    stock(&mut w, "steel", 1);
    let at = w.state.player().inventory.len() - 1;
    apply(
        &mut w,
        &format!("craft:knife:{}", (b'a' + at as u8) as char),
    );
    assert_eq!(marks(&w, "metalcraft"), 1);
    assert_eq!(marks(&w, "woodcraft"), 1);
}

#[test]
fn the_made_spear_and_javelin_are_named_by_their_heads_and_the_javelin_still_flies() {
    let mut w = world();
    train(&mut w, "woodcraft", 200);
    carry(&mut w, "knife");
    stock(&mut w, "bone", 1);
    stock(&mut w, "wood", 4);
    let plan = (w.perception())
        .craft_plan(recipe(&w, "javelin_by_hand"), &[Some(1), Some(2)])
        .expect("a plan");
    assert_eq!(plan.name, "a bone javelin");
    assert_eq!(plan.uses, "a bone shard and 4 wood billets");
    let start = w.state.time;
    apply(&mut w, "craft:javelin_by_hand:b,c");
    assert_eq!(w.state.time.since(start).as_seconds(), 1200);
    let made = (w.state.player().inventory.iter())
        .find(|s| s.object == w.content.object_id("javelin").expect("javelin"))
        .expect("a javelin made");
    assert!(
        w.lore().describe(made).ends_with("bone javelin"),
        "{}",
        said(&w)
    );
    assert_eq!(w.content.object(made.object).range, Some(8));
    // The stock is spent: only the knife is left besides it.
    assert_eq!(w.state.player().inventory.len(), 2);
}
