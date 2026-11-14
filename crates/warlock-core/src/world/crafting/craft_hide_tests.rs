//! Hide gear by hand (docs/dev/crafting.md "Hide gear"; #1353): the shields, armour, helmets,
//! cloaks, gloves, boots, and clothing Woodcraft makes with a knife, the stock each takes by the
//! base's weight (hides by the group's `unit`), the grades for gear with an aspect and plain
//! clothing, the greyings, and the ingot a stock input takes.

use super::craft::RecipeLack;
use super::craft_make::{make_out_for_test, stock_need};
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::skills::{SkillId, Trained};
use crate::test_support::{last_text as said, run as apply};
use crate::world::*;

/// The built-in content with a bare room. The player (a craftsperson) carries nothing and has
/// Woodcraft at 50.
fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_hides.toml",
        "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n\
         #######\n#.....#\n#..@..#\n#.....#\n#######\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/test_hides.toml",
        "name = \"T\"\nstart_level = \"test_hides\"\n\
         character = { race = \"human\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_hides", None).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    train(&mut w, 50);
    w
}

fn woodcraft(w: &World) -> SkillId {
    w.content.skill_id("woodcraft").expect("woodcraft")
}

/// Woodcraft at `value`, no marks, the other skills as they were.
fn train(w: &mut World, value: u16) {
    let id = woodcraft(w);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
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

fn row_lack(w: &World, id: &str) -> Option<RecipeLack> {
    let def = w.content.recipe(recipe(w, id));
    let rows = w.perception().recipes(def.skill);
    let row = rows.iter().find(|r| r.id == recipe(w, id)).expect("listed");
    row.unavailable
}

/// The pieces of `material` the main part of `id` takes.
fn need(w: &World, id: &str, material: &str) -> u32 {
    let def = w.content.recipe(recipe(w, id));
    let m = w.content.material_id(material).expect(material);
    stock_need(&w.content, def, 0, m, 1).expect("priced")
}

fn last(w: &World) -> &Stack {
    w.state
        .player()
        .inventory
        .last()
        .expect("something carried")
}

#[test]
fn the_stock_follows_the_bases_weight_as_the_design_counts_it() {
    for (id, material, count) in [
        ("buckler_by_hand", "wood", 3),
        ("buckler_by_hand", "leather", 1),
        ("buckler_by_hand", "chitin", 3),
        ("small_shield_by_hand", "wood", 13),
        ("small_shield_by_hand", "leather", 2),
        // Chitin is lighter than wood: by weight 12 shards, where the design's table wrote 13.
        ("small_shield_by_hand", "chitin", 12),
        ("large_shield_by_hand", "wood", 30),
        ("large_shield_by_hand", "leather", 4),
        ("tower_shield_by_hand", "wood", 24),
        ("tower_shield_by_hand", "leather", 3),
        ("leather_jerkin", "leather", 3),
        ("studded_leather", "leather", 5),
        ("scale_mail_by_hand", "bone", 28),
        ("scale_mail_by_hand", "dragonscale", 4),
        ("leather_cap_by_hand", "leather", 1),
        ("leather_cap_by_hand", "bone", 3),
        ("iron_helm_by_hand", "leather", 1),
        ("iron_helm_by_hand", "bone", 3),
        ("great_helm_by_hand", "leather", 1),
        ("great_helm_by_hand", "bone", 6),
        ("fur_cloak", "fur", 2),
        ("travelling_cloak", "leather", 1),
        ("leather_gloves", "leather", 1),
        ("leather_boots", "leather", 1),
        ("hide_tunic", "leather", 1),
        ("leather_apron", "leather", 1),
        ("sandals", "leather", 1),
        ("shoes", "leather", 1),
        ("turnshoes", "leather", 1),
        ("soft_boots", "leather", 1),
        ("hobnailed_boots", "leather", 1),
        ("clogs", "wood", 5),
    ] {
        let w = world();
        assert_eq!(need(&w, id, material), count, "{id} of {material}");
    }
    // The step offers the same count of what is carried: a hide is 1.4 kg (eight pieces' worth).
    let mut w = world();
    stock(&mut w, "leather", 5);
    let steps = w.perception().craft_steps(recipe(&w, "leather_jerkin"));
    assert_eq!(steps[0].options[0].takes, 3);
    let hide = &w.state.player().inventory[0];
    assert_eq!(hide.weight(&w.content) / 5, 1400);
}

#[test]
fn a_leather_jerkin_is_made_of_three_hides_with_a_knife_in_an_hour() {
    let mut w = world();
    train(&mut w, 200);
    carry(&mut w, "knife");
    stock(&mut w, "leather", 4);
    let plan = (w.perception())
        .craft_plan(recipe(&w, "leather_jerkin"), &[Some(1)])
        .expect("a plan");
    assert_eq!(plan.verb, "Make");
    assert_eq!(plan.uses, "3 leather hides");
    let start = w.state.time;
    apply(&mut w, "craft:leather_jerkin:b");
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    // Woodcraft 200 makes it (this seed's roll is no natural 2): one hide left, the knife kept,
    // and the jerkin of leather in the pack, identified.
    let jerkin = w.content.object_id("leather_jerkin").expect("jerkin");
    let pack = &w.state.player().inventory;
    let objects: Vec<_> = pack.iter().map(|s| (s.object, s.count)).collect();
    let knife = w.content.object_id("knife").expect("knife");
    let piece = w.content.object_id(crate::piece::ID).expect("piece");
    assert_eq!(
        objects,
        [(knife, 1), (piece, 1), (jerkin, 1)],
        "{}",
        said(&w)
    );
    assert!(pack[2].is_identified());
    let marks = (w.state.player().skills.get(woodcraft(&w))).map_or(0, |t| t.marks);
    assert!(marks <= 1);
}

#[test]
fn a_fur_cloak_is_of_pelts_and_a_travelling_cloak_of_a_hide() {
    let mut w = world();
    carry(&mut w, "knife");
    stock(&mut w, "fur", 2);
    stock(&mut w, "leather", 1);
    let fur = (w.perception())
        .craft_plan(recipe(&w, "fur_cloak"), &[Some(1)])
        .expect("a plan");
    assert_eq!(fur.uses, "2 fur pelts");
    assert_eq!(fur.name, "a fur cloak");
    let leather = (w.perception())
        .craft_plan(recipe(&w, "travelling_cloak"), &[Some(2)])
        .expect("a plan");
    assert_eq!(leather.uses, "a leather hide");
    assert_eq!(leather.name, "a travelling cloak");
}

#[test]
fn gear_with_an_aspect_is_graded_and_clothing_comes_out_plain() {
    let mut w = world();
    stock(&mut w, "leather", 3);
    make_out_for_test(&mut w, "leather_jerkin", &[Some(0)], 12, 20);
    let worked = last(&w).worked().expect("a Masterwork");
    assert_eq!(worked.improved.len(), 3);
    for id in ["shoes", "hide_tunic"] {
        let mut w = world();
        stock(&mut w, "leather", 1);
        make_out_for_test(&mut w, id, &[Some(0)], 24, 30);
        assert!(said(&w).starts_with("You have made a"), "{}", said(&w));
        assert_eq!(last(&w).worked(), None, "{id}");
    }
    let mut w = world();
    stock(&mut w, "wood", 5);
    make_out_for_test(&mut w, "clogs", &[Some(0)], 24, 30);
    assert_eq!(said(&w), "You have made a pair of clogs.");
    assert_eq!(last(&w).worked(), None);
}

#[test]
fn hide_gear_greys_without_a_knife_and_beyond_skill() {
    let mut w = world();
    stock(&mut w, "leather", 4);
    assert_eq!(row_lack(&w, "leather_jerkin"), Some(RecipeLack::Knife));
    let start = w.state.time;
    apply(&mut w, "craft:leather_jerkin:a");
    assert_eq!(said(&w), "You have no knife.");
    assert_eq!(w.state.time, start);
    carry(&mut w, "knife");
    assert_eq!(row_lack(&w, "leather_jerkin"), None);
    // The large shield is listed from Woodcraft 20, the tower shield and scale from 30.
    train(&mut w, 20);
    assert_eq!(row_lack(&w, "large_shield_by_hand"), None);
    assert_eq!(
        row_lack(&w, "tower_shield_by_hand"),
        Some(RecipeLack::Skill)
    );
    train(&mut w, 19);
    assert_eq!(
        row_lack(&w, "large_shield_by_hand"),
        Some(RecipeLack::Skill)
    );
}

#[test]
fn studded_leather_and_hobnailed_boots_are_made_with_an_iron_ingot() {
    // Studded leather's studs and hobnailed boots' nails are a stock input, an ingot of any
    // metal: the wizard's input step takes the iron ingot (`input_fits`) and spends it.
    for (id, hides) in [("studded_leather", 5), ("hobnailed_boots", 1)] {
        let mut w = world();
        train(&mut w, 200);
        carry(&mut w, "knife");
        stock(&mut w, "leather", hides);
        stock(&mut w, "iron", 1);
        let def = w.content.recipe(recipe(&w, id));
        let iron = &w.state.player().inventory[2];
        assert!(
            super::craft::input_fits(w.lore(), &def.inputs[0], iron),
            "{id}"
        );
        assert_eq!(row_lack(&w, id), None, "{id}");
        apply(&mut w, &format!("craft:{id}:b,c"));
        let made = w.content.object_id(id).expect(id);
        let pack = &w.state.player().inventory;
        let objects: Vec<_> = pack.iter().map(|s| s.object).collect();
        let knife = w.content.object_id("knife").expect("knife");
        // The hides and the ingot are spent: the knife and what was made are left.
        assert_eq!(objects, [knife, made], "{id}: {}", said(&w));
    }
}

#[test]
fn each_hide_recipe_is_woodcrafts_by_hand_as_the_design_tables_give_it() {
    let w = world();
    for (id, difficulty, seconds, from, ornament) in [
        ("buckler_by_hand", 10, 3600, 0, false),
        ("small_shield_by_hand", 10, 3600, 0, true),
        ("large_shield_by_hand", 12, 5400, 20, true),
        ("tower_shield_by_hand", 14, 7200, 30, false),
        ("leather_jerkin", 11, 3600, 0, false),
        ("studded_leather", 13, 5400, 20, false),
        ("scale_mail_by_hand", 14, 7200, 30, false),
        ("leather_cap_by_hand", 9, 1800, 0, false),
        ("iron_helm_by_hand", 11, 3600, 10, false),
        ("great_helm_by_hand", 13, 5400, 30, false),
        ("fur_cloak", 10, 3600, 0, false),
        ("travelling_cloak", 10, 3600, 0, false),
        ("leather_gloves", 9, 1200, 0, false),
        ("leather_boots", 9, 1800, 0, false),
        ("hide_tunic", 8, 1800, 0, false),
        ("leather_apron", 8, 1800, 0, false),
        ("sandals", 7, 1200, 0, false),
        ("shoes", 7, 1200, 0, false),
        ("turnshoes", 7, 1200, 0, false),
        ("soft_boots", 7, 1200, 0, false),
        ("hobnailed_boots", 7, 1200, 0, false),
        ("clogs", 7, 1200, 0, false),
    ] {
        let r = w.content.recipe(recipe(&w, id));
        let got = (r.difficulty, r.seconds, r.from, r.ornament);
        assert_eq!(got, (difficulty, seconds, from, ornament), "{id}");
        assert_eq!(r.skill, woodcraft(&w), "{id}");
        assert!(r.knife && r.station.is_none(), "{id}");
        assert_eq!(r.parts.len(), 1, "{id}");
        // Only the studs and the hobnails are an input: one ingot of any metal.
        let ingot = matches!(id, "studded_leather" | "hobnailed_boots");
        assert_eq!(r.inputs.len(), usize::from(ingot), "{id}");
        assert!(
            r.inputs
                .iter()
                .all(|i| i.count == 1 && i.object().is_none())
        );
    }
}
