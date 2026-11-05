//! Making out a scroll (spec §8 "Scrolls"; #1024): who reads at once and who makes the check,
//! what a failure costs and keeps, the marks, and what the rule leaves alone: identify's list,
//! Scribe, and the study odds.

use super::caster_tests::sp;
use super::*;
use crate::action::{ReadTarget, parse_actions};
use crate::check;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::rng::Rng;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::Stat;
use std::collections::BTreeMap;
use std::sync::Mutex;

const SWIMS: &str = "The writing swims before your eyes.";

/// Two scrolls of haste (level 11, read on oneself).
const HASTE: &str = "\"scroll_of_haste*2\"";
/// A scroll of fireball (level 11, aimed).
const FIREBALL: &str = "\"scroll_of_fireball\"";
const RESTORATION: &str = "\"scroll_of_restoration\"";

/// Who reads: untrained in both skills, or at these values.
#[derive(Clone, Copy, Default)]
struct Reader {
    spellcraft: Option<u16>,
    decipher: Option<u16>,
}

const UNTRAINED: Reader = Reader {
    spellcraft: None,
    decipher: None,
};

fn files(carry: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#########\n#.@.....#\n\
         #########\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = 200\ncarry = [{carry}]\n"
        ),
    );
    files
}

/// The content for a pack, loaded once: the tests start hundreds of worlds, one a seed.
fn content(carry: &str) -> Arc<Content> {
    static LOADED: Mutex<BTreeMap<String, Arc<Content>>> = Mutex::new(BTreeMap::new());
    let mut loaded = LOADED.lock().expect("no test panics holding it");
    (loaded.entry(carry.to_string()))
        .or_insert_with(|| Arc::new(Content::load(&files(carry)).expect("test content is valid")))
        .clone()
}

fn world(carry: &str, reader: Reader, seed: u64) -> World {
    let mut w = World::new(content(carry), "hall", None).expect("world");
    train(&mut w, "spellcraft", reader.spellcraft);
    train(&mut w, "decipher_script", reader.decipher);
    w.state.rng = Rng::gameplay(seed);
    w
}

/// Sets the player's skill `id` to `value` with no marks, or leaves them without it.
fn train(w: &mut World, id: &str, value: Option<u16>) {
    let skill = w.content.skill_id(id).expect("a skill");
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != skill).collect();
    if let Some(value) = value {
        all.push((skill, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text == text)
}

fn carried(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

fn marks(w: &World, id: &str) -> u32 {
    let skill = w.content.skill_id(id).expect("a skill");
    (w.state.player().skills.get(skill)).map_or(0, |t| t.marks)
}

/// The first seed from 0 at which `read:a` of [`HASTE`] by `reader` fails (`true`) or is made.
fn seed_where(failed: bool, reader: Reader) -> u64 {
    (0..500)
        .find(|&seed| {
            let mut w = world(HASTE, reader, seed);
            run(&mut w, "read:a");
            said(&w, SWIMS) == failed
        })
        .expect("a seed goes that way")
}

/// The Spellcraft value whose learning is exactly `level`.
fn spellcraft_for(level: i32) -> u16 {
    let mut w = world("", UNTRAINED, 0);
    (0..=200)
        .find(|&v| {
            train(&mut w, "spellcraft", Some(v));
            w.state.player().learning(&w.content) == Some(level)
        })
        .expect("a value teaches exactly this level")
}

#[test]
fn the_numbers_are_the_designs() {
    let w = world("", UNTRAINED, 0);
    let rules = w.content.decipher().expect("the file ships");
    let table = [
        (0, 10, [75, 93, 99]),
        (4, 10, [75, 93, 99]),
        (11, 12, [62, 85, 98]),
        (35, 17, [25, 54, 81]),
        (56, 21, [7, 25, 54]),
        (70, 24, [1, 10, 31]),
    ];
    for (level, difficulty, odds) in table {
        assert_eq!(rules.difficulty(level), difficulty, "level {level}");
        for (modifier, want) in [0, 4, 8].into_iter().zip(odds) {
            let percent = (check::ways(modifier, difficulty) * 100 + 72) / 144;
            assert_eq!(percent, want, "level {level}, modifier {modifier}");
        }
    }
    let skill = w.content.skill_id("decipher_script").expect("the skill");
    let benefit = (w.content.skill(skill).benefits.iter())
        .find(|b| b.effect == Effect::Decipher)
        .expect("the benefit");
    assert_eq!([0, 100, 200].map(|v| benefit.at(v)), [0, 8, 12]);
}

#[test]
fn a_learning_at_or_above_the_level_reads_at_once_and_below_it_does_not() {
    let exact = spellcraft_for(11);
    for value in [exact, 200] {
        let reader = Reader {
            spellcraft: Some(value),
            decipher: Some(10),
        };
        for seed in 0..40 {
            let mut w = world(HASTE, reader, seed);
            let rng = w.state.rng.clone();
            let out = run(&mut w, "read:a");
            assert!(out[0].took_turn);
            assert_eq!(carried(&w, "scroll_of_haste"), 1, "read, at {value}");
            assert!(!said(&w, SWIMS));
            assert_eq!(rng, w.state.rng, "no draw");
            assert_eq!(marks(&w, "decipher_script"), 0, "no check, no mark");
        }
    }
    // One short of it: a check, which some seed fails.
    seed_where(
        true,
        Reader {
            spellcraft: Some(exact - 1),
            decipher: None,
        },
    );
}

#[test]
fn a_failure_takes_a_reads_time_and_keeps_the_scroll_and_its_label() {
    let seed = seed_where(true, UNTRAINED);
    let mut w = world(HASTE, UNTRAINED, seed);
    let out = run(&mut w, "read:a");
    assert!(said(&w, SWIMS));
    assert!(out[0].took_turn);
    assert!(!out[0].elapsed.is_zero(), "a read's time");
    assert_eq!(carried(&w, "scroll_of_haste"), 2, "kept");
    let object = w.content.object_id("scroll_of_haste").expect("object");
    assert!(w.lore().label(object).is_some(), "still under its label");
    assert!(!said(&w, "It is a scroll of haste."));
    let haste = w.content.status_id("hasted").expect("a status");
    assert!(
        !w.state.player().effects.iter().any(|e| e.status == haste),
        "nothing cast"
    );
}

#[test]
fn a_made_check_reads_as_any_other() {
    let seed = seed_where(false, UNTRAINED);
    let mut w = world(HASTE, UNTRAINED, seed);
    run(&mut w, "read:a");
    assert_eq!(carried(&w, "scroll_of_haste"), 1, "one used up");
    assert!(said(&w, "It is a scroll of haste."));
    let object = w.content.object_id("scroll_of_haste").expect("object");
    assert!(w.lore().label(object).is_none(), "its type is known now");
}

#[test]
fn a_refusal_comes_first_with_no_draw_and_no_mark() {
    let carry = format!("{FIREBALL}, {RESTORATION}");
    let reader = Reader {
        spellcraft: None,
        decipher: Some(10),
    };
    let mut w = world(&carry, reader, 1);
    let rng = w.state.rng.clone();
    // No cell, the reader's own cell, and nothing to restore.
    for refused in ["read:a", "read:a:2,1", "read:b"] {
        assert!(!run(&mut w, refused)[0].took_turn, "{refused}");
    }
    assert_eq!(rng, w.state.rng);
    assert_eq!(marks(&w, "decipher_script"), 0);
    assert_eq!(carried(&w, "scroll_of_fireball"), 1);
    assert_eq!(carried(&w, "scroll_of_restoration"), 1);
}

#[test]
fn a_scroll_with_no_spell_counts_as_level_0() {
    let hurt = |reader, seed| {
        let mut w = world(RESTORATION, reader, seed);
        w.player_mut().damage.take(Stat::Strength, 1, 10);
        w
    };
    // Any Spellcraft reads it for certain.
    let reader = Reader {
        spellcraft: Some(0),
        decipher: None,
    };
    for seed in 0..40 {
        let mut w = hurt(reader, seed);
        run(&mut w, "read:a");
        assert!(!said(&w, SWIMS));
        assert_eq!(carried(&w, "scroll_of_restoration"), 0);
        assert!(!w.state.player().damage.is_damaged());
    }
    // Without any, a check, whose failure takes time and mends nothing.
    let failed = (0..500).find_map(|seed| {
        let mut w = hurt(UNTRAINED, seed);
        let out = run(&mut w, "read:a");
        said(&w, SWIMS).then_some((w, out))
    });
    let (w, out) = failed.expect("some seed fails");
    assert!(out[0].took_turn);
    assert_eq!(carried(&w, "scroll_of_restoration"), 1);
    assert!(w.state.player().damage.is_damaged());
}

#[test]
fn every_check_marks_decipher_script_and_spellcraft_gets_nothing() {
    let reader = Reader {
        spellcraft: Some(0),
        decipher: Some(10),
    };
    for failed in [true, false] {
        let mut w = world(HASTE, reader, seed_where(failed, reader));
        run(&mut w, "read:a");
        assert_eq!(marks(&w, "decipher_script"), 1, "failed {failed}");
        assert_eq!(marks(&w, "spellcraft"), 0, "failed {failed}");
    }
}

#[test]
fn decipher_scripts_points_are_on_the_check() {
    let fails = |decipher| {
        (0..300)
            .filter(|&seed| {
                let reader = Reader {
                    spellcraft: None,
                    decipher,
                };
                let mut w = world(HASTE, reader, seed);
                run(&mut w, "read:a");
                said(&w, SWIMS)
            })
            .count()
    };
    let (untrained, trained) = (fails(None), fails(Some(200)));
    // Level 11 (difficulty 12) at +0 and +12: 38 % and 1 %.
    assert!((80..=150).contains(&untrained), "{untrained}");
    assert!(trained < 15, "{trained}");
}

#[test]
fn a_scroll_of_identify_refuses_first_and_a_failure_identifies_nothing() {
    let carry = "\"scroll_of_identify\", \"dagger\"";
    let mut w = world(carry, UNTRAINED, 0);
    let rng = w.state.rng.clone();
    let read_on = |on| Action::Read {
        item: 0,
        target: Some(ReadTarget::Item(on)),
    };
    assert!(
        !w.apply(&read_on(1)).took_turn,
        "a plain dagger tells nothing"
    );
    assert_eq!(rng, w.state.rng, "and no draw");
    let carry = "\"scroll_of_identify\", \"scroll_of_frost\"";
    let (mut made, mut failed) = (false, false);
    for seed in 0..60 {
        let mut w = world(carry, UNTRAINED, seed);
        let frost = w.content.object_id("scroll_of_frost").expect("frost");
        assert!(w.apply(&read_on(1)).took_turn);
        if said(&w, SWIMS) {
            failed = true;
            assert_eq!(carried(&w, "scroll_of_identify"), 1);
            assert!(w.lore().label(frost).is_some(), "still labelled");
        } else {
            made = true;
            assert_eq!(carried(&w, "scroll_of_identify"), 0);
            assert!(w.lore().label(frost).is_none(), "frost is known");
        }
    }
    assert!(made && failed);
}

#[test]
fn scribe_asks_no_check_and_what_it_writes_reads_by_the_rule() {
    let scribe = |reader, seed| {
        let mut w = world("\"blank_scroll\"", reader, seed);
        let me = w.player_mut();
        me.talents.take("scribe", None);
        me.spells.insert(sp("fireball"));
        me.mana = me.mana_base;
        let rng = w.state.rng.clone();
        run(&mut w, "scribe:fireball");
        assert_eq!(rng, w.state.rng, "writing rolls nothing");
        assert_eq!(carried(&w, "scroll_of_fireball"), 1, "written");
        assert_eq!(marks(&w, "decipher_script"), 0);
        w
    };
    let reader = Reader {
        spellcraft: Some(spellcraft_for(11)),
        decipher: None,
    };
    let mut w = scribe(reader, 0);
    run(&mut w, "read:a:6,1");
    assert_eq!(
        carried(&w, "scroll_of_fireball"),
        0,
        "the writer's learning reaches it"
    );
    // A writer whose learning does not reach their own spell still knows it: it reads at once.
    for seed in 0..40 {
        let reader = Reader {
            spellcraft: Some(0),
            decipher: Some(10),
        };
        let mut w = scribe(reader, seed);
        run(&mut w, "read:a:6,1");
        assert!(!said(&w, SWIMS));
        assert_eq!(carried(&w, "scroll_of_fireball"), 0);
        assert_eq!(marks(&w, "decipher_script"), 1, "a known spell still marks");
    }
}

#[test]
fn a_known_spell_succeeds_with_no_draw_and_still_marks() {
    let reader = Reader {
        spellcraft: None,
        decipher: Some(10),
    };
    // Untrained reader who would fail on some seed: the spell they know always reads.
    let seed = seed_where(true, reader);
    let mut w = world(HASTE, reader, seed);
    w.player_mut().spells.insert(sp("haste"));
    let rng = w.state.rng.clone();
    run(&mut w, "read:a");
    assert!(!said(&w, SWIMS));
    assert_eq!(carried(&w, "scroll_of_haste"), 1, "read");
    assert_eq!(rng, w.state.rng, "no draw");
    assert_eq!(marks(&w, "decipher_script"), 1);
    // Untrained in Decipher script: nothing to mark.
    let mut w = world(HASTE, UNTRAINED, seed);
    w.player_mut().spells.insert(sp("haste"));
    run(&mut w, "read:a");
    assert_eq!(carried(&w, "scroll_of_haste"), 1, "read");
    assert_eq!(marks(&w, "decipher_script"), 0);
}

#[test]
fn a_learning_that_reaches_the_level_without_the_spell_keeps_the_bypass() {
    let reader = Reader {
        spellcraft: Some(spellcraft_for(11)),
        decipher: Some(10),
    };
    let mut w = world(HASTE, reader, 0);
    let rng = w.state.rng.clone();
    run(&mut w, "read:a");
    assert_eq!(rng, w.state.rng, "no draw");
    assert_eq!(marks(&w, "decipher_script"), 0, "no mark");
}

#[test]
fn study_odds_read_only_spellcrafts_tenth() {
    let odds = |decipher| {
        let reader = Reader {
            spellcraft: Some(10),
            decipher,
        };
        let w = world(FIREBALL, reader, 0);
        let spell = w.content.spell_id("fireball").expect("fireball");
        w.player_study_odds(spell)
    };
    let check = odds(None).and_then(|o| o.check).expect("a check to read");
    assert_eq!(odds(Some(200)).and_then(|o| o.check), Some(check));
}

#[test]
fn without_the_file_every_scroll_simply_reads() {
    let mut files = files(HASTE);
    files.remove("data/decipher.toml");
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    assert!(content.decipher().is_none());
    let mut w = World::new(content, "hall", None).expect("world");
    let rng = w.state.rng.clone();
    run(&mut w, "read:a");
    assert_eq!(rng, w.state.rng);
    assert!(!said(&w, SWIMS));
    assert_eq!(carried(&w, "scroll_of_haste"), 1);
}

#[test]
fn a_read_by_the_check_saves_and_replays() {
    let mut w = World::new(content(HASTE), "hall", Some(5)).expect("world");
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("read:a*6 wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

/// The seed from 0 at which `decipher:a` by `reader` makes the scroll out (`true`) or fails.
fn seed_deciphering(made: bool, reader: Reader) -> u64 {
    (0..500)
        .find(|&seed| {
            let mut w = world(HASTE, reader, seed);
            run(&mut w, "decipher:a");
            let object = w.content.object_id("scroll_of_haste").expect("object");
            w.lore().label(object).is_none() == made
        })
        .expect("a seed goes that way")
}

#[test]
fn the_ability_learns_the_type_unread_and_a_failure_keeps_the_label() {
    let reader = Reader {
        spellcraft: None,
        decipher: Some(10),
    };
    let object = |w: &World| w.content.object_id("scroll_of_haste").expect("object");
    let mut w = world(HASTE, reader, seed_deciphering(true, reader));
    let out = run(&mut w, "decipher:a");
    assert!(out[0].took_turn);
    assert_eq!(out[0].elapsed.as_seconds(), 600, "ten minutes");
    assert!(w.lore().label(object(&w)).is_none(), "the type is known");
    assert_eq!(carried(&w, "scroll_of_haste"), 2, "kept, unread");
    assert!(said(&w, "You make out the writing: a scroll of haste."));
    assert_eq!(marks(&w, "decipher_script"), 1, "the check's mark");
    // Nothing left to make out: refused, no time.
    let again = run(&mut w, "decipher:a");
    assert!(!again[0].took_turn);

    let mut w = world(HASTE, reader, seed_deciphering(false, reader));
    let out = run(&mut w, "decipher:a");
    assert!(out[0].took_turn, "the time is gone");
    assert_eq!(out[0].elapsed.as_seconds(), 600);
    assert!(said(&w, SWIMS));
    assert!(w.lore().label(object(&w)).is_some(), "still a label");
    assert_eq!(carried(&w, "scroll_of_haste"), 2);
    assert_eq!(marks(&w, "decipher_script"), 1);
}

#[test]
fn the_ability_is_listed_for_the_trained_and_greyed_out_without_a_scroll() {
    use crate::ability::{Ability, Unavailable};
    let reader = Reader {
        spellcraft: None,
        decipher: Some(10),
    };
    let row = |w: &World| {
        (w.perception().abilities().iter())
            .find(|a| a.ability == Ability::Decipher)
            .map(|a| a.unavailable)
    };
    let w = world(HASTE, reader, 0);
    assert_eq!(row(&w), Some(None), "usable with an unknown scroll");
    let w = world("", reader, 0);
    assert_eq!(row(&w), Some(Some(Unavailable::NoUnknownScroll)));
    let w = world(HASTE, UNTRAINED, 0);
    assert_eq!(row(&w), None, "untrained: not listed");
    assert_eq!(
        Ability::Decipher.action_on(&w.content, 3),
        Some(Action::Decipher { item: 3 })
    );
}

#[test]
fn the_ability_is_refused_with_no_time_for_a_non_scroll() {
    let reader = Reader {
        spellcraft: None,
        decipher: Some(10),
    };
    let mut w = world(&format!("{HASTE}, \"dagger\""), reader, 0);
    let before = w.state.time;
    let stack = w
        .state
        .player()
        .inventory
        .iter()
        .position(|s| w.content.object(s.object).read.is_none())
        .expect("a stack that is no scroll");
    let out = run(&mut w, &format!("decipher:{stack}"));
    assert!(!out[0].took_turn);
    assert_eq!(w.state.time, before, "no time");
}
