//! Magic crystals in play (docs/design/enchanting.md §5, §10; `world/crystal.rs`): consuming one
//! (the mana capped at the maximum, one Willpower mark a crystal, the step toward evil
//! diminishing, the refusals with no time, the ability's row, a replay), and salvage's affix
//! checks (a crystal of the affix's colour by the check, nothing from unidentified magic,
//! Enchanting's mark, a rare material now and then from a deep rare).

use super::super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::alignment::Score;
use crate::content::{ContentFiles, ObjectId, Rarity};
use crate::magic::{Affix, Magic};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};
use crate::stats::{PerStat, Stat, Stats};
use std::sync::LazyLock;

const ROOM: &str = "\
#######
#.....#
#.@c..#
#.....#
#######
";

static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_crystal.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"c\" = {{ terrain = \"floor\", objects = [\"crystal_blue\", \"crystal_blue\"] }}\n"
        ),
    );
    files.insert(
        "scenarios/test_crystal.toml",
        "name = \"T\"\nstart_level = \"test_crystal\"\n\
         character = { race = \"human\", gender = \"female\", class = \"commoner\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A human commoner alone in a room, carrying nothing, every score 10, `seed` their fate.
fn world(seed: u64) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_crystal", Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    me.alignment = Some(Score::new(0));
    w
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content.object_id(id).expect(id)
}

fn give(w: &mut World, id: &str, count: u32) {
    let stack = Stack::new(object(w, id), count);
    w.player_mut().inventory.push(stack);
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn count(w: &World, id: &str) -> u32 {
    let o = object(w, id);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == o)
        .map(|s| s.count)
        .sum()
}

fn enchanting(w: &World) -> SkillId {
    w.content.skill_id("enchanting").expect("enchanting")
}

#[test]
fn a_crystal_consumed_gives_mana_to_the_maximum_a_willpower_mark_and_a_step_toward_evil() {
    let mut w = world(1);
    give(&mut w, "crystal_red", 2);
    w.player_mut().mana_base = 10;
    let max = w.state.player().max_mana(&w.content);
    assert!(max > 6, "room for the crystal's mana: {max}");
    w.player_mut().mana = 0;
    let marks = w.state.player().stat_marks[Stat::Willpower];
    let before = w.state.time;
    w.apply(&Action::Consume { item: 0 });
    assert!(
        said(
            &w,
            "You crush the red crystal and drink what is in it. Something of it stays."
        ),
        "{:?}",
        texts(&w)
    );
    assert!(w.state.time > before, "a turn");
    assert_eq!(w.state.player().mana, 6);
    assert_eq!(w.state.player().stat_marks[Stat::Willpower], marks + 1);
    assert_eq!(w.state.player().alignment, Some(Score::new(-20)));
    assert_eq!(count(&w, "crystal_red"), 1);
    // Never past the maximum, never refused for a full pool; the last one leaves the pack.
    w.player_mut().mana = max - 2;
    w.apply(&Action::Consume { item: 0 });
    assert_eq!(w.state.player().mana, max);
    assert_eq!(count(&w, "crystal_red"), 0);
    assert!(w.state.player().inventory.is_empty());
}

#[test]
fn the_step_toward_evil_diminishes_for_the_wicked() {
    let mut w = world(2);
    give(&mut w, "crystal_black", 1);
    w.player_mut().alignment = Some(Score::new(-900));
    w.apply(&Action::Consume { item: 0 });
    let after = w.state.player().alignment.expect("score").value();
    assert!((-920..-900).contains(&after), "{after}");
}

#[test]
fn consuming_is_refused_with_no_time_for_anything_but_a_crystal() {
    let mut w = world(3);
    give(&mut w, "heart_crystal", 1);
    give(&mut w, "dagger", 1);
    let before = w.state.time;
    w.apply(&Action::Consume { item: 0 });
    w.apply(&Action::Consume { item: 1 });
    w.apply(&Action::Consume { item: 7 });
    assert_eq!(w.state.time, before);
    assert!(said(&w, "That is no magic crystal."), "{:?}", texts(&w));
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.player().alignment, Some(Score::new(0)));
}

#[test]
fn the_ability_is_everyones_and_greyed_out_without_a_crystal() {
    let row = |w: &World| {
        (w.perception().abilities().iter())
            .find(|a| a.ability == Ability::Consume)
            .map(|a| a.unavailable)
    };
    let mut w = world(4);
    assert_eq!(row(&w), Some(Some(Unavailable::NoCrystal)));
    give(&mut w, "heart_crystal", 1);
    assert_eq!(row(&w), Some(Some(Unavailable::NoCrystal)), "no crystal");
    give(&mut w, "crystal_gold", 1);
    assert_eq!(row(&w), Some(None));
    assert_eq!(
        Ability::Consume.action_on(&w.content, 1),
        Some(Action::Consume { item: 1 })
    );
    assert_eq!(Ability::Consume.id(&w.content), "consume");
}

#[test]
fn a_consume_replays() {
    // As the scenario starts: two blue crystals lie east of the player.
    let mut w = World::new(FIXTURE.clone(), "test_crystal", Some(5)).expect("world");
    let mut replay = Replay::new(&w);
    let start = w.state.player().inventory.len();
    for action in parse_actions("move:E pickup").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let at = (w.state.player().inventory.iter())
        .position(|s| s.object == object(&w, "crystal_blue"))
        .expect("picked up");
    assert_eq!(count(&w, "crystal_blue"), 2, "{start}");
    let text = crate::action::position_text(at);
    for action in parse_actions(&format!("consume:{text} wait consume:{text}")).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(count(&w, "crystal_blue"), 0, "{:?}", texts(&w));
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// A steel long sword of `rarity` at item level `level` with these affixes (id, tier).
fn sword(w: &World, rarity: Rarity, level: u8, affixes: &[(&str, u8)], known: bool) -> Stack {
    let magic = Magic {
        rarity,
        level,
        affixes: (affixes.iter())
            .map(|&(id, tier)| Affix {
                id: id.into(),
                tier,
                value: 1,
            })
            .collect(),
        name: (rarity == Rarity::Rare).then(|| "Grim Bite".into()),
        identified: known,
        carried: 0,
        unique: None,
    };
    Stack::with_magic(object(w, "long_sword"), 1, magic)
}

/// Salvages `stack` with fate `seed` and Enchanting at `skill`: the world after, done.
fn salvage(seed: u64, skill: Option<u16>, make: impl Fn(&World) -> Stack) -> World {
    let mut w = world(seed);
    if let Some(value) = skill {
        let s = enchanting(&w);
        let mut all: Vec<(SkillId, Trained)> = w.state.player().skills.iter().collect();
        all.push((s, Trained { value, marks: 0 }));
        w.player_mut().skills = all.into_iter().collect();
    }
    let stack = make(&w);
    w.player_mut().inventory.push(stack);
    w.apply(&Action::Salvage { item: 0 });
    w
}

#[test]
fn each_affix_of_identified_magic_is_a_crystal_of_its_colour_by_its_check() {
    let (mut caught, mut lost) = (0, 0);
    for seed in 0..60 {
        let w = salvage(seed, None, |w| {
            sword(
                w,
                Rarity::Magic,
                5,
                &[("flaming", 2), ("of_the_bear", 1)],
                true,
            )
        });
        let all = texts(&w);
        assert!(
            said(&w, "You take the Flaming long sword of the Bear apart."),
            "{all:?}"
        );
        let fire = said(&w, "You catch its Flaming magic in a red crystal.");
        let bear = said(&w, "You catch its magic of the Bear in a red crystal.");
        assert_eq!(
            fire,
            !said(&w, "Its Flaming magic goes out unread."),
            "{all:?}"
        );
        assert_eq!(bear, !said(&w, "Its magic of the Bear goes out unread."));
        assert_eq!(count(&w, "crystal_red"), u32::from(fire) + u32::from(bear));
        caught += u32::from(fire) + u32::from(bear);
        lost += u32::from(!fire) + u32::from(!bear);
    }
    assert!(
        caught > 0 && lost > 0,
        "both come of some fate: {caught} {lost}"
    );
}

#[test]
fn unidentified_magic_gives_nothing_and_draws_nothing() {
    let w = salvage(7, None, |w| {
        sword(w, Rarity::Magic, 5, &[("flaming", 2)], false)
    });
    assert!(said(&w, "Its magic goes out unread."), "{:?}", texts(&w));
    assert_eq!(count(&w, "crystal_red"), 0);
    // The same plain sword salvaged with the same fate draws the same materials.
    let plain = salvage(7, None, |w| Stack::one(object(w, "long_sword")));
    let mats = |w: &World| -> Vec<String> {
        (w.state.player().inventory.iter())
            .map(|s| s.describe(&w.content))
            .collect()
    };
    assert_eq!(mats(&w), mats(&plain));
}

#[test]
fn the_trained_enchanter_is_marked_and_a_deep_rare_sometimes_leaves_a_rare_material() {
    let w = salvage(9, Some(10), |w| {
        sword(w, Rarity::Magic, 5, &[("flaming", 2)], true)
    });
    let s = enchanting(&w);
    assert_eq!(w.state.player().skills.get(s).map(|t| t.marks), Some(1));
    let (mut found, mut shallow) = (0, 0);
    for seed in 0..400 {
        let deep = salvage(seed, None, |w| {
            sword(
                w,
                Rarity::Rare,
                10,
                &[("flaming", 2), ("of_the_bear", 1)],
                true,
            )
        });
        found += count(&deep, "heart_crystal") + count(&deep, "soul_fetter");
        let low = salvage(seed, None, |w| {
            sword(
                w,
                Rarity::Rare,
                9,
                &[("flaming", 2), ("of_the_bear", 1)],
                true,
            )
        });
        shallow += count(&low, "heart_crystal") + count(&low, "soul_fetter");
    }
    // 3 % of 400: about 12.
    assert!((2..40).contains(&found), "{found}");
    assert_eq!(shallow, 0, "below item level 10, never");
}
