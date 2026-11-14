//! Player summons (spec §8 "Allies": "Summons", `summon_spell.rs`): the spell kind's checks, the
//! call, its count, cap, and time, the oldest giving way, the global cap, the `ally_mod` reads,
//! the draw's bias, a slain summon's nothing, `study_difficulty`, saves, and determinism. On the
//! built-in content's spell-circle hall (peaceful kobolds) with fixture spells and talents, and
//! the druid's Pack leader (the druid's calls themselves: `caster_tests.rs`).

use super::*;
use crate::content::ContentFiles;
use crate::test_support::run;
use crate::test_support::said;

/// Two rats for five minutes, three of them at once.
const RATS: &str = r#"
[[spell]]
id = "zz_call_rats"
name = "Call rats"
description = "x"
level = 1
school = "enchantment"
kind = "summon"
shape = "self"
resolution = "auto"
effect = { summon = { creatures = ["rat"], count = "2", most = 3, minutes = 5 } }

[spell.cast]
message = "You call, and the vermin answer."
noise = { sound = "chanting", loudness = 20 }
"#;

/// One rat for a minute, one at once.
const ONE: &str = r#"
[[spell]]
id = "zz_call_one"
name = "Call one"
description = "x"
level = 1
school = "enchantment"
kind = "summon"
shape = "self"
resolution = "auto"
effect = { summon = { creatures = ["rat"], count = "1", most = 1, minutes = 1 } }
"#;

/// A rat (tagged `vermin`) or a bat, one at a time, twelve kept.
const MIXED: &str = r#"
[[spell]]
id = "zz_call_mixed"
name = "Call mixed"
description = "x"
level = 1
school = "enchantment"
kind = "summon"
shape = "self"
resolution = "auto"
effect = { summon = { creatures = ["rat", "bat"], count = "1", most = 12, minutes = 5 } }
"#;

/// A buff made very hard to learn.
const HARD: &str = r#"
[[spell]]
id = "zz_hard"
name = "Hard"
description = "x"
level = 1
school = "transmutation"
kind = "buff"
shape = "self"
resolution = "auto"
study_difficulty = 10
effect = { status = "hasted" }
"#;

const TALENTS: &str = r#"
[[talent]]
id = "zz_lord"
name = "Lord"
tree = "wayfarer"
description = "x"
effects = [{ kind = "ally_mod", lasting = true }]
systems = ["allies"]

[[talent]]
id = "zz_no_vermin"
name = "No vermin"
tree = "wayfarer"
description = "x"
effects = [{ kind = "summon_bias", tag = "vermin", weight = 0 }]
systems = ["allies"]
"#;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for (path, text) in [
        ("data/spells/zz_call_rats.toml", RATS),
        ("data/spells/zz_call_one.toml", ONE),
        ("data/spells/zz_call_mixed.toml", MIXED),
        ("data/spells/zz_hard.toml", HARD),
        ("data/talents/zz_summons.toml", TALENTS),
    ] {
        files.insert(path, text);
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("the fixtures load"))
}

/// The spell-circle hall, the player knowing the fixture calls.
fn world() -> World {
    world_on("circles")
}

fn world_on(scenario: &str) -> World {
    let content = content();
    let mut w = World::new(content.clone(), scenario, None).expect("world");
    for id in ["zz_call_rats", "zz_call_one", "zz_call_mixed"] {
        let spell = content.spell_id(id).expect("a fixture spell");
        w.player_mut().spells.insert(spell, 1);
    }
    w.player_mut().mana = 300;
    w
}

fn last_said(w: &World) -> String {
    said(w).last().cloned().unwrap_or_default()
}

/// The player's summons (not a scenario character's companion: the showcase's Osric is a ranger).
fn calls(w: &World) -> Vec<&Actor> {
    (w.state.actors.values())
        .filter(|a| a.master() == Some(w.state.player) && a.companion().is_none())
        .collect()
}

fn take(w: &mut World, talent: &str) {
    let player = w.state.player;
    (w.state.actors.get_mut(&player).unwrap().talents).take(talent, None);
}

#[test]
fn a_call_brings_its_count_beside_the_caster_on_their_side() {
    let mut w = world();
    let (time, mana) = (w.time(), w.state.player().mana);
    let spell = w.content().spell_id("zz_call_rats").expect("the spell");
    run(&mut w, "use:zz_call_rats");
    let pack = calls(&w);
    assert_eq!(pack.len(), 2, "{:?}", said(&w));
    let me = w.state.player().pos;
    for a in &pack {
        let ally = a.ally.as_ref().expect("an ally");
        assert_eq!(ally.source, AllySource::Summoned { spell: Some(spell) });
        assert_eq!(ally.order, Order::Follow);
        assert_eq!(ally.until, Some(time + Duration::minutes(5)));
        assert!(a.pos.chebyshev(me) <= 2);
        assert!(a.inventory.is_empty());
        assert_eq!(
            w.stance(w.state.player, a.id),
            crate::content::Stance::Neutral
        );
    }
    assert!(w.state.player().mana < mana, "paid");
    assert!(w.time() > time, "a cast's time");
    let lines = said(&w);
    assert!(lines.contains(&"You call, and the vermin answer.".to_string()));
    assert_eq!(last_said(&w), "Two giant rats come to your call.");
}

#[test]
fn past_its_most_the_oldest_gives_way_and_a_call_always_brings_one() {
    let mut w = world();
    run(&mut w, "use:zz_call_one");
    let first = calls(&w)[0].id;
    run(&mut w, "use:zz_call_one");
    let pack = calls(&w);
    assert_eq!(pack.len(), 1, "one kept: {:?}", said(&w));
    assert!(pack[0].id > first, "the new one stays");
    assert!(said(&w).iter().any(|m| m == "The giant rat fades away."));
    // Three at most of the other spell: a third and fourth cast keep three.
    run(&mut w, "use:zz_call_rats use:zz_call_rats");
    let spell = w.content().spell_id("zz_call_rats").expect("the spell");
    let rats = (calls(&w).into_iter())
        .filter(|a| a.ally.as_ref().unwrap().source == AllySource::Summoned { spell: Some(spell) })
        .count();
    assert_eq!(rats, 3);
}

#[test]
fn a_call_departs_when_its_minutes_run_out() {
    let mut w = world();
    run(&mut w, "use:zz_call_one");
    assert_eq!(calls(&w).len(), 1);
    run(&mut w, "rest:2m");
    assert!(calls(&w).is_empty(), "{:?}", said(&w));
    assert!(said(&w).iter().any(|m| m == "The giant rat fades away."));
}

#[test]
fn a_call_is_refused_without_room_on_the_overworld_and_at_the_cap() {
    // The global cap: seven of the dev's and one call make eight; the next call is refused,
    // unless one of its own gives way.
    let mut w = world();
    run(&mut w, "ally:rat:7 use:zz_call_one");
    assert_eq!(calls(&w).len(), 8, "{:?}", said(&w));
    let (time, mana) = (w.time(), w.state.player().mana);
    run(&mut w, "use:zz_call_rats");
    assert_eq!(last_said(&w), "You cannot lead more than this.");
    assert_eq!(
        (w.time(), w.state.player().mana),
        (time, mana),
        "no time, no mana"
    );
    run(&mut w, "use:zz_call_one");
    assert_eq!(calls(&w).len(), 8, "the old one gave way: {:?}", said(&w));
    assert!(w.state.player().mana < mana);
    // The overworld has no creatures: nothing answers there.
    let mut w = world_on("world");
    let max = w.state.player().max_mana(w.content());
    w.player_mut().mana = max;
    let time = w.time();
    run(&mut w, "use:zz_call_rats");
    assert!(calls(&w).is_empty());
    assert_eq!(last_said(&w), "Nothing answers your call out here.");
    assert_eq!((w.time(), w.state.player().mana), (time, max));
}

#[test]
fn ally_mod_brings_more_for_longer_and_keeps_more() {
    let mut w = world();
    // The druid's Pack leader (#839): one more, half again as long, one more kept.
    take(&mut w, "pack_leader");
    let time = w.time();
    run(&mut w, "use:zz_call_rats");
    let pack = calls(&w);
    assert_eq!(pack.len(), 3, "two and one more");
    // Five minutes half again as long, rounded down: seven.
    for a in &pack {
        assert_eq!(
            a.ally.as_ref().unwrap().until,
            Some(time + Duration::minutes(7))
        );
    }
    // `most` 3 + 1: a second call keeps four.
    run(&mut w, "use:zz_call_rats");
    assert_eq!(calls(&w).len(), 4);
}

#[test]
fn a_lasting_call_stays_until_slain() {
    let mut w = world();
    take(&mut w, "zz_lord");
    run(&mut w, "use:zz_call_one");
    assert_eq!(calls(&w)[0].ally.as_ref().unwrap().until, None);
    run(&mut w, "rest:1h");
    assert_eq!(calls(&w).len(), 1);
}

#[test]
fn the_draw_bias_weighs_the_pool() {
    let bat = |w: &World| w.content().creature_id("bat").expect("a bat");
    let mut plain = world();
    run(&mut plain, "use:zz_call_mixed*8");
    let bats = calls(&plain)
        .iter()
        .filter(|a| a.creature == bat(&plain))
        .count();
    assert!(
        (1..8).contains(&bats),
        "both come without a bias: {bats} bats of 8"
    );
    let mut biased = world();
    take(&mut biased, "zz_no_vermin");
    run(&mut biased, "use:zz_call_mixed*8");
    let pack = calls(&biased);
    assert_eq!(pack.len(), 8, "the global cap");
    assert!(
        pack.iter().all(|a| a.creature == bat(&biased)),
        "never a rat"
    );
}

#[test]
fn a_slain_call_leaves_no_corpse_and_nothing_else() {
    let mut w = world();
    let missile = w.content().spell_id("magic_missile").expect("shipped");
    w.player_mut().spells.insert(missile, 1);
    run(&mut w, "use:zz_call_one");
    let rat = calls(&w)[0].id;
    let at = w.state.actors[&rat].pos;
    w.state.actors.get_mut(&rat).unwrap().hp = 1;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let before = w.state.current().objects_at(at).to_vec();
    w.apply(&Action::Cast("magic_missile".into(), Some(at)));
    assert!(!w.state.actors.contains_key(&rat), "{:?}", said(&w));
    assert_eq!(w.state.current().objects_at(at), before.as_slice());
}

#[test]
fn study_difficulty_makes_a_study_a_check_at_any_learning() {
    let content = content();
    let w = World::new(content.clone(), "circles", None).expect("world");
    let decipher = content.skill_id("decipher_script").expect("skill");
    // Two spells of level 1, one made ten harder.
    let hard = content.spell_id("zz_hard").expect("the spell");
    let plain = content.spell_id("zz_call_one").expect("the spell");
    let odds = |value: u16, spell| {
        let mut me = w.state.player().clone();
        me.skills = crate::skills::Skills::default();
        me.skills.add(decipher, i32::from(value));
        // Nothing known, so the known levels add nothing to the difficulty.
        me.spells.clear();
        super::learn::study_odds(&me, &content, None, spell).expect("odds")
    };
    // At any learning, even the most: the plain one is certain, the hard one still a check, ten
    // harder than the plain one's base.
    assert_eq!(odds(150, plain).check, None);
    let (_, far) = odds(150, hard).check.expect("a check");
    assert_eq!(far, 12 + 10, "no overreach counted above its level");
}

#[test]
fn a_summon_spell_is_checked_as_it_loads() {
    let refused = |text: &str, says: &str| {
        let mut files = files();
        files.insert("data/spells/zz_bad.toml", text);
        let err = Content::load(&files).expect_err("refused");
        assert!(err.message.contains(says), "{says}: {}", err.message);
    };
    let call = |effect: &str, more: &str| {
        format!(
            "[[spell]]\nid = \"zz_bad\"\nname = \"Bad\"\ndescription = \"x\"\nlevel = 1\n\
             school = \"enchantment\"\nkind = \"summon\"\nshape = \"self\"\n\
             resolution = \"auto\"\neffect = {{ summon = {{ {effect} }} }}\n{more}"
        )
    };
    let fine = "creatures = [\"rat\"], count = \"1\", most = 1, minutes = 5";
    refused(
        &call(fine, "")
            .replace("shape = \"self\"", "shape = \"touch\"")
            .replace("resolution = \"auto\"", "resolution = \"touch\""),
        "cast on `self`",
    );
    refused(
        &call(
            "creatures = [\"rat\"], count = \"1x\", most = 1, minutes = 5",
            "",
        ),
        "summon: ",
    );
    refused(
        &call("creatures = [], count = \"1\", most = 1, minutes = 5", ""),
        "`creatures` names at least one",
    );
    refused(
        &call(
            "creatures = [\"dragonish\"], count = \"1\", most = 1, minutes = 5",
            "",
        ),
        "unknown creature `dragonish`",
    );
    refused(
        &call(
            "creatures = [\"player\"], count = \"1\", most = 1, minutes = 5",
            "",
        ),
        "cannot be called",
    );
    // The count ladder is a summon's alone; its minutes grow on `degrees`.
    refused(
        &format!("{HARD}\n[spell.grows]\ncount = \"willpower 14 2\"\n")
            .replace("zz_hard", "zz_bad"),
        "`count` is for a `summon` spell's count",
    );
    let mut files = files();
    files.insert(
        "data/spells/zz_good.toml",
        &call(
            fine,
            "[spell.grows]\ncount = \"willpower 14 1d2, 18 2d2\"\ndegrees = \"presence 14 8\"\n",
        )
        .replace("zz_bad", "zz_good"),
    );
    let content = Content::load(&files).expect("loads");
    let good = content.spell(content.spell_id("zz_good").expect("the spell"));
    assert!(good.grows.count.is_some() && good.grows.degrees.is_some());
    assert_eq!(good.tags(&content), ["enchantment", "summon", "self"]);
}

#[test]
fn a_call_survives_a_save_and_runs_the_same_twice() {
    let actions = "use:zz_call_rats wait*5 use:zz_call_mixed wait*3";
    let mut a = world();
    run(&mut a, actions);
    let mut b = world();
    run(&mut b, actions);
    assert_eq!(a.state_hash(), b.state_hash());
    let bytes = crate::save::encode(&a);
    let loaded = crate::save::decode(&bytes, a.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), a.state_hash());
    for (x, y) in calls(&a).into_iter().zip(calls(&loaded)) {
        assert_eq!(x.ally, y.ally);
    }
}
