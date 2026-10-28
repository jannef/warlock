//! Chests (spec §8 "Chests", docs/design/chests-and-treasure.md §3, §9; `world/chest.rs`): a
//! coffer opens on a bump and on `open`; a locked one refuses without a key and opens with a
//! skeleton key, Pick lock, or Knock then Open; Knock is refused with no time where nothing is
//! locked; kicks smash it (half the stacks, never coins or gems; Deft breaker's quarter; a kick
//! that holds changes nothing); a blast blows a coffer apart, coins and gems too, and not a
//! kist; what a shut chest holds is not in the snapshot; Chest-wise sees locks; legends place
//! chests with their objects and levels; chests survive a save and replay exactly; the dumps.

use super::*;
use crate::action::parse_actions;
use crate::chest::{ChestLook, ChestState};
use crate::content::ContentFiles;
use crate::dump::{self, Layer};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};

/// A lit hall: the player at the west end of the walk (row 2), chests along row 1: a coffer
/// holding a dagger and coins (`C`, x 1), a locked coffer (`L`, x 3), a locked kist (`K`, x 5,
/// "kist": level 15), an easy lock (`E`, x 7, a hutch by level), and an empty coffer (`O`,
/// x 9).
const HALL: &str = r##"
name = "Chest hall"
ambient = [255, 255, 255, 0]
item_level = 2
map = '''
###########
#C.L.K.E.O#
#@........#
#.........#
###########
'''
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"C" = { terrain = "chest", objects = ["dagger", "gold_coin*12"] }
"L" = { terrain = "chest", locked = true, objects = ["bread"] }
"K" = { terrain = "chest", chest = "kist", locked = true, objects = ["gem:ruby"] }
"E" = { terrain = "chest", chest = 6, lock = "box", lock_difficulty = 1 }
"O" = "chest"
"##;

const COFFER: Pos = Pos::new(1, 1);
const LOCKED: Pos = Pos::new(3, 1);
const KIST: Pos = Pos::new(5, 1);
const EASY: Pos = Pos::new(7, 1);
const EMPTY: Pos = Pos::new(9, 1);

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("levels/chest_hall.toml", HALL);
    for (name, extra) in [
        ("chests", "hp = 1000\n"),
        ("strong", "hp = 1000\nstats = { strength = 18 }\n"),
        ("weak", "hp = 1000\nstats = { strength = 3 }\n"),
    ] {
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"chest_hall\"\n{extra}"),
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

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn terrain(w: &World, p: Pos) -> String {
    w.content().terrain(w.state.current().terrain[p]).id.clone()
}

fn give(w: &mut World, id: &str) {
    let object = w.content().object_id(id).expect("object");
    w.player_mut().inventory.push(Stack::one(object));
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

#[test]
fn legends_place_chests_of_their_level_with_what_they_hold() {
    let w = world("chests");
    let chests = &w.state.current().chests;
    assert_eq!(chests.len(), 5, "one on every chest cell");
    assert_eq!(chests[&COFFER].level, 2, "the level's item level");
    assert_eq!(chests[&KIST].level, 15, "a band's least level");
    assert_eq!(chests[&EASY].level, 6);
    assert_eq!(chests[&COFFER].holds.len(), 2);
    assert!(chests[&EMPTY].holds.is_empty());
    assert!(
        w.state.current().objects_at(COFFER).is_empty(),
        "held, not lying"
    );
    assert!(w.state.current().locks.contains_key(&LOCKED));
    assert!(w.state.current().locks.contains_key(&KIST));
}

#[test]
fn bad_chest_legends_are_refused() {
    for (legend, why) in [
        (
            r#"{ terrain = "floor", chest = "coffer" }"#,
            "no shut chest",
        ),
        (r#"{ terrain = "chest", chest = "casket" }"#, "no band"),
        (r#"{ terrain = "chest", chest = 0 }"#, "level 0"),
        (r#"{ terrain = "chest", trap = "arrow" }"#, "a trap (C2)"),
    ] {
        let mut files = files();
        let text = HALL.replace(
            r#"{ terrain = "chest", chest = 6, lock = "box", lock_difficulty = 1 }"#,
            legend,
        );
        files.insert("levels/chest_hall.toml", &text);
        assert!(Content::load(&files).is_err(), "{why}");
    }
    let mut files = files();
    files.insert(
        "vaults/chest_vault.toml",
        "name = \"V\"\nmap = '''\n###\n#C#\n#.#\n'''\n[legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\
         \"C\" = \"chest\"\n",
    );
    assert!(Content::load(&files).is_err(), "no chests in vaults yet");
}

#[test]
fn a_coffer_opens_on_a_bump_and_spills_what_it_held() {
    let mut w = world("chests");
    let start = w.state.time;
    w.apply(&Action::Move(Dir::N));
    assert_eq!(w.player_pos(), Pos::new(1, 2), "the chest is in the way");
    assert!(said(&w, "You open the coffer."));
    assert_eq!(
        last_message(&w),
        "Inside you find a dagger and 12 gold coins."
    );
    assert_eq!(terrain(&w, COFFER), "chest_open");
    assert!(w.state.time > start, "it took time");
    assert_eq!(lying(&w, COFFER, "dagger"), 1);
    assert_eq!(lying(&w, COFFER, "gold_coin"), 12);
    assert!(w.state.current().chests[&COFFER].holds.is_empty());
    // An open chest is walkable: step in and pick up.
    run(&mut w, "move:N pickup");
    assert_eq!(w.player_pos(), COFFER);
    assert!(w.state.current().objects_at(COFFER).is_empty());
    // Nothing closes it again.
    run(&mut w, "move:S close:N");
    assert_eq!(terrain(&w, COFFER), "chest_open");
}

#[test]
fn open_says_an_empty_chest_is_empty() {
    let mut w = world("chests");
    below(&mut w, 9);
    run(&mut w, "open:N");
    assert!(said(&w, "You open the coffer."));
    assert_eq!(last_message(&w), "It is empty.");
}

#[test]
fn a_locked_chest_refuses_without_a_key_and_opens_with_a_skeleton_key() {
    let mut w = world("chests");
    below(&mut w, 3);
    let turn = w.state.turn;
    run(&mut w, "open:N");
    assert_eq!(last_message(&w), "The coffer is locked.");
    assert_eq!(w.state.turn, turn, "no time");
    run(&mut w, "move:N");
    assert_eq!(last_message(&w), "The coffer is locked.");
    assert_eq!(w.state.turn, turn, "a bump is no time either");
    give(&mut w, "iron_key");
    run(&mut w, "open:N");
    assert_eq!(
        last_message(&w),
        "The coffer is locked.",
        "only a skeleton key fits"
    );
    give(&mut w, "skeleton_key");
    let start = w.state.time;
    run(&mut w, "open:N");
    assert!(said(
        &w,
        "You unlock the coffer with your skeleton key and open it."
    ));
    assert_eq!(terrain(&w, LOCKED), "chest_open");
    assert!(!w.state.current().locks.contains_key(&LOCKED));
    assert_eq!(lying(&w, LOCKED, "bread"), 1);
    // Unlocking and opening: two actions' time.
    assert!(w.state.time.since(start) >= crate::time::Duration::seconds(12));
}

#[test]
fn pick_lock_opens_a_chest() {
    let mut w = world("chests");
    let open_lock = w.content().skill_id("open_lock").expect("Open lock");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != open_lock).collect();
    all.push((
        open_lock,
        Trained {
            value: 60,
            marks: 0,
        },
    ));
    *skills = all.into_iter().collect();
    below(&mut w, 9);
    let turn = w.state.turn;
    run(&mut w, "pick:N");
    assert_eq!(last_message(&w), "The coffer is not locked.");
    assert_eq!(w.state.turn, turn);
    run(&mut w, "move:W*2 pick:N");
    assert!(said(&w, "You pick the lock and open the hutch."));
    assert_eq!(terrain(&w, EASY), "chest_open");
    assert!(!w.state.current().locks.contains_key(&EASY));
}

/// A wizard who has Knock, with mana to spare.
fn wizard() -> World {
    let mut w = world("chests");
    let me = w.player_mut();
    me.talents.take("knock", None);
    me.mana_base = 40;
    me.mana = 40;
    w
}

#[test]
fn knock_springs_a_lock_and_opens_nothing() {
    let mut w = wizard();
    below(&mut w, 3);
    let mana = w.state.player().mana;
    run(&mut w, "use:knock:N");
    assert_eq!(last_message(&w), "The lock springs open with a click.");
    assert_eq!(w.state.player().mana, mana - 5);
    assert_eq!(terrain(&w, LOCKED), "chest", "still shut");
    assert!(!w.state.current().locks.contains_key(&LOCKED));
    run(&mut w, "open:N");
    assert!(said(&w, "You open the coffer."));
    assert_eq!(lying(&w, LOCKED, "bread"), 1);
}

#[test]
fn knock_is_refused_where_nothing_is_locked() {
    let mut w = wizard();
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    // At the open coffer's cell: unlocked.
    run(&mut w, "use:knock:N");
    assert_eq!(last_message(&w), "There is no lock there to spring.");
    // At open floor.
    run(&mut w, "use:knock:E");
    assert_eq!(last_message(&w), "There is no lock there to spring.");
    assert_eq!(w.state.turn, turn, "no time");
    assert_eq!(w.state.player().mana, mana, "no mana");
    // Without the ability it is no ability at all.
    let mut plain = world("chests");
    run(&mut plain, "use:knock:N");
    assert_eq!(last_message(&plain), "You have no such ability.");
}

#[test]
fn the_wizard_learns_knock_at_level_four() {
    let content = content();
    let wizard = (content.classes().iter())
        .find(|c| c.id == "wizard")
        .expect("wizard");
    assert!(
        (wizard.abilities.iter()).any(|(level, talent)| *level == 4 && talent == "knock"),
        "Knock at level 4"
    );
}

#[test]
fn a_strong_kick_smashes_a_coffer_sparing_coins() {
    let mut w = world("strong");
    w.player_mut().talents.take("lockbreaker", None);
    // Strength 18 + Lockbreaker's 6 against a coffer's 20: it always bursts.
    w.apply(&Action::Kick(Dir::N));
    assert!(said(&w, "The coffer bursts apart!"));
    assert_eq!(terrain(&w, COFFER), "chest_smashed");
    assert_eq!(lying(&w, COFFER, "gold_coin"), 12, "coins are spared");
    assert!(w.state.current().chests[&COFFER].holds.is_empty());
    assert!(!w.state.current().locks.contains_key(&COFFER));
}

#[test]
fn a_kick_that_holds_changes_nothing_and_kicks_may_be_tried_again() {
    let mut w = world("weak");
    below(&mut w, 5);
    let mut held = 0;
    while terrain(&w, KIST) == "chest" {
        let turn = w.state.turn;
        w.apply(&Action::Kick(Dir::N));
        assert!(w.state.turn > turn, "every kick takes time");
        if terrain(&w, KIST) == "chest" {
            held += 1;
            assert_eq!(last_message(&w), "You kick the kist. It holds.");
            assert_eq!(w.state.current().chests[&KIST].holds.len(), 1);
            assert!(w.state.current().locks.contains_key(&KIST));
        }
        assert!(held < 1000, "Strength 3 in 45 breaks it in the end");
    }
    assert!(held > 0, "Strength 3 against a kist's 45 holds at first");
    assert!(said(&w, "The kist bursts apart!"));
    assert_eq!(lying(&w, KIST, "gem"), 1, "gems are spared");
}

/// Fills the chest at `p` with `n` daggers, each its own stack, and `n` loose rubies.
fn fill(w: &mut World, p: Pos, n: usize) {
    let dagger = w.content().object_id("dagger").expect("dagger");
    // The kist's ruby, as its legend placed it.
    let ruby = w.state.current().chests[&KIST].holds[0].clone();
    let chest = w.state.current_mut().chests.get_mut(&p).expect("chest");
    chest.holds = (0..n).map(|_| Stack::one(dagger)).collect();
    for _ in 0..n {
        chest.holds.push(ruby.clone());
    }
}

#[test]
fn a_smash_destroys_about_half_and_deft_breaker_a_quarter() {
    for (deft, range) in [(false, 70..=130), (true, 25..=75)] {
        let mut w = world("strong");
        w.player_mut().talents.take("lockbreaker", None);
        if deft {
            w.player_mut().talents.take("deft_breaker", None);
        }
        fill(&mut w, COFFER, 200);
        w.apply(&Action::Kick(Dir::N));
        assert_eq!(terrain(&w, COFFER), "chest_smashed");
        let kept = lying(&w, COFFER, "dagger");
        assert!(
            range.contains(&(200 - kept)),
            "deft {deft}: {} broke",
            200 - kept
        );
        assert_eq!(
            lying(&w, COFFER, "gem"),
            200,
            "gems are never rolled on a kick"
        );
        assert!((w.messages().iter()).any(|m| m.text.ends_with("are destroyed.")));
    }
}

#[test]
fn a_blast_blows_a_coffer_apart_sparing_nothing_and_not_a_kist() {
    let mut w = world("chests");
    fill(&mut w, COFFER, 100);
    let mut events = Vec::new();
    w.blast(Pos::new(2, 2), 5, 1000, &mut events);
    assert_eq!(terrain(&w, COFFER), "chest_smashed");
    let gems = lying(&w, COFFER, "gem");
    assert!((25..=75).contains(&gems), "{gems} rubies of 100 survive");
    assert!(said(&w, "The coffer is blown apart!"));
    assert_eq!(
        terrain(&w, KIST),
        "chest",
        "a kist's 1200 outlasts a fireball"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ChestBroken { pos } if *pos == COFFER))
    );
}

#[test]
fn a_chest_is_no_wall_to_dig() {
    let mut w = world("chests");
    give(&mut w, "pick_axe");
    let axe = w.state.player().inventory.len() - 1;
    run(&mut w, &format!("equip:tool:{axe}"));
    let turn = w.state.turn;
    run(&mut w, "dig:N");
    assert_eq!(last_message(&w), "You cannot dig through the chest.");
    assert_eq!(terrain(&w, COFFER), "chest");
    assert_eq!(w.state.turn, turn, "no time");
}

#[test]
fn the_snapshot_sees_the_band_not_the_contents() {
    let mut w = world("chests");
    let view = w.perception();
    assert_eq!(
        view.chest_at(COFFER),
        Some(ChestLook {
            band: 0,
            state: ChestState::Shut,
            locked: None
        })
    );
    let kist = view.chest_at(KIST).expect("seen");
    assert_eq!(w.content().chests().bands[kist.band].id, "kist");
    assert_eq!(kist.locked, None, "a lock is not seen without Chest-wise");
    assert!(view.objects(COFFER).is_empty(), "what it holds is not seen");
    run(&mut w, "open:N");
    let view = w.perception();
    assert_eq!(
        view.chest_at(COFFER).map(|c| c.state),
        Some(ChestState::Open)
    );
    assert_eq!(view.objects(COFFER).len(), 2, "open, it is a pile");
}

#[test]
fn chest_wise_tells_locked_from_unlocked() {
    let mut w = world("chests");
    w.player_mut().talents.take("chest_wise", None);
    w.apply(&Action::Wait);
    let view = w.perception();
    let rules = w.content().chests();
    let name = |p| view.chest_at(p).expect("seen").name(rules, "chest");
    assert_eq!(name(LOCKED), "locked coffer");
    assert_eq!(name(COFFER), "unlocked coffer");
    assert_eq!(name(KIST), "locked kist");
}

#[test]
fn chests_survive_a_save_and_replay_exactly() {
    let mut w = world("strong");
    give(&mut w, "skeleton_key");
    run(&mut w, "kick:N move:E*2 open:N");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let content = content();
    for (scenario, actions) in [
        ("weak", "kick:N*6 move:E*4 kick:N*3"),
        ("chests", "move:N move:E*8 open:N"),
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

#[test]
fn the_dumps_show_chests() {
    let mut w = world("chests");
    let truth = dump::ascii(&w, &[Layer::Truth]);
    assert!(
        truth.contains("chest 2 (coffer) at (1, 1): shut, holds 2 stacks"),
        "{truth}"
    );
    assert!(truth.contains("chest 15 (kist) at (5, 1): shut, locked, holds 1 stack"));
    run(&mut w, "open:N");
    let truth = dump::ascii(&w, &[Layer::Truth]);
    assert!(truth.contains("chest 2 (coffer) at (1, 1): open"));
    let json = dump::json(&w, &[], false);
    let chests = json["chests"].as_array().expect("chests");
    assert_eq!(chests.len(), 5);
    assert_eq!(chests[0]["band"], "coffer");
    assert_eq!(chests[0]["state"], "open");
    assert!(
        !json.to_string().contains("holds"),
        "no contents in the JSON"
    );
}
