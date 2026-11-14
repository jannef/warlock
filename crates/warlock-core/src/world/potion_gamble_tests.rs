//! The gambles (spec §8 "Potions", the potions wave 2 design §4.3): a `gamble` draught draws one
//! outcome of its `data/outcomes/` table, as a healing spring's drink does; the draught of
//! chance, troll's blood, and the brewer's blunder; the outcome effects they brought (a list of
//! statuses, `drain_stat`); the loader's checks.

use super::*;
use crate::content::ContentFiles;
use crate::stats::Stat;
use crate::test_support::{run, said};

/// The test content: a lit hall, the player at `@` carrying `pack`; `more` adds files.
fn content_with(pack: &str, more: &[(&str, &str)]) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/gamble_hall.toml",
        "name = \"Hall\"\nmap = '''\n##########\n#@.......#\n#........#\n#........#\n\
         ##########\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/gamble_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"gamble_hall\"\nseed = 7\nhp = 1000\ncarry = {pack}\n"
        ),
    );
    for (path, text) in more {
        files.insert(path, text);
    }
    Content::load(&files).map_err(|e| e.to_string())
}

fn world_of(content: Content) -> World {
    World::new(Arc::new(content), "gamble_hall", Some(7)).expect("world")
}

fn world(pack: &str) -> World {
    world_of(content_with(pack, &[]).expect("test content is valid"))
}

fn severity(w: &World, id: &str) -> Option<u16> {
    let status = w.content.status_id(id).expect(id);
    w.state.player().effects.get(status).map(|e| e.severity)
}

/// A fixture gamble, `odd`, drawing from the table `odd` (`table`'s text).
fn fixture(table: &str) -> Content {
    let potion = "[[object]]\nid = \"odd\"\nname = \"odd draught\"\nglyph = \"!\"\n\
                  fg = \"#c0a0e0\"\nclass = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\n\
                  value = 1\nstackable = true\n\n[object.drink]\nkind = \"gamble\"\n\
                  outcomes = \"odd\"\ncolour = \"iridescent\"\n";
    content_with(
        "[\"odd\", \"odd\", \"odd\", \"potion_of_restoration\"]",
        &[
            ("data/objects/odd.toml", potion),
            ("data/outcomes/odd.toml", table),
        ],
    )
    .expect("fixture content")
}

#[test]
fn trolls_blood_lends_renewing_and_poisons_at_once() {
    let mut w = world("[\"trolls_blood\"]");
    let out = run(&mut w, "eat:a");
    assert!(out[0].took_turn);
    assert_eq!(severity(&w, "renewing"), Some(3));
    assert_eq!(severity(&w, "poisoned"), Some(2));
    let lines = said(&w);
    assert!(
        lines.iter().any(|m| m == "It is a vial of troll's blood."),
        "{lines:?}"
    );
    assert!(
        (lines.iter())
            .any(|m| m == "Your flesh begins to knit, but the blood sits ill in your stomach."),
        "{lines:?}"
    );
    // The outcome's line tells it, not the statuses' own start lines.
    assert!(!lines.iter().any(|m| m == "You feel sick."), "{lines:?}");
    assert_eq!(w.state.player().stat_marks[Stat::Toughness], 15);
    assert!(w.state.player().inventory.is_empty());
}

#[test]
fn a_draught_of_chance_falls_every_way_in_time() {
    let pack = format!("[{}]", ["\"draught_of_chance\""; 40].join(", "));
    let mut w = world(&pack);
    run(&mut w, &"eat:a ".repeat(40));
    assert!(w.state.player().inventory.is_empty(), "every one drunk");
    let lines = said(&w);
    let table = w
        .content
        .outcomes()
        .get("draught_of_chance")
        .expect("table");
    let heard = |kind| (table.of(kind)).any(|o| o.messages.iter().any(|m| lines.contains(m)));
    for kind in crate::content::OutcomeKind::ALL {
        assert!(heard(kind), "{kind:?} never came up in 40: {lines:?}");
    }
}

#[test]
fn the_same_seed_draws_the_same_gambles() {
    let drink = || {
        let pack = format!("[{}]", ["\"draught_of_chance\""; 10].join(", "));
        let mut w = world(&pack);
        run(&mut w, &"eat:a ".repeat(10));
        (said(&w), w.state_hash())
    };
    assert_eq!(drink(), drink());
}

#[test]
fn a_drain_stat_outcome_drains_for_good_and_restoration_gives_it_back() {
    let mut w = world_of(fixture(
        "kinds = { bane = 1 }\nexhaust = 0\n\n[[outcome]]\nkind = \"bane\"\ntier = \"minor\"\n\
         id = \"sapped\"\nweight = 1\neffect = { drain_stat = { stat = \"strength\", points = 1 } }\n\
         message = \"Sapped.\"\notherwise = \"Spared.\"\n",
    ));
    let before = w.state.player().stats.score(Stat::Strength);
    run(&mut w, "eat:a");
    let me = w.state.player();
    assert_eq!(me.stats.score(Stat::Strength), before - 1);
    assert_eq!(me.drained[Stat::Strength], 1);
    assert_eq!(said(&w).last().map(String::as_str), Some("Sapped."));
    // The potion of restoration (the last stack) gives it back.
    let letter = crate::action::position_text(w.state.player().inventory.len() - 1);
    run(&mut w, &format!("eat:{letter}"));
    assert_eq!(w.state.player().stats.score(Stat::Strength), before);
    assert_eq!(w.state.player().drained[Stat::Strength], 0);
    // At the drain's floor nothing is lost, and the `otherwise` line is said instead.
    w.player_mut().stats.drain(Stat::Strength, 30, 0);
    let floor = w.state.player().stats.score(Stat::Strength);
    run(&mut w, "eat:a");
    assert_eq!(w.state.player().stats.score(Stat::Strength), floor);
    assert_eq!(said(&w).last().map(String::as_str), Some("Spared."));
    assert!(!said(&w).iter().rev().take(1).any(|m| m == "Sapped."));
}

#[test]
fn every_gamble_draws_from_a_table_that_loads() {
    let w = world("[]");
    for id in ["draught_of_chance", "trolls_blood", "brewers_blunder"] {
        let object = w.content.object_id(id).expect(id);
        let drink = w.content.object(object).drink.as_ref().expect("a potion");
        let crate::content::Draught::Gamble { outcomes } = &drink.does else {
            panic!("{id} is a gamble");
        };
        assert_eq!(outcomes, id);
        assert!(w.content.outcomes().get(outcomes).is_some(), "{id}");
    }
}

#[test]
fn a_gamble_names_a_table_that_exists_and_nothing_else_names_one() {
    let potion = |drink: &str| {
        format!(
            "[[object]]\nid = \"odd\"\nname = \"odd\"\nglyph = \"!\"\nfg = \"#c0a0e0\"\n\
             class = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\nvalue = 1\n\
             stackable = true\n\n[object.drink]\n{drink}\ncolour = \"iridescent\"\n"
        )
    };
    let load = |drink: &str| content_with("[]", &[("data/objects/odd.toml", &potion(drink))]);
    load("kind = \"gamble\"\noutcomes = \"spring\"").expect("a gamble on the spring's table");
    for (bad, says) in [
        ("kind = \"gamble\"", "names its `outcomes`"),
        (
            "kind = \"gamble\"\noutcomes = \"nonesuch\"",
            "no outcome table `nonesuch`",
        ),
        (
            "kind = \"gamble\"\noutcomes = \"spring\"\nstatus = \"poisoned\"",
            "has no `status`",
        ),
        (
            "kind = \"restore\"\noutcomes = \"spring\"",
            "has no `outcomes`",
        ),
    ] {
        let err = load(bad).expect_err(bad);
        assert!(err.contains(says), "{bad}: {err}");
    }
}
