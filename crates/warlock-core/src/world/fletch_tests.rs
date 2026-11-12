//! Fletching (spec §8 "Fletching"; #1021): anyone with a knife carried and a bow or crossbow ready
//! makes arrows or bolts, for ten minutes that an interruption wastes; a Dexterity check plus
//! Woodcraft scales the count; under Woodcraft 25 they come out brittle; a ranger's Fletcher does
//! far better.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, ObjectId};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::{PerStat, Stats};
use crate::world::RecipeLack;
use std::sync::LazyLock;

const ROOM: &str = "\
###########
#.........#
#.@.......#
#.........#
#.......k.#
###########
";

/// The built-in content, loaded once (the tests below run batches by the hundred).
static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// The built-in content with [`ROOM`] twice: with no one in it, and with a kobold.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, k) in [
        ("test_alone", "\"floor\""),
        (
            "test_watched",
            "{ terrain = \"floor\", creature = \"kobold\" }",
        ),
    ] {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!(
                "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n"
            ),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"T\"\nstart_level = \"{name}\"\n\
                 character = {{ race = \"human\", gender = \"female\", class = \"commoner\" }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A hall with no one in it, or with a kobold in sight. The player is a human commoner carrying
/// nothing, every score 10 (a modifier of 0), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    set_woodcraft(&mut w, None);
    w
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content.object_id(id).expect(id)
}

fn woodcraft(w: &World) -> SkillId {
    w.content.skill_id("woodcraft").expect("Woodcraft")
}

/// The player trained in Woodcraft at `value` (`None`: untrained).
fn set_woodcraft(w: &mut World, value: Option<u16>) {
    let id = woodcraft(w);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

/// One of object `id` into the pack, at its end.
fn give(w: &mut World, id: &str) -> usize {
    let stack = Stack::one(object(w, id));
    let me = w.player_mut();
    me.inventory.push(stack);
    me.inventory.len() - 1
}

/// Object `id` ready in the missile weapon slot.
fn ready(w: &mut World, id: &str) {
    let item = give(w, id);
    let slot = w.content.slot_id("missile_weapon").expect("slot");
    w.player_mut().equipment.put(&[slot], item);
}

/// A knife in the pack, and a short bow ready.
fn equipped(kobold: bool, seed: u64) -> World {
    let mut w = world(kobold, seed);
    give(&mut w, "knife");
    ready(&mut w, "short_bow");
    w
}

fn count_of(w: &World, id: &str) -> u32 {
    let id = object(w, id);
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// The Fletch recipe's row on Woodcraft's screen: whether it is listed, and why it cannot be
/// made now.
fn row(w: &World) -> Option<Option<RecipeLack>> {
    (w.perception().recipes(woodcraft(w)).into_iter())
        .find(|r| r.name == "Fletch")
        .map(|r| r.unavailable)
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text == text)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn seconds_of(w: &mut World) -> u64 {
    let start = w.state.time;
    w.apply(&Action::Fletch);
    w.state.time.since(start).as_seconds()
}

/// The player has the Fletcher talent, as a ranger has.
fn grant_fletcher(w: &mut World) {
    w.player_mut().talents.take("fletcher", None);
}

/// A level-one ranger with their kit, in the showcase.
fn ranger(seed: u64) -> World {
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "healthy".into()]);
    World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

#[test]
fn a_dagger_a_stiletto_or_a_knife_is_a_knife_and_a_short_sword_is_not() {
    let w = world(false, 1);
    for (id, knife) in [
        ("dagger", true),
        ("stiletto", true),
        ("knife", true),
        ("short_sword", false),
        ("scimitar", false),
        ("long_sword", false),
        ("short_bow", false),
    ] {
        assert_eq!(w.content.is_knife(object(&w, id)), knife, "{id}");
    }
    // In the pack, not in hand.
    let mut w = world(false, 1);
    ready(&mut w, "short_bow");
    give(&mut w, "dagger");
    assert_eq!(row(&w), Some(None), "a dagger in the pack does");
    let mut w = world(false, 1);
    ready(&mut w, "short_bow");
    give(&mut w, "short_sword");
    assert_eq!(row(&w), Some(Some(RecipeLack::Knife)));
}

#[test]
fn each_reason_is_the_rows_and_the_refusal_with_no_time() {
    // Nothing at all: the knife is asked for first.
    let mut w = world(false, 1);
    assert_eq!(row(&w), Some(Some(RecipeLack::Knife)), "anyone has the row");
    let out = w.apply(&Action::Fletch);
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "You have no knife.");
    assert_eq!(Unavailable::NoKnife.reason(&w.content), "you have no knife");
    // A knife, no launcher.
    give(&mut w, "knife");
    assert_eq!(row(&w), Some(Some(RecipeLack::Launcher)));
    let out = w.apply(&Action::Fletch);
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "You have no bow or crossbow ready.");
    assert_eq!(
        Unavailable::NoLauncher.reason(&w.content),
        "you have no bow or crossbow ready"
    );
    // A sling is a launcher, of nothing to fletch.
    ready(&mut w, "sling");
    assert_eq!(row(&w), Some(Some(RecipeLack::Launcher)));
    // A bow in the pack is not ready.
    let mut w = world(false, 1);
    give(&mut w, "knife");
    give(&mut w, "short_bow");
    assert_eq!(row(&w), Some(Some(RecipeLack::Launcher)));
    // Ready, it may be made; and Woodcraft's profession row is there for the untrained too.
    let w = equipped(false, 1);
    assert_eq!(row(&w), Some(None));
    let wood = woodcraft(&w);
    assert!(
        (w.perception().abilities().iter()).any(|a| a.ability == Ability::Profession(wood)),
        "Woodcraft is listed as its profession"
    );
}

#[test]
fn a_batch_takes_ten_minutes_and_uses_nothing_up() {
    let mut w = equipped(false, 3);
    assert_eq!(seconds_of(&mut w), 600);
    assert_eq!(count_of(&w, "knife"), 1);
    assert_eq!(count_of(&w, "short_bow"), 1);
}

#[test]
fn bows_make_arrows_and_crossbows_bolts() {
    let made = |launcher: &str, value: Option<u16>, id: &str| {
        (0..40)
            .map(|seed| {
                let mut w = world(false, seed);
                give(&mut w, "knife");
                ready(&mut w, launcher);
                set_woodcraft(&mut w, value);
                w.apply(&Action::Fletch);
                count_of(&w, id)
            })
            .sum::<u32>()
    };
    assert!(made("short_bow", None, "brittle_arrow") > 0);
    assert!(made("crossbow", None, "brittle_bolt") > 0);
    assert!(made("short_bow", Some(25), "arrow") > 0);
    assert!(made("crossbow", Some(25), "bolt") > 0);
    // A horn composite bow counts: its own material does not matter.
    assert!(made("composite_bow", None, "brittle_arrow") > 0);
    assert_eq!(made("short_bow", None, "arrow"), 0, "brittle under 25");
    assert_eq!(made("short_bow", Some(24), "arrow"), 0, "brittle under 25");
    assert_eq!(
        made("short_bow", Some(25), "brittle_arrow"),
        0,
        "plain from 25"
    );
}

/// What a batch makes (of any kind) by `seed`, the player's Dexterity modifier set by `dexterity`.
fn batch(seed: u64, dexterity: u8, value: Option<u16>) -> u32 {
    let mut w = equipped(false, seed);
    let mut scores = PerStat::splat(10);
    scores[crate::stats::Stat::Dexterity] = dexterity;
    w.player_mut().stats = Stats::new(scores, scores);
    set_woodcraft(&mut w, value);
    w.apply(&Action::Fletch);
    count_of(&w, "arrow") + count_of(&w, "brittle_arrow")
}

#[test]
fn the_check_scales_the_count_by_its_bands() {
    // Dexterity so high no roll falls short: the whole count, `1d4`.
    let high = (0..200).map(|s| batch(s, 60, None)).collect::<Vec<u32>>();
    assert!(high.iter().all(|&n| (1..=4).contains(&n)), "{high:?}");
    for n in 1..=4 {
        assert!(high.contains(&n), "{n} of {high:?}");
    }
    // So low none reaches the last band: none at all, and a message for it.
    let low = (0..200).map(|s| batch(s, 1, None)).collect::<Vec<u32>>();
    let none = low.iter().filter(|&&n| n == 0).count();
    assert!(none > 0, "{low:?}");
    assert!(none > high.iter().filter(|&&n| n == 0).count());
    let mut w = equipped(false, 0);
    for seed in 0..200 {
        w = equipped(false, seed);
        let mut scores = PerStat::splat(10);
        scores[crate::stats::Stat::Dexterity] = 1;
        w.player_mut().stats = Stats::new(scores, scores);
        w.apply(&Action::Fletch);
        if count_of(&w, "brittle_arrow") == 0 {
            break;
        }
    }
    assert_eq!(last_message(&w), "You spoil every shaft you cut.");
    // Even, a mix: more than one way, and between the two.
    let even: u32 = (0..200).map(|s| batch(s, 10, None)).sum();
    assert!(even < high.iter().sum::<u32>(), "{even}");
    assert!(even > low.iter().sum::<u32>(), "{even}");
    // Woodcraft adds one for each 25, once the check allows the count.
    let trained = (0..200)
        .map(|s| batch(s, 60, Some(100)))
        .collect::<Vec<u32>>();
    assert!(trained.iter().all(|&n| (5..=8).contains(&n)), "{trained:?}");
}

#[test]
fn woodcrafts_points_help_the_check() {
    let w = |value| {
        let mut w = world(false, 1);
        set_woodcraft(&mut w, value);
        w.state.player().skill_effect(&w.content, Effect::Fletch)
    };
    assert_eq!(w(None), 0, "the untrained have none");
    assert_eq!(w(Some(100)), 8);
    assert_eq!(w(Some(200)), 12);
    // Below the top band only for those who need the points: more Woodcraft, more made, beyond
    // the count it adds.
    let wasted = |value| (0..200).filter(|&s| batch(s, 4, value) == 0).count();
    assert!(wasted(Some(100)) < wasted(None));
}

#[test]
fn fletcher_is_half_the_time_twice_the_count_a_better_check_and_never_brittle() {
    // Only a ranger has it, from the start, with the knife in the pack, not in hand.
    let mut r = ranger(5);
    assert!(r.state.player().talents.has("fletcher"), "a ranger's");
    let me = r.state.player();
    let knife = object(&r, "knife");
    let at = (me.inventory.iter().position(|s| s.object == knife)).expect("in the kit");
    let hand = r.content.slot_id("main_hand").expect("slot");
    assert_ne!(me.equipment.get(hand), Some(at), "carried, not wielded");
    let mods = me.talent_mods(&r.content);
    let numbers = mods.fletcher.expect("the effect");
    assert_eq!(
        (numbers.check, numbers.count_times, numbers.time_percent),
        (4, 2, 50)
    );
    let _ = me;
    // A ranger's first batch: five minutes.
    assert_eq!(row(&r), Some(None));
    assert_eq!(seconds_of(&mut r), 300);
    assert!(said(
        &r,
        "You sit down with your knife and some straight wood."
    ));

    let class_of = |class: &str| -> Vec<String> {
        let def = (BUILTIN.classes().iter())
            .find(|c| c.id == class)
            .expect(class);
        (def.abilities.iter()).map(|(_, id)| id.clone()).collect()
    };
    assert_eq!(
        class_of("ranger").first().map(String::as_str),
        Some("fletcher")
    );
    for class in BUILTIN.classes() {
        if class.id != "ranger" {
            assert!(
                !class_of(&class.id).contains(&"fletcher".to_string()),
                "{}",
                class.id
            );
        }
    }

    // The same character and fate with and without it (Woodcraft 0): plain at any Woodcraft, and
    // what it makes from a certain check is exactly twice.
    let with = |seed, fletches: bool| {
        let mut w = equipped(false, seed);
        let mut scores = PerStat::splat(10);
        scores[crate::stats::Stat::Dexterity] = 60;
        w.player_mut().stats = Stats::new(scores, scores);
        if fletches {
            grant_fletcher(&mut w);
        }
        let seconds = seconds_of(&mut w);
        (
            seconds,
            count_of(&w, "arrow"),
            count_of(&w, "brittle_arrow"),
        )
    };
    let mut twice = 0;
    for seed in 0..50 {
        let (slow, _, brittle) = with(seed, false);
        let (quick, plain, none) = with(seed, true);
        assert_eq!((slow, quick), (600, 300));
        assert_eq!(none, 0, "never brittle");
        assert_eq!(plain % 2, 0, "twice the count: {plain}");
        assert!((2..=8).contains(&plain), "{plain}");
        twice += u32::from(brittle > 0 && plain > 0);
    }
    assert_eq!(twice, 50);

    // +4 on the check: a character for whom a batch is sometimes wasted wastes fewer.
    let wasted = |fletches: bool| {
        (0..200)
            .filter(|&seed| {
                let mut w = equipped(false, seed);
                let mut scores = PerStat::splat(10);
                scores[crate::stats::Stat::Dexterity] = 4;
                w.player_mut().stats = Stats::new(scores, scores);
                if fletches {
                    grant_fletcher(&mut w);
                }
                w.apply(&Action::Fletch);
                count_of(&w, "arrow") + count_of(&w, "brittle_arrow") == 0
            })
            .count()
    };
    assert!(
        wasted(true) < wasted(false),
        "{} {}",
        wasted(true),
        wasted(false)
    );
}

#[test]
fn an_interrupted_batch_makes_nothing_and_spends_the_time() {
    let mut w = equipped(true, 1);
    let start = w.state.time;
    w.apply(&Action::Fletch);
    let spent = w.state.time.since(start).as_seconds();
    assert!(spent > 0 && spent < 600, "{spent}");
    assert_eq!(count_of(&w, "arrow") + count_of(&w, "brittle_arrow"), 0);
    assert!(said(&w, "You put down the knife.") || last_message(&w).contains("put down the knife"));
}

#[test]
fn a_batch_trains_woodcraft_for_those_trained() {
    let mut w = equipped(false, 1);
    set_woodcraft(&mut w, Some(10));
    let marks = |w: &World| w.state.player().skills.get(woodcraft(w)).map(|t| t.marks);
    assert_eq!(marks(&w), Some(0));
    w.apply(&Action::Fletch);
    assert_eq!(marks(&w), Some(1), "every batch");
    // A refused one trains nothing; the untrained stay so.
    let mut w = world(false, 1);
    set_woodcraft(&mut w, Some(10));
    w.apply(&Action::Fletch);
    assert_eq!(marks(&w), Some(0));
    let mut w = equipped(false, 1);
    w.apply(&Action::Fletch);
    assert_eq!(marks(&w), None);
}

#[test]
fn fletching_replays_verify() {
    let mut w = ranger(9);
    let mut replay = Replay::new(&w);
    for action in parse_actions("fletch fletch move:N fletch").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(BUILTIN.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
