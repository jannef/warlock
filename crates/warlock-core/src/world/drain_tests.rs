//! Drains (spec §8 "Stats": "Drain"; #382): a hit with a draining touch makes its target check,
//! and on a failure takes points off a stat's score for good (the potential stays), never below
//! the floor, never killing; the player is told in words; talents raise the check; a temple's
//! mending gives the points back; all of it is saved, hashed, and replayed.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::check::check;
use crate::combat::{Blow, Strike};
use crate::content::{Affliction, ContentFiles, Drain};
use crate::replay::Replay;
use crate::stats::{DRAIN_FLOOR, PerStat, Stat, Stats};
use std::sync::LazyLock;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "undead"
name = "undead"
hostile = ["player"]
"##;

// The drinker's touch drains a point of Strength against Willpower; the leech's five points of a
// random stat against Toughness; both all but surely (a difficulty of 40: only a natural 24
// holds against it).
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "drinker"
name = "drinker"
glyph = "G"
fg = "#8c8fa8"
weight = 1000
faction = "undead"
stats = { dexterity = 30 }
attack = { damage = "1", name = "touch", verb = "touches", to_hit = 20, on_hit = [{ drain = "strength", chance = 100, difficulty = 40 }] }

[[creature]]
id = "leech"
name = "leech"
glyph = "l"
fg = "#806040"
weight = 1000
faction = "undead"
attack = { damage = "1", name = "touch", verb = "touches", on_hit = [{ drain = "random", points = 5, chance = 100, check = "toughness", difficulty = 40 }] }
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"G" = { terrain = "floor", creature = "drinker" }
"l" = { terrain = "floor", creature = "leech" }
"##;

/// The player beside a drinker.
const BESIDE: &str = "#####\n#@G.#\n#####\n";

/// Willpower 1, Strength 12 (potential 15), and a thousand hit points more.
const VICTIM: &str =
    "stats = { willpower = 1, strength = 12 }\npotential = { strength = 15 }\nhp = 1000";

fn files(map: &str, scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/crypt.toml",
        &format!("name = \"Crypt\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/crypt.toml",
        &format!("name = \"Crypt\"\nstart_level = \"crypt\"\n{scenario}\n"),
    );
    files
}

fn content(map: &str, scenario: &str) -> Arc<Content> {
    Arc::new(Content::load(&files(map, scenario)).expect("test content is valid"))
}

fn world_on(map: &str, scenario: &str, seed: u64) -> World {
    World::new(content(map, scenario), "crypt", Some(seed)).expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

const HIT: Blow = Blow {
    strike: Strike::Hit { critical: false },
    damage: 1,
    kind: None,
};

/// The drain of creature `id`'s natural attack.
fn drain_of(content: &Content, id: &str) -> Drain {
    let creature = content.creature_id(id).expect(id);
    let attack = content
        .creature(creature)
        .attack
        .as_ref()
        .expect("an attack");
    match attack.on_hit[0].what {
        Affliction::Drain(drain) => drain,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_failed_check_takes_points_for_good_and_a_made_one_nothing() {
    let (mut drained, mut held) = (false, false);
    for seed in 1..=400 {
        let mut w = world_on(BESIDE, VICTIM, seed);
        let (player, drinker) = (w.player_id(), ActorId(1));
        let mut expected = w.state.rng.clone();
        let resisted = check(&mut expected, crate::stats::modifier(1), 40);
        let mut events = Vec::new();
        w.after_blow(drinker, player, AttackKind::Melee, &HIT, &mut events);
        assert_eq!(w.state.rng, expected, "the check's two draws, at 100 %");
        let me = w.state.player();
        let stat = (!resisted).then_some(Stat::Strength);
        assert_eq!(
            events,
            vec![Event::Drained {
                actor: player,
                stat
            }]
        );
        let lost = u8::from(!resisted);
        assert_eq!(me.stats.score(Stat::Strength), 12 - lost);
        assert_eq!(
            me.stats.potential(Stat::Strength),
            15,
            "the potential stays"
        );
        assert_eq!(me.drained[Stat::Strength], lost);
        drained |= !resisted;
        held |= resisted;
        if drained && held {
            return;
        }
    }
    panic!("400 seeds: drained {drained}, held {held}");
}

#[test]
fn the_player_is_told_in_words_and_the_loss_stays() {
    let mut w = world_on(BESIDE, VICTIM, 3);
    run(&mut w, "wait*5");
    let words = said(&w);
    let lost = (words.iter())
        .filter(|m| m.contains("You feel your strength drain away!"))
        .count();
    assert!(lost > 0, "{words:?}");
    let me = w.state.player();
    assert_eq!(usize::from(me.drained[Stat::Strength]), lost);
    assert_eq!(
        w.perception().own_scores()[Stat::Strength],
        12 - me.drained[Stat::Strength]
    );
    // It is no stat damage, which would mend a point a day.
    assert!(!me.damage.is_damaged());
}

#[test]
fn no_drain_goes_below_the_floor_nor_below_the_stat_damage() {
    let mut w = world_on(
        BESIDE,
        "stats = { willpower = 1, strength = 6 }\nhp = 1000",
        1,
    );
    let player = w.player_id();
    let drain = Drain {
        stat: Some(Stat::Strength),
        points: 5,
        check: Stat::Willpower,
        difficulty: 40,
    };
    for _ in 0..20 {
        w.drain(player, drain, &mut Vec::new());
    }
    let me = w.state.player();
    assert_eq!(me.stats.score(Stat::Strength), DRAIN_FLOOR);
    assert_eq!(me.drained[Stat::Strength], 6 - DRAIN_FLOOR);
    // At the floor, a failed check takes nothing, and says only the chill.
    let mut events = Vec::new();
    let mut tries = 0;
    while events.is_empty() || matches!(events[0], Event::Drained { stat: Some(_), .. }) {
        events.clear();
        w.drain(player, drain, &mut events);
        tries += 1;
        assert!(tries < 50);
    }
    assert_eq!(w.state.player().stats.score(Stat::Strength), DRAIN_FLOOR);
    // A score carries its stat damage: Dexterity 10 with 6 points of damage keeps 7.
    let me = w.player_mut();
    me.damage.take(Stat::Dexterity, 6, 10);
    let dexterity = Drain {
        stat: Some(Stat::Dexterity),
        ..drain
    };
    for _ in 0..20 {
        w.drain(player, dexterity, &mut Vec::new());
    }
    assert_eq!(w.state.player().stats.score(Stat::Dexterity), 7);
    assert!(World::from_state(w.content().clone(), w.state().clone()).is_ok());
}

#[test]
fn a_toughness_drain_lowers_the_most_hit_points_and_never_kills() {
    let mut w = world_on(BESIDE, "stats = { willpower = 1, toughness = 12 }", 1);
    let player = w.player_id();
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    let drain = Drain {
        stat: Some(Stat::Toughness),
        points: 5,
        check: Stat::Willpower,
        difficulty: 40,
    };
    for _ in 0..10 {
        w.drain(player, drain, &mut Vec::new());
    }
    let me = w.state.player();
    assert_eq!(me.stats.score(Stat::Toughness), DRAIN_FLOOR);
    let now = me.max_hp(w.content());
    assert!(now < max, "{now} < {max}");
    assert_eq!(me.hp, now, "cut down to the new most");
    assert!(me.hp >= 1 && w.state.actors.contains_key(&player));
}

#[test]
fn a_random_drain_draws_its_stat_first() {
    let mut picked = std::collections::BTreeSet::new();
    for seed in 1..=60 {
        let mut w = world_on(
            "#####\n#@l.#\n#####\n",
            "stats = { toughness = 1 }\nhp = 1000",
            seed,
        );
        let (player, leech) = (w.player_id(), ActorId(1));
        let mut expected = w.state.rng.clone();
        let stat = Stat::ALL[expected.below(Stat::COUNT as u32) as usize];
        let resisted = check(&mut expected, crate::stats::modifier(1), 40);
        let before = w.state.player().stats.score(stat);
        w.after_blow(leech, player, AttackKind::Melee, &HIT, &mut Vec::new());
        assert_eq!(w.state.rng, expected);
        let after = w.state.player().stats.score(stat);
        if !resisted && before > DRAIN_FLOOR {
            assert_eq!(after, before.saturating_sub(5).max(DRAIN_FLOOR), "{stat:?}");
            picked.insert(stat);
        }
    }
    assert!(picked.len() >= 4, "{picked:?}");
}

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

#[test]
fn the_pale_drinker_drains_strength_against_willpower() {
    let drain = drain_of(&BUILTIN, "pale_drinker");
    assert_eq!(
        drain,
        Drain {
            stat: Some(Stat::Strength),
            points: 1,
            check: Stat::Willpower,
            difficulty: 14,
        }
    );
}

#[test]
fn talents_raise_the_check_against_a_drain() {
    let drain = drain_of(&BUILTIN, "pale_drinker");
    // How many of 40 seeds hold against the pale drinker's touch with Willpower 10, by the
    // talents held: each must match the check the rules roll.
    let holds = |talents: &[&str]| -> usize {
        let mut held = 0;
        for seed in 1..=40 {
            let mut w = World::new(BUILTIN.clone(), "showcase", Some(seed)).expect("world");
            let player = w.player_id();
            let me = w.player_mut();
            me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
            for id in talents {
                me.talents.take(id, None);
            }
            let bonus = me.talent_mods(&BUILTIN);
            let modifier = bonus.drain_checks + bonus.check(Stat::Willpower);
            let mut expected = w.state.rng.clone();
            let resisted = check(&mut expected, modifier, drain.difficulty);
            let mut events = Vec::new();
            w.drain(player, drain, &mut events);
            let kept = w.state.player().stats.score(Stat::Strength) == 10;
            assert_eq!(kept, resisted, "seed {seed} {talents:?}");
            held += usize::from(resisted);
        }
        held
    };
    let (plain, unbroken, both) = (
        holds(&[]),
        holds(&["unbroken_soul"]),
        holds(&["unbroken_soul", "iron_will"]),
    );
    assert!(
        plain < unbroken && unbroken <= both,
        "{plain} {unbroken} {both}"
    );
    let book = BUILTIN.talents();
    let unbroken_soul = book.id("unbroken_soul").expect("unbroken soul");
    let mods =
        crate::talent::TalentMods::of(&BUILTIN, book.get(unbroken_soul).effects.iter().cloned());
    assert_eq!(mods.drain_checks, 4);
}

#[test]
fn a_temples_mending_gives_drained_points_back() {
    let mut w = world_on(BESIDE, VICTIM, 3);
    run(&mut w, "wait*5");
    let lost = w.state.player().drained[Stat::Strength];
    assert!(lost > 0);
    let player = w.player_id();
    w.mend(player);
    let me = w.state.player();
    assert_eq!(me.stats.score(Stat::Strength), 12);
    assert_eq!(me.stats.potential(Stat::Strength), 15);
    assert!(super::actor::undrained(&me.drained));
}

#[test]
fn drains_are_saved_hashed_and_replayed() {
    let content = content(BESIDE, VICTIM);
    let mut w = World::new(content.clone(), "crypt", Some(5)).expect("world");
    let mut replay = Replay::new(&w);
    let json = serde_json::to_string(w.state()).expect("json");
    assert!(!json.contains("drained"), "left out while none");
    for action in parse_actions("wait*6").expect("ok") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(w.state.player().drained[Stat::Strength] > 0);
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.state_hash(), w.state_hash());
}
