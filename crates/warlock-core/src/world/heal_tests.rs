//! Treating oneself with the Heal skill (spec §8 "Skills", #549): only those trained in Heal may
//! try, and only with something to treat; a try is an Intelligence check plus Heal against the
//! data's difficulty; bleeding stops and may be tried again after a failure; a poison eases a
//! degree and a disease is eased, each once only, the mark clearing as they end; refusals take
//! no time; marks; saves and replays.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::check;
use crate::content::{ContentFiles, DiseaseId, StatusId};
use crate::heal::Ailment;
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::Stat;
use crate::time::Duration;

/// A lit room with nothing in it.
const ROOM: &str = r##"
name = "Room"
ambient = [255, 255, 255, 0]
map = '''
#####
#@..#
#...#
#####
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"##;

/// The built-in content plus the room, with scenarios `room` (average stats, untrained), `sage`
/// (Intelligence 18), and `priest` (a human priest: trained in Heal); with `eager`, Heal earns a
/// mark on every try.
fn files(eager: bool) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/room.toml", ROOM);
    for (name, extra) in [
        ("room", "start_level = \"room\"\nhp = 1000\n"),
        (
            "sage",
            "start_level = \"room\"\nhp = 1000\nstats = { intelligence = 18 }\n",
        ),
        (
            "priest",
            "start_level = \"room\"\ncharacter = { race = \"human\", gender = \
             \"female\", class = \"priest\" }\n",
        ),
    ] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\n{extra}"),
        );
    }
    if eager {
        let text = include_str!("../../../../assets/data/skills.toml");
        let eager = text.replace(
            "mark = [{ on = \"heal\", chance = 30 }]",
            "mark = [{ on = \"heal\", chance = 100 }]",
        );
        assert_ne!(eager, text, "Heal earns marks on a try");
        files.insert("data/skills.toml", &eager);
    }
    files
}

fn content(eager: bool) -> Arc<Content> {
    Arc::new(Content::load(&files(eager)).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(false), scenario, None).expect("world")
}

fn heal_skill(w: &World) -> SkillId {
    w.content().skill_id("heal").expect("Heal")
}

/// The player trained in Heal at `value`.
fn train(w: &mut World, value: u16) {
    let id = heal_skill(w);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

/// What the player adds to a try: the Intelligence modifier and what Heal gives.
fn modifier(w: &World) -> i32 {
    let me = w.state.player();
    me.effective_stats(w.content()).modifier(Stat::Intelligence)
        + me.skill_effect(w.content(), Effect::Heal)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn messages(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

/// The player suffers `severity` more degrees of status `id`.
fn afflict(w: &mut World, id: &str, severity: u16) {
    let (player, status) = (w.state.player, status(w, id));
    w.afflict(player, status, severity, None, &mut Vec::new());
}

fn severity(w: &World, id: &str) -> Option<u16> {
    let status = status(w, id);
    w.state.player().effects.get(status).map(|e| e.severity)
}

/// The player has caught disease `id`, and it shows (not yet known by name).
fn sicken(w: &mut World, id: &str) -> DiseaseId {
    let (player, disease) = (w.state.player, w.content().disease_id(id).expect(id));
    assert!(w.infect(player, disease, None));
    let actor = w.state.actors.get_mut(&player).expect("player");
    actor.diseases.get_mut(disease).expect("caught").left = 1;
    w.apply(&Action::Wait);
    assert!(
        w.state
            .player()
            .diseases
            .get(disease)
            .is_some_and(|i| i.shown)
    );
    disease
}

fn heal_in_menu(w: &World) -> Option<Option<Unavailable>> {
    (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Heal)
        .map(|a| a.unavailable)
}

#[test]
fn only_those_trained_in_heal_may_treat_and_only_with_something_to_treat() {
    let mut w = world("room");
    afflict(&mut w, "bleeding", 5);
    assert_eq!(heal_in_menu(&w), None, "untrained: not in the menu");
    let out = w.apply(&Action::Heal(None));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(last_message(&w), "You know nothing of healing.");
    assert_eq!(severity(&w, "bleeding"), Some(5));

    let mut w = world("room");
    train(&mut w, 0);
    assert_eq!(
        heal_in_menu(&w),
        Some(Some(Unavailable::NothingToTreat)),
        "trained, nothing to treat: greyed out"
    );
    assert_eq!(
        Unavailable::NothingToTreat.reason(w.content()),
        "nothing to treat"
    );
    for (action, said) in [
        ("heal", "There is nothing to treat."),
        ("heal:bleeding", "There is nothing like that to treat."),
        ("heal:filth_fever", "There is nothing like that to treat."),
        ("heal:nonsense", "There is nothing to treat."),
    ] {
        let action = action.parse().expect("action");
        let out = w.apply(&action);
        assert!(!out.took_turn && out.elapsed.is_zero(), "{action}");
        assert_eq!(last_message(&w), said, "{action}");
    }
    afflict(&mut w, "bleeding", 5);
    assert_eq!(heal_in_menu(&w), Some(None), "bleeding: usable");
    let heal = heal_skill(&w);
    assert!(
        !(w.perception().abilities().iter())
            .any(|a| matches!(a.ability, Ability::Skill(id) if id == heal)),
        "Heal is listed as its use, not as a skill"
    );
    assert_eq!(
        Ability::Heal.action(w.content(), None),
        Some(Action::Heal(None))
    );
    assert_eq!(Ability::Heal.aim(w.content()), crate::ability::Aim::Ailment);
    // A priest is trained in it from the start.
    let mut priest = world("priest");
    afflict(&mut priest, "bleeding", 1);
    assert_eq!(heal_in_menu(&priest), Some(None));
}

#[test]
fn bleeding_is_an_intelligence_check_and_stops_or_may_be_tried_again() {
    // Intelligence 18 and Heal 100 (+8) against bleeding at five degrees (6 + 2 x 5 = 16), and
    // average and barely trained against it.
    let difficulty = 6 + 2 * 5;
    for (scenario, value, at_least, at_most) in [("sage", 100, 30, 40), ("room", 1, 5, 25)] {
        let mut stopped = 0;
        for seed in 0..40 {
            let mut w = World::new(content(false), scenario, Some(seed)).expect("world");
            train(&mut w, value);
            afflict(&mut w, "bleeding", 5);
            let plus = modifier(&w);
            let treat = (w.content().status(status(&w, "bleeding")).treat.clone()).expect("treat");
            assert_eq!(treat.difficulty_at(5), difficulty);
            let mut rng = w.state.rng.clone();
            let expected = check::check(&mut rng, plus, difficulty);
            let out = w.apply(&"heal:bleeding".parse().expect("action"));
            assert!(out.took_turn);
            assert_eq!(out.elapsed, Duration::seconds(15), "a quarter of a minute");
            assert_eq!(
                severity(&w, "bleeding").is_none(),
                expected,
                "{scenario} {seed}"
            );
            let said = messages(&w);
            if expected {
                stopped += 1;
                assert!(said.ends_with(&[
                    "You bind the wound.".to_string(),
                    "The bleeding stops.".to_string(),
                ]));
            } else {
                assert_eq!(last_message(&w), "The blood still runs.");
                assert_eq!(heal_in_menu(&w), Some(None), "it may be tried again");
            }
        }
        assert!(
            (at_least..=at_most).contains(&stopped),
            "{scenario}: {stopped} of 40"
        );
    }
    // Tried again and again, it stops.
    let mut w = world("room");
    train(&mut w, 0);
    afflict(&mut w, "bleeding", 5);
    let mut tries = 0;
    while severity(&w, "bleeding").is_some() {
        assert!(tries < 100, "bleeding stops within 100 tries");
        assert!(
            w.apply(&Action::Heal(None)).took_turn,
            "each try takes its time"
        );
        tries += 1;
    }
}

#[test]
fn a_poison_is_tried_once_eased_a_degree_and_tried_again_once_it_ends() {
    let mut made = 0;
    for seed in 0..30 {
        let mut w = World::new(content(false), "room", Some(seed)).expect("world");
        train(&mut w, 0);
        afflict(&mut w, "poisoned", 4);
        let mut rng = w.state.rng.clone();
        let expected = check::check(&mut rng, modifier(&w), 8 + 4);
        assert!(w.apply(&"heal:poisoned".parse().expect("action")).took_turn);
        let poisoned = status(&w, "poisoned");
        let effect = *w
            .state
            .player()
            .effects
            .get(poisoned)
            .expect("still poisoned");
        assert!(effect.treated, "{seed}: one try");
        if expected {
            made += 1;
            assert_eq!(effect.severity, 3, "eased a degree");
            assert_eq!(last_message(&w), "You draw out some of the poison.");
        } else {
            assert_eq!(effect.severity, 4);
            assert_eq!(last_message(&w), "You can do no more for the poison.");
        }
        // Not again, whatever came of it: refused, no time.
        assert_eq!(heal_in_menu(&w), Some(Some(Unavailable::NothingToTreat)));
        let out = w.apply(&"heal:poisoned".parse().expect("action"));
        assert!(!out.took_turn && out.elapsed.is_zero());
        assert_eq!(last_message(&w), "You can do no more for the poison.");
        let out = w.apply(&Action::Heal(None));
        assert!(!out.took_turn);
        assert_eq!(last_message(&w), "There is nothing to treat.");
        // A new dose after it ended may be tried again.
        let player = w.state.player;
        w.state
            .actors
            .get_mut(&player)
            .expect("player")
            .effects
            .remove(poisoned);
        afflict(&mut w, "poisoned", 2);
        assert_eq!(heal_in_menu(&w), Some(None));
    }
    assert!((5..=28).contains(&made), "{made} of 30");
}

#[test]
fn a_disease_is_tried_once_eased_never_cured_and_tried_again_when_caught_again() {
    let mut made = 0;
    for seed in 0..30 {
        let mut w = World::new(content(false), "room", Some(seed)).expect("world");
        train(&mut w, 0);
        let fever = sicken(&mut w, "filth_fever");
        let own = w.perception().own_ailments();
        assert_eq!(own.len(), 1);
        assert_eq!(own[0].name, "Sickness", "not known by name yet");
        assert_eq!(own[0].action, "heal:filth_fever".parse().expect("action"));
        let before = *w.state.player().diseases.get(fever).expect("it");
        let difficulty = w.content().disease(fever).difficulty;
        let mut rng = w.state.rng.clone();
        let expected = check::check(&mut rng, modifier(&w), difficulty);
        assert!(w.apply(&Action::Heal(None)).took_turn);
        let after = *w
            .state
            .player()
            .diseases
            .get(fever)
            .expect("never cured outright");
        assert!(after.treated);
        if expected {
            made += 1;
            assert_eq!(last_message(&w), "You ease the sickness somewhat.");
            // Filth fever is shaken off by two checks won in a row: this counts as one.
            assert_eq!(after.streak, before.streak + 1);
        } else {
            assert_eq!(last_message(&w), "You can do no more for the sickness.");
            assert_eq!(after.streak, before.streak);
        }
        let out = w.apply(&Action::Heal(Some("filth_fever".into())));
        assert!(!out.took_turn);
        assert_eq!(last_message(&w), "You can do no more for the sickness.");
        // Rid of it and caught again, it may be tried again.
        let player = w.state.player;
        w.end_disease(player, fever, &mut Vec::new());
        assert_eq!(heal_in_menu(&w), Some(Some(Unavailable::NothingToTreat)));
        sicken(&mut w, "filth_fever");
        assert_eq!(heal_in_menu(&w), Some(None));
    }
    assert!((3..=27).contains(&made), "{made} of 30");
}

#[test]
fn a_disease_that_never_recovers_or_is_one_check_from_it_has_its_next_turn_put_off() {
    for (id, streak) in [("grave_rot", 0), ("filth_fever", 1)] {
        let mut eased = false;
        for seed in 0..40 {
            let mut w = World::new(content(false), "sage", Some(seed)).expect("world");
            train(&mut w, 200);
            let disease = sicken(&mut w, id);
            let player = w.state.player;
            let infection = (w.state.actors.get_mut(&player))
                .and_then(|a| a.diseases.get_mut(disease))
                .expect("it");
            infection.streak = streak;
            infection.left = 100;
            w.apply(&Action::Heal(Some(id.into())));
            if last_message(&w).starts_with("You ease") {
                let after = *w.state.player().diseases.get(disease).expect("not cured");
                let interval = w.content().disease(disease).interval_seconds();
                assert_eq!(after.streak, streak, "{id}");
                // A full interval from the try (less the try's own seconds, which passed).
                assert!(
                    u64::from(after.left) > interval - 60,
                    "{id}: {}",
                    after.left
                );
                eased = true;
                break;
            }
        }
        assert!(eased, "{id}: eased at least once in 40 tries");
    }
}

#[test]
fn several_ailments_are_listed_bleeding_first() {
    let mut w = world("room");
    train(&mut w, 0);
    sicken(&mut w, "filth_fever");
    afflict(&mut w, "poisoned", 2);
    afflict(&mut w, "bleeding", 3);
    let names: Vec<String> = (w.perception().own_ailments().into_iter())
        .map(|a| a.name)
        .collect();
    assert_eq!(names, ["Bleeding", "Poisoned", "Sickness"]);
    let me = w.state.player();
    let first = crate::heal::treatable(me, w.content())[0];
    assert_eq!(first, Ailment::Status(status(&w, "bleeding")));
}

#[test]
fn a_try_may_earn_heal_a_mark() {
    let mut w = World::new(content(true), "room", None).expect("world");
    train(&mut w, 0);
    afflict(&mut w, "bleeding", 5);
    w.apply(&Action::Heal(None));
    let heal = heal_skill(&w);
    let marks = w.state.player().skills.get(heal).map(|t| t.marks);
    assert_eq!(marks, Some(1), "a try, whatever came of it");
}

#[test]
fn treated_flags_save_replay_and_hash() {
    let mut w = world("priest");
    let mut replay = Replay::new(&w);
    let text = "afflict:bleeding*2 afflict:poisoned*2 heal:poisoned heal heal heal wait";
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let poisoned = status(&w, "poisoned");
    assert!(
        w.state
            .player()
            .effects
            .get(poisoned)
            .is_some_and(|e| e.treated)
    );
    let (again, divergence) = replay.play(content(false)).expect("plays");
    assert!(divergence.is_none(), "the replay verifies");
    assert_eq!(again.state_hash(), w.state_hash());
    // The flags are saved state: an infection's too.
    let fever = sicken(&mut w, "filth_fever");
    let player = w.state.player;
    (w.state.actors.get_mut(&player))
        .and_then(|a| a.diseases.get_mut(fever))
        .expect("it")
        .treated = true;
    let bytes = crate::save::encode_state(w.state(), w.fingerprint());
    let loaded = crate::save::decode(&bytes, content(false)).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let me = loaded.state.player();
    assert!(me.effects.get(poisoned).is_some_and(|e| e.treated));
    assert!(me.diseases.get(fever).is_some_and(|i| i.treated));
}
