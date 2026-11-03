//! The Strange Spirit and the oath (#879; docs/design/prestige-classes.md §7; spec §8 "Prestige
//! classes"): the Ascmearc's door into the Hunter. The spirit's entries are drawn only while the
//! calling is open (a bard whose class is full, not yet called); it speaks to the good alone; the
//! oath in any case and punctuation calls the bard and sends the spirit away; wrong words cost a
//! chat's time and nothing else; killed, it leaves nothing but the stain on the killer, and the
//! road may bring it again. End to end by replay, from a new bard to Hunter 1. The test world's
//! door, the hermit's word, too. The bard's cap and the callings' level are the shipped ones, 30
//! (#874).

use super::*;
use crate::action::parse_actions;
use crate::alignment::Score;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Factor};
use crate::daylight::{self, DayPhase};
use crate::replay::Replay;
use std::sync::LazyLock;

/// The bard's cap, and the level its calling asks.
const CAP: u8 = 30;
/// The road one cell east of Cēaptūn: seven steps east of the pass.
const ROAD: Pos = Pos::new(19, 28);
const TO_THE_ROAD: &str = "move:E*7 skip:4h";
const ENTRY: &str = "strange_spirit_roads";
const OATH: &str = "LIFE, before... death!";

/// The built-in content, nobody hungering, the class tables stripped, and the bard started Good
/// (so a test's replay need not do good deeds first).
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    crate::content::strip_class_tables(&mut files);
    files.edit_table("classes", |text| match text.contains("id = \"bard\"") {
        true => format!("{text}alignment = 500\n"),
        false => text.to_string(),
    });
    Arc::new(Content::load(&files).expect("fixture content"))
});

/// The shipped content, nobody hungering.
static SHIPPED: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn character(class: &str) -> CharacterSpec {
    format!("human,female,{class}")
        .parse()
        .expect("a character")
}

/// A human of `class` in `scenario`, on `content`.
fn start(content: &Arc<Content>, scenario: &str, class: &str) -> World {
    World::with_character(content.clone(), scenario, None, Some(&character(class))).expect("world")
}

/// Applies `actions`, recording each in `replay` if there is one.
fn act(w: &mut World, actions: &str, replay: &mut Option<&mut Replay>) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        if let Some(r) = replay.as_deref_mut() {
            r.push(&action, w);
        }
    }
}

fn run(w: &mut World, actions: &str) {
    act(w, actions, &mut None);
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

/// The XP that takes the player to `level`'s threshold, every choice due then spent.
fn reach(w: &mut World, level: u8, replay: &mut Option<&mut Replay>) {
    let spec = w.state.character.as_ref().map(|c| &c.spec);
    let factor = Factor::of(w.content(), spec);
    let need = w.content().experience().threshold_for(level, factor);
    let more = need.saturating_sub(w.player_xp());
    act(w, &format!("xp:{more}"), replay);
    settle(w, replay);
}

fn settle(w: &mut World, replay: &mut Option<&mut Replay>) {
    while w.due().is_some() {
        act(w, "levelup:auto", replay);
    }
}

fn set_score(w: &mut World, value: i32) {
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").alignment = Some(Score::new(value));
}

fn score(w: &World) -> i32 {
    (w.state.player().alignment).map_or(0, Score::value)
}

fn prestige(w: &World) -> Option<&str> {
    (w.state.character.as_ref()).and_then(|c| c.prestige.as_deref())
}

/// The spirit on the player's level, if any.
fn spirit(w: &World) -> Option<ActorId> {
    let id = w
        .content()
        .creature_id("strange_spirit")
        .expect("the spirit");
    (w.state.actors.values())
        .find(|a| a.creature == id && a.level == w.state.current_level)
        .map(|a| a.id)
}

/// Whether a party of the spirit's entry has anyone left (one with none is forgotten at the
/// next meeting).
fn known_party(w: &World) -> bool {
    (w.state.parties.iter())
        .filter(|p| p.entry == ENTRY)
        .any(|p| p.members.iter().any(|m| w.state.actors.contains_key(m)))
}

/// Walks the player up beside the spirit, step by step (recorded), and says which way it is.
fn approach(w: &mut World, replay: &mut Option<&mut Replay>) -> Dir {
    for _ in 0..40 {
        let me = w.player_pos();
        let there = w.state.actors[&spirit(w).expect("the spirit")].pos;
        if me.chebyshev(there) == 1 {
            return Dir::toward(me, there).expect("beside it");
        }
        let toward = Dir::toward(me, there).expect("a way");
        let level = &w.state.levels[&w.state.current_level];
        let free = |d: Dir| {
            let p = me.step(d);
            level.is_walkable(w.content(), p)
                && w.state.actor_at(&w.state.current_level, p).is_none()
                && p.chebyshev(there) < me.chebyshev(there)
        };
        let dir = (Dir::ALL.into_iter())
            .filter(|&d| free(d))
            .min_by_key(|&d| (me.step(d).dist2(there), d != toward))
            .expect("a free step closer");
        act(w, &format!("move:{dir:?}"), replay);
    }
    panic!("never reached the spirit");
}

/// A bard of `CAP` on the road at noon (fixture content), with the spirit hailed.
fn hailed(score_value: i32) -> World {
    let mut w = start(&FIXTURE, "ascmearc", "bard");
    run(&mut w, TO_THE_ROAD);
    assert_eq!(w.player_pos(), ROAD);
    assert_eq!(daylight::phase(w.state.time), DayPhase::Day);
    reach(&mut w, CAP, &mut None);
    set_score(&mut w, score_value);
    run(&mut w, &format!("meet:{ENTRY} encounter:hail"));
    w
}

#[test]
fn the_callings_ask_for_the_bards_full_class() {
    // The spirit comes the moment the class is full, never before and never after.
    let bard = SHIPPED.class("bard").expect("the bard").max_level;
    assert_eq!(bard, CAP);
    for campaign in ["ascmearc", "showcase"] {
        let calling = SHIPPED.calling(campaign, "hunter").expect("a door");
        assert_eq!(calling.campaign.as_deref(), Some(campaign));
        assert_eq!(calling.from, ["bard"], "{campaign}");
        assert_eq!(calling.level, bard, "{campaign}");
    }
    // Four day tables of the open country, rare, peaceful, and only while the door is open.
    for table in ["roads", "vale", "downs", "woods"] {
        let id = format!("strange_spirit_{table}");
        let (t, entry) = SHIPPED.encounter_entry("ascmearc", &id).expect("an entry");
        assert_eq!(t.id, table);
        assert_eq!(entry.weight, 2);
        assert_eq!(entry.when.calling.as_deref(), Some("hunter"));
        assert_eq!(entry.when.phase, [DayPhase::Day]);
    }
    let def = SHIPPED.creature(SHIPPED.creature_id("strange_spirit").expect("it"));
    assert!(!def.corpse && def.xp == 0);
}

#[test]
fn an_unknown_calling_on_an_entry_is_refused() {
    let mut files = ContentFiles::builtin();
    files.edit_table("encounters", |t| {
        t.replace("calling = \"hunter\"", "calling = \"skald\"")
    });
    let err = Content::load(&files).expect_err("no such calling");
    assert!(err.message.contains("no calling `skald`"), "{err}");
    // A word may only call a door of its own world.
    let mut files = ContentFiles::builtin();
    files.remove("campaigns/ascmearc/data/callings/hunter.toml");
    files.remove_encounter_tables();
    let err = Content::load(&files).expect_err("the spirit's word calls nothing");
    assert!(
        err.message.contains("no calling `hunter` in its campaign"),
        "{err}"
    );
}

#[test]
fn the_spirit_is_met_only_while_the_calling_is_open() {
    let entries = |w: &World| w.encounter_entries_here();
    // A warrior never: the door is not theirs.
    let mut w = start(&FIXTURE, "ascmearc", "warrior");
    run(&mut w, TO_THE_ROAD);
    reach(&mut w, 40, &mut None);
    assert!(
        !entries(&w).contains(&ENTRY.to_string()),
        "{:?}",
        entries(&w)
    );
    // A bard below the cap: not yet; at it: the spirit may come.
    let mut w = start(&FIXTURE, "ascmearc", "bard");
    run(&mut w, TO_THE_ROAD);
    reach(&mut w, CAP - 1, &mut None);
    assert!(!entries(&w).contains(&ENTRY.to_string()));
    assert!(entries(&w).contains(&"road_pedlar".to_string()));
    reach(&mut w, CAP, &mut None);
    assert!(
        entries(&w).contains(&ENTRY.to_string()),
        "{:?}",
        entries(&w)
    );
    // Not by night.
    run(&mut w, "skip:12h");
    assert_eq!(daylight::phase(w.state.time), DayPhase::Night);
    assert!(!entries(&w).contains(&ENTRY.to_string()));
    // Once called (here by the dev door), never again.
    run(&mut w, "skip:12h call:hunter");
    assert_eq!(prestige(&w), Some("hunter"));
    assert!(!entries(&w).contains(&ENTRY.to_string()));
}

#[test]
fn the_spirit_waits_and_speaks_only_to_the_good() {
    let mut w = hailed(0);
    assert!(said(&w, "You meet a strange spirit"), "{:?}", w.messages());
    let id = spirit(&w).expect("hailed: the spirit is here");
    let at = w.state.actors[&id].pos;
    run(&mut w, "wait*5");
    assert_eq!(w.state.actors[&id].pos, at, "it stands and waits");
    // Neutral: it looks through the bard; no screen, no topics, no row.
    let dir = approach(&mut w, &mut None);
    let out = w.apply(&Action::Chat(dir));
    assert!(out.took_turn);
    assert!(!out.events.iter().any(|e| matches!(e, Event::Talked(_))));
    assert_eq!(
        last_message(&w),
        "The spirit looks through you as if you were not there."
    );
    // The oath said to it then is not heard either: it does not listen.
    run(
        &mut w,
        &format!(
            "say:{dir:?}:{}",
            crate::words::normalise(OATH).replace(' ', "_")
        ),
    );
    assert_eq!(prestige(&w), None);
    assert!(spirit(&w).is_some());
    // Kind: it speaks; nothing to ask, and the Say row.
    set_score(&mut w, 200);
    let out = w.apply(&Action::Chat(dir));
    let talk = out.events.iter().find_map(|e| match e {
        Event::Talked(t) => Some(t),
        _ => None,
    });
    let talk = talk.expect("the dialogue screen");
    assert!(talk.words && talk.topics.is_empty());
    assert!(said(&w, "I have waited long on this road."));
}

#[test]
fn wrong_words_cost_a_chat_and_the_oath_calls_the_bard() {
    let mut w = hailed(500);
    let dir = approach(&mut w, &mut None);
    let level = w.player_level();
    for _ in 0..3 {
        let start = w.time();
        let out = w.apply(&Action::Say {
            dir,
            words: "life after death".into(),
        });
        assert!(out.took_turn);
        assert_eq!(w.time().since(start), crate::time::Duration::seconds(3));
        assert_eq!(
            last_message(&w),
            "The strange spirit says: \"Those are not the words.\""
        );
    }
    assert_eq!((prestige(&w), w.player_level()), (None, level));
    let entry_party = |w: &World| known_party(w);
    assert!(entry_party(&w));
    w.apply(&Action::Say {
        dir,
        words: OATH.into(),
    });
    let texts: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
    let from = (texts.iter())
        .position(|t| t == "The strange spirit says: \"Life before death.\"")
        .expect("the oath echoed");
    let oath = [
        "Life before death.",
        "Strength before weakness.",
        "Journey before destination.",
        "I will protect those who stand in darkness.",
        "I will strive to mend what is broken.",
        "And when I cannot see the light, I will take the next step.",
    ]
    .map(|line| format!("The strange spirit says: \"{line}\""));
    assert_eq!(texts[from..from + 6], oath, "{texts:?}");
    assert_eq!(
        texts[from + 6],
        "The oath is in you now, as a blade is in its sheath. You are a hunter."
    );
    // The spirit fades as the action ends; the level's news comes after it, as it always does.
    assert_eq!(texts[from + 7], "The strange spirit fades, and is gone.");
    let welcome = (texts.iter()).position(|t| t == "Welcome to level 31.");
    assert!(welcome.is_some_and(|i| i > from + 7), "{texts:?}");
    assert_eq!(prestige(&w), Some("hunter"));
    assert_eq!(w.player_level(), CAP + 1, "Hunter 1, at once");
    assert!(spirit(&w).is_none(), "gone, with no corpse");
    assert!(!entry_party(&w), "its party forgotten: no one of it left");
}

#[test]
fn the_spirit_killed_leaves_nothing_and_may_come_again() {
    let mut w = hailed(500);
    let dir = approach(&mut w, &mut None);
    let xp = w.player_xp();
    let level = w.state.current_level.clone();
    let at = w.player_pos().step(dir);
    for _ in 0..40 {
        if spirit(&w).is_none() {
            break;
        }
        w.apply(&Action::Kick(dir));
    }
    assert!(spirit(&w).is_none(), "one hit point");
    assert_eq!(w.player_xp(), xp, "no experience");
    assert!(score(&w) < 500, "a peaceful good soul killed");
    assert!(
        w.state.levels[&level].objects_at(at).is_empty(),
        "no corpse"
    );
    assert_eq!(prestige(&w), None);
    // No one of its party is left: the next meeting forgets it, and the road may bring the
    // spirit again while the door stays open.
    assert!(!known_party(&w));
    assert!(w.calling_open("hunter"));
}

#[test]
fn a_new_bard_becomes_a_hunter_by_replay() {
    let mut w = start(&FIXTURE, "ascmearc", "bard");
    let mut replay = Replay::new(&w);
    let mut r = Some(&mut replay);
    act(&mut w, TO_THE_ROAD, &mut r);
    reach(&mut w, CAP, &mut r);
    assert!(
        crate::alignment::Band::Kind <= w.state.player().band(w.content()),
        "the fixture's bard starts good"
    );
    act(&mut w, &format!("meet:{ENTRY} encounter:hail"), &mut r);
    let dir = approach(&mut w, &mut r);
    act(
        &mut w,
        &format!("chat:{dir:?} say:{dir:?}:life_before_death wait"),
        &mut r,
    );
    assert_eq!(prestige(&w), Some("hunter"));
    assert_eq!(w.player_level(), CAP + 1);
    let json = replay.to_json();
    let replayed = Replay::from_json(&json)
        .expect("json")
        .run(FIXTURE.clone())
        .expect("the replay verifies");
    assert_eq!(replayed.state_hash(), w.state_hash());
    let loaded = crate::save::decode(&crate::save::encode(&w), FIXTURE.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}

#[test]
fn the_hermit_hears_the_oath_from_a_full_bard_only() {
    // The test world's door: the hermit by the hub's stairs (south-east of the start after
    // the walk the screen snapshots take; here, set beside the player).
    for (class, called) in [("warrior", false), ("bard", true)] {
        let mut w = start(&SHIPPED, "showcase", class);
        let hermit = SHIPPED.creature_id("hermit").expect("the hermit");
        let id = (w.state.actors.values())
            .find(|a| a.creature == hermit && a.level == w.state.current_level)
            .map(|a| a.id)
            .expect("the hermit in the hub");
        let me = w.player_pos();
        let level = &w.state.levels[&w.state.current_level];
        let dir = (Dir::ALL.into_iter())
            .find(|&d| {
                level.is_walkable(&SHIPPED, me.step(d))
                    && w.state
                        .actor_at(&w.state.current_level, me.step(d))
                        .is_none()
            })
            .expect("room beside the player");
        w.state.actors.get_mut(&id).expect("hermit").pos = me.step(dir);
        w.perceive(&mut Vec::new());
        // Saying it below the full class: nothing.
        run(&mut w, &format!("say:{dir:?}:life_before_death"));
        assert_eq!(last_message(&w), "The old hermit says nothing.", "{class}");
        reach(&mut w, CAP, &mut None);
        run(&mut w, &format!("say:{dir:?}:life_before_death"));
        assert_eq!(prestige(&w).is_some(), called, "{class}");
        if called {
            assert!(said(&w, "You are a hunter."));
            assert!(said(&w, "Go on, then, hunter."));
            assert!(w.state.actors.contains_key(&id), "the hermit stays");
            // The shipped bard, called at its cap: Hunter 1 at once, at total level 31.
            assert_eq!(w.player_level(), CAP + 1);
        } else {
            assert_eq!(last_message(&w), "The old hermit says nothing.");
        }
    }
}
