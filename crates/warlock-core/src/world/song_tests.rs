//! Songs (spec §8 "Class abilities", the bard's songs; #870): Inspire paid in inspiration alone
//! and refused without it, its degrees by Music, the noise, the overworld and Silenced refused,
//! Virtuoso on every song, Music's marks, and a replay.

use super::class_active_tests::{last, reach, run};
use super::vitals_tests::train;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ClassActive, ResourceId};
use crate::noise::{self, Act, SONG_NOISE, SoundKind};
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};

/// A human bard in `scenario` (seed 7), with their kit, born under no sign, every score 14.
fn bard_in(scenario: &str) -> World {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,male,bard".parse().expect("a character");
    // Creation picks that no test here reads (a bard's one more, a human's Versatile one more).
    let picks = ["keen_ears", "lean", "healthy", "tunneler"];
    let spec = spec.with_talents(picks.iter().map(|p| p.to_string()).collect());
    let w = World::start_run(content.clone(), scenario, Some(7), Some(&spec), None, true)
        .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

const INSPIRE: Ability = Ability::Class(ClassActive::Inspire);

fn inspiration(w: &World) -> ResourceId {
    w.content().resource_id("inspiration").expect("inspiration")
}

fn held(w: &World) -> Option<u16> {
    (w.state.player().resources.get(inspiration(w))).map(|p| p.value)
}

fn set(w: &mut World, value: u16) {
    let r = inspiration(w);
    w.player_mut().resources.set(r, value);
}

/// Why the player cannot use Inspire now, as the abilities menu has it; `None` without it.
fn own(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == INSPIRE)
        .map(|a| a.unavailable)
}

/// The degrees of status `id` the player suffers (0: none).
fn degrees(w: &World, id: &str) -> u16 {
    let status = w.content().status_id(id).expect("a status");
    (w.state.player().effects.get(status)).map_or(0, |e| e.severity)
}

/// A bard of level 9 (Inspire's), holding three inspiration and some mana.
fn bard() -> World {
    let mut w = bard_in("showcase");
    reach(&mut w, 9);
    set(&mut w, 3);
    w.player_mut().mana = 10;
    w
}

#[test]
fn inspire_is_paid_in_inspiration_alone() {
    let mut w = bard_in("showcase");
    reach(&mut w, 9);
    w.player_mut().mana = 10;
    let pool = inspiration(&w);
    assert_eq!(held(&w), Some(0));
    // Short of inspiration, plenty of mana: no song (no fallback on mana).
    assert_eq!(own(&w), Some(Some(Unavailable::Short(pool))));
    let turn = w.state.turn;
    run(&mut w, "use:inspire");
    assert_eq!(last(&w), "You have too little inspiration.");
    assert_eq!(
        (w.state.turn, w.state.player().mana),
        (turn, 10),
        "no time, no mana"
    );
    assert_eq!(degrees(&w, "inspired"), 0);
    // With three, those are spent and the mana is left alone.
    set(&mut w, 3);
    assert_eq!(own(&w), Some(None));
    run(&mut w, "use:inspire");
    assert!(w.state.turn > turn, "a song's time");
    assert_eq!((held(&w), w.state.player().mana), (Some(0), 10));
    assert!(degrees(&w, "inspired") >= 1);
}

#[test]
fn music_lengthens_the_song_up_to_the_status_max() {
    for (music, expected) in [(10, 1), (49, 1), (50, 2), (99, 2), (100, 3), (300, 3)] {
        let mut w = bard();
        train(&mut w, "music", music);
        run(&mut w, "use:inspire");
        assert_eq!(degrees(&w, "inspired"), expected, "Music {music}");
    }
    // The bonus is the status's, whatever the degrees: degrees are time.
    let content = Content::builtin().expect("content");
    let inspired = content.status(content.status_id("inspired").expect("inspired"));
    assert_eq!((inspired.max, inspired.combat.to_hit), (3, 1));
}

#[test]
fn a_song_is_heard_as_a_voice() {
    let w = bard();
    let content = w.content().clone();
    let me = w.state.player();
    let burden = w.state.load(&content, me).penalty();
    let song = noise::noise(&content, w.state.current(), me, burden, Act::Sing).expect("heard");
    assert_eq!(
        (song.kind, song.at, song.loudness),
        (SoundKind::Voice, me.pos, SONG_NOISE)
    );
    // It carries farther than a word.
    let mutter = w.heard_by(Act::Mutter);
    let sing = w.heard_by(Act::Sing);
    assert!(mutter.iter().all(|id| sing.contains(id)));
    // A creature beside the bard hears it (the world's hearing, not the bard's perception).
    let mut w = bard();
    let kobold = super::class_active_tests::beside(&mut w, "kobold", 10);
    assert!(w.heard_by(Act::Sing).contains(&kobold));
}

#[test]
fn silence_and_the_overworld_refuse_a_song() {
    let mut w = bard();
    run(&mut w, "afflict:silenced");
    let silenced = w.content().status_id("silenced").expect("silenced");
    assert_eq!(own(&w), Some(Some(Unavailable::While(silenced))));
    let turn = w.state.turn;
    run(&mut w, "use:inspire");
    assert_eq!(last(&w), "You cannot sing while silenced.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(3)));
    // On the overworld, which is silent.
    let mut w = bard_in("world");
    reach(&mut w, 9);
    set(&mut w, 3);
    let turn = w.state.turn;
    run(&mut w, "use:inspire");
    assert_eq!(last(&w), "You cannot do that on the overworld.");
    assert_eq!((w.state.turn, held(&w)), (turn, Some(3)));
}

#[test]
fn a_virtuoso_sings_greater_for_nothing() {
    let mut w = bard();
    reach(&mut w, 30);
    train(&mut w, "music", 10);
    set(&mut w, 0);
    w.player_mut().mana = 0;
    assert_eq!(own(&w), Some(None), "free");
    run(&mut w, "use:inspire");
    assert_eq!(degrees(&w, "inspired"), 0);
    assert_eq!(degrees(&w, "inspired_greatly"), 2, "a degree more");
    assert_eq!(held(&w), Some(0));
}

#[test]
fn a_song_marks_music() {
    let mut w = bard();
    train(&mut w, "music", 20);
    let music = w.content().skill_id("music").expect("music");
    let marks = |w: &World| (w.state.player().skills.get(music)).map_or(0, |t| t.marks);
    run(&mut w, "use:inspire");
    assert_eq!(marks(&w), 1, "every song");
}

#[test]
fn a_song_replays() {
    let content = Arc::new(Content::builtin().expect("content"));
    let spec: CharacterSpec = "human,female,bard".parse().expect("a character");
    let mut w =
        World::start_run(content, "showcase", Some(7), Some(&spec), None, true).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("xp:3000000 levelup:auto use:inspire wait").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(w.content().clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
