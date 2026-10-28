//! Elixirs (spec §8 "Potions"; `world/elixir.rs`): drunk through the Consume action (`eat:`),
//! the stat +1 for good and its potential +1d2, the hit points and mana that follow, the marks
//! of the `elixir` use, Quick study of the body, and where elixirs come from.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::stats::{PerStat, Stat, Stats};

/// The six elixirs, by id, with the stat each raises.
const ELIXIRS: &[(&str, Stat)] = &[
    ("elixir_of_might", Stat::Strength),
    ("elixir_of_grace", Stat::Dexterity),
    ("elixir_of_vigour", Stat::Toughness),
    ("elixir_of_wit", Stat::Intelligence),
    ("elixir_of_will", Stat::Willpower),
    ("elixir_of_presence", Stat::Presence),
];

/// A lit hall with the player carrying `pack` (a scenario's `carry`), seeded `seed`, every stat
/// 11 with a potential of `potential`.
fn world_seeded(pack: &str, seed: u64, potential: u8) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/elixir_hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n######\n#@...#\n######\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/elixir_hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"elixir_hall\"\nseed = 7\ncarry = {pack}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "elixir_hall", Some(seed)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(11), PerStat::splat(potential));
    let max = w.state.player().max_hp(&w.content);
    w.player_mut().hp = max;
    w
}

fn world(pack: &str) -> World {
    world_seeded(pack, 7, 11)
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn stat(w: &World, stat: Stat) -> (u8, u8) {
    let stats = &w.state.player().stats;
    (stats.score(stat), stats.potential(stat))
}

#[test]
fn an_elixir_raises_its_stat_by_one_and_its_potential_by_one_or_two() {
    let mut rises = BTreeSet::new();
    for seed in 0..40 {
        let mut w = world_seeded("[\"elixir_of_might\"]", seed, 11);
        let before = w.state.time;
        run(&mut w, "eat:a");
        assert!(w.state.time > before, "a turn passed");
        assert!(w.state.player().inventory.is_empty(), "used up");
        let (score, potential) = stat(&w, Stat::Strength);
        assert_eq!(score, 12, "seed {seed}");
        assert!((13..=14).contains(&potential), "seed {seed}: {potential}");
        rises.insert(potential - 12);
        let lines = said(&w);
        let widened = "You feel stronger, and that you could be stronger yet.";
        match potential {
            14 => assert!(lines.iter().any(|m| m == widened), "{lines:?}"),
            _ => assert!(lines.iter().any(|m| m == "You feel stronger."), "{lines:?}"),
        }
        // The other stats stay.
        assert_eq!(stat(&w, Stat::Dexterity), (11, 11));
    }
    assert_eq!(rises, BTreeSet::from([1, 2]), "both rolls come up");
}

#[test]
fn the_potential_is_lifted_to_the_score_first_when_below() {
    // A score at its potential: the trained rise lifts the potential with it, then the roll.
    let mut w = world_seeded("[\"elixir_of_wit\"]", 3, 11);
    run(&mut w, "eat:a");
    let (score, potential) = stat(&w, Stat::Intelligence);
    assert_eq!(score, 12);
    assert!(potential > score);
    // A potential above the score: the score rises within it, and the roll adds to it.
    let mut w = world_seeded("[\"elixir_of_wit\"]", 3, 15);
    run(&mut w, "eat:a");
    let (score, potential) = stat(&w, Stat::Intelligence);
    assert_eq!(score, 12);
    assert!((16..=17).contains(&potential), "{potential}");
}

#[test]
fn hit_points_follow_a_toughness_elixir_and_mana_an_intelligence_one() {
    let mut w = world("[\"elixir_of_vigour\", \"elixir_of_wit\"]");
    let (hp, max) = (w.state.player().hp, w.state.player().max_hp(&w.content));
    run(&mut w, "eat:a");
    let grown = w.state.player().max_hp(&w.content) - max;
    assert!(grown > 0, "Toughness 11 to 12 raises the modifier");
    assert_eq!(
        w.state.player().hp,
        hp + grown,
        "the new hit points come at once"
    );
    let mana = w.state.player().max_mana(&w.content);
    run(&mut w, "eat:a");
    assert!(w.state.player().max_mana(&w.content) > mana);
}

#[test]
fn an_elixir_gives_twenty_marks_and_quick_study_doubles_them() {
    let mut w = world("[\"elixir_of_grace\"]");
    run(&mut w, "eat:a");
    assert_eq!(w.state.player().stat_marks[Stat::Dexterity], 20);
    assert_eq!(w.state.player().stat_marks[Stat::Strength], 0);
    let mut w = world("[\"elixir_of_grace\"]");
    w.player_mut().talents.take("quick_study_of_the_body", None);
    run(&mut w, "eat:a");
    assert_eq!(w.state.player().stat_marks[Stat::Dexterity], 40);
}

#[test]
fn every_elixir_raises_its_own_stat_and_says_so_in_words() {
    for &(id, raised) in ELIXIRS {
        let mut w = world(&format!("[\"{id}\"]"));
        run(&mut w, "eat:a");
        for s in Stat::ALL {
            let want = if s == raised { 12 } else { 11 };
            assert_eq!(stat(&w, s).0, want, "{id}: {s:?}");
        }
        let feel = format!("You feel {}", raised.comparative());
        assert!(said(&w).iter().any(|m| m.starts_with(&feel)), "{id}");
        let object = w.content.object_id(id).expect("elixir");
        let def = w.content.object(object);
        assert_eq!((def.level, def.frequency, def.value), (8, 2, 2000), "{id}");
        assert_eq!(def.drink.as_ref().expect("a drink").colour, "gold");
    }
}

#[test]
fn an_unknown_elixir_is_a_gold_potion_and_a_known_one_says_what_it_does_in_words() {
    let mut w = world("[\"elixir_of_might*2\"]");
    let describe = |w: &World| -> Vec<String> {
        let stack = &w.state.player().inventory[0];
        (crate::magic::describe::appraised(stack, w.lore(), Default::default()))
            .into_iter()
            .map(|l| l.text)
            .collect()
    };
    let stack = w.state.player().inventory[0].clone();
    assert!(w.lore().describe(&stack).ends_with("gold potions"));
    assert!(
        describe(&w)
            .iter()
            .any(|l| l == "You do not know what this potion does.")
    );
    run(&mut w, "eat:a");
    assert!(said(&w).iter().any(|m| m == "It is an elixir of might."));
    let lines = describe(&w);
    assert!(
        lines.iter().any(|l| l == "Makes you stronger for good"),
        "{lines:?}"
    );
    assert!(!lines.join(" ").contains("1d2"), "never the dice");
}

#[test]
fn an_elixir_at_the_top_of_the_scores_says_no_different() {
    let mut w = world("[\"elixir_of_might\"]");
    let top = *crate::stats::SCORES.end();
    w.player_mut().stats = Stats::new(PerStat::splat(top), PerStat::splat(top));
    run(&mut w, "eat:a");
    assert_eq!(stat(&w, Stat::Strength), (top, top));
    assert!(said(&w).iter().any(|m| m == "You feel no different."));
}

#[test]
fn no_general_store_or_provisioner_stocks_an_elixir_and_an_alchemist_may() {
    let w = world("[]");
    let shops = w.content.shops().expect("shops");
    for &(id, _) in ELIXIRS {
        let elixir = w.content.object_id(id).expect("elixir");
        for kind in ["general_store", "provisioner"] {
            let shop = shops.id(kind).expect("shop");
            assert!(!shops.stocks(&w.content, shop, elixir), "{kind}: {id}");
        }
        let alchemist = shops.id("alchemist").expect("alchemist");
        assert!(shops.stocks(&w.content, alchemist, elixir), "{id}");
    }
}

#[test]
fn the_forgotten_altar_may_leave_an_elixir() {
    let w = world("[]");
    let table = w.content.outcomes().get("forgotten_altar").expect("table");
    let elixir = (table.outcomes.iter())
        .find(|o| o.id == "elixir")
        .expect("the elixir outcome");
    let named = format!("{:?}", elixir.effect);
    for &(id, _) in ELIXIRS {
        let object = w.content.object_id(id).expect("elixir");
        assert!(named.contains(&format!("{object:?}")), "{id}: {named}");
    }
}

#[test]
fn content_refuses_an_elixir_without_a_stat_or_with_another_kinds_fields() {
    let load = |drink: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/objects/elixir_of_test.toml",
            &format!(
                "[[object]]\nid = \"elixir_of_test\"\nname = \"elixir of test\"\nglyph = \"!\"\n\
                 fg = \"#e8c860\"\nclass = \"potions\"\nmaterials = [\"glass\"]\nweight = 250\n\
                 value = 1\nstackable = true\n[object.drink]\ncolour = \"gold\"\n{drink}"
            ),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    load("kind = \"elixir\"\nstat = \"strength\"\npotential = \"1d2\"\n").expect("valid");
    for (drink, says) in [
        (
            "kind = \"elixir\"\npotential = \"1d2\"\n",
            "names its `stat`",
        ),
        (
            "kind = \"elixir\"\nstat = \"luck\"\npotential = \"1d2\"\n",
            "no stat `luck`",
        ),
        (
            "kind = \"elixir\"\nstat = \"strength\"\n",
            "needs `potential`",
        ),
        (
            "kind = \"elixir\"\nstat = \"strength\"\npotential = \"1d2\"\npoints = 9\n",
            "`points` are 1 to 3",
        ),
        (
            "kind = \"elixir\"\nstat = \"strength\"\npotential = \"1d2\"\ndice = \"1d4\"\n",
            "has no `dice`",
        ),
        (
            "kind = \"mana\"\ndice = \"1d4\"\nstat = \"strength\"\n",
            "has no `stat`",
        ),
    ] {
        let err = load(drink).expect_err(drink);
        assert!(err.contains(says), "{drink}: {err}");
    }
}
