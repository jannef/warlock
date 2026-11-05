//! Learning spells (spec §8 "Spells"): refusals, the check by overreach and known levels, the
//! time, interruption, the scroll used up, mishaps by band, saves and replays.

use super::caster_tests::sp;
use super::learn::Lesson;
use super::*;
use crate::action::parse_actions;
use crate::content::{Band, ContentFiles, drawn_among};
use crate::replay::Replay;

/// The pack: a, two scrolls of fireball (Fireball, level 11); b, one of lightning (Lightning
/// bolt, level 11); c, two of frost (Frost, level 6); d, one of identify; e, a dagger.
const PACK: &str = "carry = [\"scroll_of_fireball*2\", \"scroll_of_lightning\", \
     \"scroll_of_frost*2\", \"scroll_of_identify\", \"dagger\"]";

/// A lit hall of `map` (`@` the player, `r` a giant rat), the player with `mana` base mana.
fn world_with(map: &str, mana: u16) -> World {
    seeded(map, mana, None)
}

fn seeded(map: &str, mana: u16, seed: Option<u64>) -> World {
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
    // Trained in Spellcraft, at nothing: learning's floor of 1.
    set_spellcraft(&mut w, 0);
    w
}

/// The player is trained in Spellcraft at `value`.
fn set_spellcraft(w: &mut World, value: u16) {
    let spellcraft = spellcraft(w);
    let skills = &mut w.player_mut().skills;
    let by = i32::from(value) - skills.value(spellcraft).map_or(0, i32::from);
    skills.add(spellcraft, by);
}

fn spellcraft(w: &World) -> crate::skills::SkillId {
    w.content.skill_id("spellcraft").expect("skill")
}

const HALL: &str = "#######\n#.....#\n#..@..#\n#.....#\n#######\n";

fn world() -> World {
    world_with(HALL, 10)
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn last(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map(|m| m.display())
        .unwrap_or_default()
}

fn count(w: &World, object: &str) -> u32 {
    let id = w.content.object_id(object).expect("object");
    (w.state.player().inventory.iter())
        .filter(|s| s.object == id)
        .map(|s| s.count)
        .sum()
}

/// Knows every scroll type carried (the dev identify).
fn identified(mut w: World) -> World {
    run(&mut w, "identify");
    w
}

#[test]
fn refusals_take_no_time_and_keep_the_scroll() {
    let mut w = world();
    let refused = |w: &mut World, action: &str, said: &str| {
        let out = run(w, action).remove(0);
        assert!(!out.took_turn, "{action}");
        assert_eq!(last(w), said, "{action}");
    };
    refused(&mut w, "study:c", "You do not know what this says.");
    refused(&mut w, "study:z", "You carry no such thing.");
    refused(&mut w, "study:e", "There is no spell in a dagger to learn.");
    let mut w = identified(w);
    // The scroll of identify teaches Identify (#592).
    let mut knows = w.clone();
    knows.player_mut().spells.insert(sp("identify"));
    refused(&mut knows, "study:d", "You know this spell already.");
    w.player_mut().spells.insert(sp("frost"));
    refused(&mut w, "study:c", "You know this spell already.");
    assert_eq!(count(&w, "scroll_of_frost"), 2, "kept");
    // No mana at all: no gift for magic.
    let mut w = identified(world_with(HALL, 0));
    w.player_mut().mana_base = -100;
    refused(&mut w, "study:c", "You have no gift for magic.");
}

#[test]
fn the_check_grows_with_overreach_and_the_levels_known() {
    let w = world();
    let content = w.content.clone();
    let me = w.state.player().clone();
    let odds = |me: &Actor, value: u16, spell| {
        let mut me = me.clone();
        let spellcraft = content.skill_id("spellcraft").expect("skill");
        me.skills.add(spellcraft, i32::from(value));
        study_odds(&me, &content, None, spell).expect("odds")
    };
    let int = me
        .effective_stats(&content)
        .modifier(crate::stats::Stat::Intelligence);
    // (Spellcraft, spell): overreach in spell levels and difficulty (12 and a point a level
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
            assert_eq!(bonus, int + tenth, "Intelligence and Spellcraft's tenth");
        } else {
            assert_eq!(o.ways(), 144, "certain");
        }
    }
    // Twenty levels known make it a point harder; harder is less likely.
    let mut learned = me.clone();
    learned
        .spells
        .extend([sp("call_lightning"), sp("fireball"), sp("frost")]);
    learned.spells.extend([sp("blessing"), sp("mend")]);
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
    // Quick study's bonus and Spellcraft's tenth count.
    let mut studious = me.clone();
    studious.talents.take("quick_study", None);
    let o = odds(&studious, 59, sp("call_lightning"));
    assert_eq!(o.check.map(|c| c.0), Some(int + 2 + 5));
}

#[test]
fn a_spell_within_reach_is_learned_for_certain_in_its_time() {
    let mut w = identified(world());
    // Learning 6: Frost's level.
    set_spellcraft(&mut w, 40);
    assert!(w.state.player().max_mana(&w.content) > 0);
    let hash = w.state_hash();
    let out = run(&mut w, "study:c").remove(0);
    assert_ne!(w.state_hash(), hash);
    // Frost is level 6: 100 + 6 × 20 = 220 turns, 22 minutes.
    assert_eq!(out.elapsed, crate::time::Duration::minutes(22));
    assert!(w.state.player().spells.contains(&sp("frost")));
    assert_eq!(count(&w, "scroll_of_frost"), 1, "one used up");
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "You begin to study the scroll of frost.")
    );
    assert!(
        said.iter().any(|m| m.ends_with("you have learned Frost.")),
        "{said:?}"
    );
    // It is in the abilities menu, and cannot be studied again.
    let abilities = w.perception().abilities();
    assert!(
        abilities
            .iter()
            .any(|a| a.ability == crate::ability::Ability::Spell(sp("frost")))
    );
    assert!(!run(&mut w, "study:c")[0].took_turn);
    // A scroll of a known spell still reads.
    assert!(run(&mut w, "read:c:3,1")[0].took_turn);
}

#[test]
fn quick_study_halves_the_time() {
    let mut w = identified(world());
    set_spellcraft(&mut w, 40);
    w.player_mut().talents.take("quick_study", None);
    let out = run(&mut w, "study:c").remove(0);
    assert_eq!(out.elapsed, crate::time::Duration::minutes(11));
}

#[test]
fn a_blow_interrupts_the_study_which_keeps_the_scroll() {
    let map = "#######\n#.....#\n#..@r.#\n#.....#\n#######\n";
    let mut w = identified(world_with(map, 10));
    set_spellcraft(&mut w, 40);
    let out = run(&mut w, "study:c").remove(0);
    assert!(out.took_turn);
    assert!(out.elapsed < crate::time::Duration::minutes(22));
    assert_eq!(last(&w), "You stop studying.");
    assert_eq!(count(&w, "scroll_of_frost"), 2, "kept");
    assert!(!w.state.player().spells.contains(&sp("frost")));
}

#[test]
fn far_beyond_the_reader_a_study_fails_into_a_mishap() {
    // Lightning bolt at learning 1 (overreach 10, a bad mishap's band) against 22.
    let mut failed = 0;
    for seed in 0..8 {
        let mut w = identified(seeded(HALL, 10, Some(seed)));
        run(&mut w, "study:b");
        assert_eq!(count(&w, "scroll_of_lightning"), 0, "used up either way");
        let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
        if said.iter().any(|m| m.contains("crumbles to dust")) {
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
        turns: 1,
    };
    let mut w = identified(world());
    let mut events = Vec::new();
    let me = w.state.player;
    let frost = lesson(&w, "scroll_of_frost", sp("frost"));
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
    assert!(w.state.player().spells.contains(&sp("frost")));
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
    // The scroll's own spell goes off on its reader.
    let hp = w.state.player().hp;
    let fire = lesson(&w, "scroll_of_fireball", sp("fireball"));
    w.mishap(&mishap(&w, "eruption"), fire, &mut events);
    let lost = hp - w.state.player().hp;
    assert!(lost > 0, "burnt");
}

#[test]
fn learned_spells_survive_saves_and_replays() {
    let mut w = identified(world());
    set_spellcraft(&mut w, 40);
    let empty = serde_json::to_string(w.state.player()).expect("json");
    assert!(!empty.contains("spells"), "left out while empty");
    run(&mut w, "study:c");
    let json = serde_json::to_string(w.state.player()).expect("json");
    let saved = format!("\"spells\":[{}]", sp("frost").0);
    assert!(json.contains(&saved), "by index: {json}");
    let back: Actor = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.spells, w.state.player().spells);
    let content = w.content.clone();
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // Replays: recorded and played back to the same state, by a character trained by their class.
    let wizard: CharacterSpec = "human,female,wizard".parse().expect("a character");
    let mut w = World::with_character(content.clone(), "hall", None, Some(&wizard)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("identify study:b wait study:c").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn the_dumps_say_the_spells_known_and_the_chances_in_words() {
    use crate::dump::{json, layer_rows, parse_layers};
    let spells = parse_layers("spells").expect("layer")[0];
    let progress = parse_layers("progress").expect("layer")[0];
    let mut w = world();
    assert_eq!(
        layer_rows(&w, spells),
        vec!["no spells"],
        "unknown scrolls are not offered"
    );
    w = identified(w);
    w.player_mut().spells.insert(sp("blessing"));
    let rows = layer_rows(&w, spells);
    assert!(rows[0].starts_with("Blessing (level 3, "), "{rows:?}");
    let frost = "study c, a scroll of frost: Frost, a struggle".to_string();
    assert!(rows.contains(&frost), "{rows:?}");
    assert!(
        rows.iter()
            .any(|r| r.starts_with("study a, a scroll of fireball: Fireball, "))
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("study d, a scroll of identify: Identify, ")),
        "it teaches Identify (#592): {rows:?}"
    );
    // The exact odds only in the dev layer.
    let dev = layer_rows(&w, progress);
    let exact = |r: &String| r.starts_with("study c frost: overreach 5, ") && r.ends_with("/144");
    assert!(dev.iter().any(exact), "{dev:?}");
    assert!(
        dev.iter().any(|r| r == "learning 1 (Spellcraft 0)"),
        "{dev:?}"
    );
    let value = json(&w, &[spells], false);
    assert_eq!(value["spells"][0]["id"], "blessing");
    assert_eq!(value["studies"][2]["chance"], "a struggle");
    assert_eq!(value["spells"][0]["level"], 3);
    // Untrained: no study lines anywhere, and the dev layer says why.
    w.player_mut().skills = crate::skills::Skills::default();
    let rows = layer_rows(&w, spells);
    assert!(!rows.iter().any(|r| r.starts_with("study ")), "{rows:?}");
    let dev = layer_rows(&w, progress);
    assert!(
        dev.iter()
            .any(|r| r == "cannot learn: not trained in Spellcraft"),
        "{dev:?}"
    );
    assert!(!dev.iter().any(|r| r.starts_with("study ")), "{dev:?}");
    assert_eq!(json(&w, &[spells], false)["studies"], serde_json::json!([]));
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
fn learning_is_spellcrafts_curve_at_least_one_for_the_trained() {
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
        set_spellcraft(&mut w, value);
        assert_eq!(learning(&w), Some(reach), "Spellcraft {value}");
    }
    // Not trained: none at all, which is not the floor.
    w.player_mut().skills = crate::skills::Skills::default();
    assert_eq!(learning(&w), None);
}

#[test]
fn study_odds_read_spellcraft_a_priest_at_17_a_wizard_at_108() {
    let mut w = world();
    let content = w.content.clone();
    let odds = |w: &World, spell| study_odds(w.state.player(), &content, None, spell);
    set_spellcraft(&mut w, 17);
    assert_eq!(
        odds(&w, sp("spark")).expect("odds").check,
        None,
        "a cantrip"
    );
    assert_eq!(odds(&w, sp("fireball")).expect("odds").overreach, 10);
    set_spellcraft(&mut w, 108);
    let far = odds(&w, sp("fireball")).expect("odds");
    assert_eq!((far.overreach, far.check), (-25, None), "learning 36");
    assert_eq!(odds(&w, sp("call_lightning")).expect("odds").overreach, -25);
    // A hard spell (the druid's calls) is a check at any learning, its overreach not counted
    // when the learning passes its level.
    for (spell, hard) in [("call_of_the_forest", 10), ("call_the_elements", 12)] {
        let def = content.spell(sp(spell));
        assert_eq!(def.study_difficulty, hard, "{spell}");
        set_spellcraft(&mut w, 999);
        let (_, top) = odds(&w, sp(spell)).expect("odds").check.expect("a check");
        assert_eq!(top, 12 + hard, "{spell} at learning 70");
    }
}

#[test]
fn a_character_not_trained_in_spellcraft_cannot_study_cantrips_included() {
    let mut w = identified(world());
    w.player_mut().skills = crate::skills::Skills::default();
    let (time, spells) = (w.state.time, w.state.player().spells.clone());
    let out = run(&mut w, "study:c").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last(&w), UNTRAINED);
    assert_eq!(w.state.time, time, "no time");
    assert_eq!(w.state.player().spells, spells);
    assert_eq!(count(&w, "scroll_of_frost"), 2, "the scroll kept");
    // None are offered, and no odds are given.
    assert!(w.perception().studies().is_empty());
    assert!(!w.perception().can_learn());
    let me = w.state.player();
    assert_eq!(study_odds(me, &w.content, None, sp("spark")), None);
    // A scroll of a spell still reads, to cast it.
    assert!(run(&mut w, "read:c:3,1")[0].took_turn);
    assert_eq!(count(&w, "scroll_of_frost"), 1);
}

#[test]
fn a_bonus_alone_opens_nothing_and_counts_for_the_trained() {
    let mut w = identified(world());
    w.player_mut().skills = crate::skills::Skills::default();
    wear_scholar(&mut w, 50);
    assert_eq!(w.state.player().learning(&w.content), None);
    assert!(!run(&mut w, "study:c")[0].took_turn);
    assert_eq!(last(&w), UNTRAINED);
    // Skill-raising talents do not train it either.
    let spellcraft = spellcraft(&w);
    assert_eq!(w.state.player().skills.value(spellcraft), None);
    // Once trained, the ring counts: 0 + 50 is learning 8.
    set_spellcraft(&mut w, 0);
    assert_eq!(w.state.player().learning(&w.content), Some(8));
}

#[test]
fn the_nine_classes_start_with_their_spellcraft() {
    let w = world();
    let content = w.content.clone();
    let spellcraft = spellcraft(&w);
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
        let grant = (class(name).skills.iter()).find(|(s, _)| *s == spellcraft);
        let dice: crate::dice::Dice = dice.parse().expect("dice");
        assert_eq!(grant.map(|&(_, d)| d), Some(dice), "{name}");
    }
    for (name, chance, dice) in [("aristocrat", 35, "2d4+4"), ("ascetic", 20, "2d4")] {
        let grant = (class(name).optional_skills.iter()).find(|o| o.skill == spellcraft);
        let grant = grant.unwrap_or_else(|| panic!("{name}"));
        let dice: crate::dice::Dice = dice.parse().expect("dice");
        assert_eq!((grant.chance, grant.dice), (chance, dice), "{name}");
    }
    // Everyone else: none, for sure or by chance; the priest's old chance is gone with it.
    for name in [
        "berserker",
        "thief",
        "warrior",
        "commoner",
        "craftsperson",
        "merchant",
    ] {
        let c = class(name);
        assert!(c.skills.iter().all(|(s, _)| *s != spellcraft), "{name}");
        assert!(
            c.optional_skills.iter().all(|o| o.skill != spellcraft),
            "{name}"
        );
    }
    assert!(
        class("priest")
            .optional_skills
            .iter()
            .all(|o| o.skill != spellcraft)
    );
}

/// A level-1 `class` with its kit (spells from the kit's scrolls, spells from the class's table).
fn kitted(class: &str) -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    super::caster_tests::world_with(content, class)
}

#[test]
fn level_one_casters_learn_their_kits_scrolls_and_keep_their_granted_spells() {
    // The kit's cantrip scroll is certain for the two classes that carry one.
    for class in ["priest", "wizard"] {
        let mut w = identified(kitted(class));
        assert!(w.state.player().learning(&w.content).is_some(), "{class}");
        let at = (w.state.player().inventory.iter())
            .position(|s| s.object == w.content.object_id("scroll_of_light").expect("object"))
            .unwrap_or_else(|| panic!("{class}'s kit"));
        assert!(run(&mut w, &format!("study:{at}"))[0].took_turn, "{class}");
        assert!(w.state.player().spells.contains(&sp("light")), "{class}");
    }
    // The class gives its spell whatever the Spellcraft: taken away, it stays known and cast.
    let mut w = kitted("wizard");
    w.player_mut().skills = crate::skills::Skills::default();
    assert!(w.state.player().knows_spell(&w.content, sp("ray_of_frost")));
    // And a class without Spellcraft is refused, though its kit may hold a scroll to read.
    let mut w = identified(kitted("merchant"));
    assert_eq!(w.state.player().learning(&w.content), None);
    let at = (w.state.player().inventory.iter())
        .position(|s| s.object == w.content.object_id("scroll_of_identify").expect("object"))
        .expect("the merchant's kit");
    assert!(!run(&mut w, &format!("study:{at}"))[0].took_turn);
    assert_eq!(last(&w), UNTRAINED);
}
