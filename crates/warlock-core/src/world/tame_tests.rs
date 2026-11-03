//! Taming (`tame.rs`; spec §8 "Allies": Tame): who may try (trained, no companion), which beasts
//! (tameable, not hostile or unaware, any level), the channel and what stops it, the food, the
//! check's modifier and difficulty, what a tamed beast becomes, the refusals' cost, the mark,
//! the menu's reason, and a replay.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Action, parse_actions};
use crate::content::{ContentFiles, Stance};
use crate::geom::Dir;
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::Stat;
use std::sync::Arc;

/// A lit nook: the player below a beast walled in on three sides, so it cannot wander off.
fn level(beast: &str, faction: Option<&str>) -> String {
    let faction = faction.map_or(String::new(), |f| format!(", faction = \"{f}\""));
    format!(
        "name = \"Nook\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n##b##\n#.@.#\n#...#\n#####\n'''\n\
         [legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
         \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
         \"b\" = {{ terrain = \"floor\", creature = \"{beast}\"{faction} }}\n"
    )
}

/// The built-in content and the nook with `beast` (of `faction`, if given), the player a human
/// of `class` carrying `carry`.
fn world_with(beast: &str, faction: Option<&str>, class: &str, carry: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/nook.toml", &level(beast, faction));
    files.insert(
        "scenarios/nook.toml",
        &format!(
            "name = \"Nook\"\nstart_level = \"nook\"\n\
             character = {{ race = \"human\", gender = \"male\", class = \"{class}\" }}\n\
             carry = [{carry}]\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "nook", Some(7)).expect("world")
}

fn world(beast: &str) -> World {
    world_with(beast, None, "commoner", "")
}

fn handle(w: &World) -> SkillId {
    w.content()
        .skill_id("handle_animal")
        .expect("Handle animal")
}

/// The player's Handle animal at `value`, or untrained.
fn train(w: &mut World, value: Option<u16>) {
    let id = handle(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn beast(w: &World) -> ActorId {
    let at = Pos::new(2, 1);
    (w.state.actors.values())
        .find(|a| a.pos == at && a.id != w.state.player)
        .map(|a| a.id)
        .expect("the beast")
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn last(w: &World) -> String {
    said(w).last().cloned().unwrap_or_default()
}

fn tame(w: &mut World) -> Outcome {
    w.apply(&Action::Tame(Dir::N))
}

fn menu(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Tame)
        .map(|a| a.unavailable)
}

fn asleep(w: &mut World, id: ActorId) {
    let status = w.content().status_id("asleep").expect("asleep");
    w.afflict(id, status, 1, None, &mut Vec::new());
}

#[test]
fn handle_animal_gives_tame_in_its_place_and_untrained_is_refused() {
    let mut w = world("hound");
    assert_eq!(menu(&w), Some(None), "a commoner is trained: usable");
    let skills: Vec<&str> = (Ability::every(w.content()).into_iter())
        .filter(|a| a.source() == crate::ability::Source::Skill)
        .map(|a| a.id(w.content()))
        .collect();
    assert!(skills.contains(&"tame") && !skills.contains(&"handle_animal"));
    train(&mut w, None);
    assert_eq!(menu(&w), None, "untrained: not in the menu");
    let time = w.time();
    let out = tame(&mut w);
    assert!(!out.took_turn);
    assert_eq!(w.time(), time);
    assert_eq!(last(&w), "You do not know how to tame beasts.");
}

#[test]
fn a_calm_beast_is_tamed_after_the_turns_and_becomes_a_companion() {
    let mut w = world_with("hound", None, "commoner", "\"dried_meat*2\"");
    train(&mut w, Some(200));
    let hound = beast(&w);
    let start = w.time();
    let out = tame(&mut w);
    let turns = w.content().companions().expect("rules").tame.turns;
    let said = said(&w);
    assert!(
        said.iter()
            .any(|t| t.starts_with("You crouch by the hound and hold out")),
        "{said:?}"
    );
    assert!(out.took_turn);
    assert_eq!(
        w.time().since(start),
        crate::time::Duration::seconds(u64::from(turns) * 6),
        "{turns} turns of the tame action (a normal action's six seconds)"
    );
    let actor = &w.state.actors[&hound];
    let data = actor.companion().expect("a companion now");
    assert_eq!(w.companion(), Some(hound));
    assert_eq!(actor.master(), Some(w.state.player));
    assert_ne!(w.stance(w.state.player, hound), Stance::Hostile);
    assert_eq!(
        actor.faction(w.content()),
        w.state.player().faction(w.content())
    );
    assert_eq!(
        (actor.mind.last_seen, actor.mind.noise, actor.mind.post),
        (None, None, None),
        "it forgot what it was about"
    );
    assert_eq!(data.grade, 1, "at a level 1 master, its own level");
    assert_eq!(
        last(&w),
        format!("The hound comes to your hand. You call it {}.", data.name)
    );
    // The food went to it: one of the two.
    let meat = w.content().object_id("dried_meat").expect("dried meat");
    let left: u32 = (w.state.player().inventory.iter())
        .filter(|s| s.object == meat)
        .map(|s| s.count)
        .sum();
    assert_eq!(left, 1);
    // Tame now greys out with the reason, and is refused.
    assert_eq!(menu(&w), Some(Some(Unavailable::HaveCompanion)));
    assert_eq!(
        Unavailable::HaveCompanion.reason(w.content()),
        "you already have a companion"
    );
    let time = w.time();
    assert!(!tame(&mut w).took_turn);
    assert_eq!(w.time(), time);
    assert_eq!(last(&w), "You already have a companion.");
}

#[test]
fn the_check_is_willpower_skill_food_and_the_rangers_bonus_against_the_beasts_level() {
    for (kind, level) in [("hound", 1), ("wolf", 4), ("bear", 12)] {
        let mut w = world_with(kind, None, "commoner", "\"dried_meat\"");
        train(&mut w, Some(100));
        let content = w.content().clone();
        let rules = content.companions().expect("rules").tame;
        let me = w.state.player();
        let mods = me.talent_mods(&content);
        let base = me.effective_stats(&content).modifier(Stat::Willpower)
            + mods.check(Stat::Willpower)
            + me.skill_effect(&content, Effect::Tame);
        assert_eq!(me.skill_effect(&content, Effect::Tame), 10, "+10 at 100");
        assert_eq!(mods.tame, 0, "a commoner has no bonus");
        let (modifier, difficulty) = w.tame_odds(beast(&w), true);
        assert_eq!(modifier, base + rules.food);
        assert_eq!(w.tame_odds(beast(&w), false).0, base);
        assert_eq!(difficulty, rules.difficulty + rules.per_level * level);
    }
    // The ranger's Animal companion adds its 2.
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: crate::character::CharacterSpec = "human,male,ranger".parse().expect("a ranger");
    let w = World::start_run(
        content.clone(),
        "ascmearc",
        Some(3),
        Some(&spec),
        None,
        false,
    )
    .expect("world");
    assert_eq!(w.state.player().talent_mods(&content).tame, 2);
}

#[test]
fn no_level_gate_a_bear_may_be_tried_and_a_failure_costs_only_the_time() {
    let mut w = world("bear");
    train(&mut w, Some(0));
    let bear = beast(&w);
    let marks = |w: &World| {
        w.state
            .player()
            .skills
            .get(handle(w))
            .expect("trained")
            .marks
    };
    let mut tries = 0;
    let mut tamed = false;
    while tries < 20 && !tamed {
        let out = tame(&mut w);
        assert!(out.took_turn, "never refused for its level");
        tries += 1;
        tamed = w.companion() == Some(bear);
        if !tamed {
            assert_eq!(last(&w), "The bear snaps at you and backs away.");
            assert!(w.state.actors[&bear].ally.is_none());
            assert_eq!(w.stance(w.state.player, bear), Stance::Neutral);
        }
    }
    assert!(
        tries > 1,
        "a bear at level 1 is the natural 24 alone: tried again at once"
    );
    assert!(marks(&w) > 0, "every try may mark Handle animal");
}

#[test]
fn a_hostile_beast_that_has_seen_you_is_refused_and_an_asleep_one_may_be_tried() {
    let mut w = world_with("rat", None, "commoner", "");
    let rat = beast(&w);
    assert_eq!(w.stance(w.state.player, rat), Stance::Hostile);
    let time = w.time();
    assert!(!tame(&mut w).took_turn);
    assert_eq!(w.time(), time);
    assert_eq!(
        last(&w),
        "The giant rat has seen you: it will not be tamed now."
    );
    // Asleep, it has seen no one.
    let mut w = world_with("rat", None, "commoner", "");
    let rat = beast(&w);
    asleep(&mut w, rat);
    w.perceive(&mut Vec::new());
    assert!(tame(&mut w).took_turn, "{:?}", said(&w));
    let rat_now = &w.state.actors[&rat];
    assert!(
        rat_now.companion().is_some() || said(&w).iter().any(|t| t.contains("snaps at you")),
        "it was tried: {:?}",
        said(&w)
    );
}

#[test]
fn what_cannot_be_tamed_is_refused_without_time() {
    for (beast, faction, line) in [
        ("kobold", None, "The kobold cannot be tamed."),
        (
            "hound",
            Some("kobolds"),
            "The hound has seen you: it will not be tamed now.",
        ),
    ] {
        let mut w = world_with(beast, faction, "commoner", "");
        let time = w.time();
        assert!(!tame(&mut w).took_turn, "{beast}");
        assert_eq!(w.time(), time);
        assert_eq!(last(&w), line);
    }
    let mut w = world("hound");
    let time = w.time();
    assert!(!w.apply(&Action::Tame(Dir::S)).took_turn);
    assert_eq!(w.time(), time);
    assert_eq!(last(&w), "There is no beast there to tame.");
}

#[test]
fn a_blow_stops_the_taming_with_nothing_lost_but_the_time() {
    let mut w = world_with("hound", None, "commoner", "\"dried_meat\"");
    train(&mut w, Some(200));
    let hound = beast(&w);
    // Bleeding hurts the player every turn: the first stops it.
    let bleeding = w.content().status_id("bleeding").expect("bleeding");
    let player = w.state.player;
    w.afflict(player, bleeding, 3, None, &mut Vec::new());
    let out = tame(&mut w);
    assert!(out.took_turn);
    assert!(w.state.actors[&hound].ally.is_none(), "not tamed");
    let said = said(&w);
    assert!(
        said.iter().any(|t| t == "You stop taming the hound."),
        "{said:?}"
    );
    let meat = w.content().object_id("dried_meat").expect("dried meat");
    assert!(
        (w.state.player().inventory.iter()).any(|s| s.object == meat),
        "an interrupted try eats nothing"
    );
}

#[test]
fn a_tame_replays() {
    // The commoner's own Handle animal: nothing set by hand, so the replay starts as the run.
    let mut w = world_with("hound", None, "commoner", "\"dried_meat*3\"");
    let content = w.content().clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait tame:N tame:N tame:N wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(said(&w).iter().any(|t| t.starts_with("You crouch by")));
    let replay = Replay::from_json(&replay.to_json()).expect("parses");
    let (played, divergence) = replay.play(content).expect("plays");
    assert!(divergence.is_none(), "{divergence:?}");
    assert_eq!(played.state_hash(), w.state_hash());
}
