//! Armoursmithing and the metal tools (docs/dev/crafting.md "Armour and tools"; #1354): the
//! stock each takes by the base's weight (mail by weight: chain 22 ingots, plate 44), the anvil,
//! the stone pick by Woodcraft with a knife and no anvil, plate beyond skill below 60, a graded
//! suit against a plain lantern, and salvage giving half back.

use super::craft::RecipeLack;
use super::craft_make::make_out_for_test;
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::{last_text as said, run as apply};
use crate::world::*;

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
/// Metalcraft and Woodcraft at 60.
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
    train(&mut w, "metalcraft", 60);
    train(&mut w, "woodcraft", 60);
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

fn carry(w: &mut World, object: &str) {
    let id = w.content.object_id(object).expect(object);
    w.player_mut().inventory.push(Stack::one(id));
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

fn last(w: &World) -> &Stack {
    w.state.player().inventory.last().expect("a stack")
}

/// The row of recipe `id` among its profession's: what it takes and its reason if greyed.
fn row(w: &World, id: &str) -> (String, Option<RecipeLack>) {
    let def = w.content.recipe(recipe(w, id));
    let rows = w.perception().recipes(def.skill);
    let row = rows.iter().find(|r| r.id == recipe(w, id)).expect("listed");
    (row.takes.clone(), row.unavailable)
}

#[test]
fn the_stock_follows_the_bases_weight_as_the_design_counts_it() {
    for (id, material, count) in [
        // Mail by weight: a chain shirt is 11 kg, 22 ingots; plate 22 kg, 44.
        ("chain_mail", "steel", 22),
        ("chain_mail", "iron", 22),
        // A mithril or bronze suit takes as many of its own ingots (a piece weighs by its metal).
        ("chain_mail", "mithril", 22),
        ("scale_mail", "bronze", 28),
        ("banded_mail", "steel", 34),
        ("plate_armour", "steel", 44),
        ("buckler", "iron", 3),
        // A wooden shield's weight in iron.
        ("small_shield", "iron", 13),
        ("large_shield", "steel", 30),
        ("tower_shield", "iron", 24),
        // A cap of leather in metal: a skull cap.
        ("leather_cap", "steel", 3),
        ("iron_helm", "iron", 3),
        ("great_helm", "steel", 6),
        ("gauntlets", "iron", 2),
        ("iron_shod_boots", "iron", 4),
        ("lantern", "iron", 1),
        ("lantern", "brass", 1),
        ("pick_axe", "iron", 6),
        ("mattock", "iron", 9),
        ("pick_axe_by_hand", "flint", 6),
        ("mattock_by_hand", "stone", 9),
    ] {
        let mut w = world();
        stock(&mut w, material, 60);
        let steps = w.perception().craft_steps(recipe(&w, id));
        assert_eq!(steps[0].options.len(), 1, "{id} offers the {material}");
        assert_eq!(steps[0].options[0].takes, count, "{id} of {material}");
    }
}

#[test]
fn the_rows_price_in_the_bases_metal_or_steel_never_the_first_metal_on_the_list() {
    let w = world();
    for (id, words) in [
        ("chain_mail", "22 iron ingots"),
        ("small_shield", "13 steel ingots"),
        ("leather_cap", "3 steel ingots"),
        ("pick_axe_by_hand", "6 blocks of flint"),
    ] {
        let (takes, _) = row(&w, id);
        assert!(takes.starts_with(words), "{id}: {takes}");
    }
}

#[test]
fn a_lantern_is_of_iron_or_brass_only() {
    let mut w = world();
    stock(&mut w, "steel", 1);
    stock(&mut w, "brass", 1);
    let steps = w.perception().craft_steps(recipe(&w, "lantern"));
    let offered: Vec<usize> = steps[0].options.iter().map(|o| o.at).collect();
    assert_eq!(offered, [1]);
}

#[test]
fn the_smiths_work_needs_the_anvil_and_the_stone_pick_a_knife_and_no_anvil() {
    let mut w = world();
    stock(&mut w, "iron", 30);
    stock(&mut w, "flint", 6);
    apply(&mut w, "move:S move:S");
    assert!(matches!(
        row(&w, "chain_mail").1,
        Some(RecipeLack::Station(_))
    ));
    let start = w.state.time;
    apply(&mut w, "craft:chain_mail:a");
    assert_eq!(said(&w), "There is no anvil here.");
    assert_eq!(w.state.time, start, "no time");
    // The stone pick: greyed without a knife, made away from the anvil with one.
    assert_eq!(row(&w, "pick_axe_by_hand").1, Some(RecipeLack::Knife));
    carry(&mut w, "knife");
    assert_eq!(row(&w, "pick_axe_by_hand").1, None);
    let start = w.state.time;
    apply(&mut w, "craft:pick_axe_by_hand:b");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    assert_eq!(marks(&w, "woodcraft"), 1);
    assert_eq!(marks(&w, "metalcraft"), 0);
}

#[test]
fn plate_is_beyond_skill_below_60() {
    let mut w = world();
    stock(&mut w, "steel", 44);
    assert_eq!(row(&w, "plate_armour").1, None);
    train(&mut w, "metalcraft", 59);
    assert_eq!(row(&w, "plate_armour").1, Some(RecipeLack::Skill));
    let start = w.state.time;
    apply(&mut w, "craft:plate_armour:a");
    assert_eq!(said(&w), "That is beyond your skill.");
    assert_eq!(w.state.time, start, "no time");
}

#[test]
fn a_suit_is_graded_and_a_lantern_comes_out_plain() {
    let mut w = world();
    stock(&mut w, "steel", 22);
    make_out_for_test(&mut w, "chain_mail", &[Some(0)], 12, 20);
    assert_eq!(
        said(&w),
        "You have made a Masterwork steel chain mail shirt!"
    );
    let worked = last(&w).worked().expect("worked");
    assert_eq!(worked.improved.len(), 3);
    // Mail gives Defence and Protection, so its improvements are drawn among those.
    assert!((worked.improved.iter()).all(|a| matches!(
        a,
        crate::magic::Aspect::Defence | crate::magic::Aspect::Protection
    )));
    stock(&mut w, "iron", 1);
    let at = w.state.player().inventory.len() - 1;
    make_out_for_test(&mut w, "lantern", &[Some(at)], 24, 30);
    assert_eq!(said(&w), "You have made a lantern.");
    assert_eq!(last(&w).worked(), None);
    // A pick-axe has no attack: plain too.
    stock(&mut w, "iron", 6);
    let at = w.state.player().inventory.len() - 1;
    make_out_for_test(&mut w, "pick_axe", &[Some(at)], 24, 30);
    assert_eq!(last(&w).worked(), None);
}

#[test]
fn a_helm_forged_at_the_anvil_takes_its_hour_and_marks_metalcraft() {
    let mut w = world();
    stock(&mut w, "iron", 3);
    let start = w.state.time;
    apply(&mut w, "craft:iron_helm:a,-");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    assert_eq!(marks(&w, "metalcraft"), 1);
    // Made or failed (half the iron back), the stock is spent at the end.
    let piece = w.content.object_id(crate::piece::ID).expect("piece");
    let left = (w.state.player().inventory.iter()).find(|s| s.object == piece);
    assert!(left.is_none_or(|s| s.count < 3), "{}", said(&w));
}

#[test]
fn salvaging_a_chain_shirt_gives_half_its_ingots_back() {
    let w = world();
    let mail = Stack::one(w.content.object_id("chain_mail").expect("chain_mail"));
    let falls = crate::piece::break_up(&mail, &w.content, 50);
    assert_eq!(falls.len(), 1);
    assert_eq!(falls[0].count, 11);
}
