//! Escorts (`escort.rs`; spec §8 "Quests": "Escort quests"): a fixture wise woman in a hamlet
//! gives an escort quest for Eadgyth, lost in the cave below; chatted with, Eadgyth follows in
//! the coward stance, across the stairs, wields what she is given and drops what she held,
//! refuses what fits nothing, and home with the player completes the quest; home alone does
//! not; her death fails it; a save and a replay.

use crate::ability::{Ability, Unavailable};
use crate::action::{Command, parse_actions};
use crate::content::{CombatStance, ContentFiles, QuestKind};
use crate::replay::Replay;
use crate::test_support::{run, texts};
use crate::world::combat::fight::Harm;
use crate::world::*;

/// The wise woman and Eadgyth, added to the built-in dialogue.
const DIALOGUE: &str = r##"
[[dialogue]]
npc = "wise_woman"
name = "Wynn"
lines = ["Mind the cave."]

[[dialogue]]
npc = "eadgyth"
name = "Eadgyth"
greet = "Who's there?"
lines = ["Is it much farther?"]
"##;

/// The escort quest, added to the built-in quests.
const QUEST: &str = r##"
[[quest]]
id = "eadgyth_home"
name = "Eadgyth"
giver = "wise_woman"
type = "escort"
target = "hamlet"
escort = "eadgyth"
joins = "Oh, thank you! I'll keep right behind you."
reward = { gold = 30, xp = 40 }
offer = "Eadgyth went into the cave and has not come back. Bring her home."
reminder = "Is she with you?"
thanks = "Home! I never thought I'd see it again."
done = "You found her! Bless you."
"##;

// The hamlet: the wise woman south of the player, stairs down at the east end.
const HAMLET: &str = r##"
name = "Hamlet"
ambient = [255, 255, 255, 0]
map = '''
#######
#@....#
#w...>#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"w" = { terrain = "floor", creature = "villager", npc = "wise_woman" }
">" = { terrain = "stairs_down", to = "cave" }
"##;

// The cave: stairs up at the west end, Eadgyth with a club at the east end.
const CAVE: &str = r##"
name = "Cave"
ambient = [255, 255, 255, 0]
map = '''
##########
#<......e#
#........#
##########
'''
[legend]
"#" = "wall"
"." = "floor"
"<" = { terrain = "stairs_up", to = "hamlet" }
"e" = { terrain = "floor", creature = "villager", npc = "eadgyth", carry = ["club"], wield = "club" }
"##;

fn files(quest: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for (table, extra) in [("dialogue", DIALOGUE), ("quests", quest)] {
        let (path, text) = files.table(table).next().expect(table);
        let (path, text) = (path.to_string(), format!("{text}{extra}"));
        files.insert(&path, &text);
    }
    files.insert("levels/hamlet.toml", HAMLET);
    files.insert("levels/cave.toml", CAVE);
    files.insert(
        "scenarios/hamlet.toml",
        "name = \"hamlet\"\nstart_level = \"hamlet\"\nhp = 200\n\
         carry = [\"long_sword\", \"great_sword\", \"apple\"]\n",
    );
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files(QUEST)).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "hamlet", None).expect("world")
}

fn heard(w: &World, line: &str) -> bool {
    texts(w).iter().any(|m| m == line)
}

fn eadgyth(w: &World) -> &Actor {
    (w.state.actors.values())
        .find(|a| a.npc.as_deref() == Some("eadgyth"))
        .expect("Eadgyth")
}

fn stage(w: &World) -> Option<Stage> {
    w.state.quests.record("eadgyth_home").map(|r| r.stage)
}

fn object(w: &World, id: &str) -> crate::content::ObjectId {
    w.content.object_id(id).expect(id)
}

/// What Eadgyth holds in her main hand.
fn wields(w: &World) -> Option<String> {
    let e = eadgyth(w);
    let hand = w.content.slot_id("main_hand").expect("main hand");
    let i = e.equipment.get(hand)?;
    Some(w.content.object(e.inventory[i].object).id.clone())
}

/// Given the quest, down in the cave beside Eadgyth (west of her), and spoken with.
const FOUND: &str = "ask:S:quest-eadgyth_home move:E*3 move:SE descend move:E*6 chat:E";

#[test]
fn chatted_with_while_the_quest_is_active_she_follows_in_the_coward_stance() {
    let mut w = world();
    // Before the quest she is only someone in a cave.
    run(&mut w, "move:E*3 move:SE descend move:E*6 chat:E");
    assert!(eadgyth(&w).ally.is_none());
    assert!(
        heard(&w, "Eadgyth says: \"Who's there?\""),
        "{:?}",
        texts(&w)
    );
    let mut w = world();
    run(&mut w, FOUND);
    let e = eadgyth(&w);
    let ally = e.ally.as_ref().expect("an ally");
    assert_eq!(ally.master, w.player_id());
    assert_eq!(ally.order, Order::Follow);
    assert_eq!(e.escorting(), Some("eadgyth_home"));
    assert_eq!(e.stance, CombatStance::Coward);
    assert_eq!(e.faction(&w.content), w.state.player().faction(&w.content));
    assert!(
        heard(
            &w,
            "Eadgyth says: \"Oh, thank you! I'll keep right behind you.\""
        ),
        "{:?}",
        texts(&w)
    );
    // The quest log says so.
    let log = w.perception().quests();
    assert_eq!(log[0].kind, QuestKind::Escort);
    assert_eq!(
        log[0].objective,
        "Wynn wants you to bring Eadgyth safely to Hamlet."
    );
    assert_eq!(log[0].status, "Eadgyth follows you.");
    // Chatting again is a chat.
    run(&mut w, "chat:E");
    assert!(heard(&w, "Eadgyth says: \"Is it much farther?\""));
}

#[test]
fn home_together_the_quest_completes_and_she_stays() {
    let mut w = world();
    run(&mut w, FOUND);
    assert_eq!(stage(&w), Some(Stage::Active));
    let gold = object(&w, "gold_coin");
    run(&mut w, "move:W*6 ascend");
    assert_eq!(w.state.current_level, "hamlet");
    let e = eadgyth(&w);
    assert_eq!(e.level, "hamlet", "she came up the stairs");
    assert!(e.ally.is_none(), "no longer following");
    assert_eq!(e.stance, CombatStance::Neutral);
    assert_eq!(e.faction_override, None, "her own faction back");
    assert_eq!(stage(&w), Some(Stage::Done));
    assert!(heard(
        &w,
        "Eadgyth says: \"Home! I never thought I'd see it again.\""
    ));
    // The giver is here: she thanks the player and pays.
    assert!(heard(&w, "Wynn says: \"You found her! Bless you.\""));
    assert!(heard(&w, "Wynn gives you 30 gold coins."));
    assert!(heard(&w, "Quest complete: Eadgyth."));
    assert_eq!(super::shop::count_of(&w.state.player().inventory, gold), 30);
    // She stays: the player goes down, and she does not come.
    let at = eadgyth(&w).pos;
    run(&mut w, "descend");
    assert_eq!(eadgyth(&w).level, "hamlet");
    assert_eq!(eadgyth(&w).pos, at);
}

#[test]
fn home_alone_does_not_complete_it() {
    let mut w = world();
    run(&mut w, &format!("{FOUND} order:stay move:W*6 ascend"));
    assert_eq!(w.state.current_level, "hamlet");
    assert_eq!(eadgyth(&w).level, "cave", "left behind, holding");
    assert_eq!(stage(&w), Some(Stage::Active));
    assert_eq!(
        w.perception().quests()[0].status,
        "Eadgyth is not with you."
    );
    // Back for her: she comes, and it is done.
    run(&mut w, "descend move:E*6 order:follow move:W*6 ascend");
    assert_eq!(stage(&w), Some(Stage::Done));
}

#[test]
fn she_is_set_on_nothing() {
    let mut w = world();
    run(&mut w, FOUND);
    let at = w.player_pos().step(Dir::W);
    let out = w.apply(&Action::Order(Command::SetOn(at)));
    assert!(!out.took_turn);
    assert!(heard(&w, "Eadgyth will not go looking for a fight."));
    assert_eq!(eadgyth(&w).ally.as_ref().unwrap().order, Order::Follow);
}

#[test]
fn given_a_sword_she_wields_it_and_drops_her_club() {
    let mut w = world();
    run(&mut w, FOUND);
    assert_eq!(wields(&w).as_deref(), Some("club"));
    let view = w.perception();
    let give = view
        .abilities()
        .into_iter()
        .find(|a| a.ability == Ability::Give);
    assert_eq!(give.expect("Give is listed").unavailable, None);
    let at = eadgyth(&w).pos;
    let start = w.time();
    // The long sword is the pack's first stack.
    let out = w.apply(&Action::Give {
        item: 0,
        ally: None,
    });
    assert!(out.took_turn);
    assert!(w.time() > start);
    assert_eq!(wields(&w).as_deref(), Some("long_sword"));
    assert!(
        heard(&w, "You give the long sword to Eadgyth."),
        "{:?}",
        texts(&w)
    );
    assert!(heard(&w, "Eadgyth wields the long sword."));
    assert!(heard(&w, "Eadgyth drops the club."));
    let club = object(&w, "club");
    assert!((w.state.current().objects_at(at).iter()).any(|s| s.object == club));
    assert!(!(eadgyth(&w).inventory.iter()).any(|s| s.object == club));
    // A two-handed sword takes both hands, and the long sword falls.
    let sword = object(&w, "long_sword");
    w.apply(&Action::Give {
        item: 0,
        ally: Some(Dir::E),
    });
    assert_eq!(wields(&w).as_deref(), Some("great_sword"));
    assert!((w.state.current().objects_at(at).iter()).any(|s| s.object == sword));
    assert!(w.state.player().inventory.iter().all(|s| s.object != sword));
}

#[test]
fn what_fits_nothing_is_refused_with_no_time() {
    let mut w = world();
    run(&mut w, FOUND);
    let apple = (w.state.player().inventory.iter())
        .position(|s| s.object == object(&w, "apple"))
        .expect("an apple");
    let start = w.time();
    let out = w.apply(&Action::Give {
        item: apple,
        ally: None,
    });
    assert!(!out.took_turn);
    assert_eq!(w.time(), start);
    assert!(heard(&w, "Eadgyth has no use for that."));
    assert!(fits_escort(&w.content, &w.state.player().inventory[0]));
    assert!(!fits_escort(&w.content, &w.state.player().inventory[apple]));
}

#[test]
fn give_before_she_follows_is_not_listed() {
    let mut w = world();
    run(&mut w, "move:E*3 move:SE descend move:E*6");
    let view = w.perception();
    assert!(!view.abilities().iter().any(|a| a.ability == Ability::Give));
    let mut w = world();
    run(&mut w, &format!("{FOUND} order:stay move:W*3"));
    let view = w.perception();
    let give = view
        .abilities()
        .into_iter()
        .find(|a| a.ability == Ability::Give);
    assert_eq!(give.unwrap().unavailable, Some(Unavailable::NotBeside));
}

#[test]
fn her_death_fails_the_quest_and_locks_nothing_else() {
    let mut w = world();
    run(&mut w, FOUND);
    let id = eadgyth(&w).id;
    let player = w.player_id();
    assert!(w.hurt(id, 10_000, Harm::Actor(player), &mut Vec::new()));
    assert_eq!(stage(&w), Some(Stage::Failed));
    let log = w.perception().quests();
    assert!(!log[0].active);
    assert!(log[0].failed);
    assert_eq!(log[0].status, "Failed: Eadgyth is dead.");
    // Her death before she follows fails it too.
    let mut w = world();
    run(&mut w, "ask:S:quest-eadgyth_home move:E*3 move:SE descend");
    let id = eadgyth(&w).id;
    assert!(w.hurt(id, 10_000, Harm::Actor(player), &mut Vec::new()));
    assert_eq!(stage(&w), Some(Stage::Failed));
}

#[test]
fn the_load_checks() {
    let bad = |from: &str, to: &str| {
        let quest = QUEST.replace(from, to);
        Content::load(&files(&quest)).expect_err(to).to_string()
    };
    assert!(bad("escort = \"eadgyth\"\n", "").contains("needs `escort`"));
    assert!(bad("joins = ", "accept = ").contains("needs `joins`"));
    assert!(bad("target = \"hamlet\"", "target = \"nowhere\"").contains("unknown level"));
    assert!(bad("escort = \"eadgyth\"", "escort = \"villager_x\"").contains("no one who talks"));
    assert!(bad("type = \"escort\"", "type = \"reach\"").contains("no `escort`"));
}

#[test]
fn a_save_and_a_replay() {
    let mut w = world();
    let mut replay = Replay::new(&w);
    let actions = parse_actions(&format!("{FOUND} move:W*6 ascend")).expect("actions");
    for (i, action) in actions.iter().enumerate() {
        w.apply(action);
        replay.push(action, &w);
        if i == 13 {
            // Following, on the way: saved and loaded as she is.
            let loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("loads");
            assert_eq!(loaded.state_hash(), w.state_hash());
            assert_eq!(eadgyth(&loaded).escorting(), Some("eadgyth_home"));
        }
    }
    assert_eq!(stage(&w), Some(Stage::Done));
    let played = replay.run(content()).expect("replay verifies");
    assert_eq!(played.state(), w.state());
}
