//! Potions (spec §8 "Potions"): drunk through the Consume action (`eat:`), each kind through its
//! one path; known by a word and their type's colour until drunk or identified; marks; Thrifty
//! drinker and Nose for draughts; the content checks.

use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::ContentFiles;

/// Every potion type, by id.
const POTIONS: &[&str] = &[
    "murky_draught",
    "potion_of_antidote",
    "potion_of_cure_disease",
    "potion_of_full_healing",
    "potion_of_greater_healing",
    "potion_of_healing",
    "potion_of_mana",
];

/// The test content: a lit hall, the player at `@` carrying `pack` (a scenario's `carry`), a
/// potion of healing lying at `p` and two loaves of bread at `b`; `more` adds or replaces files.
fn content_with(pack: &str, more: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/potion_hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n##########\n#@pb.....#\n\
         ##########\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"p\" = { terrain = \"floor\", objects = [\"potion_of_healing\"] }\n\
         \"b\" = { terrain = \"floor\", objects = [\"bread*2\"] }\n",
    );
    files.insert(
        "scenarios/potion_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"potion_hall\"\nseed = 7\nhp = 1000\ncarry = {pack}\n"
        ),
    );
    for (path, text) in more {
        files.insert(path, text);
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world(pack: &str) -> World {
    World::new(content_with(pack, &[]), "potion_hall", Some(7)).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn id(w: &World, object: &str) -> ObjectId {
    w.content.object_id(object).expect("object")
}

fn count(w: &World, object: &str) -> u32 {
    let object = id(w, object);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

fn has(w: &World, status: &str) -> bool {
    let status = w.content.status_id(status).expect("status");
    w.state.player().effects.get(status).is_some()
}

fn afflict(w: &mut World, status: &str, degrees: u16) {
    let status = w.content.status_id(status).expect("status");
    let player = w.state.player;
    w.afflict(player, status, degrees, None, &mut Vec::new());
}

#[test]
fn a_potion_of_healing_heals_at_once_in_a_turn_and_is_used_up() {
    let mut w = world("[\"potion_of_healing*2\"]");
    w.player_mut().hp = 1;
    let before = w.state.time;
    run(&mut w, "eat:a");
    assert!(w.state.player().hp > 1, "healed");
    assert!(w.state.time > before, "a turn passed");
    assert_eq!(count(&w, "potion_of_healing"), 1);
    let lines = said(&w);
    assert!(
        lines.iter().any(|m| m == "You feel your wounds close."),
        "{lines:?}"
    );
}

#[test]
fn healing_never_goes_past_the_maximum_and_greater_healing_stops_bleeding() {
    let mut w = world("[\"potion_of_greater_healing\", \"potion_of_full_healing\"]");
    let max = w.state.player().max_hp(&w.content);
    afflict(&mut w, "bleeding", 2);
    run(&mut w, "eat:a");
    assert_eq!(w.state.player().hp, max);
    assert!(!has(&w, "bleeding"));
    afflict(&mut w, "poisoned", 3);
    run(&mut w, "eat:a");
    assert!(!has(&w, "poisoned"), "full healing draws poison out");
}

#[test]
fn an_antidote_cures_poison_and_cure_disease_every_disease() {
    let mut w = world("[\"potion_of_antidote\", \"potion_of_cure_disease\"]");
    afflict(&mut w, "poisoned", 5);
    run(&mut w, "eat:a");
    assert!(!has(&w, "poisoned"));
    let player = w.state.player;
    let rot = w.content.disease_id("filth_fever").expect("filth fever");
    assert!(w.infect(player, rot, None));
    assert!(!w.state.player().diseases.is_empty());
    run(&mut w, "eat:a");
    assert!(w.state.player().diseases.is_empty(), "every disease ends");
}

#[test]
fn a_potion_of_mana_gives_mana_back() {
    let spec: CharacterSpec = "human,male,wizard".parse().expect("a character");
    let content = content_with("[\"potion_of_mana\"]", &[]);
    let mut w = World::with_character(content, "potion_hall", Some(7), Some(&spec)).expect("world");
    let max = w.state.player().max_mana(&w.content);
    assert!(max > 0);
    w.player_mut().mana = 0;
    run(&mut w, "eat:a");
    assert!(w.state.player().mana > 0);
    assert!(w.state.player().mana <= max);
}

#[test]
fn a_murky_draught_sickens_and_gives_no_marks() {
    let mut w = world("[\"murky_draught\"]");
    run(&mut w, "eat:a");
    assert!(has(&w, "poisoned"));
    let lines = said(&w);
    assert!(lines.iter().any(|m| m == "You feel sick."), "{lines:?}");
    let marks = w.state.player().stat_marks;
    assert!(crate::stats::Stat::ALL.iter().all(|&s| marks[s] == 0));
}

#[test]
fn a_potion_gives_its_marks_by_count() {
    let mut w = world("[\"potion_of_healing\", \"potion_of_mana\", \"potion_of_greater_healing\"]");
    run(&mut w, "eat:a");
    let marks = |w: &World| w.state.player().stat_marks;
    assert_eq!(marks(&w)[crate::stats::Stat::Toughness], 15);
    run(&mut w, "eat:a");
    assert_eq!(marks(&w)[crate::stats::Stat::Willpower], 15);
    run(&mut w, "eat:a");
    assert_eq!(marks(&w)[crate::stats::Stat::Toughness], 45);
}

#[test]
fn a_floor_potion_is_drunk_where_it_lies_and_food_is_still_eaten() {
    let mut w = world("[]");
    run(&mut w, "move:E eat:floor:a");
    let here = w.player_pos();
    assert!(w.state.current().objects_at(here).is_empty(), "drunk");
    let lines = said(&w);
    assert!(
        lines.iter().any(|m| m.starts_with("You drink the ")),
        "{lines:?}"
    );
    // Bread is a meal, as before.
    run(&mut w, "move:E eat:floor:a");
    let here = w.player_pos();
    assert_eq!(
        w.state.current().objects_at(here)[0].count,
        1,
        "{:?}",
        said(&w)
    );
}

#[test]
fn what_is_neither_food_nor_a_potion_is_refused_with_no_time() {
    let mut w = world("[\"dagger\"]");
    let before = w.state.time;
    run(&mut w, "eat:a");
    assert_eq!(w.state.time, before);
    assert_eq!(said(&w).last().unwrap(), "You cannot consume a dagger.");
}

#[test]
fn an_unknown_potion_shows_a_word_and_its_type_colour_until_drunk() {
    let mut w = world("[\"potion_of_healing*2\", \"murky_draught\"]");
    let healing = id(&w, "potion_of_healing");
    let word = w.labels.get(healing).expect("a label").to_string();
    let name = w.lore().describe(&w.state.player().inventory[0]);
    assert_eq!(name, format!("2 {word} red potions"));
    assert_eq!(
        w.lore().fg(&w.state.player().inventory[0]),
        w.content.object(healing).fg,
        "the type's own colour"
    );
    run(&mut w, "eat:a");
    let lines = said(&w);
    assert!(
        lines.contains(&format!("You drink the {word} red potion.")),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"It is a potion of healing.".to_string()),
        "{lines:?}"
    );
    assert!(w.state.known_scrolls.contains(&healing));
    assert_eq!(
        w.lore().describe(&w.state.player().inventory[0]),
        "a potion of healing"
    );
    // The draught is still unknown.
    let murky = id(&w, "murky_draught");
    assert!(w.lore().label(murky).is_some());
}

#[test]
fn every_potion_type_has_its_own_word_and_earlier_labels_are_unchanged() {
    let w = world("[]");
    let words: BTreeSet<&str> = (POTIONS.iter())
        .map(|p| w.labels.get(id(&w, p)).expect("a word"))
        .collect();
    assert_eq!(words.len(), POTIONS.len(), "all different");
    // Without potion labels, the scrolls', wands', and rods' labels are what they are with them.
    let mut files = ContentFiles::builtin();
    files.remove("data/potion_labels.toml");
    let without = Arc::new(Content::load(&files).expect("content"));
    let old = crate::scroll_label::Labels::draw(&without, 1234);
    let new = crate::scroll_label::Labels::draw(&w.content, 1234);
    for o in 0..w.content.object_defs().len() {
        let o = ObjectId(o as u16);
        if w.content.object(o).drink.is_none() {
            assert_eq!(old.get(o), new.get(o), "{}", w.content.object(o).id);
        } else {
            assert_eq!(old.get(o), None);
        }
    }
}

#[test]
fn the_scroll_of_identify_identifies_a_potion_type() {
    let mut w = world("[\"scroll_of_identify\", \"potion_of_mana\"]");
    let mana = id(&w, "potion_of_mana");
    w.state.known_scrolls.insert(id(&w, "scroll_of_identify"));
    run(&mut w, "read:a:b");
    assert!(w.state.known_scrolls.contains(&mana), "{:?}", said(&w));
}

#[test]
fn the_witch_knows_her_kit_potion() {
    let spec: CharacterSpec = "human,female,witch".parse().expect("a character");
    let w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let potion = (w.state.player().inventory.iter())
        .find(|s| w.content.object(s.object).drink.is_some())
        .expect("a potion in her kit");
    assert!(w.lore().label(potion.object).is_none());
}

static BUILTIN: std::sync::LazyLock<Arc<Content>> =
    std::sync::LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

#[test]
fn thrifty_drinker_keeps_potions_by_its_chance_and_draws_only_when_held() {
    // Without the talent, a cure (no dice) draws nothing from the gameplay stream.
    let mut w = world("[\"potion_of_antidote*3\"]");
    let rng = w.state.rng.clone();
    run(&mut w, "eat:a");
    assert_eq!(w.state.rng, rng);
    assert_eq!(count(&w, "potion_of_antidote"), 2);
    // With it, at a sure chance, the potion is kept.
    let sure = "[[talent]]\nid = \"thrifty_drinker\"\nname = \"Thrifty drinker\"\n\
                tree = \"delver\"\ndescription = \"x\"\n\
                effects = [{ kind = \"potion_kept\", chance = 100 }]\nsystems = [\"potions\"]\n";
    let content = content_with(
        "[\"potion_of_antidote*3\"]",
        &[("data/talents/thrifty_drinker.toml", sure)],
    );
    let mut w = World::new(content, "potion_hall", Some(7)).expect("world");
    w.player_mut().talents.take("thrifty_drinker", None);
    run(&mut w, "eat:a");
    assert_eq!(count(&w, "potion_of_antidote"), 3);
    assert!(said(&w).contains(&super::potion::KEPT.to_string()));
    assert!(
        w.state
            .known_scrolls
            .contains(&id(&w, "potion_of_antidote"))
    );
}

#[test]
fn content_refuses_a_potion_with_another_kinds_fields_or_a_split_colour() {
    let potion = |drink: &str, fg: &str| {
        format!(
            "[[object]]\nid = \"odd_potion\"\nname = \"odd potion\"\nglyph = \"!\"\n\
             fg = \"{fg}\"\nclass = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\n\
             stackable = true\n\n[object.drink]\n{drink}\n"
        )
    };
    for (drink, fg) in [
        (
            "kind = \"mana\"\ndice = \"1d4\"\ncures = [\"bandage\"]\ncolour = \"blue\"",
            "#3c6ec8",
        ),
        ("kind = \"heal\"\ncolour = \"red\"", "#c83c3c"),
        ("kind = \"cure\"\ncolour = \"green\"", "#3ca050"),
        (
            "kind = \"heal\"\ndice = \"1d4\"\ncolour = \"red\"",
            "#ffffff",
        ),
        (
            "kind = \"heal\"\ndice = \"1d4\"\ncures = [\"nothing\"]\ncolour = \"red\"",
            "#c83c3c",
        ),
        (
            "kind = \"harm\"\ndice = \"1\"\ncolour = \"brown\"",
            "#5a4030",
        ),
        (
            "kind = \"heal\"\ndice = \"1d4\"\ncolour = \"red\"\nmarks = { luck = 5 }",
            "#c83c3c",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        files.insert("data/objects/odd_potion.toml", &potion(drink, fg));
        assert!(Content::load(&files).is_err(), "{drink}");
    }
    // Too few words for the potion types.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/potion_labels.toml",
        "name = \"{label} {colour} potion\"\nplural = \"{label} {colour} potions\"\n\
         fg = \"#c0c0c0\"\nsyllables = [1, 1]\n\
         parts = [\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\"]\n",
    );
    let mut more = String::new();
    for i in 0..3 {
        more.push_str(
            &potion(
                "kind = \"heal\"\ndice = \"1d4\"\ncolour = \"red\"",
                "#c83c3c",
            )
            .replace("odd_potion", &format!("odd_potion_{i}")),
        );
    }
    files.insert("data/objects/odd_potions.toml", &more);
    assert!(Content::load(&files).is_err());
}
