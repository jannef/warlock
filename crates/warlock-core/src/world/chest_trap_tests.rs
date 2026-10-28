//! Trapped chests (spec §8 "Chests", docs/design/chests-and-treasure.md §3.5, §9;
//! `world/chest.rs`): a chest's trap springs at whoever opens it (a bump, a key, Pick lock) and
//! at whoever smashes it with a kick, never on a kick that holds; its arrow flies out of the box;
//! a burst blows the chest apart with nothing spared (the fireball's own blast on a coffer, the
//! frost's or a kist's by the rule); Disarm made leaves the chest shut and untrapped, failed
//! badly at a burst blows it apart; searching finds it with the chest's sign; Chest-wise sees
//! it; Trap sense halves its blow; a blast from afar springs nothing and drops it; levels refuse
//! a pit on a chest and a trap on an open one; the dumps; a replay.

use super::search::Hidden;
use super::*;
use crate::action::parse_actions;
use crate::check::{self, Checked};
use crate::content::ContentFiles;
use crate::dump::{self, Layer};
use crate::replay::Replay;
use crate::rng::Rng;
use crate::skills::{SkillId, Trained};

/// A lit hall: the player at the west end of the walk (row 2), trapped chests along row 1: a
/// coffer with a hidden arrow trap holding bread (`A`, x 1), a coffer with a fireball trap
/// holding a ruby (`F`, x 3), a kist with a frost trap (`K`, x 5), a locked coffer with a spear
/// trap holding a dagger and coins (`S`, x 7), and a coffer with a known alarm trap (`N`, x 9).
const HALL: &str = r##"
name = "Trap chest hall"
ambient = [255, 255, 255, 0]
item_level = 2
map = '''
#############
#A.F.K.S.N..#
#@..........#
#...........#
#############
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"A" = { terrain = "chest", trap = "arrow", objects = ["bread"] }
"F" = { terrain = "chest", trap = "fireball", objects = ["gem:ruby"] }
"K" = { terrain = "chest", chest = "kist", trap = "frost", objects = ["gem:ruby"] }
"S" = { terrain = "chest", locked = true, trap = "spear", objects = ["dagger", "gold_coin*12"] }
"N" = { terrain = "chest", trap = "alarm", known = true }
"##;

const ARROW: Pos = Pos::new(1, 1);
const FIRE: Pos = Pos::new(3, 1);
const KIST: Pos = Pos::new(5, 1);
const SPEAR: Pos = Pos::new(7, 1);
const ALARM: Pos = Pos::new(9, 1);

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/trap_chest_hall.toml", HALL);
    for (name, extra) in [
        ("tchests", "hp = 1000\n"),
        ("tstrong", "hp = 1000\nstats = { strength = 18 }\n"),
        ("tweak", "hp = 1000\nstats = { strength = 3 }\n"),
        ("tclumsy", "hp = 1000\nstats = { dexterity = 3 }\n"),
        ("tsharp", "hp = 1000\nstats = { intelligence = 18 }\n"),
    ] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"trap_chest_hall\"\n{extra}"),
        );
    }
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("fixture content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(), scenario, None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn terrain(w: &World, p: Pos) -> String {
    w.content().terrain(w.state.current().terrain[p]).id.clone()
}

fn trapped(w: &World, p: Pos) -> bool {
    w.state.current().traps.contains_key(&p)
}

/// How many of object `id` lie in the pile at `p`.
fn lying(w: &World, p: Pos, id: &str) -> u32 {
    let object = w.content().object_id(id).expect("object");
    (w.state.current().objects_at(p).iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// The player walks along row 2 to stand below x.
fn below(w: &mut World, x: i32) {
    while w.player_pos().x < x {
        w.apply(&Action::Move(Dir::E));
    }
    assert_eq!(w.player_pos(), Pos::new(x, 2));
}

/// Whether the trap at `at` went off at the player in `events`.
fn sprung_at_player(w: &World, events: &[Event], at: Pos) -> bool {
    let player = w.state.player;
    (events.iter()).any(
        |e| matches!(e, Event::TrapSprung { at: a, victim, .. } if *a == at && *victim == player),
    )
}

/// Fills the chest at `p` with `n` loose rubies, each its own stack.
fn rubies(w: &mut World, p: Pos, n: usize) {
    let ruby = w.state.current().chests[&FIRE].holds[0].clone();
    let chest = w.state.current_mut().chests.get_mut(&p).expect("chest");
    chest.holds = vec![ruby; n];
}

/// The player trained in `skill` to `value`.
fn train(w: &mut World, skill: &str, value: u16) {
    let id = w.content().skill_id(skill).expect("skill");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

#[test]
fn levels_trap_shut_chests_but_never_with_a_pit_nor_an_open_chest() {
    let w = world("tchests");
    let level = w.state.current();
    assert!(!level.traps[&ARROW].known, "hidden unless `known`");
    assert!(level.traps[&ALARM].known);
    for (legend, why) in [
        (r#"{ terrain = "chest", trap = "pit" }"#, "a pit on a chest"),
        (
            r#"{ terrain = "chest_open", trap = "arrow" }"#,
            "a trap on an open chest",
        ),
        (
            r#"{ terrain = "chest_smashed", trap = "arrow" }"#,
            "a trap on a smashed chest",
        ),
    ] {
        let mut files = files();
        let text = HALL.replace(
            r#"{ terrain = "chest", trap = "alarm", known = true }"#,
            legend,
        );
        files.insert("levels/trap_chest_hall.toml", &text);
        let error = Content::load(&files).expect_err(why);
        assert!(error.to_string().contains("chest"), "{why}: {error}");
    }
}

#[test]
fn an_arrow_flies_out_of_a_chest_at_whoever_opens_it() {
    let mut w = world("tchests");
    let outcome = w.apply(&Action::Move(Dir::N));
    let events = &outcome.events;
    assert!(said(&w, "You open the coffer."));
    assert!(sprung_at_player(&w, events, ARROW), "{events:?}");
    let from = (events.iter()).find_map(|e| match e {
        Event::Missile { from, .. } => Some(*from),
        _ => None,
    });
    assert_eq!(from, Some(Some(ARROW)), "out of the box itself");
    assert_eq!(terrain(&w, ARROW), "chest_open", "an arrow leaves it whole");
    assert!(!trapped(&w, ARROW), "its trap went with the lid");
    assert_eq!(lying(&w, ARROW, "bread"), 1);
    assert!(said(&w, "Inside you find a bread.") || said(&w, "Inside you find a loaf of bread."));
}

#[test]
fn a_fireball_in_a_coffer_blows_it_apart_sparing_nothing() {
    let mut w = world("tchests");
    rubies(&mut w, FIRE, 200);
    below(&mut w, 3);
    let outcome = w.apply(&Action::Open(Dir::N));
    assert!(sprung_at_player(&w, &outcome.events, FIRE));
    assert_eq!(terrain(&w, FIRE), "chest_smashed");
    assert!(said(&w, "The coffer is blown apart!"));
    let gems = lying(&w, FIRE, "gem");
    assert!((60..=140).contains(&gems), "{gems} rubies of 200 survive");
    assert!(!trapped(&w, FIRE));
    assert!(!said(&w, "Inside you find 200 rubies."));
    // Said after the burst's own report, the lines in the order they happened.
    let texts: Vec<&str> = w.messages().iter().map(|m| m.text.as_str()).collect();
    let at = |line: &str| texts.iter().rposition(|t| *t == line).expect(line);
    assert!(
        at("You open the coffer.") < at("A fireball explodes!"),
        "{texts:?}"
    );
    assert!(
        at("A fireball explodes!") < at("The coffer is blown apart!"),
        "{texts:?}"
    );
}

#[test]
fn a_frost_trap_blows_a_kist_apart_though_its_blast_is_nothing() {
    let mut w = world("tchests");
    rubies(&mut w, KIST, 200);
    below(&mut w, 5);
    let outcome = w.apply(&Action::Open(Dir::N));
    assert!(sprung_at_player(&w, &outcome.events, KIST));
    assert_eq!(terrain(&w, KIST), "chest_smashed");
    assert!(said(&w, "The kist is blown apart!"));
    let gems = lying(&w, KIST, "gem");
    assert!((60..=140).contains(&gems), "{gems} rubies of 200 survive");
}

#[test]
fn a_kick_that_holds_springs_nothing_and_one_that_smashes_springs_the_trap() {
    let mut w = world("tweak");
    below(&mut w, 7);
    loop {
        let outcome = w.apply(&Action::Kick(Dir::N));
        if terrain(&w, SPEAR) == "chest" {
            assert!(
                !(outcome.events.iter()).any(|e| matches!(e, Event::TrapSprung { .. })),
                "a kick that holds springs nothing"
            );
            assert!(trapped(&w, SPEAR));
            continue;
        }
        assert!(sprung_at_player(&w, &outcome.events, SPEAR), "the kicker");
        break;
    }
    assert_eq!(
        terrain(&w, SPEAR),
        "chest_smashed",
        "a spear leaves it smashed"
    );
    assert!(said(&w, "The coffer bursts apart!"));
    assert_eq!(lying(&w, SPEAR, "gold_coin"), 12, "a smash spares coins");
    assert!(!trapped(&w, SPEAR));
}

#[test]
fn a_kick_at_a_fireball_chest_blows_it_apart() {
    let mut w = world("tstrong");
    w.player_mut().talents.take("lockbreaker", None);
    rubies(&mut w, FIRE, 200);
    below(&mut w, 3);
    w.apply(&Action::Kick(Dir::N));
    assert_eq!(terrain(&w, FIRE), "chest_smashed");
    assert!(said(&w, "The coffer is blown apart!"));
    assert!(!said(&w, "The coffer bursts apart!"));
    assert!(lying(&w, FIRE, "gem") < 200, "gems are not spared");
}

#[test]
fn pick_lock_springs_the_trap() {
    let mut w = world("tchests");
    train(&mut w, "open_lock", 100);
    below(&mut w, 7);
    loop {
        let outcome = w.apply(&Action::PickLock(Dir::N));
        if terrain(&w, SPEAR) != "chest" {
            assert!(sprung_at_player(&w, &outcome.events, SPEAR));
            break;
        }
    }
    assert!(said(&w, "You pick the lock and open the coffer."));
    assert_eq!(terrain(&w, SPEAR), "chest_open");
}

/// Sets the gameplay stream to the first whose next check at disarming the trap at `at` comes
/// out as `want` says.
fn rig(w: &mut World, at: Pos, want: impl Fn(Checked) -> bool) {
    let kind = w.state.current().traps[&at].kind;
    let difficulty = w.difficulty_of(Hidden::Trap(kind));
    let modifier = w.disarm_modifier();
    let seed = (0..10_000)
        .find(|&s| want(check::roll(&mut Rng::gameplay(s), modifier, difficulty)))
        .expect("a stream that comes out so");
    w.state.rng = Rng::gameplay(seed);
}

#[test]
fn disarming_a_chest_leaves_it_shut_and_failing_badly_at_a_burst_blows_it_apart() {
    let mut w = world("tchests");
    train(&mut w, "disable_device", 20);
    below(&mut w, 9);
    rig(&mut w, ALARM, |c| c == Checked::Made);
    run(&mut w, "disarm:N");
    assert!(said(&w, "You disarm the alarm trap."));
    assert!(!trapped(&w, ALARM));
    assert_eq!(terrain(&w, ALARM), "chest", "still shut");

    let mut w = world("tclumsy");
    train(&mut w, "disable_device", 1);
    w.state
        .current_mut()
        .traps
        .get_mut(&FIRE)
        .expect("trap")
        .known = true;
    below(&mut w, 3);
    rig(
        &mut w,
        FIRE,
        |c| matches!(c, Checked::Failed { by } if by >= 5),
    );
    run(&mut w, "disarm:N");
    assert!(said(&w, "You set off the fireball trap!"));
    assert_eq!(terrain(&w, FIRE), "chest_smashed");
    assert!(said(&w, "The coffer is blown apart!"));
}

#[test]
fn searching_finds_a_chests_trap_by_the_chests_sign() {
    let mut w = world("tsharp");
    train(&mut w, "search", 40);
    run(&mut w, "search:100");
    assert!(
        said(&w, "You notice that the coffer is trapped: an arrow trap."),
        "{:?}",
        w.messages().iter().map(|m| &m.text).collect::<Vec<_>>()
    );
    assert!(w.state.current().traps[&ARROW].known);
    // The chest keeps its glyph in the view; the trap is in the trap list.
    let view = dump::ascii(&w, &[Layer::View, Layer::Traps]);
    assert!(view.contains("\n#⊟.⊟.⊟.⊟.⊟..#\n"), "{view}");
    assert!(view.contains("(1, 1) arrow trap"), "{view}");
}

#[test]
fn chest_wise_sees_a_chests_trap() {
    let mut w = world("tchests");
    w.apply(&Action::Wait);
    assert!(
        !w.state.current().traps[&ARROW].known,
        "unseen without the talent"
    );
    w.player_mut().talents.take("chest_wise", None);
    w.apply(&Action::Wait);
    let level = w.state.current();
    assert!(
        [ARROW, FIRE, KIST, SPEAR]
            .iter()
            .all(|p| level.traps[p].known)
    );
    assert!(said(
        &w,
        "You notice that the coffer is trapped: an arrow trap."
    ));
    assert!(said(
        &w,
        "You notice that the kist is trapped: a frost trap."
    ));
    assert_eq!(w.perception().trap(FIRE), w.content().trap_id("fireball"));
}

#[test]
fn trap_sense_halves_a_chests_blow() {
    let lost = |sense: bool| {
        let mut w = world("tchests");
        if sense {
            w.player_mut().talents.take("trap_sense", None);
        }
        below(&mut w, 5);
        let hp = w.state.player().hp;
        w.apply(&Action::Open(Dir::N));
        hp - w.state.player().hp
    };
    let (plain, sensed) = (lost(false), lost(true));
    assert!(plain > 0, "the frost hurts");
    assert_eq!(sensed, (plain / 2).max(1), "{plain} halved");
}

#[test]
fn a_blast_from_afar_springs_nothing_and_drops_the_trap() {
    let mut w = world("tchests");
    let mut events = Vec::new();
    w.blast(Pos::new(2, 3), 5, 1000, &mut events);
    assert_eq!(terrain(&w, ARROW), "chest_smashed");
    assert!(!trapped(&w, ARROW));
    assert!(!(events.iter()).any(|e| matches!(e, Event::TrapSprung { .. })));
    assert_eq!(terrain(&w, KIST), "chest", "a kist outlasts it");
    assert!(trapped(&w, KIST));
}

#[test]
fn the_truth_dump_lists_a_chests_trap() {
    let w = world("tchests");
    let truth = dump::ascii(&w, &[Layer::Truth]);
    assert!(
        truth.contains("chest 2 (coffer) at (1, 1): shut, trapped (arrow), holds 1 stack"),
        "{truth}"
    );
    assert!(
        truth.contains("chest 2 (coffer) at (7, 1): shut, locked, trapped (spear), holds 2 stacks")
    );
}

#[test]
fn trapped_chests_replay_exactly() {
    let content = content();
    for (scenario, actions) in [
        ("tchests", "move:N move:E*2 open:N move:E*2 open:N"),
        ("tweak", "move:E*6 kick:N*8"),
    ] {
        let mut w = World::new(content.clone(), scenario, None).expect("world");
        let mut replay = Replay::new(&w);
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        let (verified, divergence) = replay.play(content.clone()).expect("replays");
        assert_eq!(divergence, None, "{scenario}");
        assert_eq!(verified.state_hash(), w.state_hash());
    }
}
