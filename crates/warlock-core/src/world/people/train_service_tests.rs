//! A skill taught for gold (`does = "train_skill"`, #1391; spec §8 "Dialogue": "Services"): a
//! fixture teacher who trains Mining, the skill digging needs. Listed while the player is
//! untrained, the price paid, the skill at the data's value, gone once trained, refusals for
//! nothing, digging refused before and allowed after, saves and replays, and the load checks.

use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;
use crate::test_support::{last_text as last_message, run};
use crate::world::*;

/// The teacher's words and service, added to the built-in dialogue.
const TEACHER: &str = r##"
[[dialogue]]
npc = "teacher"
lines = ["Rock's like a purse."]

[[dialogue.service]]
id = "train_mining"
name = "Teach me to mine"
price = 500
does = "train_skill"
temple = true
train = { skill = "mining", value = 5 }
reply = "Noo gan an' dig."
nothing = "Ye can mine already, man."
"##;

// A lit gallery: the teacher east of the player, a pick-axe underfoot, packed earth to the
// north.
const GALLERY: &str = r##"
name = "Gallery"
ambient = [255, 255, 255, 0]
map = '''
#######
#E#####
#@T...#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"E" = "packed_earth"
"@" = { terrain = "floor", spawn = "player", objects = ["pick_axe"] }
"T" = { terrain = "floor", creature = "hermit", npc = "teacher" }
"##;

fn files(gold: u32) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), format!("{text}{TEACHER}"));
    files.insert(&path, &text);
    files.insert("levels/gallery.toml", GALLERY);
    let carry = match gold {
        0 => String::new(),
        n => format!("carry = [\"gold_coin*{n}\"]\n"),
    };
    files.insert(
        "scenarios/gallery.toml",
        &format!("name = \"gallery\"\nstart_level = \"gallery\"\n{carry}"),
    );
    files
}

fn content(gold: u32) -> Arc<Content> {
    Arc::new(Content::load(&files(gold)).expect("test content is valid"))
}

/// The gallery with `gold` gold coins, the pick-axe ready in the Tool slot.
fn world(gold: u32) -> World {
    let mut w = World::new(content(gold), "gallery", None).expect("world");
    let pick = (w.state().player().inventory.len()).to_string();
    run(&mut w, &format!("pickup equip:tool:{pick}"));
    w
}

fn gold(w: &World) -> u32 {
    let coin = w.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&w.state().player().inventory, coin)
}

fn mining(w: &World) -> Option<u16> {
    let id = w.content().skill_id("mining").expect("Mining");
    w.state().player().skills.value(id)
}

fn earth_dug(w: &World) -> bool {
    let (level, _) = w.ground_truth();
    let t = level.terrain_at(Pos::new(1, 1)).expect("in bounds");
    w.content().terrain(t).id == "rubble"
}

#[test]
fn untrained_the_player_cannot_dig_and_is_told_why_for_no_time() {
    let mut w = world(0);
    assert_eq!(mining(&w), None);
    assert_eq!(
        w.perception().dig_refusal(),
        Some(crate::world::crafting::dig::UNTRAINED),
        "perception reads the rules' refusal"
    );
    let time = w.time();
    run(&mut w, "dig:N");
    assert_eq!(last_message(&w), crate::world::crafting::dig::UNTRAINED);
    assert_eq!(w.time(), time);
    assert!(!earth_dug(&w));
}

#[test]
fn the_teaching_is_listed_with_its_price_while_untrained() {
    let w = world(600);
    let topics = w.topics(ActorId(1));
    let listed: Vec<(&str, Option<u32>)> =
        (topics.iter()).map(|t| (t.id.as_str(), t.price)).collect();
    assert_eq!(listed, [("train_mining", Some(500))]);
}

#[test]
fn bought_it_trains_mining_takes_the_gold_and_a_chat_and_then_digging_works() {
    let mut w = world(600);
    let time = w.time();
    run(&mut w, "ask:E:train_mining");
    assert_eq!(gold(&w), 100);
    assert_eq!(mining(&w), Some(5));
    let said: Vec<&str> = (w.messages().iter()).map(|m| m.text.as_str()).collect();
    assert!(
        said.contains(&"The old hermit trains you in mining."),
        "{said:?}"
    );
    assert_eq!(
        last_message(&w),
        "The old hermit says: \"Noo gan an' dig.\""
    );
    assert!(w.time() > time, "it takes a chat's time");
    // Gone from the topics once trained; asked for anyway, its `nothing`, for no time or gold.
    assert!(w.topics(ActorId(1)).is_empty());
    let time = w.time();
    run(&mut w, "ask:E:train_mining");
    assert_eq!(
        last_message(&w),
        "The old hermit says: \"Ye can mine already, man.\""
    );
    assert_eq!((gold(&w), w.time(), mining(&w)), (100, time, Some(5)));
    // Now the packed earth gives way.
    assert_eq!(w.perception().dig_refusal(), None);
    run(&mut w, "dig:N");
    assert!(earth_dug(&w));
}

#[test]
fn short_of_gold_it_is_refused_for_nothing() {
    let mut w = world(499);
    let time = w.time();
    run(&mut w, "ask:E:train_mining");
    assert_eq!(last_message(&w), "You cannot pay.");
    assert_eq!((gold(&w), w.time(), mining(&w)), (499, time, None));
    assert_eq!(w.topics(ActorId(1)).len(), 1, "still offered");
}

#[test]
fn a_teaching_is_saved_and_replayed() {
    let mut w = World::new(content(600), "gallery", None).expect("world");
    let mut replay = Replay::new(&w);
    let pick = (w.state().player().inventory.len()).to_string();
    let walk = format!("pickup equip:tool:{pick} dig:N ask:E:train_mining dig:N wait");
    let actions = parse_actions(&walk).expect("text");
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert!(earth_dug(&w));
    let played = replay.run(content(600)).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let loaded = crate::save::decode(&crate::save::encode(&w), content(600)).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn a_teaching_names_a_known_skill_and_only_a_teaching_has_one() {
    let load = |fields: &str| {
        let mut files = files(0);
        let (path, text) = files.table("dialogue").next().expect("dialogue");
        let service = format!(
            "[[dialogue]]\nnpc = \"tutor\"\nlines = [\"x\"]\n[[dialogue.service]]\nid = \"s\"\n\
             name = \"S\"\nprice = 5\nreply = \"r\"\nnothing = \"n\"\n{fields}"
        );
        let (path, text) = (path.to_string(), format!("{text}\n{service}"));
        files.insert(&path, &text);
        Content::load(&files).map(|_| ()).map_err(|e| e.message)
    };
    let ok = "does = \"train_skill\"\ntrain = { skill = \"mining\", value = 5 }\n";
    assert_eq!(load(ok), Ok(()));
    let cases = [
        (
            ok.replace("\"mining\"", "\"delving\""),
            "unknown skill `delving`",
        ),
        ("does = \"train_skill\"\n".to_string(), "needs `train"),
        (
            "does = \"cure\"\ntrain = { skill = \"mining\", value = 5 }\n".to_string(),
            "only `train_skill` has `train`",
        ),
    ];
    for (fields, error) in cases {
        let got = load(&fields).expect_err(error);
        assert!(got.contains(error), "{got}");
    }
}
