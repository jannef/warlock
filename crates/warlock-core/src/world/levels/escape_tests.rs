//! Escape artist (#1015, `hold.rs`, `trap.rs`): the automatic check when bonds stop or slow what
//! the player tries, what it adds up to, what it costs, its marks, and a pit's shorter climb.

use crate::ability::Ability;
use crate::check::check;
use crate::content::{ContentFiles, StatusId};
use crate::geom::{Dir, Pos};
use crate::skills::{Effect as SkillEffect, SkillId, Trained};
use crate::stats::{PerStat, Stat, Stats};
use crate::time::Duration;
use crate::world::*;

/// A marked status that only slows, as the data may one day hold.
const SNARED: &str = "[[status]]\nid = \"zz_snared\"\nname = \"Snared\"\nseen = \"snared\"\n\
                      max = 3\neases = 60\ncost = 200\nescape = 13\nend = \"You kick free.\"\n";

/// The player, room to step west, and a pit east.
const ROOM: &str = "#########\n#...@p..#\n#.......#\n#########\n";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    let statuses = files.table_text("status");
    let statuses = format!("{statuses}{SNARED}");
    files.replace_table("status", &statuses);
    files.insert(
        "levels/escape.toml",
        &format!(
            "name = \"Escape\"\nambient = [255, 255, 255, 0]\nmap = '''\n{ROOM}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"p\" = {{ terrain = \"floor\", trap = \"pit\" }}\n"
        ),
    );
    files.insert(
        "scenarios/escape.toml",
        "name = \"Escape\"\nstart_level = \"escape\"\n",
    );
    files
}

fn world() -> World {
    let content = Content::load(&files()).expect("test content is valid");
    let mut w = World::new(Arc::new(content), "escape", Some(3)).expect("world");
    let me = w.player_mut();
    me.hp_base = 500;
    me.hp = 500;
    w
}

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect("status")
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content().skill_id(id).expect("skill")
}

/// The player trained in `id` at `value`, marks gone.
fn train(w: &mut World, id: &str, value: u16) {
    let id = skill(w, id);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> u32 {
    (w.state.player().skills.get(skill(w, id))).map_or(0, |t| t.marks)
}

/// The player's Strength and Dexterity at these scores, the rest at 10.
fn strong_and_quick(w: &mut World, strength: u8, dexterity: u8) {
    let mut scores = PerStat::splat(10);
    scores[Stat::Strength] = strength;
    scores[Stat::Dexterity] = dexterity;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
}

fn suffer(w: &mut World, id: &str, degrees: u16) {
    let status = status(w, id);
    let player = w.state.player;
    w.player_mut().effects.remove(status);
    w.afflict(player, status, degrees, None, &mut Vec::new());
}

fn said(w: &World, turn: u64) -> Vec<String> {
    (w.messages().iter())
        .filter(|m| m.turn >= turn)
        .map(|m| m.text.clone())
        .collect()
}

/// A step west from the start under a fresh `id`: whether the player got away, with what it cost
/// and what was said.
fn step_under(w: &mut World, start: Pos, id: &str) -> (bool, Duration, Vec<String>) {
    w.player_mut().pos = start;
    suffer(w, id, 3);
    let turn = w.state.turn;
    let out = w.apply(&Action::Move(Dir::W));
    (w.player_pos() != start, out.elapsed, said(w, turn))
}

#[test]
fn the_skill_gives_what_the_design_says_and_no_ability() {
    let w = world();
    let def = w.content().skill(skill(&w, "escape_artist"));
    let at = |effect: SkillEffect, value: u16| {
        (def.benefits.iter())
            .find(|b| b.effect == effect)
            .expect("benefit")
            .at(value)
    };
    assert_eq!(
        [0, 50, 100, 150, 200, 999].map(|v| at(SkillEffect::Escape, v)),
        [0, 4, 8, 10, 12, 12]
    );
    assert_eq!(
        [0, 49, 50, 99, 100, 999].map(|v| at(SkillEffect::Climb, v)),
        [0, 0, 1, 1, 2, 2]
    );
    // What is bonds and what is not.
    for id in ["webbed", "pinned"] {
        assert_eq!(w.content().status(status(&w, id)).escape, Some(13), "{id}");
    }
    assert!(w.content().status(status(&w, "held")).hold.is_some());
    for id in [
        "staggered",
        "stunned",
        "surprised",
        "transfixed",
        "sluggish",
        "drowsy",
    ] {
        assert_eq!(w.content().status(status(&w, id)).escape, None, "{id}");
    }
}

#[test]
fn trained_or_not_the_menu_has_no_row() {
    let mut w = world();
    train(&mut w, "escape_artist", 100);
    w.apply(&Action::Wait);
    let abilities = w.perception().abilities();
    let id = skill(&w, "escape_artist");
    assert!(!abilities.iter().any(|a| a.ability == Ability::Skill(id)));
}

#[test]
fn untrained_webbed_and_pinned_are_refused_for_no_time_and_no_draw() {
    for id in ["webbed", "pinned"] {
        let mut w = world();
        suffer(&mut w, id, 3);
        let (rng, start, turn) = (w.state.rng.clone(), w.player_pos(), w.state.turn);
        let out = w.apply(&Action::Move(Dir::W));
        assert!(!out.took_turn, "{id}");
        assert_eq!(w.player_pos(), start);
        assert_eq!(w.state.rng, rng, "{id}");
        let name = w.content().status(status(&w, id)).name.to_lowercase();
        assert!(
            said(&w, turn).contains(&format!("You cannot move while {name}.")),
            "{:?}",
            said(&w, turn)
        );
        assert!(w.state.player().effects.get(status(&w, id)).is_some());
    }
}

#[test]
fn trained_webbed_and_pinned_struggle_as_the_step_is_tried() {
    for (id, end, still) in [
        (
            "webbed",
            "You tear free of the web.",
            "You struggle, but you are still caught in a web.",
        ),
        (
            "pinned",
            "You can move again.",
            "You struggle, but you are still pinned.",
        ),
    ] {
        let mut w = world();
        train(&mut w, "escape_artist", 10);
        strong_and_quick(&mut w, 10, 10);
        let start = w.player_pos();
        let step = Duration::seconds(6);
        let (mut made, mut failed) = (0, 0);
        for _ in 0..40 {
            let (away, took, lines) = step_under(&mut w, start, id);
            assert_eq!(took, step, "{id}: the step's price either way, {lines:?}");
            if away {
                made += 1;
                assert!(lines.iter().any(|l| l == end), "{lines:?}");
                assert!(w.state.player().effects.get(status(&w, id)).is_none());
            } else {
                failed += 1;
                assert!(lines.iter().any(|l| l == still), "{lines:?}");
            }
        }
        // About even at 2d12 against 13 with no modifier.
        assert!(
            made >= 8 && failed >= 8,
            "{id}: {made} made, {failed} failed"
        );
    }
}

#[test]
fn a_failed_struggle_against_a_pin_spends_the_action_that_eases_it() {
    let mut w = world();
    train(&mut w, "escape_artist", 1);
    strong_and_quick(&mut w, 1, 1);
    let pinned = status(&w, "pinned");
    let start = w.player_pos();
    suffer(&mut w, "pinned", 3);
    // Given before this action, as a creature's or a trap's is.
    w.fresh.clear();
    w.apply(&Action::Move(Dir::W));
    if w.player_pos() == start {
        assert_eq!(
            w.state.player().effects.get(pinned).map(|e| e.severity),
            Some(2)
        );
    }
}

#[test]
fn the_better_of_strength_and_dexterity_counts_both_ways_with_the_skill() {
    let mut w = world();
    train(&mut w, "escape_artist", 100);
    let plus = |score: u8| {
        let scores = Stats::new(PerStat::splat(score), PerStat::splat(40));
        scores.modifier(Stat::Strength)
    };
    strong_and_quick(&mut w, 30, 10);
    assert_eq!(w.slipping(Stat::Strength), plus(30) + 8, "the Strength");
    strong_and_quick(&mut w, 10, 30);
    assert_eq!(w.slipping(Stat::Strength), plus(30) + 8, "the Dexterity");
    // The attempts bear it out: a modifier of 20 or more slips all but a natural 2.
    train(&mut w, "escape_artist", 200);
    let start = w.player_pos();
    for (strength, dexterity) in [(40, 10), (10, 40)] {
        strong_and_quick(&mut w, strength, dexterity);
        let away = (0..20)
            .filter(|_| step_under(&mut w, start, "webbed").0)
            .count();
        assert!(away >= 19, "St {strength}, Dx {dexterity}: {away}");
    }
}

#[test]
fn held_trained_may_slip_by_dexterity_untrained_only_by_strength() {
    // No holder: the hold's own grip of 13.
    let attempts = |value: Option<u16>| {
        let mut w = world();
        if let Some(value) = value {
            train(&mut w, "escape_artist", value);
        }
        strong_and_quick(&mut w, 1, 40);
        let start = w.player_pos();
        (0..30)
            .filter(|_| {
                let (away, took, lines) = step_under(&mut w, start, "held");
                assert_eq!(took, Duration::seconds(6), "{lines:?}");
                let line = if away {
                    "You break free."
                } else {
                    "You struggle, but cannot break free."
                };
                assert!(lines.iter().any(|l| l == line), "{lines:?}");
                away
            })
            .count()
    };
    assert!(attempts(Some(100)) >= 28);
    assert!(attempts(None) <= 12, "Strength 1 alone");
}

#[test]
fn a_marked_status_that_slows_is_checked_and_the_action_goes_ahead_either_way() {
    let mut w = world();
    train(&mut w, "escape_artist", 10);
    strong_and_quick(&mut w, 10, 10);
    let start = w.player_pos();
    let (mut made, mut failed) = (0, 0);
    for _ in 0..40 {
        let (away, took, lines) = step_under(&mut w, start, "zz_snared");
        assert!(away, "it never stops the step: {lines:?}");
        if w.state
            .player()
            .effects
            .get(status(&w, "zz_snared"))
            .is_none()
        {
            made += 1;
            assert_eq!(took, Duration::seconds(6), "its plain price");
            assert!(lines.iter().any(|l| l == "You kick free."), "{lines:?}");
        } else {
            failed += 1;
            assert_eq!(took, Duration::seconds(12), "slowed, no worse");
            let still = "You struggle, but you are still snared.";
            assert!(lines.iter().any(|l| l == still), "{lines:?}");
        }
    }
    assert!(made >= 8 && failed >= 8, "{made} made, {failed} failed");
}

#[test]
fn no_check_for_a_status_with_no_mark_nor_for_the_untrained_slowed() {
    let mut w = world();
    train(&mut w, "escape_artist", 100);
    suffer(&mut w, "staggered", 1);
    let gameplay = w.state.rng.clone();
    assert_eq!(w.struggle(&Action::Move(Dir::W), &mut Vec::new()), None);
    assert_eq!(w.state.rng, gameplay);
    assert_eq!(marks(&w, "escape_artist"), 0);

    let mut w = world();
    suffer(&mut w, "zz_snared", 1);
    let gameplay = w.state.rng.clone();
    assert_eq!(w.struggle(&Action::Move(Dir::W), &mut Vec::new()), None);
    assert_eq!(w.state.rng, gameplay);
    assert!(
        w.state
            .player()
            .effects
            .get(status(&w, "zz_snared"))
            .is_some()
    );
}

#[test]
fn what_bonds_do_not_stop_is_no_struggle() {
    let mut w = world();
    train(&mut w, "escape_artist", 100);
    suffer(&mut w, "webbed", 3);
    let gameplay = w.state.rng.clone();
    assert_eq!(w.struggle(&Action::Wait, &mut Vec::new()), None);
    assert_eq!(w.state.rng, gameplay);
    assert_eq!(marks(&w, "escape_artist"), 0);
}

#[test]
fn a_check_is_two_draws_of_the_gameplay_stream_and_a_mark_whatever_came_of_it() {
    let mut w = world();
    train(&mut w, "escape_artist", 10);
    strong_and_quick(&mut w, 10, 10);
    let mut checks = 0;
    let (mut made, mut failed) = (false, false);
    while !(made && failed) {
        suffer(&mut w, "webbed", 3);
        let mut gameplay = w.state.rng.clone();
        check(&mut gameplay, 0, 13);
        let training = w.state.training.clone();
        let spent = w.struggle(&Action::Move(Dir::W), &mut Vec::new());
        assert_eq!(w.state.rng, gameplay);
        assert_ne!(w.state.training, training, "the stats' chances draw");
        checks += 1;
        assert_eq!(marks(&w, "escape_artist"), checks);
        made |= spent.is_none();
        failed |= spent.is_some();
        assert!(checks < 100);
    }
}

#[test]
fn a_second_status_blocking_the_step_asks_on_the_next_attempt() {
    let mut w = world();
    train(&mut w, "escape_artist", 200);
    strong_and_quick(&mut w, 40, 40);
    suffer(&mut w, "webbed", 3);
    suffer(&mut w, "held", 3);
    let start = w.player_pos();
    // Made (all but a natural 2): the first gone, the other refuses this step for no time.
    let mut out = w.apply(&Action::Move(Dir::W));
    while out.took_turn {
        out = w.apply(&Action::Move(Dir::W));
    }
    assert_eq!(w.player_pos(), start);
    let left = [status(&w, "webbed"), status(&w, "held")]
        .iter()
        .filter(|&&s| w.state.player().effects.get(s).is_some())
        .count();
    assert_eq!(left, 1);
    assert_eq!(marks(&w, "escape_artist"), 1 + u32::from(left == 0));
    // The next attempt asks that one.
    w.player_mut().pos = start;
    while w.player_pos() == start {
        w.apply(&Action::Move(Dir::W));
    }
    assert!(marks(&w, "escape_artist") >= 2);
}

#[test]
fn a_pits_climb_is_halved_from_50_and_skipped_from_100() {
    for (value, took) in [
        (None, 12),
        (Some(49), 12),
        (Some(50), 9),
        (Some(99), 9),
        (Some(100), 6),
        (Some(300), 6),
    ] {
        let mut w = world();
        if let Some(value) = value {
            train(&mut w, "escape_artist", value);
        }
        let out = w.apply(&Action::Move(Dir::E));
        assert!(
            w.messages()
                .iter()
                .any(|m| m.text == "You fall into a pit."),
            "{value:?}"
        );
        assert_eq!(out.elapsed, Duration::seconds(took), "{value:?}");
    }
}

#[test]
fn a_climb_marks_escape_artist_always_and_athletics_half_the_time_with_no_effect() {
    let mut w = world();
    train(&mut w, "escape_artist", 1);
    train(&mut w, "athletics", 1);
    let start = w.player_pos();
    // 99 climbs: the 100th mark would roll Escape artist by itself (`[rolls] auto_at`).
    for _ in 0..99 {
        w.player_mut().pos = start;
        // Hidden again, with no surprise left from the last: the player cannot sidestep it.
        let surprised = status(&w, "surprised");
        w.player_mut().effects.remove(surprised);
        let at = start.step(Dir::E);
        (w.state.current_mut().traps.get_mut(&at).expect("the pit")).known = false;
        let out = w.apply(&Action::Move(Dir::E));
        assert_eq!(out.elapsed, Duration::seconds(12), "no cut at 1");
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
    }
    assert_eq!(marks(&w, "escape_artist"), 99);
    let athletics = marks(&w, "athletics");
    assert!((30..=70).contains(&athletics), "{athletics}");
}

#[test]
fn the_data_checks_refuse_a_bad_escape() {
    for (fields, why) in [
        ("seen = \"x\"\nescape = 13\n", "must block"),
        ("seen = \"x\"\ncost = 75\nescape = 13\n", "must block"),
        (
            "seen = \"x\"\nescape = 13\nstacking = \"highest\"\neases = 6\nblocks = [\"step\"]\n\
             hold = { difficulty = 13, turns = \"1d2\" }\n",
            "grip",
        ),
    ] {
        let mut files = ContentFiles::builtin();
        let statuses = files.table_text("status");
        let text =
            format!("{statuses}[[status]]\nid = \"zz_bad\"\nname = \"Bad\"\nmax = 3\n{fields}");
        files.replace_table("status", &text);
        let error = Content::load(&files).expect_err(why).to_string();
        assert!(error.contains(why), "{why}: {error}");
    }
}
