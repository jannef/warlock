//! Cēaptūn's quests (#340; docs/design/campaign.md §6.2): the three chains of Reeve Leofric,
//! Sister Hild, and Factor Wulfrun, each offered one at a time; the two reveals (Fennmynster as
//! the fen minster is accepted, Fyrdelf as the lost shaft is, which waits for both the ledger
//! and the barrow's heart); the ledger carried to Godgifu; the chalice handed in; the branches'
//! arrival lines; and a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::dump::{self, Layer};
use crate::object::Stack;
use crate::replay::Replay;
use std::sync::LazyLock;

const HILD: Pos = Pos::new(27, 25);
const GODGIFU: Pos = Pos::new(46, 26);
const LEOFRIC: Pos = Pos::new(60, 26);
const WULFRUN: Pos = Pos::new(72, 26);

/// Built-in content with nobody hungering.
static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.remove("data/hunger.toml");
    Arc::new(Content::load(&files).expect("builtin content"))
});

fn town() -> World {
    World::new(CONTENT.clone(), "ceaptun", None).expect("a run in Cēaptūn")
}

fn act(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.text.clone()).collect()
}

/// Puts the player west of `npc`, so they talk to the east.
fn beside(world: &mut World, npc: Pos) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = npc.step(Dir::W);
    world.perceive(&mut Vec::new());
}

fn topics(world: &World, npc: Pos) -> Vec<String> {
    let id = (world.state.actor_at("ceaptun", npc))
        .expect("someone there")
        .id;
    (world.topics(id).into_iter()).map(|t| t.id).collect()
}

fn carried(world: &World, object: &str) -> u32 {
    let object = world.content().object_id(object).expect("an object");
    super::shop::count_of(&world.state().player().inventory, object)
}

/// Marks `done` quests done, as if handed in.
fn done(world: &mut World, done: &[&str]) {
    let records: Vec<String> = (done.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    world.state.quests = serde_json::from_str(&json).expect("quests");
}

#[test]
fn each_giver_offers_the_first_quest_of_their_chain() {
    let w = town();
    assert_eq!(
        topics(&w, HILD),
        ["quest-grave_goods", "mynster", "fen", "cure", "mend"]
    );
    assert_eq!(topics(&w, LEOFRIC), ["quest-barrow_rats", "town", "barrow"]);
    assert_eq!(
        topics(&w, WULFRUN),
        ["quest-foremans_ledger", "diggings", "trade"]
    );
    // Godgifu gives none: she takes the ledger in.
    assert_eq!(
        topics(&w, GODGIFU),
        ["barrows", "marsh", "mines", "volcano", "pass"]
    );
    // Each chain goes on once its quest before is done.
    let mut w = town();
    done(&mut w, &["grave_goods", "barrow_rats"]);
    assert_eq!(topics(&w, HILD)[0], "quest-fen_minster");
    assert_eq!(topics(&w, LEOFRIC)[0], "quest-barrow_heart");
    done(
        &mut w,
        &["grave_goods", "fen_minster", "barrow_rats", "barrow_heart"],
    );
    assert_eq!(topics(&w, HILD)[0], "quest-water_keeps");
    assert_eq!(topics(&w, LEOFRIC)[0], "quest-wight");
}

#[test]
fn the_ledger_is_carried_to_godgifu() {
    let mut w = town();
    beside(&mut w, WULFRUN);
    act(
        &mut w,
        "ask:E:quest-foremans_ledger ask:E:accept-foremans_ledger",
    );
    assert_eq!(carried(&w, "ledger"), 1, "handed over as it is accepted");
    let log = &w.perception().quests()[0];
    assert!(
        log.objective
            .starts_with("Deliver a foreman's ledger to Godgifu. The Gæsthūs"),
        "{}",
        log.objective
    );
    assert!(topics(&w, GODGIFU).contains(&"quest-foremans_ledger".to_string()));
    beside(&mut w, GODGIFU);
    act(&mut w, "ask:E:quest-foremans_ledger");
    assert_eq!(carried(&w, "ledger"), 0);
    assert_eq!(carried(&w, "gold_coin"), 10);
    assert!(
        said(&w)
            .iter()
            .any(|l| l.starts_with("Godgifu says: \"The foreman's ledger"))
    );
}

#[test]
fn the_lost_shaft_waits_for_the_ledger_and_the_barrows_heart_and_reveals_fyrdelf() {
    let mut w = town();
    done(&mut w, &["foremans_ledger"]);
    assert_eq!(
        topics(&w, WULFRUN)[0],
        "diggings",
        "not before the barrow's heart"
    );
    done(&mut w, &["barrow_rats", "barrow_heart"]);
    assert_eq!(
        topics(&w, WULFRUN)[0],
        "quest-foremans_ledger",
        "not before the ledger"
    );
    done(&mut w, &["foremans_ledger", "barrow_rats", "barrow_heart"]);
    assert_eq!(topics(&w, WULFRUN)[0], "quest-lost_shaft");

    beside(&mut w, WULFRUN);
    act(&mut w, "ask:E:quest-lost_shaft");
    assert!(
        w.state.revealed.is_empty(),
        "revealed as it is accepted, not offered"
    );
    act(&mut w, "ask:E:accept-lost_shaft");
    assert!(w.state.revealed.contains("fyrdelf"));
    assert_eq!(
        said(&w).last().unwrap(),
        "Factor Wulfrun marks Fyrdelf on your map."
    );
    let log = &w.perception().quests()[0];
    assert_eq!(
        log.objective,
        "Reach Fyrdelf 2. The shaft lies in Ascdūn, in the hills under Fyrbeorg, north of the ford."
    );
    // Out on the overworld, made after the reveal, the way in is marked and known.
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = Pos::new(45, 17);
    act(&mut w, "ascend"); // the quick leave: nobody in town is hostile
    assert_eq!(w.perception().level_id(), "ascmearc");
    let known = dump::layer_rows(&w, Layer::Locations).join("\n");
    assert!(known.contains("(97, 21) Fyrdelf"), "{known}");
    assert!(!known.contains("Fennmynster"), "{known}");
}

#[test]
fn the_fen_minster_is_marked_as_sister_hild_sends_the_player() {
    let mut w = town();
    beside(&mut w, HILD);
    // The chalice first: brought from the barrow and handed in.
    act(&mut w, "ask:E:quest-grave_goods ask:E:accept-grave_goods");
    let chalice = CONTENT.object_id("silver_chalice").expect("the chalice");
    let id = w.state.player;
    (w.state.actors.get_mut(&id).expect("player").inventory).push(Stack::new(chalice, 1));
    act(&mut w, "ask:E:quest-grave_goods");
    assert_eq!(carried(&w, "silver_chalice"), 0);
    assert_eq!(carried(&w, "gold_coin"), 40);
    assert!(w.state.revealed.is_empty());

    act(&mut w, "ask:E:quest-fen_minster ask:E:accept-fen_minster");
    assert!(w.state.revealed.contains("fennmynster"));
    let lines = said(&w);
    let n = lines.len();
    assert!(lines[n - 2].starts_with("Sister Hild says: \"Here, let me show you"));
    assert_eq!(lines[n - 1], "Sister Hild marks Fennmynster on your map.");
    let active = (w.perception().quests().into_iter())
        .find(|q| q.name == "The fen minster")
        .expect("in the log");
    assert_eq!(
        active.objective,
        "Reach Fennmynster 3. The minster lies in Micelfenn, east of Fennhām."
    );
    // Asking again reminds, and marks nothing twice.
    act(&mut w, "ask:E:quest-fen_minster");
    assert!(!said(&w).last().unwrap().contains("marks"));
}

#[test]
fn kills_in_cealdhlaw_count_for_the_reeve() {
    let mut w = town();
    w.state.quests = serde_json::from_str(
        r#"{"kills": {"cealdhlaw:1": {"rat": 3}, "cealdhlaw:3": {"rat": 2}}}"#,
    )
    .expect("quests");
    beside(&mut w, LEOFRIC);
    act(
        &mut w,
        "ask:E:quest-barrow_rats ask:E:accept-barrow_rats ask:E:quest-barrow_rats",
    );
    assert_eq!(
        carried(&w, "gold_coin"),
        25,
        "five rats on any of its levels"
    );
    assert_eq!(topics(&w, LEOFRIC)[0], "quest-barrow_heart");
}

#[test]
fn the_branches_say_their_arrival_lines_on_their_first_level() {
    for (level, line) in [
        ("cealdhlaw:1", "Cold air breathes up from the mound."),
        ("fennmynster:1", "The water is up to the third step."),
        ("fyrdelf:1", "The galleries smell of sulphur."),
    ] {
        assert_eq!(CONTENT.arrival(level), Some(line));
        let mut w = town();
        w.travel(level, None).expect("travels");
        assert_eq!(said(&w).last().map(String::as_str), Some(line));
    }
    assert_eq!(CONTENT.arrival("cealdhlaw:2"), None, "the first level only");
    assert_eq!(
        CONTENT.arrival("ceaptun"),
        Some("Market day in Cēaptūn: the square is loud.")
    );
}

#[test]
fn taking_up_the_chalice_replays_exactly() {
    let mut w = town();
    let mut replay = Replay::new(&w);
    let actions = "move:SW*4 move:W*15 open:SW move:SW move:S*2 chat:SE \
                   ask:SE:quest-grave_goods ask:SE:accept-grave_goods";
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(w.perception().quests()[0].name, "Grave goods");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
