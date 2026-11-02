//! The identification service (#432; spec §8 "Services"): a fixture appraiser who identifies
//! everything the player carries for 100 gold per unidentified stack; the count, the refusals,
//! magic items and scroll types identified, Pious left out, saves, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::magic::{Affix, Magic, Rarity};
use crate::replay::Replay;

/// The appraiser's words and service, added to the built-in dialogue.
const APPRAISER: &str = r##"
[[dialogue]]
npc = "appraiser"
lines = ["Let me see that."]

[[dialogue.service]]
id = "identify"
name = "An appraisal of your goods"
price = 100
does = "identify"
reply = "Entered and accounted for."
nothing = "You know all you carry."
"##;

// A lit counting house: the appraiser east of the player.
const HOUSE: &str = r##"
name = "Counting house"
ambient = [255, 255, 255, 0]
map = '''
#######
#@A...#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"A" = { terrain = "floor", creature = "hermit", npc = "appraiser" }
"##;

/// Content where the player starts with `gold` gold and `carry` (scenario carry entries).
fn content(gold: u32, carry: &[&str]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), format!("{text}{APPRAISER}"));
    files.insert(&path, &text);
    files.insert("levels/house.toml", HOUSE);
    let mut items: Vec<String> = carry.iter().map(|c| format!("\"{c}\"")).collect();
    if gold > 0 {
        items.insert(0, format!("\"gold_coin*{gold}\""));
    }
    files.insert(
        "scenarios/house.toml",
        &format!(
            "name = \"house\"\nstart_level = \"house\"\ncarry = [{}]\n",
            items.join(", ")
        ),
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world(gold: u32, carry: &[&str]) -> World {
    World::new(content(gold, carry), "house", Some(7)).expect("world")
}

/// Three fireball scrolls and two lightning scrolls: two unidentified stacks.
const SCROLLS: &[&str] = &["scroll_of_fireball*3", "scroll_of_lightning*2"];

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn gold(w: &World) -> u32 {
    let coin = w.content().object_id("gold_coin").expect("gold");
    super::shop::count_of(&w.state().player().inventory, coin)
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

/// The player carries an unidentified iron ring of the Bear.
fn carry_ring(w: &mut World) {
    let ring = w.content().object_id("iron_ring").expect("ring");
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: "of_the_bear".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    w.player_mut()
        .inventory
        .push(crate::object::Stack::with_magic(ring, 1, magic));
}

/// Whether the player knows everything they carry.
fn knows_all(w: &World) -> bool {
    let view = w.perception();
    view.inventory().iter().all(|s| view.lore().knows(s))
}

#[test]
fn it_is_listed_at_its_price_each_with_the_count_of_unidentified_stacks() {
    let mut w = world(500, SCROLLS);
    let topic = |w: &World| {
        let t = w.topics(ActorId(1)).pop().expect("the service");
        (t.id, t.price, t.items)
    };
    // Per stack: three fireball scrolls count once, as do two of lightning; gold is known.
    assert_eq!(topic(&w), ("identify".into(), Some(100), Some(2)));
    carry_ring(&mut w);
    assert_eq!(w.unidentified().len(), 3);
    assert_eq!(topic(&w), ("identify".into(), Some(100), Some(3)));
    // Pious halves a temple's price, never an appraisal's.
    w.player_mut().talents.take("pious", None);
    assert_eq!(topic(&w).1, Some(100));
}

#[test]
fn everything_is_identified_for_the_total_and_it_takes_a_chats_time() {
    let mut w = world(500, SCROLLS);
    carry_ring(&mut w);
    assert!(!knows_all(&w));
    let before = w.time();
    let out = run(&mut w, "ask:E:identify").pop().expect("asked");
    assert!(out.took_turn);
    assert!(w.time() > before, "it takes time");
    assert_eq!(gold(&w), 200, "three stacks at 100 gold each");
    assert!(knows_all(&w), "magic items and scroll types alike");
    let fireball = w
        .content()
        .object_id("scroll_of_fireball")
        .expect("fireball");
    assert!(w.lore().is_identified(fireball), "the type, from now on");
    let said = said(&w);
    let names = [
        "This is 3 scrolls of fireball!",
        "This is 2 scrolls of lightning!",
        "This is an iron ring of the Bear!",
    ];
    for name in names {
        assert!(said.iter().any(|m| m == name), "{name}: {said:?}");
    }
    let paid = said
        .iter()
        .position(|m| m.starts_with("You pay "))
        .expect("paid");
    assert_eq!(said[paid], "You pay the old hermit 300 gold.");
    assert_eq!(
        said.last().map(String::as_str),
        Some("The old hermit says: \"Entered and accounted for.\"")
    );
    // The coins went into the appraiser's purse.
    let coin = w.content().object_id("gold_coin").expect("gold");
    let purse = super::shop::count_of(&w.state().actors[&ActorId(1)].inventory, coin);
    assert_eq!(purse, 300);
}

#[test]
fn nothing_to_identify_or_too_little_gold_is_refused_for_nothing() {
    // Nothing unidentified: no charge, no time.
    let mut w = world(500, &[]);
    let out = run(&mut w, "ask:E:identify").pop().expect("asked");
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(gold(&w), 500);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("The old hermit says: \"You know all you carry.\"")
    );
    assert_eq!(w.topics(ActorId(1))[0].items, Some(0));
    // 150 gold for two stacks: all or nothing, so nothing.
    let mut w = world(150, SCROLLS);
    let out = run(&mut w, "ask:E:identify").pop().expect("asked");
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(gold(&w), 150);
    assert_eq!(w.unidentified().len(), 2, "nothing identified");
    assert_eq!(said(&w).last().map(String::as_str), Some("You cannot pay."));
    // No gold at all.
    let mut w = world(0, SCROLLS);
    run(&mut w, "ask:E:identify");
    assert_eq!(said(&w).last().map(String::as_str), Some("You cannot pay."));
    // Once everything is known, the next asking is free and refused.
    let mut w = world(200, SCROLLS);
    run(&mut w, "ask:E:identify ask:E:identify");
    assert_eq!(gold(&w), 0);
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("The old hermit says: \"You know all you carry.\"")
    );
}

#[test]
fn an_appraisal_is_saved_and_replayed() {
    let mut w = world(300, SCROLLS);
    let mut replay = Replay::new(&w);
    for action in &parse_actions("ask:E:identify wait ask:E:identify").expect("text") {
        w.apply(action);
        replay.push(action, &w);
    }
    assert!(knows_all(&w));
    let played = replay.run(content(300, SCROLLS)).expect("replay verifies");
    assert_eq!(played.state(), w.state());
    let loaded =
        crate::save::decode(&crate::save::encode(&w), content(300, SCROLLS)).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
}
