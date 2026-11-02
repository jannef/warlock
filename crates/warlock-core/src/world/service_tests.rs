//! Services (spec §8 "Services"; the campaign design §6.1): a fixture sister who cures and mends
//! for gold, asked for like topics; the price, the refusals, Pious, saves, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::stats::Stat;

/// The sister's words and services, added to the built-in dialogue.
const SISTER: &str = r##"
[[dialogue]]
npc = "sister"
lines = ["Peace be with you."]

[[dialogue.topic]]
id = "temple"
name = "The temple"
reply = "It is old, and so am I."

[[dialogue.service]]
id = "cure"
name = "A cure for what ails you"
price = 60
does = "cure"
reply = "Go in health."
nothing = "Nothing ails you, child."

[[dialogue.service]]
id = "mend"
name = "A mending of body"
price = 40
does = "mend"
reply = "Be whole again."
nothing = "You are whole, child."
"##;

// A lit chapel: the sister east of the player.
const CHAPEL: &str = r##"
name = "Chapel"
ambient = [255, 255, 255, 0]
map = '''
#######
#@S...#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"S" = { terrain = "floor", creature = "hermit", npc = "sister" }
"##;

fn files(gold: u32) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), format!("{text}{SISTER}"));
    files.insert(&path, &text);
    files.insert("levels/chapel.toml", CHAPEL);
    let carry = match gold {
        0 => String::new(),
        n => format!("carry = [\"gold_coin*{n}\"]\n"),
    };
    files.insert(
        "scenarios/chapel.toml",
        &format!("name = \"chapel\"\nstart_level = \"chapel\"\n{carry}"),
    );
    files
}

fn content(gold: u32) -> Arc<Content> {
    Arc::new(Content::load(&files(gold)).expect("test content is valid"))
}

fn world(gold: u32) -> World {
    World::new(content(gold), "chapel", None).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn gold(w: &World) -> u32 {
    let coin = w.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&w.state().player().inventory, coin)
}

fn last_message(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn sick(w: &World) -> bool {
    w.state().player().diseases.iter().next().is_some()
}

/// Dexterity damage on the player, `points` of it.
fn hurt(w: &mut World, points: i32) {
    let player = w.player_mut();
    let score = player.stats.score(Stat::Dexterity);
    player.damage.take(Stat::Dexterity, points, score);
}

fn damage(w: &World) -> u8 {
    w.state().player().damage.get(Stat::Dexterity)
}

#[test]
fn services_are_listed_after_the_topics_with_their_price() {
    let w = world(100);
    let topics = w.topics(ActorId(1));
    let listed: Vec<(&str, Option<u32>)> =
        (topics.iter()).map(|t| (t.id.as_str(), t.price)).collect();
    assert_eq!(
        listed,
        [("temple", None), ("cure", Some(60)), ("mend", Some(40))]
    );
}

#[test]
fn a_cure_ends_every_disease_for_its_price_and_takes_a_chats_time() {
    let mut w = world(100);
    run(&mut w, "infect:the_shakes rest:24h*3");
    assert!(sick(&w), "the shakes took hold");
    let before = w.time();
    let out = run(&mut w, "ask:E:cure");
    assert!(!sick(&w), "cured");
    assert_eq!(gold(&w), 40);
    assert!(w.time() > before, "it takes time");
    let said: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        said.iter().any(|m| m == "You pay the old hermit 60 gold."),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|m| m == "The old hermit says: \"Go in health.\""),
        "{said:?}"
    );
    assert!(
        said.iter().any(|m| m == "Your hands are steady again."),
        "{said:?}"
    );
    // The conversation goes on, on the service asked for.
    let talked = out[0].events.iter().find_map(|e| match e {
        Event::Talked(talk) => Some(talk),
        _ => None,
    });
    assert_eq!(talked.and_then(|t| t.asked.as_deref()), Some("cure"));
    // The sister has the gold.
    let coin = w.content().object_id("gold_coin").expect("gold");
    let hers = super::shop::count_of(&w.state().actors[&ActorId(1)].inventory, coin);
    assert_eq!(hers, 60);
}

#[test]
fn a_cure_ends_a_disease_still_incubating() {
    let mut w = world(100);
    run(&mut w, "infect:the_shakes");
    assert!(sick(&w) && !w.state().player().diseases.shows());
    run(&mut w, "ask:E:cure");
    assert!(!sick(&w));
    assert_eq!(gold(&w), 40);
}

#[test]
fn nothing_to_cure_or_mend_or_no_gold_is_refused_for_nothing() {
    let mut w = world(100);
    let time = w.time();
    run(&mut w, "ask:E:cure");
    assert_eq!(
        last_message(&w),
        "The old hermit says: \"Nothing ails you, child.\""
    );
    run(&mut w, "ask:E:mend");
    assert_eq!(
        last_message(&w),
        "The old hermit says: \"You are whole, child.\""
    );
    assert_eq!((gold(&w), w.time()), (100, time));
    // Without enough gold.
    let mut w = world(30);
    run(&mut w, "infect:the_shakes");
    let time = w.time();
    run(&mut w, "ask:E:cure");
    assert_eq!(last_message(&w), "You cannot pay.");
    assert!(sick(&w));
    assert_eq!((gold(&w), w.time()), (30, time));
    let mut w = world(0);
    hurt(&mut w, 2);
    run(&mut w, "ask:E:mend");
    assert_eq!(last_message(&w), "You cannot pay.");
    assert_eq!(damage(&w), 2);
}

#[test]
fn a_mending_restores_every_point_but_not_while_a_disease_shows() {
    let mut w = world(100);
    hurt(&mut w, 3);
    let score = |w: &World| {
        w.state()
            .player()
            .effective_stats(w.content())
            .score(Stat::Dexterity)
    };
    let whole = score(&w) + 3;
    run(&mut w, "infect:the_shakes rest:24h*3");
    assert!(w.state().player().diseases.shows(), "the shakes show");
    let hurt_now = damage(&w);
    run(&mut w, "ask:E:mend");
    assert_eq!(last_message(&w), "The old hermit says: \"Be cured first.\"");
    assert_eq!((gold(&w), damage(&w)), (100, hurt_now));
    run(&mut w, "ask:E:cure ask:E:mend");
    assert_eq!(damage(&w), 0);
    assert_eq!(score(&w), whole);
    assert_eq!(gold(&w), 0);
    assert_eq!(last_message(&w), "The old hermit says: \"Be whole again.\"");
}

#[test]
fn pious_at_its_first_rank_pays_half_and_a_cure_mends_a_point() {
    let mut w = world(100);
    w.player_mut().talents.take("pious", None);
    let prices: Vec<Option<u32>> = w.topics(ActorId(1)).iter().map(|t| t.price).collect();
    assert_eq!(prices, [None, Some(30), Some(20)]);
    run(&mut w, "infect:the_shakes");
    hurt(&mut w, 2);
    run(&mut w, "ask:E:cure");
    assert!(!sick(&w));
    assert_eq!((gold(&w), damage(&w)), (70, 1));
    run(&mut w, "ask:E:mend");
    assert_eq!((gold(&w), damage(&w)), (50, 0));
}

#[test]
fn services_are_saved_and_replayed() {
    let mut w = world(100);
    let mut replay = Replay::new(&w);
    let actions =
        parse_actions("infect:the_shakes rest:24h*3 ask:E:cure ask:E:mend ask:E:cure wait")
            .expect("text");
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert!(!sick(&w));
    let played = replay.run(content(100)).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let loaded = crate::save::decode(&crate::save::encode(&w), content(100)).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn every_disease_is_cured_by_prayer() {
    let content = Content::builtin().expect("content");
    for d in content.disease_defs() {
        assert!(
            d.cured_by.iter().any(|c| c == super::service::PRAYER),
            "{} lacks prayer",
            d.id
        );
    }
}

#[test]
fn service_data_is_validated() {
    let load = |service: &str| {
        let mut files = files(0);
        let (path, text) = files.table("dialogue").next().expect("dialogue");
        let (path, text) = (path.to_string(), format!("{text}\n{service}"));
        files.insert(&path, &text);
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let service = |fields: &str| {
        format!(
            "[[dialogue]]\nnpc = \"abbot\"\nlines = [\"x\"]\n[[dialogue.topic]]\nid = \"t\"\n\
             name = \"T\"\nreply = \"r\"\n[[dialogue.service]]\n{fields}"
        )
    };
    let ok = "id = \"cure\"\nname = \"A cure\"\nprice = 5\ndoes = \"cure\"\nreply = \"r\"\nnothing = \"n\"\n";
    assert_eq!(load(&service(ok)), Ok(()));
    let cases = [
        (ok.replace("price = 5", "price = 0"), "at least 1 gold"),
        (
            ok.replace("\"cure\"\nname", "\"t\"\nname"),
            "another service",
        ),
        (ok.replace("\"cure\"\nname", "\"Cure\"\nname"), "lower-case"),
        (
            ok.replace("does = \"cure\"", "does = \"bless\""),
            "unknown variant",
        ),
        (ok.replace("nothing = \"n\"", "nothing = \"\""), "not empty"),
    ];
    for (fields, error) in cases {
        let got = load(&service(&fields)).expect_err(error);
        assert!(got.contains(error), "{got}");
    }
    // Paid in the shops' currency.
    let mut files = files(0);
    files.remove("data/shops.toml");
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), text.to_string());
    files.insert(&path, &text);
    let got = Content::load(&files).map(|_| ()).map_err(|e| e.message);
    assert!(got.is_err_and(|e| e.contains("data/shops.toml")));
}
