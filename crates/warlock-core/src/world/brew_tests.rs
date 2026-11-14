//! Brewing (docs/dev/crafting.md "Brewing"; #1357): Alchemy's herb recipes as the design's table
//! gives them, a brew choosing nothing (its herbs taken from the pack, the dialog's plan naming
//! the potion truly), twenty minutes' work, the potion made known for what it is, no grade at any
//! margin, a failure spending the herbs for a murky draught and a natural 2 for a brewer's
//! blunder, the refusals with no time, Intelligence the check's stat, the mark, and a replay.

use super::craft_make::make_out_for_test;
use super::*;
use crate::content::{ContentFiles, RecipeId};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::Stat;

/// A cave with glowing fungus north-east of the player.
const ROOM: &str = "\
#######
#...F.#
#..@..#
#.....#
#.....#
#######
";

/// The built-in content with [`ROOM`]; the player (a witch) carries nothing and has Alchemy at
/// `value`.
fn world(value: u16) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_still.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"F\" = \"fungus\"\n"
        ),
    );
    files.insert(
        "scenarios/test_still.toml",
        "name = \"T\"\nstart_level = \"test_still\"\n\
         character = { race = \"human\", gender = \"female\", class = \"witch\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_still", Some(3)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    train(&mut w, value);
    w
}

fn alchemy(w: &World) -> SkillId {
    w.content.skill_id("alchemy").expect("alchemy")
}

/// Alchemy at `value`, no marks, the other skills as they were.
fn train(w: &mut World, value: u16) {
    let id = alchemy(w);
    let mut all: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((id, Trained { value, marks: 0 }));
    w.player_mut().skills = all.into_iter().collect();
}

/// `count` of herb (or any stackable object) `id` at the end of the pack.
fn give(w: &mut World, id: &str, count: u32) {
    let object = w.content.object_id(id).expect(id);
    let mut stack = Stack::one(object);
    stack.count = count;
    w.player_mut().inventory.push(stack);
}

fn said(w: &World, back: usize) -> String {
    let all = w.messages();
    (all.iter().rev().nth(back)).map_or_else(String::new, |m| m.text.clone())
}

fn recipe(w: &World, id: &str) -> RecipeId {
    w.content.recipes().id(id).expect(id)
}

fn apply(w: &mut World, text: &str) {
    for action in crate::action::parse_actions(text).expect("parses") {
        w.apply(&action);
    }
}

/// The pack as the player knows it.
fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| w.lore().describe(s))
        .collect()
}

fn rows(w: &World) -> Vec<String> {
    (w.perception().recipes(alchemy(w)).into_iter())
        .map(|r| match r.unavailable {
            Some(why) => format!("{}: {} ({})", r.name, r.takes, why.reason(&w.content)),
            None => format!("{}: {}", r.name, r.takes),
        })
        .collect()
}

/// The design's essence rows (recipes.md §6; #1358): inputs, difficulty, from.
type Row = (&'static str, &'static [(&'static str, u32)], i32, u16);
const ESSENCE_ROWS: [Row; 9] = [
    (
        "potion_of_frost",
        &[("bog_myrtle", 1), ("essence_of_ice", 1)],
        13,
        40,
    ),
    (
        "potion_of_earthing",
        &[("bog_myrtle", 1), ("essence_of_earth", 1)],
        13,
        40,
    ),
    (
        "potion_of_sea_legs",
        &[("essence_of_water", 1), ("glowcap", 1)],
        13,
        40,
    ),
    (
        "flask_of_fire",
        &[("essence_of_fire", 1), ("flask_of_oil", 1)],
        14,
        50,
    ),
    (
        "flask_of_frost",
        &[("essence_of_ice", 1), ("glowcap", 1)],
        14,
        50,
    ),
    (
        "flask_of_thunder",
        &[("essence_of_lightning", 1), ("glowcap", 1)],
        15,
        60,
    ),
    (
        "flask_of_smoke",
        &[("essence_of_air", 1), ("henbane", 1)],
        12,
        25,
    ),
    (
        "pitch_fire",
        &[("essence_of_fire", 2), ("flask_of_tar", 1)],
        17,
        100,
    ),
    (
        "blasting_flask",
        &[("essence_of_fire", 1), ("essence_of_lightning", 1)],
        18,
        100,
    ),
];

#[test]
fn the_brews_are_the_designs_table_and_nothing_else() {
    let content = Content::builtin().expect("content");
    let skill = content.skill_id("alchemy").expect("alchemy");
    let murky = content.object_id("murky_draught");
    let blunder = content.object_id("brewers_blunder");
    #[allow(clippy::type_complexity)]
    let mut shown: Vec<(String, Vec<(String, u32)>, i32, u16)> = Vec::new();
    for (_, r) in content.recipes().of(skill) {
        let result = r.result.expect("a result");
        assert_eq!(content.object(result.object).id, r.id, "{}", r.id);
        assert_eq!(r.seconds, 1200, "{}", r.id);
        assert_eq!(r.station, None, "{}", r.id);
        assert!(!r.knife && !r.ornament && r.parts.is_empty(), "{}", r.id);
        assert_eq!(r.failure, murky, "{}", r.id);
        assert_eq!(r.fumble, blunder, "{}", r.id);
        let inputs = (r.inputs.iter())
            .map(|i| {
                let object = i.object().expect("an object input");
                (content.object(object).id.clone(), i.count)
            })
            .collect();
        shown.push((r.id.clone(), inputs, r.difficulty, r.from));
    }
    assert_eq!(shown.len(), 38, "29 of herbs, 9 with an essence");
    let row = |id: &str| shown.iter().find(|r| r.0 == id).cloned().expect(id);
    let herbs = |list: &[(&str, u32)]| -> Vec<(String, u32)> {
        list.iter().map(|(h, n)| (h.to_string(), *n)).collect()
    };
    assert_eq!(
        row("potion_of_healing"),
        ("potion_of_healing".into(), herbs(&[("moonwort", 2)]), 10, 0)
    );
    assert_eq!(
        row("potion_of_full_healing"),
        (
            "potion_of_full_healing".into(),
            herbs(&[("moonwort", 4), ("comfrey", 2), ("mandrake", 1)]),
            18,
            100
        )
    );
    assert_eq!(
        row("greater_potion_of_wit"),
        (
            "greater_potion_of_wit".into(),
            herbs(&[("mandrake", 2), ("vervain", 2)]),
            17,
            100
        )
    );
    assert_eq!(
        row("stinking_flask"),
        (
            "stinking_flask".into(),
            herbs(&[("henbane", 1), ("bog_myrtle", 1)]),
            11,
            10
        )
    );
    // The essence rows (#1358).
    for (id, inputs, difficulty, from) in ESSENCE_ROWS {
        assert_eq!(
            row(id),
            (id.into(), herbs(inputs), difficulty, from),
            "{id}"
        );
    }
    // Never brewed: the elixirs, the gambles, the deeper draughts, oil and tar, the hexing flasks.
    for never in [
        "elixir_of_might",
        "draught_of_chance",
        "murky_draught",
        "potion_of_restoration",
        "potion_of_quickness",
        "flask_of_oil",
        "flask_of_tar",
        "flask_of_darkness",
    ] {
        assert!(content.recipes().id(never).is_none(), "{never}");
    }
}

#[test]
fn a_brew_chooses_nothing_and_its_plan_names_the_potion_truly() {
    let mut w = world(20);
    give(&mut w, "moonwort", 1);
    give(&mut w, "glowcap", 1);
    give(&mut w, "moonwort", 3);
    let id = recipe(&w, "potion_of_healing");
    let view = w.perception();
    assert!(view.craft_steps(id).is_empty(), "nothing to choose");
    let plan = view.craft_plan(id, &[]).expect("a plan");
    assert_eq!(plan.verb, "Brew");
    // Unknown to the player yet, the potion is still named: the recipe says what it makes.
    assert_eq!(plan.name, "a potion of healing");
    assert_eq!(plan.uses, "2 sprigs of moonwort");
    assert_eq!(plan.station, None);
    assert_eq!(plan.time, "about 20 minutes");
    assert_eq!(
        view.craft_plan(id, &[Some(0)]).err().as_deref(),
        Some("That is more than the work takes.")
    );
    // The first stacks are taken: one sprig from the first, one from the third.
    let start = w.state.time;
    apply(&mut w, "craft:potion_of_healing");
    assert_eq!(w.state.time.since(start).as_seconds(), 1200);
    assert_eq!(
        pack(&w),
        ["a glowcap", "2 sprigs of moonwort", "a potion of healing"]
    );
}

#[test]
fn a_brewed_potion_is_known_for_what_it_is_and_alchemy_is_marked() {
    let mut w = world(20);
    give(&mut w, "moonwort", 2);
    let healing = w.content.object_id("potion_of_healing").expect("healing");
    assert!(w.lore().label(healing).is_some(), "unknown at first");
    // A natural 24 always brews it, whatever the margin.
    make_out_for_test(&mut w, "potion_of_healing", &[], 24, -10);
    assert!(w.lore().label(healing).is_none(), "the type is known");
    assert_eq!(pack(&w), ["a potion of healing"]);
    assert!(w.state.player().inventory[0].is_identified());
    assert_eq!(said(&w, 0), "You have made a potion of healing.");
    // End to end, with the mark.
    let mut w = world(20);
    give(&mut w, "glowcap", 3);
    apply(&mut w, "craft:potion_of_sustenance");
    let id = alchemy(&w);
    let marks = (w.state.player().skills.get(id)).expect("trained").marks;
    assert_eq!(
        marks, 1,
        "one mark, at a hundred percent, whatever came of it"
    );
    let words: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
    assert!(
        words.contains(&"You set to work on the potion of sustenance."),
        "{words:?}"
    );
}

#[test]
fn a_potion_has_no_grade_at_any_margin() {
    let mut w = world(20);
    give(&mut w, "moonwort", 2);
    make_out_for_test(&mut w, "potion_of_healing", &[], 12, 20);
    assert_eq!(pack(&w), ["a potion of healing"]);
    assert_eq!(said(&w, 0), "You have made a potion of healing.");
    let stack = &w.state.player().inventory[0];
    assert_eq!(*stack, {
        let mut plain = Stack::one(stack.object);
        plain.identify();
        plain
    });
}

#[test]
fn a_failed_brew_spends_the_herbs_for_a_murky_draught_and_a_natural_2_for_a_blunder() {
    let mut w = world(20);
    give(&mut w, "moonwort", 5);
    make_out_for_test(&mut w, "potion_of_healing", &[], 12, -1);
    assert_eq!(pack(&w), ["3 sprigs of moonwort", "a murky draught"]);
    assert_eq!(
        said(&w, 1),
        "The brew clouds over and settles murky. That was not what you meant to make."
    );
    assert_eq!(said(&w, 0), "You are left with a murky draught.");
    let murky = w.content.object_id("murky_draught").expect("murky");
    assert!(w.lore().label(murky).is_none(), "the brewer knows it");
    // A natural 2, however good the brewer: a brewer's blunder.
    make_out_for_test(&mut w, "potion_of_healing", &[], 2, 30);
    assert_eq!(
        pack(&w),
        [
            "a sprig of moonwort",
            "a murky draught",
            "a brewer's blunder"
        ]
    );
    assert_eq!(
        said(&w, 1),
        "The brew seethes and turns a colour no book speaks of."
    );
    assert_eq!(said(&w, 0), "You are left with a brewer's blunder.");
}

#[test]
fn a_brew_is_refused_with_no_time_without_the_herbs_or_beyond_skill() {
    let mut w = world(20);
    give(&mut w, "moonwort", 1);
    let rows = rows(&w);
    let antidote = "Potion of antidote: a sprig of feverfew, a glowcap, about 20 minutes (not \
                    enough sprigs of feverfew)";
    assert!(rows.iter().any(|r| r == antidote), "{rows:?}");
    let healing = (rows.iter())
        .find(|r| r.starts_with("Potion of healing"))
        .expect("listed");
    assert_eq!(
        healing,
        "Potion of healing: 2 sprigs of moonwort, about 20 minutes (not enough sprigs of moonwort)"
    );
    let full = (rows.iter())
        .find(|r| r.starts_with("Potion of full healing"))
        .expect("listed");
    assert!(full.ends_with("(beyond your skill)"), "{full}");
    let start = w.state.time;
    for (text, words) in [
        (
            "craft:potion_of_healing",
            "You have too few sprigs of moonwort.",
        ),
        ("craft:potion_of_full_healing", "That is beyond your skill."),
    ] {
        apply(&mut w, text);
        assert_eq!(said(&w, 0), words, "{text}");
        assert_eq!(w.state.time, start, "{text}");
    }
    assert_eq!(pack(&w), ["a sprig of moonwort"]);
    // The untrained see none of them.
    let id = alchemy(&w);
    let rest: Vec<(SkillId, Trained)> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    w.player_mut().skills = rest.into_iter().collect();
    assert!(w.perception().recipes(id).is_empty());
}

#[test]
fn intelligence_is_the_brews_stat() {
    let w = world(100);
    let def = w.content.recipe(recipe(&w, "potion_of_healing"));
    let me = w.state.player();
    let want = me.effective_stats(&w.content).modifier(Stat::Intelligence)
        + me.skill_effect_of(&w.content, alchemy(&w), crate::skills::Effect::Craft)
        + me.talent_mods(&w.content).vigour.check(Stat::Intelligence);
    assert_eq!(w.craft_modifier(def, None), want);
    assert_eq!(
        me.skill_effect_of(&w.content, alchemy(&w), crate::skills::Effect::Craft),
        8,
        "Alchemy's craft at 100"
    );
}

#[test]
fn a_brew_from_gathered_herbs_replays() {
    let content = world(20).content.clone();
    // The witch as the scenario makes her: Alchemy and Herbalism trained, nothing carried.
    let mut w = World::new(content.clone(), "test_still", Some(3)).expect("world");
    let mut replay = Replay::new(&w);
    let text = "gather:NE*4 craft:potion_of_mana wait";
    for action in crate::action::parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        pack(&w)
            .iter()
            .any(|p| p.contains("potion of mana") || p.contains("murky")),
        "{:?}",
        pack(&w)
    );
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn a_flask_of_fire_from_a_found_essence_replays() {
    // A witch south of an essence of fire and a flask of oil lying on the floor.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_brewery.toml",
        "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#.@.#\n#.E.#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"E\" = { terrain = \"floor\", objects = [\"essence_of_fire\", \"flask_of_oil\"] }\n",
    );
    files.insert(
        "scenarios/test_brewery.toml",
        "name = \"T\"\nstart_level = \"test_brewery\"\n\
         character = { race = \"human\", gender = \"female\", class = \"witch\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content.clone(), "test_brewery", Some(3)).expect("world");
    let mut replay = Replay::new(&w);
    // Levels for Alchemy past the flask's 50 (the dev grant and the rolls), then the brew.
    // The oil identified (the dev `identify`): an unknown flask is no input (#1383).
    let text = "xp:30000 advance:alchemy*40 levelup:auto move:S pickup identify \
                craft:flask_of_fire wait";
    for action in crate::action::parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let made = pack(&w);
    assert!(
        made.iter()
            .any(|p| p == "a flask of fire" || p == "a murky draught"),
        "{made:?}"
    );
    assert!(!made.iter().any(|p| p.contains("essence")), "{made:?}");
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// Every input of `inputs` given, each its count, in order.
fn give_all(w: &mut World, inputs: &[(&str, u32)]) {
    for (id, count) in inputs {
        give(w, id, *count);
    }
}

/// The types of `inputs` known (a flask's; #1383), as if identified.
fn know_all(w: &mut World, inputs: &[(&str, u32)]) {
    for (id, _) in inputs {
        let object = w.content.object_id(id).expect(id);
        w.learn_scroll(object);
    }
}

#[test]
fn each_essence_recipe_makes_its_potion_or_flask_from_its_inputs() {
    for (id, inputs, _, _) in ESSENCE_ROWS {
        let mut w = world(100);
        give_all(&mut w, inputs);
        know_all(&mut w, inputs);
        make_out_for_test(&mut w, id, &[], 12, 5);
        let made = w.content.object_id(id).expect(id);
        let pack = &w.state.player().inventory;
        assert_eq!(pack.len(), 1, "{id}: the inputs spent, {pack:?}");
        assert_eq!((pack[0].object, pack[0].count), (made, 1), "{id}");
        assert!(pack[0].is_identified(), "{id}");
        assert!(w.lore().label(made).is_none(), "{id}: its type known");
    }
}

#[test]
fn a_flask_of_fire_spends_an_essence_and_a_bought_flask_of_oil() {
    let mut w = world(100);
    give(&mut w, "flask_of_oil", 2);
    give(&mut w, "essence_of_fire", 1);
    know_all(&mut w, &[("flask_of_oil", 1)]);
    let start = w.state.time;
    apply(&mut w, "craft:flask_of_fire");
    assert_eq!(w.state.time.since(start).as_seconds(), 1200);
    let ids: Vec<(String, u32)> = (w.state.player().inventory.iter())
        .map(|s| (w.content.object(s.object).id.clone(), s.count))
        .collect();
    let made = (ids.iter()).any(|(id, _)| id == "flask_of_fire" || id == "murky_draught");
    assert!(made, "{ids:?}");
    assert!(
        ids.contains(&("flask_of_oil".into(), 1)),
        "one oil spent: {ids:?}"
    );
    assert!(
        !ids.iter().any(|(id, _)| id == "essence_of_fire"),
        "{ids:?}"
    );
}

#[test]
fn an_unknown_flask_of_oil_is_no_input_until_identified() {
    // #1383: the flask of fire's row must not tell an unknown clear potion is oil.
    let mut w = world(100);
    give(&mut w, "flask_of_oil", 1);
    give(&mut w, "essence_of_fire", 1);
    let oil = w.content.object_id("flask_of_oil").expect("oil");
    assert!(w.lore().label(oil).is_some(), "a labelled potion type");
    let row = |w: &World| {
        (rows(w).into_iter())
            .find(|r| r.starts_with("Flask of fire"))
            .expect("listed")
    };
    let unknown = row(&w);
    assert!(unknown.ends_with("(not enough flasks of oil)"), "{unknown}");
    let start = w.state.time;
    apply(&mut w, "craft:flask_of_fire");
    assert_eq!(said(&w, 0), "You have too few flasks of oil.");
    assert_eq!(w.state.time, start, "no time");
    assert_eq!(w.state.player().inventory.len(), 2, "nothing spent");
    // Identified, it counts.
    w.learn_scroll(oil);
    let known = row(&w);
    assert!(!known.contains('('), "usable: {known}");
    apply(&mut w, "craft:flask_of_fire");
    assert_eq!(w.state.time.since(start).as_seconds(), 1200);
}

#[test]
fn a_brew_is_refused_with_no_time_without_its_essence() {
    let mut w = world(100);
    give(&mut w, "flask_of_oil", 1);
    give(&mut w, "essence_of_fire", 1);
    let rows = rows(&w);
    let pitch = (rows.iter())
        .find(|r| r.starts_with("Pitch fire"))
        .expect("listed");
    assert!(pitch.ends_with("(not enough essences of fire)"), "{pitch}");
    let thunder = (rows.iter())
        .find(|r| r.starts_with("Flask of thunder"))
        .expect("listed");
    assert!(
        thunder.ends_with("(not enough essences of lightning)"),
        "{thunder}"
    );
    let start = w.state.time;
    apply(&mut w, "craft:flask_of_thunder");
    assert_eq!(said(&w, 0), "You have too few essences of lightning.");
    apply(&mut w, "craft:pitch_fire");
    assert_eq!(said(&w, 0), "You have too few essences of fire.");
    assert_eq!(w.state.time, start);
    assert_eq!(w.state.player().inventory.len(), 2, "nothing spent");
}

#[test]
fn a_failed_essence_brew_spends_its_essences_for_a_murky_draught_or_a_blunder() {
    let mut w = world(100);
    give(&mut w, "essence_of_fire", 2);
    give(&mut w, "essence_of_lightning", 2);
    make_out_for_test(&mut w, "blasting_flask", &[], 12, -1);
    assert_eq!(
        pack(&w),
        [
            "an essence of fire",
            "an essence of lightning",
            "a murky draught"
        ]
    );
    make_out_for_test(&mut w, "blasting_flask", &[], 2, 30);
    assert_eq!(pack(&w), ["a murky draught", "a brewer's blunder"]);
    assert_eq!(said(&w, 0), "You are left with a brewer's blunder.");
}

#[test]
fn every_essence_a_brew_takes_drops_from_some_creature_and_oil_and_tar_are_sold() {
    let content = Content::builtin().expect("content");
    let skill = content.skill_id("alchemy").expect("alchemy");
    let shops = content.shops().expect("shops");
    let dropped = |object| {
        (content.creature_defs().iter())
            .any(|c| (c.drops.iter()).any(|d| d.goods == crate::content::DropGoods::Object(object)))
    };
    let sold = |object| {
        (0..shops.defs().len()).any(|i| {
            let kind = crate::content::ShopId(i as u8);
            shops.get(kind).generated && shops.stocks(&content, kind, object)
        })
    };
    let mut essences = 0;
    for (_, r) in content.recipes().of(skill) {
        for input in &r.inputs {
            let object = input.object().expect("an object input");
            let def = content.object(object);
            if def.id.starts_with("essence_of_") {
                essences += 1;
                assert!(dropped(object), "{}: {} drops from nothing", r.id, def.id);
            } else if content.item_class(def.class).id == "potions" {
                assert!(sold(object), "{}: {} is sold nowhere", r.id, def.id);
            }
        }
    }
    assert_eq!(essences, 10, "the essence inputs of the nine rows");
}
