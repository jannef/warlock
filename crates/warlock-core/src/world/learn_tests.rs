//! Learning spells from grimoires (spec §8 "Spells": "Learning"): refusals, the check by
//! overreach and known levels, the time, interruption, the grimoire kept, mishaps by band, saves
//! and replays. Deepening a known spell by reading its grimoire again ("Ranks", #1264) is
//! `spell_rank_tests.rs`.

use super::caster_tests::sp;
use super::learn::Lesson;
use super::*;
use crate::action::parse_actions;
use crate::content::{Band, ContentFiles, drawn_among};
use crate::replay::Replay;
use crate::test_support::builtin;
pub(super) use crate::test_support::last;
pub(super) use crate::test_support::run;

/// The pack: a, two grimoires of fireball (Fireball, level 11); b, one of lightning (Lightning
/// bolt, level 11); c, two of frost (Frost, level 6); d, one of identify; e, a dagger; f, two
/// scrolls of frost.
const PACK: &str = "carry = [\"grimoire_of_fireball*2\", \"grimoire_of_lightning\", \
     \"grimoire_of_frost*2\", \"grimoire_of_identify\", \"dagger\", \"scroll_of_frost*2\"]";

/// A lit hall of `map` (`@` the player, `r` a giant rat), the player with `mana` base mana.
pub(super) fn world_with(map: &str, mana: u16) -> World {
    seeded(map, mana, None)
}

pub(super) fn seeded(map: &str, mana: u16, seed: Option<u64>) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = {mana}\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "hall", seed).expect("world");
    // Trained in Decipher script, at nothing: learning's floor of 1.
    set_decipher(&mut w, 0);
    w
}

/// The player is trained in Decipher script, the skill of learning (#1270), at `value`.
pub(super) fn set_decipher(w: &mut World, value: u16) {
    let decipher = decipher(w);
    let skills = &mut w.player_mut().skills;
    let by = i32::from(value) - skills.value(decipher).map_or(0, i32::from);
    skills.add(decipher, by);
}

fn decipher(w: &World) -> crate::skills::SkillId {
    w.content.skill_id("decipher_script").expect("skill")
}

pub(super) const HALL: &str = "#######\n#.....#\n#..@..#\n#.....#\n#######\n";

fn world() -> World {
    world_with(HALL, 10)
}

pub(super) fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// Knows every scroll type carried (the dev identify).
pub(super) fn identified(mut w: World) -> World {
    run(&mut w, "identify");
    w
}

#[test]
fn refusals_take_no_time_and_keep_the_grimoire() {
    let mut w = world();
    let refused = |w: &mut World, action: &str, said: &str| {
        let out = run(w, action).remove(0);
        assert!(!out.took_turn, "{action}");
        assert_eq!(last(w), said, "{action}");
    };
    refused(&mut w, "read:z", "You carry no such thing.");
    refused(&mut w, "read:e", "You cannot read a dagger.");
    refused(
        &mut w,
        "read:c:3,1",
        "A grimoire is read, not aimed: `read:a`.",
    );
    // The grimoire of identify teaches Identify (#592); known at the last rank, it has nothing
    // more to teach (below it, a reading deepens the spell: `spell_rank_tests.rs`).
    let mut knows = w.clone();
    knows.player_mut().spells.insert(sp("identify"), 10);
    refused(&mut knows, "read:d", "You know all there is of Identify.");
    w.player_mut().spells.insert(sp("frost"), 10);
    refused(&mut w, "read:c", "You know all there is of Frost.");
    assert_eq!(count(&w, "grimoire_of_frost"), 2, "kept");
    // No mana at all: no gift for magic.
    let mut w = identified(world_with(HALL, 0));
    w.player_mut().mana_base = -100;
    refused(&mut w, "read:c", "You have no gift for magic.");
}

#[test]
fn the_check_grows_with_overreach_and_the_levels_known() {
    let w = world();
    let content = w.content.clone();
    let me = w.state.player().clone();
    let odds = |me: &Actor, value: u16, spell| {
        let mut me = me.clone();
        let decipher = content.skill_id("decipher_script").expect("skill");
        me.skills.add(decipher, i32::from(value));
        study_odds(&me, &content, None, spell).expect("odds")
    };
    let int = me
        .effective_stats(&content)
        .modifier(crate::stats::Stat::Intelligence);
    // (Decipher script, spell): overreach in spell levels and difficulty (12 and a point a level
    // over), nothing known. Learning: 1 at 15, 2 at 20, 3 at 25, 6 at 40, 12 at 64.
    let table = [
        (15, sp("evil_eye"), -1, None),
        (15, sp("spark"), 0, None),
        (40, sp("frost"), 0, None),
        (64, sp("frost"), -6, None),
        (15, sp("blessing"), 2, Some(14)),
        (15, sp("frost"), 5, Some(17)),
        (15, sp("fireball"), 10, Some(22)),
        (25, sp("lightning_bolt"), 8, Some(20)),
        (20, sp("haste"), 9, Some(21)),
    ];
    for (value, spell, overreach, difficulty) in table {
        let o = odds(&me, value, spell);
        assert_eq!(o.overreach, overreach, "{spell:?} at {value}");
        assert_eq!(o.check.map(|c| c.1), difficulty, "{spell:?} at {value}");
        if let Some((bonus, _)) = o.check {
            let tenth = i32::from(value) / 10;
            assert_eq!(
                bonus,
                int + tenth,
                "Intelligence and Decipher script's tenth"
            );
        } else {
            assert_eq!(o.ways(), 144, "certain");
        }
    }
    // Twenty levels known make it a point harder; harder is less likely.
    let mut learned = me.clone();
    learned
        .spells
        .extend([sp("call_lightning"), sp("fireball"), sp("frost")].map(|s| (s, 1)));
    learned
        .spells
        .extend([sp("blessing"), sp("mend")].map(|s| (s, 1)));
    let (before, after) = (
        odds(&me, 0, sp("hex_of_weakness")),
        odds(&learned, 0, sp("hex_of_weakness")),
    );
    assert_eq!(
        after.check.map(|c| c.1),
        Some(15),
        "11 + 11 + 6 + 3 + 1 = 32 levels: a point"
    );
    assert!(after.ways() < before.ways());
    // Quick study's bonus and Decipher script's tenth count.
    let mut studious = me.clone();
    studious.talents.take("quick_study", None);
    let o = odds(&studious, 59, sp("call_lightning"));
    assert_eq!(o.check.map(|c| c.0), Some(int + 2 + 5));
}

#[test]
fn a_spell_within_reach_is_learned_for_certain_in_its_time() {
    let mut w = identified(world());
    // Learning 6: Frost's level.
    set_decipher(&mut w, 40);
    assert!(w.state.player().max_mana(&w.content) > 0);
    let hash = w.state_hash();
    let out = run(&mut w, "read:c").remove(0);
    assert_ne!(w.state_hash(), hash);
    // Frost is level 6: 100 + 6 × 20 = 220 turns, 22 minutes.
    assert_eq!(out.elapsed, crate::time::Duration::minutes(22));
    assert!(w.state.player().spells.contains_key(&sp("frost")));
    assert_eq!(count(&w, "grimoire_of_frost"), 2, "kept");
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "You begin to read the grimoire of frost.")
    );
    assert!(
        said.iter().any(|m| m.ends_with("you have learned Frost.")),
        "{said:?}"
    );
    // It is in the abilities menu at rank 1, and the grimoire now deepens it (a reading twice as
    // long, for rank 2; `spell_rank_tests.rs`).
    let abilities = w.perception().abilities();
    assert!(
        abilities
            .iter()
            .any(|a| a.ability == crate::ability::Ability::Spell(sp("frost")))
    );
    assert_eq!(
        w.state.player().spell_rank(&w.content, sp("frost")),
        Some(1)
    );
    let again = run(&mut w, "read:c").remove(0);
    assert!(again.took_turn);
    assert_eq!(again.elapsed, crate::time::Duration::minutes(44));
    // A scroll of a known spell still reads.
    assert!(run(&mut w, "read:f:3,1")[0].took_turn);
}

#[test]
fn quick_study_halves_the_time() {
    let mut w = identified(world());
    set_decipher(&mut w, 40);
    w.player_mut().talents.take("quick_study", None);
    let out = run(&mut w, "read:c").remove(0);
    assert_eq!(out.elapsed, crate::time::Duration::minutes(11));
}

#[test]
fn a_blow_interrupts_the_study() {
    let map = "#######\n#.....#\n#..@r.#\n#.....#\n#######\n";
    let mut w = identified(world_with(map, 10));
    set_decipher(&mut w, 40);
    let out = run(&mut w, "read:c").remove(0);
    assert!(out.took_turn);
    assert!(out.elapsed < crate::time::Duration::minutes(22));
    assert_eq!(last(&w), "You stop reading.");
    assert_eq!(count(&w, "grimoire_of_frost"), 2, "kept");
    assert!(!w.state.player().spells.contains_key(&sp("frost")));
}

#[test]
fn far_beyond_the_reader_a_study_fails_into_a_mishap() {
    // Lightning bolt at learning 1 (overreach 10, a bad mishap's band) against 22.
    let mut failed = 0;
    for seed in 0..8 {
        let mut w = identified(seeded(HALL, 10, Some(seed)));
        run(&mut w, "read:b");
        assert_eq!(count(&w, "grimoire_of_lightning"), 1, "kept either way");
        let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
        if said.iter().any(|m| m == "The words twist away from you.") {
            failed += 1;
            // A bad mishap, or a lucky one.
            let messages: Vec<&str> = drawn_among(w.content.mishaps(), Band::Bad)
                .map(|m| m.message.as_str())
                .collect();
            assert!(
                said.iter().any(|m| messages.contains(&m.as_str())),
                "{said:?}"
            );
        }
    }
    assert!(failed >= 6, "{failed}");
}

#[test]
fn mishaps_are_drawn_by_band_beside_the_lucky_ones() {
    let mut w = world();
    let content = w.content.clone();
    for band in [Band::Slight, Band::Bad, Band::Terrible] {
        let among: Vec<Band> = drawn_among(content.mishaps(), band)
            .map(|m| m.band)
            .collect();
        assert!(among.contains(&band) && among.contains(&Band::Lucky));
        assert!(among.iter().all(|&b| b == band || b == Band::Lucky));
        for _ in 0..40 {
            let m = w.draw_mishap(band).expect("drawn");
            assert!(m.band == band || m.band == Band::Lucky);
        }
    }
    assert_eq!(
        w.state.rng,
        {
            let mut again = world();
            for band in [Band::Slight, Band::Bad, Band::Terrible] {
                for _ in 0..40 {
                    again.draw_mishap(band);
                }
            }
            again.state.rng
        },
        "one draw each, deterministic"
    );
}

#[test]
fn each_mishap_does_what_it_says() {
    let mishap = |w: &World, id: &str| {
        (w.content.mishaps().iter())
            .find(|m| m.id == id)
            .cloned()
            .expect("mishap")
    };
    let lesson = |w: &World, object: &str, spell| Lesson {
        item: 0,
        object: w.content.object_id(object).expect("object"),
        spell,
        rank: None,
        turns: 1,
    };
    let mut w = identified(world());
    let mut events = Vec::new();
    let me = w.state.player;
    let frost = lesson(&w, "grimoire_of_frost", sp("frost"));
    // A status.
    let m = mishap(&w, "swimming_head");
    w.mishap(&m, frost, &mut events);
    let sluggish = w.content.status_id("sluggish").expect("status");
    assert!(w.state.player().effects.has(sluggish));
    assert_eq!(last(&w).as_str(), m.message.as_str());
    // Mana.
    w.player_mut().mana = 8;
    w.mishap(&mishap(&w, "mana_drain"), frost, &mut events);
    assert_eq!(w.state.player().mana, 2);
    // Marks lost and won.
    w.player_mut().stat_marks[crate::stats::Stat::Strength] = 1;
    w.mishap(&mishap(&w, "forgetting"), frost, &mut events);
    assert_eq!(w.state.player().stat_marks[crate::stats::Stat::Strength], 0);
    w.mishap(&mishap(&w, "hard_lesson"), frost, &mut events);
    assert_eq!(
        w.state.player().stat_marks[crate::stats::Stat::Intelligence],
        1
    );
    // Learned anyway.
    w.mishap(&mishap(&w, "learned_anyway"), frost, &mut events);
    assert!(w.state.player().spells.contains_key(&sp("frost")));
    // A teleport moves the reader.
    let from = w.player_pos();
    w.mishap(&mishap(&w, "twisted_space"), frost, &mut events);
    assert_ne!(w.player_pos(), from);
    // A summoned hostile stands next to them.
    let before = w.state.actors.len();
    w.mishap(&mishap(&w, "rift"), frost, &mut events);
    assert_eq!(w.state.actors.len(), before + 1);
    let summoned = w
        .state
        .actors
        .values()
        .find(|a| a.id != me)
        .expect("summoned");
    assert_eq!(summoned.pos.chebyshev(w.player_pos()), 1);
    assert_eq!(w.content.creature(summoned.creature).id, "dretch");
    // The grimoire's own spell goes off on its reader.
    let hp = w.state.player().hp;
    let fire = lesson(&w, "grimoire_of_fireball", sp("fireball"));
    w.mishap(&mishap(&w, "eruption"), fire, &mut events);
    let lost = hp - w.state.player().hp;
    assert!(lost > 0, "burnt");
}

#[test]
fn learned_spells_survive_saves_and_replays() {
    let mut w = identified(world());
    set_decipher(&mut w, 40);
    let empty = serde_json::to_string(w.state.player()).expect("json");
    assert!(!empty.contains("spells"), "left out while empty");
    run(&mut w, "read:c");
    let json = serde_json::to_string(w.state.player()).expect("json");
    let saved = format!("\"spells\":{{\"{}\":1}}", sp("frost").0);
    assert!(json.contains(&saved), "by index, with the rank: {json}");
    let back: Actor = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.spells, w.state.player().spells);
    let content = w.content.clone();
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // Replays: recorded and played back to the same state, by a character trained by their class.
    let wizard: CharacterSpec = "human,female,wizard".parse().expect("a character");
    let mut w = World::with_character(content.clone(), "hall", None, Some(&wizard)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("identify read:b wait read:c").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn the_dumps_say_the_spells_known_and_the_learning() {
    use crate::dump::{json, layer_rows, parse_layers};
    let spells = parse_layers("spells").expect("layer")[0];
    let progress = parse_layers("progress").expect("layer")[0];
    let mut w = world();
    assert_eq!(
        layer_rows(&w, spells),
        vec!["no spells"],
        "grimoires are no spells"
    );
    w.player_mut().spells.insert(sp("blessing"), 1);
    let rows = layer_rows(&w, spells);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0].starts_with("Blessing (level 3, "), "{rows:?}");
    let dev = layer_rows(&w, progress);
    assert!(
        dev.iter().any(|r| r == "learning 1 (Decipher script 0)"),
        "{dev:?}"
    );
    let value = json(&w, &[spells], false);
    assert_eq!(value["spells"][0]["id"], "blessing");
    assert_eq!(value["spells"][0]["level"], 3);
    assert!(value.get("studies").is_none());
    // Untrained: the dev layer says why.
    w.player_mut().skills = crate::skills::Skills::default();
    let dev = layer_rows(&w, progress);
    assert!(
        dev.iter()
            .any(|r| r == "cannot learn: not trained in Decipher script"),
        "{dev:?}"
    );
}

#[test]
fn the_read_list_tells_the_chance_of_each_grimoire_to_learn() {
    let mut w = world();
    w.player_mut().spells.insert(sp("identify"), 10);
    let studies = w.perception().studies();
    // Fireball, lightning, and frost, each to learn; not identify, known at its last rank.
    let items: Vec<usize> = studies.iter().map(|s| s.item).collect();
    assert_eq!(items, vec![0, 1, 2]);
    assert!(studies.iter().all(|s| s.rank.is_none()));
    assert_eq!(studies[2].chance, "a struggle");
    // Known below it, identify's grimoire is listed for its next rank.
    w.player_mut().spells.insert(sp("identify"), 3);
    let studies = w.perception().studies();
    assert_eq!(studies[3].item, 3);
    assert_eq!(studies[3].rank, Some(4));
}

/// A ring of the Scholar worth `value` on the player's left hand.
fn wear_scholar(w: &mut World, value: i32) {
    use crate::magic::{Affix, Magic, Rarity};
    let content = w.content.clone();
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: "of_the_scholar".into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = content.object_id("iron_ring").expect("object");
    let slot = content.slot_id("left_ring").expect("slot");
    let me = w.player_mut();
    me.inventory.push(Stack::with_magic(object, 1, magic));
    me.equipment.put(&[slot], me.inventory.len() - 1);
}

#[test]
fn learning_is_decipher_scripts_curve_at_least_one_for_the_trained() {
    let mut w = world();
    let learning = |w: &World| w.state.player().learning(&w.content);
    // The design's table: 0 below 10, 10 at 60, 30 at 100, 70 from 150; 0 is floored to 1.
    let table = [
        (0, 1),
        (9, 1),
        (10, 1),
        (17, 1),
        (22, 2),
        (27, 3),
        (35, 5),
        (50, 8),
        (60, 10),
        (75, 17),
        (90, 25),
        (100, 30),
        (108, 36),
        (110, 38),
        (125, 50),
        (140, 62),
        (150, 70),
        (999, 70),
    ];
    for (value, reach) in table {
        set_decipher(&mut w, value);
        assert_eq!(learning(&w), Some(reach), "Decipher script {value}");
    }
    // Not trained: none at all, which is not the floor.
    w.player_mut().skills = crate::skills::Skills::default();
    assert_eq!(learning(&w), None);
}

#[test]
fn study_odds_read_decipher_script_a_priest_at_17_a_wizard_at_108() {
    let mut w = world();
    let content = w.content.clone();
    let odds = |w: &World, spell| study_odds(w.state.player(), &content, None, spell);
    set_decipher(&mut w, 17);
    assert_eq!(
        odds(&w, sp("spark")).expect("odds").check,
        None,
        "a cantrip"
    );
    assert_eq!(odds(&w, sp("fireball")).expect("odds").overreach, 10);
    set_decipher(&mut w, 108);
    let far = odds(&w, sp("fireball")).expect("odds");
    assert_eq!((far.overreach, far.check), (-25, None), "learning 36");
    assert_eq!(odds(&w, sp("call_lightning")).expect("odds").overreach, -25);
    // A hard spell (the druid's calls) is a check at any learning, its overreach not counted
    // when the learning passes its level.
    for (spell, hard) in [("call_of_the_forest", 10), ("call_the_elements", 12)] {
        let def = content.spell(sp(spell));
        assert_eq!(def.study_difficulty, hard, "{spell}");
        set_decipher(&mut w, 999);
        let (_, top) = odds(&w, sp(spell)).expect("odds").check.expect("a check");
        assert_eq!(top, 12 + hard, "{spell} at learning 70");
    }
}

#[test]
fn a_character_not_trained_in_decipher_script_cannot_study_cantrips_included() {
    let mut w = identified(world());
    w.player_mut().skills = crate::skills::Skills::default();
    let (time, spells) = (w.state.time, w.state.player().spells.clone());
    let out = run(&mut w, "read:c").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last(&w), UNTRAINED);
    assert_eq!(w.state.time, time, "no time");
    assert_eq!(w.state.player().spells, spells);
    assert_eq!(count(&w, "grimoire_of_frost"), 2, "the grimoire kept");
    // None are offered, and no odds are given.
    assert!(w.perception().studies().is_empty());
    let me = w.state.player();
    assert_eq!(study_odds(me, &w.content, None, sp("spark")), None);
    // A scroll of a spell still reads, to cast it: by the check, which takes time and may be
    // tried again (`decipher_tests.rs`).
    for _ in 0..40 {
        assert!(run(&mut w, "read:f:3,1")[0].took_turn);
        if count(&w, "scroll_of_frost") == 1 {
            return;
        }
    }
    panic!("never read");
}

#[test]
fn a_bonus_alone_opens_nothing_and_counts_for_the_trained() {
    let mut w = identified(world());
    w.player_mut().skills = crate::skills::Skills::default();
    wear_scholar(&mut w, 50);
    assert_eq!(w.state.player().learning(&w.content), None);
    assert!(!run(&mut w, "read:c")[0].took_turn);
    assert_eq!(last(&w), UNTRAINED);
    // Skill-raising talents do not train it either.
    let decipher = decipher(&w);
    assert_eq!(w.state.player().skills.value(decipher), None);
    // Once trained, the ring counts: 0 + 50 is learning 8.
    set_decipher(&mut w, 0);
    assert_eq!(w.state.player().learning(&w.content), Some(8));
}

#[test]
fn the_classes_that_learn_start_with_decipher_script() {
    let w = world();
    let content = w.content.clone();
    let decipher = decipher(&w);
    let class = |name: &str| {
        (content.classes().iter())
            .find(|c| c.id == name)
            .unwrap_or_else(|| panic!("class {name}"))
    };
    for (name, dice) in [
        ("wizard", "2d6+10"),
        ("witch", "2d6+10"),
        ("priest", "2d6+10"),
        ("druid", "2d6+10"),
        ("bard", "2d4+4"),
        ("paladin", "2d4+4"),
        ("ranger", "2d4+4"),
    ] {
        let grant = (class(name).skills.iter()).find(|(s, _)| *s == decipher);
        let dice: crate::dice::Dice = dice.parse().expect("dice");
        assert_eq!(grant.map(|&(_, d)| d), Some(dice), "{name}");
    }
    for (name, chance, dice) in [
        ("aristocrat", 35, "2d4+4"),
        ("ascetic", 20, "2d4"),
        ("merchant", 25, "2d4"),
    ] {
        let grant = (class(name).optional_skills.iter()).find(|o| o.skill == decipher);
        let grant = grant.unwrap_or_else(|| panic!("{name}"));
        let dice: crate::dice::Dice = dice.parse().expect("dice");
        assert_eq!((grant.chance, grant.dice), (chance, dice), "{name}");
    }
    // Everyone else: none, for sure or by chance.
    for name in ["berserker", "thief", "warrior", "commoner", "craftsperson"] {
        let c = class(name);
        assert!(c.skills.iter().all(|(s, _)| *s != decipher), "{name}");
        assert!(
            c.optional_skills.iter().all(|o| o.skill != decipher),
            "{name}"
        );
    }
}

/// A level-1 `class` with its kit (spells from the class's table).
pub(super) fn kitted(class: &str) -> World {
    let content = builtin();
    super::caster_tests::world_with(content, class)
}

/// Gives the player one `object` and says where it is.
pub(super) fn given(w: &mut World, object: &str) -> usize {
    let object = w.content.object_id(object).expect("object");
    let me = w.player_mut();
    me.inventory.push(Stack::new(object, 1));
    me.inventory.len() - 1
}

#[test]
fn level_one_casters_learn_cantrips_and_keep_their_granted_spells() {
    // A cantrip's grimoire is certain for the two classes whose kit carries its scroll.
    for class in ["priest", "wizard"] {
        let mut w = kitted(class);
        assert!(w.state.player().learning(&w.content).is_some(), "{class}");
        let at = given(&mut w, "grimoire_of_light");
        assert!(run(&mut w, &format!("read:{at}"))[0].took_turn, "{class}");
        assert!(
            w.state.player().spells.contains_key(&sp("light")),
            "{class}"
        );
    }
    // The class gives its spell whatever the Decipher script: taken away, it stays known and cast.
    let mut w = kitted("wizard");
    w.player_mut().skills = crate::skills::Skills::default();
    assert!(w.state.player().knows_spell(&w.content, sp("ray_of_frost")));
    // And a class without Decipher script is refused.
    let mut w = kitted("warrior");
    assert_eq!(w.state.player().learning(&w.content), None);
    let at = given(&mut w, "grimoire_of_identify");
    assert!(!run(&mut w, &format!("read:{at}"))[0].took_turn);
    assert_eq!(last(&w), UNTRAINED);
}

#[test]
fn learning_a_spell_from_a_grimoire_pays_five_experience_a_spell_level_once() {
    let mut w = identified(world());
    set_decipher(&mut w, 40);
    let before = w.state.experience;
    // Frost is level 6.
    run(&mut w, "read:c");
    assert!(w.state.player().spells.contains_key(&sp("frost")));
    assert_eq!(w.state.experience - before, 30);
    // Fireball is level 11: 55. A level-0 spell pays 5.
    let level = w.content.spell_of(sp("fireball")).expect("spell").level;
    assert_eq!(level, 11);
    let before = w.state.experience;
    w.learn_spell(sp("fireball"));
    assert_eq!(w.state.experience - before, 55);
    assert_eq!(w.content.experience().spell_xp(0), 5);
    // Learned again (a lucky mishap insight): nothing more.
    let before = w.state.experience;
    w.learn_spell(sp("fireball"));
    assert_eq!(w.state.experience, before);
}
