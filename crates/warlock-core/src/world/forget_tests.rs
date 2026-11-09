//! A miscast's rank loss and forgetting (spec §8 "Spells": "Miscasts", #1289, `forget.rs`):
//! rank loss stops at rank 1; forgetting removes a learned spell and marks a class's at rank 0
//! (not known, not cast, not listed); the grimoire teaches either again at rank 1 with its
//! experience; `amnesia` takes one more, `blank` every spell; a creature loses nothing; the
//! protected half may lose ranks but never forget.

use super::caster_tests::{kobold, said, sp};
use super::learn_tests::{given, kitted, run, set_decipher};
use super::miscast::Aim;
use super::*;
use crate::content::{ContentFiles, Miscast};

/// The built-in entry `id`.
fn entry(w: &World, id: &str) -> Miscast {
    (w.content.miscasts().iter())
        .find(|m| m.id == id)
        .cloned()
        .expect("a built-in entry")
}

/// Miscast `id` befalls `caster` after a cast of `spell`.
fn befall(w: &mut World, caster: ActorId, spell: &str, id: &str) {
    let content = w.content.clone();
    let spell = sp(spell);
    let me = w.state.actors[&caster].clone();
    let cast = super::cast::Cast::by_creature(&me, &content, (spell, content.spell(spell)));
    let entry = entry(w, id);
    let aim = if caster == w.state.player {
        Aim::Player(None)
    } else {
        Aim::Creature(w.state.player)
    };
    w.miscast(caster, &cast, aim, &entry, &mut Vec::new());
}

/// The player's rank in `spell`.
fn rank(w: &World, spell: &str) -> Option<u8> {
    w.state.player().spell_rank(&w.content, sp(spell))
}

#[test]
fn rank_loss_stops_at_the_first_rank() {
    let mut w = kitted("wizard");
    let player = w.state.player;
    w.player_mut().spells.insert(sp("ray_of_frost"), 5);
    befall(&mut w, player, "ray_of_frost", "stumble");
    assert_eq!(rank(&w, "ray_of_frost"), Some(4));
    befall(&mut w, player, "ray_of_frost", "falter");
    assert_eq!(rank(&w, "ray_of_frost"), Some(2));
    befall(&mut w, player, "ray_of_frost", "falter");
    assert_eq!(rank(&w, "ray_of_frost"), Some(1), "never below 1");
    // A class's spell never deepened is at 1 already: nothing more, nothing written.
    let mut w = kitted("wizard");
    befall(&mut w, player, "ray_of_frost", "falter");
    assert_eq!(rank(&w, "ray_of_frost"), Some(1));
    assert!(!w.state.player().spells.contains_key(&sp("ray_of_frost")));
}

#[test]
fn forgetting_takes_a_learned_spell_and_a_class_spell_alike() {
    let mut w = kitted("wizard");
    let player = w.state.player;
    w.player_mut().spells.insert(sp("spark"), 3);
    befall(&mut w, player, "spark", "forgetting");
    assert!(!w.state.player().spells.contains_key(&sp("spark")), "gone");
    assert_eq!(rank(&w, "spark"), None);

    // The class's Ray of frost, deepened or not, is marked at rank 0.
    let ray = sp("ray_of_frost");
    assert!(w.state.player().knows_spell(&w.content, ray));
    befall(&mut w, player, "ray_of_frost", "forgetting");
    let me = w.state.player();
    assert_eq!(me.spells.get(&ray), Some(&0));
    assert!(!me.knows_spell(&w.content, ray), "not known");
    assert_eq!(me.spell_rank(&w.content, ray), None);
    assert!(!me.known_spells(&w.content).contains(&ray), "not listed");
    assert!(
        (w.perception().abilities().iter())
            .all(|a| a.ability != crate::ability::Ability::Spell(ray)),
        "not in the abilities menu"
    );
    // Casting it is refused, with no time.
    kobold(&mut w, 2, 1000);
    w.player_mut().mana = 40;
    let here = w.player_pos();
    let out = w.apply(&Action::Cast(
        "ray_of_frost".into(),
        Some(Pos::new(here.x + 2, here.y)),
    ));
    assert!(!out.took_turn, "not cast");
    // Forgotten again: nothing more happens.
    befall(&mut w, player, "ray_of_frost", "forgetting");
    assert_eq!(w.state.player().spells.get(&ray), Some(&0));
}

#[test]
fn the_grimoire_teaches_a_forgotten_spell_again_with_its_experience() {
    let mut w = kitted("wizard");
    let player = w.state.player;
    set_decipher(&mut w, 200);
    let ray = sp("ray_of_frost");
    befall(&mut w, player, "ray_of_frost", "forgetting");
    let level = w.content.spell(ray).level;
    let xp = w.content.experience().spell_xp(level);
    assert!(xp > 0);
    let item = given(&mut w, "grimoire_of_ray_of_frost");
    let before = w.state.experience;
    run(&mut w, &format!("read:{item}"));
    let me = w.state.player();
    assert!(me.knows_spell(&w.content, ray), "learned again");
    assert_eq!(me.spell_rank(&w.content, ray), Some(1));
    assert_eq!(w.state.experience - before, xp, "paid as for any learning");

    // A learned spell forgotten is learned again the same way, and paid again too.
    w.player_mut().spells.insert(sp("frost"), 4);
    befall(&mut w, player, "frost", "forgetting");
    let item = given(&mut w, "grimoire_of_frost");
    let before = w.state.experience;
    run(&mut w, &format!("read:{item}"));
    assert_eq!(rank(&w, "frost"), Some(1), "at rank 1, the ranks gone");
    let paid = w
        .content
        .experience()
        .spell_xp(w.content.spell(sp("frost")).level);
    assert_eq!(w.state.experience - before, paid);
}

#[test]
fn amnesia_takes_one_more_and_blank_takes_every_spell() {
    let mut w = kitted("wizard");
    let player = w.state.player;
    for s in ["spark", "frost", "fireball"] {
        w.player_mut().spells.insert(sp(s), 1);
    }
    let known = w.state.player().known_spells(&w.content).len();
    assert!(known >= 4, "{known}");
    befall(&mut w, player, "spark", "amnesia");
    let after = w.state.player().known_spells(&w.content);
    assert_eq!(after.len(), known - 2);
    assert!(!after.contains(&sp("spark")), "the spell cast first");
    befall(&mut w, player, "frost", "blank");
    assert!(w.state.player().known_spells(&w.content).is_empty());
    // The same amnesia, from the same state and seed, takes the same second spell.
    let draw = |seed| {
        let mut w = kitted("wizard");
        w.state.rng = crate::rng::Rng::gameplay(seed);
        for s in ["spark", "frost", "fireball"] {
            w.player_mut().spells.insert(sp(s), 1);
        }
        befall(&mut w, player, "spark", "amnesia");
        w.state.player().known_spells(&w.content)
    };
    assert_eq!(draw(7), draw(7));
}

#[test]
fn the_player_is_told_what_was_lost_by_name() {
    let mut w = kitted("wizard");
    let player = w.state.player;
    let last = |w: &World| said(w).last().cloned().unwrap_or_default();
    // At rank 1 nothing is lost, and only the entry's own line is said.
    befall(&mut w, player, "ray_of_frost", "stumble");
    assert_eq!(last(&w), entry(&w, "stumble").message);
    w.player_mut().spells.insert(sp("ray_of_frost"), 4);
    befall(&mut w, player, "ray_of_frost", "falter");
    assert_eq!(last(&w), "You know Ray of frost less well than you did.");
    w.player_mut().spells.insert(sp("frost"), 1);
    befall(&mut w, player, "frost", "forgetting");
    assert_eq!(last(&w), "You have forgotten Frost.");
}

#[test]
fn a_creatures_ranks_and_spells_are_its_kinds() {
    let mut w = kitted("wizard");
    let k = kobold(&mut w, 2, 1000);
    let rng = w.state.rng.clone();
    let before = w.state.actors[&k].clone();
    for id in ["stumble", "falter", "forgetting", "amnesia", "blank"] {
        befall(&mut w, k, "spark", id);
    }
    assert_eq!(w.state.actors[&k].spells, before.spells);
    assert_eq!(w.state.rng, rng, "no draw");
}

#[test]
fn the_protected_half_may_lose_ranks_but_never_forget() {
    let load = |effect: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/miscasts/glimmer.toml",
            &format!(
                "[[miscast]]\nid = \"glimmer\"\nfrom = 1\nto = 2\nmessage = \"x\"\n\
                 seen = \"{{who}}\"\neffect = {{ {effect} }}\n"
            ),
        );
        Content::load(&files)
    };
    for refused in [
        "kind = \"forget\", spells = 1",
        "kind = \"forget\", all = true",
        "kind = \"rank_loss\", ranks = 0",
        "kind = \"forget\", spells = 0",
        "kind = \"forget\", all = false",
        "kind = \"forget\", spells = 1, all = true",
        "kind = \"rank_loss\"",
    ] {
        assert!(load(refused).is_err(), "{refused} is refused");
    }
    assert!(load("kind = \"rank_loss\", ranks = 2").is_ok());
    // Neither is a study's mishap.
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/mishaps/zz.toml",
        "[[mishap]]\nid = \"zz\"\nband = \"slight\"\nweight = 1\nmessage = \"x\"\n\
         effect = { kind = \"rank_loss\", ranks = 1 }\n",
    );
    assert!(Content::load(&files).is_err());
}
