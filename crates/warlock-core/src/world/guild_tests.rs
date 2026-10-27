//! The Weyfarers' Gilde (#425; the guild design §4, §7, §10): Wat One-Eye in the Gildhalle and
//! his topics, the chain's first three links given one at a time with no gate but the chain's
//! own, the ranks derived from the quests (Tin as link 1 is given, Pewter as link 2 is done, each
//! said once), the scroll handed in (an unpaid one stays on the bill), the `stats` dump, a
//! replay, and the guilds' load checks.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::dump::{self, Layer};
use crate::replay::Replay;
use std::sync::LazyLock;

/// Wat One-Eye's post, by the Gildhalle's far wall.
const WAT: Pos = Pos::new(81, 27);
/// From the square to Wat, through the Gildhalle's door: the player ends north of him.
const TO_WAT: &str = "move:E*26 move:S*4 move:E*9 open:S move:S*5";

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

/// Puts the player north of Wat, so they talk to the south.
fn beside_wat(world: &mut World) {
    let id = world.state.player;
    world.state.actors.get_mut(&id).expect("player").pos = WAT.step(Dir::N);
    world.perceive(&mut Vec::new());
}

fn topics(world: &World) -> Vec<String> {
    let id = (world.state.actor_at("ceaptun", WAT))
        .expect("Wat at his post")
        .id;
    (world.topics(id).into_iter()).map(|t| t.id).collect()
}

/// Marks `done` quests done, as if handed in.
fn done(world: &mut World, done: &[&str]) {
    let records: Vec<String> = (done.iter())
        .map(|q| format!(r#""{q}": {{"stage": "done", "completed": 1}}"#))
        .collect();
    let json = format!(r#"{{"records": {{{}}}}}"#, records.join(", "));
    world.state.quests = serde_json::from_str(&json).expect("quests");
}

fn rank(world: &World) -> Option<String> {
    let view = world.perception();
    let guilds = view.own_guilds();
    assert!(guilds.len() <= 1, "one guild");
    guilds.first().map(|g| g.title().to_string())
}

fn promotions(world: &World) -> Vec<String> {
    (said(world).into_iter())
        .filter(|l| l.contains("names you"))
        .collect()
}

fn carried(world: &World, object: &str) -> u32 {
    let object = world.content().object_id(object).expect("an object");
    super::shop::count_of(&world.state().player().inventory, object)
}

#[test]
fn wat_keeps_the_gildhalle_and_reaches_his_post_from_the_square() {
    let mut w = town();
    let wat = w.state.actor_at("ceaptun", WAT).expect("Wat").id;
    assert_eq!(w.state.actors[&wat].npc.as_deref(), Some("wat"));
    act(&mut w, TO_WAT);
    assert_eq!(w.state.player().pos, WAT.step(Dir::N), "{:?}", said(&w));
    act(&mut w, "chat:S");
    let line = said(&w).last().cloned().expect("said");
    assert!(
        line.starts_with("Wat One-Eye says: \"'Nother one"),
        "{line}"
    );
}

#[test]
fn wats_topics_give_the_chain_one_link_at_a_time() {
    let mut w = town();
    assert_eq!(topics(&w), ["quest-guild_quarry", "guild", "board"]);
    done(&mut w, &["guild_quarry"]);
    assert_eq!(topics(&w), ["quest-guild_hundred", "guild", "board"]);
    done(&mut w, &["guild_quarry", "guild_hundred"]);
    assert_eq!(topics(&w), ["quest-guild_scroll", "guild", "board"]);
    done(&mut w, &["guild_quarry", "guild_hundred", "guild_scroll"]);
    assert_eq!(topics(&w), ["guild", "board"]);
    // No gate but the chain's own: a Wicked character at level 1 is given every link.
    let mut w = town();
    w.player_mut().alignment = Some(crate::alignment::Score::new(-1000));
    assert_eq!(topics(&w)[0], "quest-guild_quarry");
    done(&mut w, &["guild_quarry"]);
    assert_eq!(topics(&w)[0], "quest-guild_hundred");
    done(&mut w, &["guild_quarry", "guild_hundred"]);
    assert_eq!(topics(&w)[0], "quest-guild_scroll");
}

#[test]
fn being_given_the_bounty_makes_the_player_a_tin_once() {
    let mut w = town();
    assert_eq!(rank(&w), None);
    beside_wat(&mut w);
    act(&mut w, "ask:S:quest-guild_quarry");
    assert_eq!(rank(&w).as_deref(), Some("Tin"));
    assert_eq!(
        promotions(&w),
        ["Wat One-Eye names you a Tin of the Weyfarers' Gilde."]
    );
    let log = (w.perception().quests().into_iter())
        .find(|q| q.id == "guild_quarry")
        .expect("logged");
    assert!(
        log.objective.starts_with("Wat One-Eye wants you to kill "),
        "{}",
        log.objective
    );
    // Asked again: the reminder, and no second promotion.
    act(&mut w, "ask:S:quest-guild_quarry");
    assert_eq!(promotions(&w).len(), 1);
    let stats = dump::layer_rows(&w, Layer::Stats);
    assert!(
        stats.contains(&"Guild Weyfarers' Gilde, Tin".to_string()),
        "{stats:?}"
    );
}

#[test]
fn the_hundred_done_makes_the_player_a_pewter() {
    let mut w = town();
    done(&mut w, &["guild_quarry"]);
    assert_eq!(rank(&w).as_deref(), Some("Tin"), "done counts as given");
    beside_wat(&mut w);
    act(&mut w, "ask:S:quest-guild_hundred");
    assert!(promotions(&w).is_empty(), "already Tin");
    let tag = (w
        .quests()
        .record("guild_hundred")
        .and_then(|r| r.roll.clone()))
    .expect("drawn")
    .target;
    let tag = tag.strip_prefix("tag:").unwrap_or(&tag);
    let member = (CONTENT.tagged(tag).next()).unwrap_or_else(|| panic!("a member of {tag}"));
    for _ in 0..100 {
        w.tally_kill(member, "cealdhlaw");
    }
    act(&mut w, "ask:S:quest-guild_hundred");
    assert!(
        said(&w).contains(&"Quest complete: The hundred.".to_string()),
        "{:?}",
        said(&w)
    );
    assert_eq!(rank(&w).as_deref(), Some("Pewter"));
    assert_eq!(
        promotions(&w),
        ["Wat One-Eye names you a Pewter of the Weyfarers' Gilde."]
    );
    assert!(carried(&w, "red_potion") >= 2);
    let stats = dump::layer_rows(&w, Layer::Stats);
    assert!(stats.contains(&"Guild Weyfarers' Gilde, Pewter".to_string()));
}

#[test]
fn the_scroll_handed_in_leaves_the_pack_and_an_unpaid_one_stays_billed() {
    let mut w = town();
    done(&mut w, &["guild_quarry", "guild_hundred"]);
    beside_wat(&mut w);
    act(
        &mut w,
        "ask:S:quest-guild_scroll quest_roll:guild_scroll:scroll_of_identify",
    );
    // The general store's scrolls, picked up and not paid for.
    let id = w.state.player;
    w.state.actors.get_mut(&id).expect("player").pos = Pos::new(29, 8);
    act(&mut w, "pickup");
    let identify = CONTENT.object_id("scroll_of_identify").expect("a scroll");
    let billed = |w: &World| {
        (w.state.current().shops[0].unpaid.iter())
            .filter(|s| s.object == identify)
            .map(|s| s.count)
            .sum::<u32>()
    };
    assert_eq!((carried(&w, "scroll_of_identify"), billed(&w)), (3, 3));
    beside_wat(&mut w);
    act(&mut w, "ask:S:quest-guild_scroll");
    assert!(
        said(&w).contains(&"Quest complete: A scroll for the Gilde.".to_string()),
        "{:?}",
        said(&w)
    );
    assert_eq!(carried(&w, "scroll_of_identify"), 2, "one handed in");
    assert_eq!(billed(&w), 3, "still owed for");
    assert_eq!(
        rank(&w).as_deref(),
        Some("Pewter"),
        "no rank for the scroll"
    );
}

#[test]
fn joining_the_gilde_replays_exactly() {
    let mut w = town();
    let mut replay = Replay::new(&w);
    let actions = format!("{TO_WAT} chat:S ask:S:guild ask:S:board ask:S:quest-guild_quarry");
    for action in parse_actions(&actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(rank(&w).as_deref(), Some("Tin"));
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    assert_eq!(rank(&verified).as_deref(), Some("Tin"));
}

/// Content with `guild` added to the Ascmearc's guilds, or the error it is refused with.
fn with_guild(guild: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert("campaigns/ascmearc/data/guilds/test.toml", guild);
    Content::load(&files).map_err(|e| e.message)
}

#[test]
fn guilds_are_checked_at_load() {
    let guild = |ranks: &str| {
        format!(
            "[[guild]]\nid = \"test\"\nname = \"the Test\"\nmaster = \"wat\"\nranks = [{ranks}]\n"
        )
    };
    with_guild(&guild(
        r#"{ title = "Tin", material = "tin", given = "guild_quarry" }"#,
    ))
    .expect("a valid guild");
    for (ranks, error) in [
        (
            r#"{ title = "Mithril", material = "mithril" }"#,
            "unknown material `mithril`",
        ),
        (
            r#"{ title = "Tin", material = "tin", given = "nothing" }"#,
            "unknown quest `nothing`",
        ),
        (
            r#"{ title = "Tin", material = "tin", given = "barrow_rats", done = "barrow_rats" }"#,
            "not both",
        ),
        (
            r#"{ title = "Tin", material = "tin" }, { title = "Iron", material = "iron", done = "barrow_rats" }"#,
            "comes after",
        ),
        (
            r#"{ title = "Tin", material = "tin" }, { title = "Tin", material = "iron" }"#,
            "listed twice",
        ),
        (r#"{ title = "", material = "tin" }"#, "`title`"),
    ] {
        let err = with_guild(&guild(ranks)).expect_err(ranks);
        assert!(err.contains(error), "{ranks}: {err}");
    }
    let err = with_guild(&guild("")).expect_err("no ranks");
    assert!(err.contains("`ranks`"), "{err}");
    let nobody = r#"[[guild]]
id = "test"
name = "the Test"
master = "nobody"
ranks = [{ title = "Tin", material = "tin" }]
"#;
    assert!(
        with_guild(nobody)
            .expect_err("no master")
            .contains("`master`")
    );
}
