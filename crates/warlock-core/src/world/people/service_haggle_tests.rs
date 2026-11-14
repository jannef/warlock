//! Services haggle a little (#1031): a healer's and a sage's price follows the player's favour,
//! a temple's stays flat, and every service paid trains Negotiation.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;

const FOLK: &str = r##"
[[dialogue]]
npc = "folk"
lines = ["Yes?"]

[[dialogue.service]]
id = "cure"
name = "A healer's cure"
price = 60
does = "cure"
reply = "There."
nothing = "Nothing ails you."

[[dialogue.service]]
id = "mend"
name = "A temple's mending"
price = 40
does = "mend"
temple = true
reply = "There."
nothing = "You are whole."

[[dialogue.service]]
id = "identify"
name = "A sage's appraisal"
price = 100
does = "identify"
reply = "There."
nothing = "You know all you carry."
"##;

const HOUSE: &str = r##"
name = "House"
ambient = [255, 255, 255, 0]
map = '''
#######
#@F...#
#######
'''
[legend]
"#" = "wall"
"." = "floor"
"@" = { terrain = "floor", spawn = "player" }
"F" = { terrain = "floor", creature = "hermit", npc = "folk" }
"##;

/// The house, with Negotiation marked by every deal when `eager`.
fn world(eager: bool) -> World {
    let mut files = ContentFiles::builtin();
    let (path, text) = files.table("dialogue").next().expect("dialogue");
    let (path, text) = (path.to_string(), format!("{text}{FOLK}"));
    files.insert(&path, &text);
    files.insert("levels/house.toml", HOUSE);
    files.insert(
        "scenarios/house.toml",
        "name = \"house\"\nstart_level = \"house\"\ncarry = [\"gold_coin*500\"]\n",
    );
    if eager {
        files.replace_in_table(
            "skills",
            "{ on = \"deal\", chance = 25 }",
            "{ on = \"deal\", chance = 100 }",
        );
    }
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "house", Some(7)).expect("world")
}

fn prices(w: &World) -> Vec<(String, u32)> {
    (w.topics(ActorId(1)).iter())
        .filter_map(|t| Some((t.id.clone(), t.price?)))
        .collect()
}

fn negotiation(w: &mut World, value: i32) {
    let id = w.content().skill_id("negotiation").expect("Negotiation");
    w.player_mut().skills.add(id, value);
}

#[test]
fn a_healers_and_a_sages_price_follow_favour_and_a_temples_stays_flat() {
    let mut w = world(false);
    let flat = prices(&w);
    let at = |name: &str, list: &[(String, u32)]| list.iter().find(|p| p.0 == name).expect(name).1;
    negotiation(&mut w, 100);
    let content = w.content().clone();
    let favour = shop::haggling(&content, content.shops().expect("shops"), w.state.player());
    assert!(favour >= 20, "Negotiation 100 gives 20: {favour}");
    let haggled = prices(&w);
    let less = |price: u32| (u64::from(price) * (100 - favour as u64)).div_ceil(100) as u32;
    assert_eq!(at("cure", &haggled), less(at("cure", &flat)));
    assert_eq!(at("identify", &haggled), less(at("identify", &flat)));
    assert!(at("cure", &haggled) < at("cure", &flat));
    assert_eq!(
        at("mend", &haggled),
        at("mend", &flat),
        "a temple does not haggle"
    );
}

#[test]
fn a_service_paid_trains_negotiation() {
    let mut w = world(true);
    negotiation(&mut w, 1);
    let id = w.content().skill_id("negotiation").expect("Negotiation");
    let marks = |w: &World| w.state.player().skills.get(id).expect("trained").marks;
    let before = marks(&w);
    for a in parse_actions("infect:the_shakes ask:E:cure").expect("actions") {
        w.apply(&a);
    }
    assert!(marks(&w) > before, "a deal was marked");
}
