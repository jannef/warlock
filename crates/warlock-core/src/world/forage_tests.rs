//! Foraging (spec §8 "Foraging", #547): only those trained in Survival may forage, only in the
//! wild (the overworld's land and its local areas, never water, a town, or a dungeon); a forage
//! takes its time, then finds food by the ground's chance plus Survival, fresh, into the pack;
//! refusals take none; it trains Survival; replays verify.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::content::{ContentFiles, Landscape};
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};

/// The built-in content, with Survival earning a mark on every forage when `eager`, and every
/// ground's chance `chance` percent when given.
fn content(eager: bool, chance: Option<u8>) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    if eager {
        let text = include_str!("../../../../assets/data/skills.toml");
        let eager = text.replace(
            "{ on = \"forage\", chance = 25 }",
            "{ on = \"forage\", chance = 100 }",
        );
        assert_ne!(eager, text, "Survival earns marks on a forage");
        files.insert("data/skills.toml", &eager);
    }
    if let Some(chance) = chance {
        let text = include_str!("../../../../assets/data/forage.toml");
        let mut out = String::new();
        for line in text.lines() {
            if line.starts_with("chance = ") {
                out.push_str(&format!("chance = {chance}\n"));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        files.insert("data/forage.toml", &out);
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(false, None), scenario, None).expect("world")
}

fn survival(w: &World) -> SkillId {
    w.content().skill_id("survival").expect("Survival")
}

/// The player trained in Survival at `value` (`None`: untrained).
fn train(w: &mut World, value: Option<u16>) {
    let id = survival(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn forage_in_menu(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Forage)
        .map(|a| a.unavailable)
}

fn food_count(w: &World) -> u32 {
    let content = w.content();
    (w.state.player().inventory.iter())
        .filter(|s| content.object(s.object).food.is_some())
        .map(|s| s.count)
        .sum()
}

/// The landscape the player forages on, if any.
fn ground(w: &World) -> Option<Landscape> {
    forage_ground(w.content(), w.state.current(), w.player_pos()).map(|g| g.landscape)
}

#[test]
fn only_those_trained_in_survival_may_forage() {
    let mut w = world("world");
    train(&mut w, None);
    assert_eq!(forage_in_menu(&w), None, "untrained: not in the menu");
    let out = w.apply(&Action::Forage);
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(
        last_message(&w),
        "You do not know how to live off the land."
    );
    train(&mut w, Some(0));
    assert!(forage_in_menu(&w).is_some(), "trained at 0: listed");
    let survival = survival(&w);
    assert!(
        !(w.perception().abilities().iter())
            .any(|a| matches!(a.ability, Ability::Skill(id) if id == survival)),
        "Survival is listed as its use, not as a skill"
    );
    // A ranger is trained in it from the start.
    assert!(forage_in_menu(&world("forest")).is_some());
}

#[test]
fn foraging_is_for_the_wild_only() {
    // A dungeon, a hand-made courtyard: listed, greyed out, refused with no time.
    for scenario in ["showcase", "courtyard", "depths"] {
        let mut w = world(scenario);
        train(&mut w, Some(20));
        assert_eq!(ground(&w), None, "{scenario}");
        assert_eq!(
            forage_in_menu(&w),
            Some(Some(Unavailable::NotHere)),
            "{scenario}"
        );
        let out = w.apply(&Action::Forage);
        assert!(!out.took_turn && out.elapsed.is_zero(), "{scenario}");
        assert_eq!(last_message(&w), "There is nothing to forage here.");
    }
    // A local area forages as its overworld cell: the Thornwood's glade is forest.
    let w = world("forest");
    assert!(
        matches!(ground(&w), Some(Landscape::Forest | Landscape::DenseForest)),
        "{:?}",
        ground(&w)
    );
    assert_eq!(forage_in_menu(&w), Some(None));
    // On the overworld, by the terrain underfoot: a road gives nothing, wild land does.
    let w = world("world");
    let content = w.content().clone();
    let (level, _) = w.ground_truth();
    let mut seen = (false, false);
    for (p, t) in level.terrain.iter() {
        let landscape = content.terrain(*t).landscape;
        let found = forage_ground(&content, level, p).map(|g| g.landscape);
        match landscape {
            Some(l) if content.forage().expect("rules").ground(l).is_some() => {
                assert_eq!(found, Some(l), "{p:?}");
                seen.0 = true;
            }
            _ => {
                assert_eq!(found, None, "{p:?}");
                seen.1 = true;
            }
        }
    }
    assert_eq!(seen, (true, true));
    // Water underfoot is never foraged, whatever the area.
    let mut w = world("forest");
    let content = w.content().clone();
    let level = w.state.current_mut();
    let water = (level.terrain.iter())
        .find(|(_, t)| content.terrain(**t).freeze.is_some())
        .map(|(p, _)| p);
    if let Some(p) = water {
        assert!(forage_ground(&content, w.state.current(), p).is_none());
    }
}

#[test]
fn a_forage_takes_its_time_and_finds_fresh_food_by_chance() {
    let rules_seconds = content(false, None).forage().expect("rules").seconds;
    // Every ground at 100 %: always found, one of the ground's finds, fresh, in the pack.
    let mut interrupted = 0;
    for seed in 0..10 {
        let mut w = World::new(content(false, Some(100)), "forest", Some(seed)).expect("world");
        train(&mut w, Some(0));
        let before = food_count(&w);
        let start = w.state.time;
        w.apply(&Action::Forage);
        // A creature coming into view cuts it short, finding nothing.
        if w.state.time.since(start) < Duration::seconds(u64::from(rules_seconds)) {
            assert!(
                last_message(&w).contains("stop foraging"),
                "{}",
                last_message(&w)
            );
            assert_eq!(food_count(&w), before, "seed {seed}");
            interrupted += 1;
            continue;
        }
        assert_eq!(
            w.state.time.since(start),
            Duration::seconds(u64::from(rules_seconds)),
            "seed {seed}"
        );
        let got = food_count(&w) - before;
        let ground = forage_ground(w.content(), w.state.current(), w.player_pos()).expect("wild");
        assert!(
            (1..=ground.count.max().unsigned_abs()).contains(&got),
            "seed {seed}: {got}"
        );
        assert!(
            last_message(&w).starts_with("You gather "),
            "{}",
            last_message(&w)
        );
        let made: Vec<_> = (w.state.player().inventory.iter())
            .filter(|s| ground.finds.iter().any(|(o, _)| *o == s.object))
            .map(|s| s.died)
            .collect();
        assert!(
            made.contains(&Some(w.state.time)),
            "fresh: made as it was found"
        );
    }
    assert!(interrupted < 5, "{interrupted} of 10 cut short");
    // At the data's chance, sometimes nothing: the same spot may be tried again at once.
    let mut w = world("forest");
    train(&mut w, Some(0));
    let (mut found, mut empty) = (0, 0);
    for _ in 0..40 {
        let before = food_count(&w);
        w.apply(&Action::Forage);
        if food_count(&w) > before {
            found += 1;
        } else {
            assert_eq!(last_message(&w), "You find nothing worth eating.");
            empty += 1;
        }
    }
    assert!(found > 0 && empty > 0, "{found} found, {empty} empty");
}

#[test]
fn survival_raises_the_chance() {
    let w = world("forest");
    let mut w100 = world("forest");
    train(&mut w100, Some(100));
    let mut w0 = world("forest");
    train(&mut w0, Some(0));
    assert_eq!(
        w0.state.player().skill_effect(w.content(), Effect::Forage),
        0
    );
    assert_eq!(
        w100.state
            .player()
            .skill_effect(w.content(), Effect::Forage),
        15
    );
    let mut w999 = world("forest");
    train(&mut w999, Some(999));
    assert_eq!(
        w999.state
            .player()
            .skill_effect(w.content(), Effect::Forage),
        30
    );
    // Found more often by a master than by a novice, over the same seeds.
    let finds = |value: u16| {
        (0..60)
            .filter(|&seed| {
                let mut w = World::new(content(false, None), "forest", Some(seed)).expect("world");
                train(&mut w, Some(value));
                let before = food_count(&w);
                w.apply(&Action::Forage);
                food_count(&w) > before
            })
            .count()
    };
    assert!(finds(999) > finds(0));
}

#[test]
fn a_forage_trains_survival() {
    let mut w = World::new(content(true, None), "forest", None).expect("world");
    train(&mut w, Some(10));
    let marks = |w: &World| w.state.player().skills.get(survival(w)).map(|t| t.marks);
    assert_eq!(marks(&w), Some(0));
    w.apply(&Action::Forage);
    assert_eq!(marks(&w), Some(1));
    // A refused forage trains nothing.
    let mut w = World::new(content(true, None), "showcase", None).expect("world");
    train(&mut w, Some(10));
    w.apply(&Action::Forage);
    assert_eq!(marks(&w), Some(0));
}

#[test]
fn forage_replays_verify() {
    let content = content(false, None);
    let mut w = World::new(content.clone(), "forest", None).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("forage forage move:N forage").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (verified, divergence) = replay.play(content).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}
