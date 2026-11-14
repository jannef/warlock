//! The Lullaby (spec §8 "Class abilities", the bard's songs; #873; `world/song.rs`): the bard's
//! song at 20, for 6 mana and a song's time; every hostile that hears it, seen or not, checks
//! Willpower against 13 plus Music's craft, and the failed are Drowsy; friends and the peaceful
//! draw nothing; sung to an empty room all the same.

use super::class_active_tests::{reach, suffers};
use super::vitals_tests::train;
use super::war_cry_tests::foe_at;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::check;
use crate::content::ClassActive;
use crate::noise::Act;
use crate::replay::Replay;
use crate::skills::Effect;
use crate::stats::{PerStat, Stats};
use crate::test_support::{builtin, last, run};

const LULLABY: Ability = Ability::Class(ClassActive::Lullaby);

/// A human bard (seed 7) with their kit, every score 14, born under no sign.
fn bard_at(level: u8) -> World {
    let content = builtin();
    let spec: CharacterSpec = "human,male,bard".parse().expect("a character");
    let picks = ["keen_ears", "lean", "healthy", "tunneler"];
    let spec = spec.with_talents(picks.iter().map(|p| p.to_string()).collect());
    let w = World::start_run(
        content.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    reach(&mut w, level);
    w
}

/// A bard of 20 with the Lullaby and 20 mana, alone on the level.
fn singer() -> World {
    let mut w = bard_at(20);
    let player = w.state.player;
    w.state.actors.retain(|&id, _| id == player);
    w.player_mut().mana = 20;
    w
}

fn own(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == LULLABY)
        .map(|a| a.unavailable)
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

/// Two cells east of the player.
fn near(w: &World) -> Pos {
    let me = w.player_pos();
    Pos::new(me.x + 2, me.y)
}

fn drowsy(w: &World, id: ActorId) -> u16 {
    let status = w.content().status_id("drowsy").expect("drowsy");
    (w.state.actors[&id].effects.get(status)).map_or(0, |e| e.severity)
}

fn sing(w: &mut World) -> Outcome {
    w.apply(&Action::Use(ClassActive::Lullaby, None))
}

#[test]
fn the_lullaby_comes_at_20_and_wants_mana() {
    let mut w = bard_at(19);
    assert_eq!(own(&w), None, "not before 20");
    reach(&mut w, 20);
    assert!(said(&w, "You gain Lullaby."), "{}", last(&w));
    let rows: Vec<_> = (w.perception().abilities().into_iter())
        .filter_map(|a| match a.ability {
            Ability::Class(active) => Some(active),
            _ => None,
        })
        .collect();
    assert_eq!(rows, [ClassActive::Inspire, ClassActive::Lullaby]);
    w.player_mut().mana = 5;
    assert_eq!(own(&w), Some(Some(Unavailable::NoMana)));
    let time = w.state.time;
    assert!(!sing(&mut w).took_turn);
    assert_eq!((w.state.time, w.state.player().mana), (time, 5));
    w.player_mut().mana = 6;
    assert_eq!(own(&w), Some(None));
}

#[test]
fn sung_to_an_empty_room_it_spends_its_mana_and_its_time() {
    let mut w = singer();
    let mut chatted = w.clone();
    let rng = w.state.rng.clone();
    let out = sing(&mut w);
    assert!(out.took_turn);
    assert_eq!(w.state.player().mana, 14, "six mana");
    assert_eq!(last(&w), "You sing a lullaby.");
    assert_eq!(w.state.rng, rng, "nobody to check: no draw");
    // The time of a song (Inspire's).
    chatted.player_mut().mana = 20;
    let inspiration = w.content().resource_id("inspiration").expect("inspiration");
    chatted.player_mut().resources.set(inspiration, 3);
    run(&mut chatted, "use:inspire");
    assert_eq!(w.state.time, chatted.state.time, "a song's time");
}

#[test]
fn the_weak_willed_grow_drowsy_and_the_strong_shake_it_off() {
    // Music 200 (difficulty 21) against a kobold of Willpower 1 (-5): only a natural 24 makes it.
    let mut w = singer();
    train(&mut w, "music", 200);
    let player = w.state.player;
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 1);
    w.perceive(&mut Vec::new());
    sing(&mut w);
    assert!(said(&w, "The kobold's eyes grow heavy."));
    let status = w.content().status_id("drowsy").expect("drowsy");
    let effect = w.state.actors[&kobold].effects.get(status).copied();
    assert_eq!(effect.and_then(|e| e.by), Some(player), "the bard's doing");
    // Music 200 adds three degrees to one: Drowsy's most, three.
    assert_eq!(drowsy(&w, kobold), 3);
    // A kobold of Willpower 40 (+15) against 13: only a natural 2 fails it.
    let mut w = singer();
    let at = near(&w);
    let kobold = foe_at(&mut w, "kobold", at, 40);
    w.perceive(&mut Vec::new());
    sing(&mut w);
    assert!(said(&w, "The kobold shakes off the song."));
    assert!(!suffers(&w, kobold, "drowsy"));
}

#[test]
fn the_check_is_willpower_against_13_and_musics_craft_two_draws_a_hearer() {
    for (music, score) in [
        (10, 10),
        (10, 14),
        (100, 14),
        (100, 20),
        (200, 20),
        (200, 26),
    ] {
        let mut w = singer();
        train(&mut w, "music", music);
        let at = near(&w);
        let kobold = foe_at(&mut w, "kobold", at, score);
        w.perceive(&mut Vec::new());
        let craft = w
            .state
            .player()
            .skill_effect(w.content(), Effect::SongCraft);
        let expected_craft = match music {
            200 => 8,
            100 => 4,
            _ => 0,
        };
        assert_eq!(craft, expected_craft, "Music {music}");
        let will = crate::stats::modifier(i32::from(score));
        let mut rng = w.state.rng.clone();
        let made = check::check(&mut rng, will, 13 + craft);
        sing(&mut w);
        assert_eq!(
            suffers(&w, kobold, "drowsy"),
            !made,
            "Music {music}, Wp {score}"
        );
        assert_eq!(w.state.rng, rng, "two draws for the one hearer");
    }
}

#[test]
fn an_unseen_hearer_is_checked_like_any_other() {
    // Out of the bard's sight, a kobold that hears the song is lulled all the same: who hears is
    // the world's, not the player's to know.
    let mut w = singer();
    train(&mut w, "music", 200);
    // A screen of wall two cells east, the kobold behind it: the song goes round.
    let me = w.player_pos();
    let wall = w.content().terrain_id("wall").expect("wall");
    for y in -1..=1 {
        w.set_terrain(Pos::new(me.x + 2, me.y + y), wall);
    }
    let behind = Pos::new(me.x + 3, me.y);
    let kobold = foe_at(&mut w, "kobold", behind, 1);
    w.perceive(&mut Vec::new());
    assert!(!w.perception.sees_actors_at(behind), "out of sight");
    assert!(w.heard_by(Act::Sing).contains(&kobold), "in earshot");
    sing(&mut w);
    assert!(suffers(&w, kobold, "drowsy"));
    assert!(said(&w, "You sing a lullaby."));
    assert!(
        !said(&w, "The kobold's eyes grow heavy."),
        "nothing said of the unseen"
    );
}

#[test]
fn the_peaceful_hear_it_and_draw_nothing() {
    let mut w = singer();
    train(&mut w, "music", 200);
    let at = near(&w);
    let friend = foe_at(&mut w, "kobold", at, 1);
    let peaceful = w.content().factions().id("peaceful").expect("peaceful");
    w.state
        .actors
        .get_mut(&friend)
        .expect("there")
        .faction_override = Some(peaceful);
    assert_ne!(w.stance(w.state.player, friend), Stance::Hostile);
    w.perceive(&mut Vec::new());
    let rng = w.state.rng.clone();
    sing(&mut w);
    assert_eq!(w.state.rng, rng, "no draw");
    assert!(!suffers(&w, friend, "drowsy"));
}

#[test]
fn a_lullaby_replays_and_restores() {
    let content = builtin();
    let spec: CharacterSpec = "human,female,bard".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    let actions = "xp:3000000 levelup:auto use:lullaby wait use:lullaby";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    // The two songs, one message line counted twice.
    let sang = (w.messages().iter()).find(|m| m.text == "You sing a lullaby.");
    assert_eq!(sang.map(|m| m.count), Some(2));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restored");
    assert_eq!(restored.state_hash(), w.state_hash());
}
