//! Sanctuary (`world/sanctuary.rs`, spec §8 "Class abilities", §7 "Special rooms": hallowed
//! ground; the priest design §6.4, §7): the rite and its refusals, who keeps out, the pace, the
//! break, the fade, and saves and replays.

use super::caster_tests::{reach, run, said};
use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::alignment::Score;
use crate::character::CharacterSpec;
use crate::content::{ClassActive, ContentFiles, Factor};
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::stats::{PerStat, Stats};
use crate::test_support::CONTENT as BUILTIN;

/// A human priest of `level` (every score 14), Neutral, on `scenario`'s start.
fn priest_on(scenario: &str, level: u8) -> World {
    let spec: CharacterSpec = "human,female,priest".parse().expect("a character");
    let spec = spec.with_talents(vec!["keen_ears".into(), "lean".into(), "pious".into()]);
    let w = World::start_run(BUILTIN.clone(), scenario, Some(7), Some(&spec), None, true)
        .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(BUILTIN.clone(), state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w.player_mut().alignment = Some(Score::new(0));
    reach(&mut w, level);
    w
}

/// A hall, 25 wide, the player at (19, 3), a giant rat (hostile) at `rat` if any; no class.
fn hall(rat: Option<Pos>) -> World {
    let mut rows: Vec<Vec<char>> = "\
#########################
#.......................#
#.......................#
#.......................#
#.......................#
#.......................#
#########################"
        .lines()
        .map(|l| l.chars().collect())
        .collect();
    rows[3][19] = '@';
    if let Some(r) = rat {
        rows[r.y as usize][r.x as usize] = 'r';
    }
    let map: String = rows
        .iter()
        .map(|r| r.iter().collect::<String>() + "\n")
        .collect();
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_hallow.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"r\" = {{ terrain = \"floor\", creature = \"rat\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_hallow.toml",
        "name = \"T\"\nstart_level = \"test_hallow\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_hallow", None).expect("world")
}

/// Ground hallowed about `at` (radius 2, three times the pace) for an hour from now.
fn lay(w: &mut World, at: Pos) {
    let until = w.state.time + crate::time::Duration::seconds(3600);
    let hallow = Hallow {
        at,
        radius: 2,
        until,
        regen: 300,
        dark: false,
    };
    w.state.current_mut().hallows.push(hallow);
}

fn rat(w: &World) -> Option<Pos> {
    (w.state.actors.values())
        .find(|a| w.content.creature(a.creature).id == "rat")
        .map(|a| a.pos)
}

const HERE: Pos = Pos::new(19, 3);

#[test]
fn the_rite_hallows_the_ground_for_its_mana_and_a_minute() {
    let mut w = priest_on("showcase", 50);
    let ability = Ability::Class(ClassActive::Sanctuary);
    assert!(ability.had_by(w.state.player(), w.content()));
    assert!(!ability.had_by(priest_on("showcase", 49).state.player(), w.content()));
    let (mana, time, rng) = (w.state.player().mana, w.state.time, w.state.rng.clone());
    let cost = w.state.player().mana_cost(w.content(), 30);
    run(&mut w, "use:sanctuary");
    let told = said(&w);
    assert!(told.iter().any(|m| m == "You kneel and pray."), "{told:?}");
    assert!(
        told.iter()
            .any(|m| m == "The ground about you is hallowed."),
        "{told:?}"
    );
    assert!(w.state.time.since(time).as_seconds() >= 60, "a minute");
    let left = w.state.player().mana;
    assert!(
        cost > 0 && left < mana && left >= mana - cost,
        "its mana spent: {left}"
    );
    assert_eq!(w.state.rng, rng, "no draw");
    let here = w.player_pos();
    let hallows = w.hallows();
    assert_eq!(hallows.len(), 1);
    assert_eq!((hallows[0].at(), hallows[0].radius()), (here, 2));
    assert_eq!(
        w.perception().hallow(here),
        Some(false),
        "the player knows it"
    );
    assert!(
        (dump::layer_rows(&w, Layer::Truth).iter()).any(|r| r.starts_with("hallow at ")),
        "the truth layer lists it"
    );
    // Ground already hallowed is refused, with no time.
    w.player_mut().mana = 40;
    let turn = w.state.turn;
    run(&mut w, "use:sanctuary");
    assert_eq!(w.state.turn, turn);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("This ground is already hallowed.")
    );
}

#[test]
fn the_rite_is_refused_on_the_overworld_and_without_its_mana() {
    let mut w = priest_on("world", 50);
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    run(&mut w, "use:sanctuary");
    assert_eq!(
        (w.state.turn, w.state.player().mana),
        (turn, mana),
        "refused free"
    );
    assert!(
        said(&w)
            .last()
            .is_some_and(|m| m.starts_with("The open land"))
    );
    assert!(w.hallows().is_empty());
    let mut w = priest_on("showcase", 50);
    w.player_mut().mana = 5;
    let turn = w.state.turn;
    run(&mut w, "use:sanctuary");
    assert_eq!(w.state.turn, turn, "too little mana");
    assert!(w.hallows().is_empty());
}

#[test]
fn a_hostile_outside_never_steps_onto_hallowed_ground() {
    let mut w = hall(Some(Pos::new(12, 3)));
    lay(&mut w, HERE);
    for _ in 0..40 {
        w.apply(&Action::Wait);
        let at = rat(&w).expect("the rat");
        assert!(at.chebyshev(HERE) > 2, "the rat stays out, at {at}");
    }
    let (hp, max) = w.perception().own_hp();
    assert_eq!(hp, max, "unhurt");
    // Look mode names the ground.
    assert_eq!(w.perception().hallow(Pos::new(18, 3)), Some(false));
    assert_eq!(w.perception().hallow(Pos::new(16, 3)), None);
    // Without it, the rat comes.
    w.state.current_mut().hallows.clear();
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    let at = rat(&w).expect("the rat");
    assert!(at.chebyshev(HERE) <= 1, "it closes in, at {at}");
}

#[test]
fn one_inside_when_it_is_laid_comes_on_and_its_blows_break_nothing() {
    let mut w = hall(Some(Pos::new(17, 3)));
    lay(&mut w, HERE);
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    let at = rat(&w).expect("the rat");
    assert!(at.chebyshev(HERE) <= 1, "it closes in, at {at}");
    assert_eq!(w.hallows().len(), 1, "being struck breaks nothing");
}

#[test]
fn hallowed_ground_heals_and_restores_mana_three_times_as_fast() {
    let healed = |hallowed: bool| {
        let mut w = hall(None);
        if hallowed {
            lay(&mut w, HERE);
        }
        w.player_mut().hp = 1;
        w.player_mut().mana_base = 60;
        w.player_mut().mana = 0;
        // Four minutes: one hit point back at Toughness 10.
        for _ in 0..40 {
            w.apply(&Action::Wait);
        }
        let me = w.state.player();
        (me.hp - 1, me.mana)
    };
    let (hp, mana) = healed(false);
    let (hallowed_hp, hallowed_mana) = healed(true);
    assert_eq!((hp, hallowed_hp), (1, 3));
    assert!(
        mana > 0 && hallowed_mana >= 3 * mana - 1,
        "{mana} {hallowed_mana}"
    );
}

#[test]
fn a_blow_from_within_breaks_the_peace() {
    let mut w = hall(Some(Pos::new(16, 3)));
    w = {
        let mut state = w.state().clone();
        let player = state.player;
        state.actors.get_mut(&player).expect("player").pos = Pos::new(17, 3);
        World::from_state(w.content().clone(), state).expect("state")
    };
    lay(&mut w, HERE);
    w.apply(&Action::Move(Dir::W));
    assert!(w.hallows().is_empty(), "broken");
    assert!(said(&w).iter().any(|m| m == "The peace is broken."));
}

#[test]
fn the_hallowing_fades_after_its_hour() {
    let mut w = hall(None);
    lay(&mut w, HERE);
    run(&mut w, "rest:2h");
    assert!(w.hallows().is_empty(), "faded");
    assert!(said(&w).iter().any(|m| m == "The hallowing fades."));
    assert_eq!(w.perception().hallow(HERE), None);
}

#[test]
fn sanctuary_saves_and_replays_the_same() {
    let spec: CharacterSpec = "human,female,priest".parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut replay = Replay::new(&w);
    let factor = Factor::of(w.content(), Some(&spec));
    let need = w.content().experience().threshold_for(50, factor);
    let actions = format!("xp:{need} levelup:auto use:sanctuary wait*5");
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.hallows().len(), 1, "{:?}", said(&w));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(BUILTIN.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
}
