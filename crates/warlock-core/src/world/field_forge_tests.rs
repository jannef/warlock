//! Field forge (spec §8 "Class abilities"; the craftsperson design §7.7; `world/field_forge.rs`):
//! the craftsperson's level-32 active sets an ingot on a heap of stones beside them in an hour,
//! a makeshift anvil for good, which serves every recipe at the anvil, Temper, and the walk-in,
//! and which a pick clears to floor with nothing mined out of it.

use super::*;
use crate::ability::{Ability, Aim};
use crate::action::parse_actions;
use crate::beside::{Beside, Target};
use crate::content::{ClassActive, ContentFiles, Offer, RecipeUse};
use crate::object::Stack;
use crate::test_support::run;

const FORGE: ClassActive = ClassActive::FieldForge;

/// A lit hall: the player at (2, 2), a door at (2, 1), a kobold's place at (6, 2) when asked
/// for, and floor about.
const HALL: &str = "\
#########
##+######
#.@.....#
#.......#
#########
";

/// The hall's player, a craftsperson trained in Metalcraft and Mining, with Field forge (when
/// `talent`) and nothing carried; a kobold at (6, 2) with `kobold`.
fn world(talent: bool, kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    let map = HALL.replacen("#.@.....#", "#.@...k.#", 1);
    files.insert(
        "levels/test_hall.toml",
        &format!(
            "name = \"Test\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"+\" = \"door_closed\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_hall.toml",
        "name = \"T\"\nstart_level = \"test_hall\"\n\
         character = { race = \"human\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_hall", None).expect("world");
    let metal = w.content.skill_id("metalcraft").expect("metalcraft");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    if me.skills.get(metal).is_none() {
        me.skills.add(metal, 10);
    }
    w.learn_mining();
    if talent {
        w.player_mut().talents.take("field_forge", None);
    }
    w
}

/// `count` steel ingots put at the end of the pack.
fn ingots(w: &mut World, count: u32) {
    let steel = w.content.material_id("steel").expect("steel");
    let stack = crate::piece::make(&w.content, steel, count).expect("ingots");
    w.player_mut().inventory.push(stack);
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn terrain_at(w: &World, p: Pos) -> String {
    let t = w.state.current().terrain_at(p).expect("in bounds");
    w.content.terrain(t).id.clone()
}

/// How many ingots the player carries.
fn carried(w: &World) -> u32 {
    let piece = w.content.object_id(crate::piece::ID).expect("piece");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == piece)
        .map(|s| s.count)
        .sum()
}

/// Refused, taking no time, saying `line`.
fn refused(w: &mut World, actions: &str, line: &str) {
    let before = w.state.time;
    let out = run(w, actions);
    assert!(!out[0].took_turn, "{actions}: no time");
    assert_eq!(w.state.time, before);
    assert_eq!(last(w), line, "{actions}");
}

#[test]
fn the_craftsperson_gains_it_at_level_thirty_two() {
    let w = world(false, false);
    let def = w.content.class("craftsperson").expect("craftsperson");
    let at = (def.abilities.iter()).find(|(_, t)| t == "field_forge");
    assert_eq!(at.map(|(level, _)| *level), Some(32));
    let talents = w.content.talents();
    let id = talents.id("field_forge").expect("the talent");
    assert!(talents.get(id).granted, "a class ability, never picked");
    assert_eq!(Ability::Class(FORGE).aim(&w.content), Aim::Direction);
    assert_eq!(
        Beside::Use(Ability::Class(FORGE)).target(&w.content),
        Some(Target::Ground)
    );
    let action = Action::Use(FORGE, Some(Dir::E));
    assert_eq!(action.to_string(), "use:field_forge:E");
    assert_eq!(
        parse_actions("use:field_forge:E").expect("parses"),
        vec![action]
    );
}

#[test]
fn it_is_refused_with_no_time_without_the_talent_an_ingot_or_open_ground() {
    let mut w = world(false, false);
    ingots(&mut w, 1);
    refused(&mut w, "use:field_forge:E", "You have no such ability.");
    let mut w = world(true, false);
    refused(&mut w, "use:field_forge:E", "You have no ingot to set.");
    // A billet of wood is no ingot.
    let oak = w.content.material_id("hardwood").expect("hardwood");
    let billet = crate::piece::make(&w.content, oak, 3).expect("billets");
    w.player_mut().inventory.push(billet);
    refused(&mut w, "use:field_forge:E", "You have no ingot to set.");
    ingots(&mut w, 1);
    // A wall, and a door.
    let wall = "There is no open ground there to set an anvil on.";
    refused(&mut w, "use:field_forge:NW", wall);
    refused(&mut w, "use:field_forge:N", wall);
    // Something lying there.
    let dagger = w.content.object_id("dagger").expect("dagger");
    w.state
        .current_mut()
        .set_pile(Pos::new(3, 3), vec![Stack::one(dagger)]);
    refused(
        &mut w,
        "use:field_forge:SE",
        "Clear the ground there first.",
    );
    assert_eq!(carried(&w), 4, "nothing spent: 3 billets and the ingot");
}

#[test]
fn an_hour_sets_a_makeshift_anvil_and_spends_one_ingot() {
    let mut w = world(true, false);
    ingots(&mut w, 2);
    let start = w.state.time;
    let out = run(&mut w, "use:field_forge:E");
    assert!(out[0].took_turn);
    assert_eq!(w.state.time.since(start).as_seconds(), 3600);
    assert_eq!(terrain_at(&w, Pos::new(3, 2)), "makeshift_anvil");
    assert_eq!(carried(&w), 1);
    assert_eq!(
        last(&w),
        "You set the block on a heap of stones: a makeshift anvil."
    );
    // It stands in the way, and is no open ground for another.
    refused(
        &mut w,
        "use:field_forge:E",
        "There is no open ground there to set an anvil on.",
    );
}

#[test]
fn an_interrupted_forge_makes_nothing_and_keeps_the_ingot() {
    let mut w = world(true, true);
    ingots(&mut w, 1);
    let start = w.state.time;
    run(&mut w, "use:field_forge:S");
    assert!(w.state.time.since(start).as_seconds() < 3600);
    assert_eq!(terrain_at(&w, Pos::new(2, 3)), "floor");
    assert_eq!(carried(&w), 1);
    assert!(
        last(&w).ends_with("eave the anvil unfinished."),
        "{}",
        last(&w)
    );
}

#[test]
fn the_makeshift_anvil_works_as_the_anvil() {
    let mut w = world(true, false);
    ingots(&mut w, 2);
    run(&mut w, "use:field_forge:E");
    let content = w.content.clone();
    let made = content.terrain_id("makeshift_anvil").expect("terrain");
    let anvil = content.terrain_id("anvil").expect("anvil");
    assert_eq!(content.works_as(made), anvil);
    assert_eq!(content.works_as(anvil), anvil);
    // The walk-in: it offers Temper as the anvil does.
    assert_eq!(content.offer(made), Some(Offer::Temper));
    // Every recipe at the anvil may be made beside it: Temper and the smelt.
    let metal = content.skill_id("metalcraft").expect("metalcraft");
    let rows = w.perception().recipes(metal);
    assert!(rows.iter().any(|r| r.name == "Temper"), "Temper listed");
    assert!(rows.len() > 1, "the anvil's recipes listed");
    assert!(
        !(rows.iter()).any(|r| r.unavailable == Some(RecipeLack::Station(anvil))),
        "no recipe lacks the anvil"
    );
    let temper = content.recipes().of_use(RecipeUse::Temper).expect("temper");
    assert_eq!(temper.station, Some(anvil));
    // Temper itself is begun there.
    let sword = content.object_id("long_sword").expect("sword");
    w.player_mut().inventory.insert(0, Stack::one(sword));
    let start = w.state.time;
    w.apply(&Action::Temper { item: 0 });
    assert!(w.state.time.since(start).as_seconds() > 0, "worked");
    assert!(!(w.messages().iter()).any(|m| m.text == "There is no anvil here."));
}

#[test]
fn a_pick_clears_it_to_floor_with_nothing_mined() {
    let mut w = world(true, false);
    ingots(&mut w, 1);
    run(&mut w, "use:field_forge:E");
    let content = w.content.clone();
    let made = content.terrain(content.terrain_id("makeshift_anvil").expect("terrain"));
    let breaks = made.breaks.expect("it breaks");
    assert_eq!(content.terrain(breaks.becomes).id, "floor");
    let pick = content.object_id("pick_axe").expect("pick-axe");
    w.player_mut().inventory.push(Stack::one(pick));
    let at = w.state.player().inventory.len() - 1;
    run(
        &mut w,
        &format!("equip:tool:{} dig:E", crate::action::position_text(at)),
    );
    assert_eq!(terrain_at(&w, Pos::new(3, 2)), "floor");
    assert!(
        w.state.current().objects_at(Pos::new(3, 2)).is_empty(),
        "nothing mined"
    );
}
