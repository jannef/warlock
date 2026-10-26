//! Learning spells (spec §8 "Spells"): refusals, the check by overreach and known tiers, the
//! time, interruption, the scroll used up, mishaps by band, saves and replays.

use super::learn::Lesson;
use super::*;
use crate::action::parse_actions;
use crate::content::{Band, ClassSpell, ContentFiles, drawn_among};
use crate::replay::Replay;

/// The pack: a, two scrolls of fireball (Fire burst, tier 3); b, one of lightning (Call
/// lightning, tier 4); c, two of frost (Frost, tier 2); d, one of identify; e, a dagger.
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
    World::new(content, "hall", seed).expect("world")
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

/// Raises the player to at least `level`, spending the picks it brings.
fn raise_to(w: &mut World, level: u8) {
    while w.player_level() < level {
        run(w, "xp:2000 levelup:auto");
    }
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
    refused(
        &mut w,
        "study:d",
        "There is no spell in a scroll of identify to learn.",
    );
    w.player_mut().spells.insert(ClassSpell::Frost);
    refused(&mut w, "study:c", "You know this spell already.");
    assert_eq!(count(&w, "scroll_of_frost"), 2, "kept");
    // No mana at all: no gift for magic.
    let mut w = identified(world_with(HALL, 0));
    w.player_mut().mana_base = -100;
    refused(&mut w, "study:c", "You have no gift for magic.");
}

#[test]
fn the_check_grows_with_overreach_and_the_tiers_known() {
    let w = world();
    let content = w.content.clone();
    let me = w.state.player().clone();
    let odds =
        |me: &Actor, level: i32, spell| study_odds(me, &content, None, level, spell).expect("odds");
    let int = me
        .effective_stats(&content)
        .modifier(crate::stats::Stat::Intelligence);
    // (caster level, spell): overreach and difficulty, nothing known.
    let table = [
        (1, ClassSpell::Blessing, 0, None),
        (6, ClassSpell::Frost, 0, None),
        (12, ClassSpell::Frost, -1, None),
        (1, ClassSpell::EvilEye, 1, Some(16)),
        (1, ClassSpell::Frost, 2, Some(20)),
        (1, ClassSpell::FireBurst, 3, Some(24)),
        (3, ClassSpell::CallLightning, 3, Some(24)),
        (1, ClassSpell::CallLightning, 4, Some(28)),
    ];
    for (level, spell, overreach, difficulty) in table {
        let o = odds(&me, level, spell);
        assert_eq!(o.overreach, overreach, "{spell:?} at {level}");
        assert_eq!(o.check.map(|c| c.1), difficulty, "{spell:?} at {level}");
        if let Some((bonus, _)) = o.check {
            assert_eq!(bonus, int, "Intelligence alone");
        } else {
            assert_eq!(o.ways(), 144, "certain");
        }
    }
    // Ten tiers known make it a point harder; harder is less likely.
    let mut learned = me.clone();
    learned
        .spells
        .extend([ClassSpell::CallLightning, ClassSpell::FireBurst]);
    learned
        .spells
        .extend([ClassSpell::Blessing, ClassSpell::Frost]);
    let (before, after) = (
        odds(&me, 1, ClassSpell::EvilEye),
        odds(&learned, 1, ClassSpell::EvilEye),
    );
    assert_eq!(
        after.check.map(|c| c.1),
        Some(17),
        "4 + 3 + 1 + 2 = 10 tiers"
    );
    assert!(after.ways() < before.ways());
    // Quick study's bonus and Spellcraft's tenth count.
    let mut studious = me.clone();
    studious.talents.take("quick_study", None);
    let spellcraft = content.skill_id("spellcraft").expect("skill");
    studious.skills.add(spellcraft, 100);
    let o = odds(&studious, 1, ClassSpell::EvilEye);
    assert_eq!(o.check.map(|c| c.0), Some(int + 2 + 10));
}

#[test]
fn a_spell_within_reach_is_learned_for_certain_in_its_time() {
    let mut w = identified(world());
    raise_to(&mut w, 6);
    assert!(w.state.player().max_mana(&w.content) > 0);
    let hash = w.state_hash();
    let out = run(&mut w, "study:c").remove(0);
    assert_ne!(w.state_hash(), hash);
    // Frost is tier 2: three hundred turns, half an hour.
    assert_eq!(out.elapsed, crate::time::Duration::minutes(30));
    assert!(w.state.player().spells.contains(&ClassSpell::Frost));
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
            .any(|a| a.ability == crate::ability::Ability::Spell(ClassSpell::Frost))
    );
    assert!(!run(&mut w, "study:c")[0].took_turn);
    // A scroll of a known spell still reads.
    assert!(run(&mut w, "read:c:3,1")[0].took_turn);
}

#[test]
fn quick_study_halves_the_time() {
    let mut w = identified(world());
    raise_to(&mut w, 6);
    w.player_mut().talents.take("quick_study", None);
    let out = run(&mut w, "study:c").remove(0);
    assert_eq!(out.elapsed, crate::time::Duration::minutes(15));
}

#[test]
fn a_blow_interrupts_the_study_which_keeps_the_scroll() {
    let map = "#######\n#.....#\n#..@r.#\n#.....#\n#######\n";
    let mut w = identified(world_with(map, 10));
    raise_to(&mut w, 6);
    let out = run(&mut w, "study:c").remove(0);
    assert!(out.took_turn);
    assert!(out.elapsed < crate::time::Duration::minutes(30));
    assert_eq!(last(&w), "You stop studying.");
    assert_eq!(count(&w, "scroll_of_frost"), 2, "kept");
    assert!(!w.state.player().spells.contains(&ClassSpell::Frost));
}

#[test]
fn far_beyond_the_reader_a_study_fails_into_a_mishap() {
    // Call lightning at caster level 1 (overreach 4) against 28: a natural 24 alone makes it.
    let mut failed = 0;
    for seed in 0..8 {
        let mut w = identified(seeded(HALL, 10, Some(seed)));
        run(&mut w, "study:b");
        assert_eq!(count(&w, "scroll_of_lightning"), 0, "used up either way");
        let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
        if said.iter().any(|m| m.contains("crumbles to dust")) {
            failed += 1;
            // A terrible mishap, or a lucky one.
            let messages: Vec<&str> = drawn_among(w.content.mishaps(), Band::Terrible)
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
    let frost = lesson(&w, "scroll_of_frost", ClassSpell::Frost);
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
    assert!(w.state.player().spells.contains(&ClassSpell::Frost));
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
    let fire = lesson(&w, "scroll_of_fireball", ClassSpell::FireBurst);
    w.mishap(&mishap(&w, "eruption"), fire, &mut events);
    let lost = hp - w.state.player().hp;
    let def = w.content.spell_of(ClassSpell::FireBurst).expect("spell");
    let cap = w.content.spell_rules().expect("rules").cap_of(def);
    assert!(
        lost > 0 && lost <= cap,
        "burnt, never past the cap: {lost} of {cap}"
    );
}

#[test]
fn learned_spells_survive_saves_and_replays() {
    let mut w = identified(world());
    raise_to(&mut w, 6);
    let empty = serde_json::to_string(w.state.player()).expect("json");
    assert!(!empty.contains("spells"), "left out while empty");
    run(&mut w, "study:c");
    let json = serde_json::to_string(w.state.player()).expect("json");
    assert!(json.contains("\"spells\":[\"frost\"]"), "{json}");
    let back: Actor = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.spells, w.state.player().spells);
    let content = w.content.clone();
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // Replays: recorded and played back to the same state.
    let mut w = world();
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
    w.player_mut().spells.insert(ClassSpell::Blessing);
    let rows = layer_rows(&w, spells);
    assert!(rows[0].starts_with("Blessing: tier 0, "), "{rows:?}");
    let frost = "study c, a scroll of frost: Frost, far beyond you".to_string();
    assert!(rows.contains(&frost), "{rows:?}");
    assert!(
        rows.iter()
            .any(|r| r.starts_with("study a, a scroll of fireball: Fire burst, "))
    );
    assert!(
        !rows.iter().any(|r| r.contains("identify")),
        "it teaches nothing"
    );
    // The exact odds only in the dev layer.
    let dev = layer_rows(&w, progress);
    let exact = |r: &String| r.starts_with("study c frost: overreach 2, ") && r.ends_with("/144");
    assert!(dev.iter().any(exact), "{dev:?}");
    let value = json(&w, &[spells], false);
    assert_eq!(value["spells"][0]["id"], "blessing");
    assert_eq!(value["studies"][2]["chance"], "far beyond you");
}
