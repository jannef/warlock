//! The utility draughts (spec §8 "Potions", the potions wave 2 design §4.2): sustenance
//! (`nourish`), restoration (`restore`, the scroll of restoration's cleanse), blinking
//! (`blink`, the teleport trap's move), and clear head (a `cure` of the `clear_head` means),
//! drunk through `eat:`; their content checks.

use super::*;
use crate::content::ContentFiles;
use crate::stats::Stat;
use crate::test_support::{run, said};

/// The test content: a lit hall, the player at `@` carrying `pack`; `more` adds files.
fn content_with(pack: &str, more: &[(&str, &str)]) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/utility_hall.toml",
        "name = \"Hall\"\nmap = '''\n##########\n#@.......#\n#........#\n#........#\n\
         ##########\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/utility_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"utility_hall\"\nseed = 7\nhp = 1000\ncarry = {pack}\n"
        ),
    );
    for (path, text) in more {
        files.insert(path, text);
    }
    Content::load(&files).map_err(|e| e.to_string())
}

fn world(pack: &str) -> World {
    let content = Arc::new(content_with(pack, &[]).expect("test content is valid"));
    World::new(content, "utility_hall", Some(7)).expect("world")
}

fn nutrition(w: &World) -> i32 {
    w.state.player().satiation.expect("hungers").nutrition
}

fn set_nutrition(w: &mut World, n: i32) {
    w.player_mut().satiation = Some(crate::hunger::Satiation::new(n));
}

#[test]
fn sustenance_feeds_a_meals_worth_never_into_bloated() {
    let mut w =
        world("[\"potion_of_sustenance\", \"potion_of_sustenance\", \"potion_of_sustenance\"]");
    set_nutrition(&mut w, 1000);
    let out = run(&mut w, "eat:a");
    assert!(out[0].took_turn);
    assert_eq!(nutrition(&w), 2500, "a ration's 1500");
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "It fills you as a good meal would.")
    );
    assert_eq!(w.state.player().stat_marks[Stat::Toughness], 15);
    // Near Bloated it stops below it.
    let bloated = w.content.hunger().expect("hunger").states.bloated;
    set_nutrition(&mut w, bloated - 200);
    run(&mut w, "eat:a");
    assert_eq!(nutrition(&w), bloated - 1);
    // Bloated already, it does nothing (and never lowers it).
    set_nutrition(&mut w, bloated + 100);
    run(&mut w, "eat:a");
    assert_eq!(nutrition(&w), bloated + 100);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("You feel no fuller.")
    );
    assert!(w.state.player().inventory.is_empty(), "all three drunk");
}

#[test]
fn restoration_mends_damage_restores_drains_and_ends_stiffening() {
    let mut w = world("[\"potion_of_restoration\", \"potion_of_restoration\"]");
    let strength = w.state.player().stats.score(Stat::Strength);
    let me = w.player_mut();
    me.damage.take(Stat::Dexterity, 3, 10);
    let lost = me.stats.drain(Stat::Strength, 2, 3);
    me.drained[Stat::Strength] = lost;
    assert!(lost > 0);
    let stiffening = w.content.status_id("stiffening").expect("stiffening");
    let player = w.state.player;
    let mut events = Vec::new();
    w.afflict(player, stiffening, 1, None, &mut events);
    assert!(w.state.player().effects.has(stiffening));
    let out = run(&mut w, "eat:a");
    assert!(out[0].took_turn);
    let me = w.state.player();
    assert!(!me.damage.is_damaged());
    assert_eq!(me.drained[Stat::Strength], 0);
    assert_eq!(me.stats.score(Stat::Strength), strength);
    assert!(!me.effects.has(stiffening));
    assert_eq!(me.stat_marks[Stat::Willpower], 15);
    let lines = said(&w);
    for line in [
        "You feel your strength return.",
        "You feel your dexterity return.",
    ] {
        assert!(lines.iter().any(|t| t == line), "{line}: {lines:?}");
    }
    // With nothing to restore it is drunk all the same: an unknown potion is never refused.
    let out = run(&mut w, "eat:a");
    assert!(out[0].took_turn);
    assert!(w.state.player().inventory.is_empty());
    assert_eq!(said(&w).last().map(String::as_str), Some("You feel whole."));
}

#[test]
fn blinking_moves_the_drinker_to_a_free_cell_without_a_draw() {
    let mut w = world("[\"potion_of_blinking\"]");
    let here = w.player_pos();
    let rng = w.state.rng.clone();
    let out = run(&mut w, "eat:a");
    assert!(out[0].took_turn);
    let there = w.player_pos();
    assert_ne!(there, here);
    assert!(w.lands_on(there));
    assert!(
        said(&w)
            .iter()
            .any(|m| m == "In a blink, you are elsewhere.")
    );
    assert_eq!(w.state.player().stat_marks, Default::default(), "no marks");
    assert_eq!(w.state.rng, rng, "the blink draws nothing");
    // The same run blinks to the same cell.
    let mut again = world("[\"potion_of_blinking\"]");
    run(&mut again, "eat:a");
    assert_eq!(again.player_pos(), there);
}

#[test]
fn blinking_on_the_overworld_does_nothing() {
    let content = Arc::new(content_with("[]", &[]).expect("content"));
    let mut w = World::new(content, "world", Some(7)).expect("world");
    let blink = w.content.object_id("potion_of_blinking").expect("potion");
    w.player_mut().inventory.push(Stack::one(blink));
    let here = w.player_pos();
    let letter = crate::action::position_text(w.state.player().inventory.len() - 1);
    let out = run(&mut w, &format!("eat:{letter}"));
    assert!(out[0].took_turn);
    assert_eq!(w.player_pos(), here);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("The land lurches, and holds you where you are.")
    );
}

#[test]
fn clear_head_ends_the_clouded_mind_and_the_numbed_body() {
    const CLEARED: [&str; 6] = [
        "frightened",
        "drowsy",
        "sluggish",
        "stunned",
        "staggered",
        "transfixed",
    ];
    let mut w = world("[\"potion_of_clear_head\"]");
    let player = w.state.player;
    let poisoned = w.content.status_id("poisoned").expect("poisoned");
    let mut events = Vec::new();
    for id in CLEARED {
        let status = w.content.status_id(id).expect(id);
        w.afflict(player, status, 1, None, &mut events);
    }
    w.afflict(player, poisoned, 2, None, &mut events);
    run(&mut w, "eat:a");
    for id in CLEARED {
        let status = w.content.status_id(id).expect(id);
        assert!(!w.state.player().effects.has(status), "{id} ends");
    }
    assert!(w.state.player().effects.has(poisoned), "not a poison");
    assert_eq!(w.state.player().stat_marks[Stat::Willpower], 15);
}

#[test]
fn utility_kinds_take_only_their_fields() {
    let potion = |drink: &str| {
        format!(
            "[[object]]\nid = \"odd\"\nname = \"odd\"\nglyph = \"!\"\nfg = \"#70b0e0\"\n\
             class = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\nvalue = 1\nstackable = true\n\n[object.drink]\n\
             {drink}\ncolour = \"sky blue\"\n"
        )
    };
    let load = |drink: &str| content_with("[]", &[("data/objects/odd.toml", &potion(drink))]);
    load("kind = \"nourish\"\nsatiation = 900").expect("nourish");
    assert!(load("kind = \"restore\"").is_ok());
    assert!(load("kind = \"blink\"").is_ok());
    for (bad, says) in [
        ("kind = \"nourish\"", "needs `satiation`"),
        ("kind = \"blink\"\nstatus = \"poisoned\"", "has no `status`"),
        ("kind = \"restore\"\ndice = \"1d4\"", "has no `dice`"),
        (
            "kind = \"heal\"\ndice = \"1d4\"\nsatiation = 10",
            "has no `satiation`",
        ),
    ] {
        let err = load(bad).expect_err(bad);
        assert!(err.contains(says), "{bad}: {err}");
    }
}
