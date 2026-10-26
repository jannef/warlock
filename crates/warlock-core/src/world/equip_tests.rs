//! Equipment (spec §6, §7): slots, wearing and wielding, swaps, two hands, rings on either hand,
//! the time it takes, dropping what is worn, weight, saves and replays, and what creatures wear
//! and wield.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::equipment;
use crate::replay::Replay;

// The player stands on a pile of arms and armor in a fully lit room; a kobold with a short sword
// stands in sight, and one with a spear and a jerkin behind a closed door.
const LEVEL: &str = r##"
name = "Armory"
ambient = [255, 255, 255, 0]
map = '''
###########
#@....#...#
#.....+.k.#
#...v.#...#
###########
'''
[legend]
"@" = { terrain = "floor", spawn = "player", objects = ["quarterstaff", "small_shield", "short_sword", "leather_jerkin", "chain_mail", "silver_ring*2", "short_bow", "arrow*12", "iron_helm"] }
"v" = { terrain = "floor", creature = "kobold", carry = ["copper_coin*3", "short_sword"], wield = "short_sword" }
"k" = { terrain = "floor", creature = "kobold", carry = ["spear", "leather_jerkin"], wield = ["spear"], wear = "leather_jerkin" }
"##;

// Inventory letters once everything is picked up.
const PICK_ALL: &str = "pickup";

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    // At peace: the kobolds stand by.
    files.insert(
        "data/factions.toml",
        "[[faction]]\nid = \"player\"\nname = \"you\"\n\
         [[faction]]\nid = \"vermin\"\nname = \"vermin\"\n\
         [[faction]]\nid = \"kobolds\"\nname = \"kobolds\"\n\
         [[faction]]\nid = \"goblins\"\nname = \"goblins\"\n\
         [[faction]]\nid = \"undead\"\nname = \"undead\"\n\
         [[faction]]\nid = \"marsh\"\nname = \"marsh things\"\n\
         [[faction]]\nid = \"cult\"\nname = \"cultists\"\n\
         [[faction]]\nid = \"duergar\"\nname = \"duergar\"\n\
         [[faction]]\nid = \"fire\"\nname = \"fire things\"\n\
         [[faction]]\nid = \"devil\"\nname = \"devils\"\n\
         [[faction]]\nid = \"wildlife\"\nname = \"wildlife\"\n\
         [[faction]]\nid = \"peaceful\"\nname = \"peaceful folk\"\n\
         [[faction]]\nid = \"merchants\"\nname = \"merchants\"\n\
         [[faction]]\nid = \"outraged\"\nname = \"outraged\"\nhostile = [\"player\"]\n",
    );
    files.insert("levels/armory.toml", LEVEL);
    // Strong enough to carry the whole armory (22.7 kg) unburdened: up to 25 kg (spec §8
    // "Burden").
    files.insert(
        "scenarios/armory.toml",
        "name = \"Armory\"\nstart_level = \"armory\"\n\
         stats = { strength = 18, toughness = 14 }\n",
    );
    files
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(&files()).expect("test content is valid"))
}

/// The armory with everything picked up: a quarterstaff, b small shield, c short sword,
/// d leather jerkin, e chain mail, f and g silver rings, h short bow, i 12 arrows, j iron helm.
fn world() -> World {
    let mut w = World::new(content(), "armory", None).expect("world");
    run(&mut w, PICK_ALL);
    assert_eq!(w.perception().inventory().len(), 10);
    w
}

fn run(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions parse") {
        last = w.apply(&action);
    }
    last
}

/// Seconds the action took.
fn took(w: &mut World, action: &str) -> u64 {
    run(w, action).elapsed.as_seconds()
}

/// What is in each slot that holds something: slot id and object id.
fn worn(w: &World) -> Vec<(String, String)> {
    let view = w.perception();
    let content = view.content();
    (view.equipment().iter())
        .map(|(slot, item)| {
            let object = view.inventory()[item].object;
            (
                content.slot(slot).id.clone(),
                content.object(object).id.clone(),
            )
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

fn last_message(w: &World) -> String {
    w.messages()
        .iter()
        .last()
        .map_or(String::new(), |m| m.text.clone())
}

#[test]
fn objects_go_only_where_they_fit_and_refusals_take_no_time() {
    let mut w = world();
    let turn = w.turn();
    for (action, says) in [
        ("equip:head:d", "A leather jerkin does not go there."),
        ("equip:missile_weapon:i", "12 arrows do not go there."),
        ("equip:main_hand:h", "A short bow does not go there."),
        ("equip:tail:a", "There is no slot `tail`."),
        ("equip:body:z", "You carry no such thing."),
        ("remove:body", "You have nothing there (body)."),
    ] {
        assert_eq!(took(&mut w, action), 0, "{action}");
        assert_eq!(last_message(&w), says, "{action}");
    }
    assert_eq!(w.turn(), turn);
    assert!(worn(&w).is_empty());
    // Putting on what is already there is refused too.
    run(&mut w, "equip:body:d");
    assert_eq!(took(&mut w, "equip:body:d"), 0);
    assert_eq!(last_message(&w), "You already have a leather jerkin there.");
}

#[test]
fn equipping_takes_the_time_of_the_slots_action() {
    let mut w = world();
    // Body armor costs 3000 (18 s at normal speed); the rest 1000 (6 s), on and off alike.
    assert_eq!(took(&mut w, "equip:body:d"), 18);
    assert_eq!(last_message(&w), "You put on a leather jerkin.");
    assert_eq!(took(&mut w, "equip:head:j"), 6);
    assert_eq!(took(&mut w, "equip:left_ring:f"), 6);
    assert_eq!(took(&mut w, "equip:main_hand:c"), 6);
    assert_eq!(last_message(&w), "You wield a short sword.");
    assert_eq!(took(&mut w, "equip:missile_weapon:h"), 6);
    assert_eq!(took(&mut w, "equip:missiles:i"), 6);
    assert_eq!(last_message(&w), "You ready 12 arrows.");
    assert_eq!(took(&mut w, "remove:head"), 6);
    assert_eq!(last_message(&w), "You take off an iron helm.");
    assert_eq!(took(&mut w, "remove:missile_weapon"), 6);
    assert_eq!(last_message(&w), "You put away a short bow.");
    // A swap takes the old thing off and puts the new one on: both take their time.
    assert_eq!(took(&mut w, "equip:body:e"), 36);
    assert!(
        (w.messages().iter().rev().take(2).map(|m| m.text.as_str())).eq([
            "You put on a chain mail shirt.",
            "You take off a leather jerkin."
        ]),
        "{:?}",
        w.messages()
    );
    assert_eq!(took(&mut w, "remove:body"), 18);
    assert_eq!(
        worn(&w),
        pairs(&[
            ("main_hand", "short_sword"),
            ("left_ring", "silver_ring"),
            ("missiles", "arrow")
        ])
    );
}

#[test]
fn rings_fit_either_hand() {
    let mut w = world();
    run(&mut w, "equip:left_ring:f equip:right_ring:g");
    let rings = |w: &World| {
        let view = w.perception();
        let slot = |id| view.content().slot_id(id).expect("slot");
        let eq = view.equipment();
        (eq.get(slot("left_ring")), eq.get(slot("right_ring")))
    };
    assert_eq!(rings(&w), (Some(5), Some(6)));
    // Moving the left ring to the right hand takes it off, takes off the ring there, and puts
    // it on: three actions' time. The left hand is bare.
    assert_eq!(took(&mut w, "equip:right_ring:f"), 18);
    assert_eq!(rings(&w), (None, Some(5)));
    assert_eq!(last_message(&w), "You put on a silver ring.");
    assert_eq!(
        equipment::mark(w.content(), w.perception().equipment(), 5).as_deref(),
        Some("on right hand")
    );
}

#[test]
fn a_two_handed_weapon_takes_both_hands() {
    let mut w = world();
    run(&mut w, "equip:main_hand:c equip:off_hand:b");
    assert_eq!(
        worn(&w),
        pairs(&[("main_hand", "short_sword"), ("off_hand", "small_shield")])
    );
    // The staff puts away the sword and the shield: three things move, three actions' time.
    assert_eq!(took(&mut w, "equip:main_hand:a"), 18);
    assert_eq!(
        worn(&w),
        pairs(&[("main_hand", "quarterstaff"), ("off_hand", "quarterstaff")])
    );
    assert_eq!(last_message(&w), "You wield a quarterstaff in both hands.");
    let view = w.perception();
    assert_eq!(
        equipment::mark(view.content(), view.equipment(), 0).as_deref(),
        Some("wielded in both hands")
    );
    // A shield in the off hand puts the staff away from both hands.
    assert_eq!(took(&mut w, "equip:off_hand:b"), 12);
    assert_eq!(worn(&w), pairs(&[("off_hand", "small_shield")]));
    // Taking the staff out of either hand frees both.
    run(&mut w, "equip:main_hand:a");
    assert_eq!(took(&mut w, "remove:off_hand"), 6);
    assert!(worn(&w).is_empty());
    assert_eq!(last_message(&w), "You put away a quarterstaff.");
}

#[test]
fn launchers_and_missiles_pair_by_kind_and_ready_apart() {
    let mut w = world();
    let content = w.content().clone();
    let id = |name| content.object_id(name).expect("object");
    assert!(content.fires(id("short_bow"), id("arrow")));
    assert!(!content.fires(id("short_bow"), id("rock")));
    // The bow and its arrows go in their own slots.
    run(&mut w, "equip:missile_weapon:h equip:missiles:i");
    assert_eq!(
        worn(&w),
        pairs(&[("missile_weapon", "short_bow"), ("missiles", "arrow")])
    );
    let view = w.perception();
    let (bow, arrows) = (view.inventory()[7].object, view.inventory()[8].object);
    assert!(content.fires(bow, arrows));
}

#[test]
fn dropping_takes_off_first_and_the_letters_follow() {
    let mut w = world();
    run(&mut w, "equip:body:d equip:head:j");
    // The jerkin comes off (18 s) and is dropped (6 s); the helm moves up a letter, still worn.
    assert_eq!(took(&mut w, "drop:d"), 24);
    let said: Vec<&str> = (w.messages().iter().rev().take(2).map(|m| m.text.as_str())).collect();
    assert_eq!(
        said,
        [
            "You drop a leather jerkin.",
            "You take off a leather jerkin."
        ]
    );
    assert_eq!(worn(&w), pairs(&[("head", "iron_helm")]));
    let view = w.perception();
    let helm = view.equipment().iter().next().map(|(_, i)| i);
    assert_eq!(helm, Some(8));
    // Dropping something not worn costs only the drop.
    assert_eq!(took(&mut w, "drop:a"), 6);
    assert_eq!(worn(&w), pairs(&[("head", "iron_helm")]));
}

#[test]
fn equipping_changes_no_weight() {
    let mut w = world();
    let before = w.perception().own_weight();
    run(
        &mut w,
        "equip:body:e equip:main_hand:a equip:missiles:i equip:left_ring:f",
    );
    assert_eq!(worn(&w).len(), 5, "the staff is in both hands");
    assert_eq!(w.perception().own_weight(), before);
    assert_eq!(w.actor_weight(w.player_id()), Some(before));
}

#[test]
fn equipment_is_saved_hashed_and_replayed() {
    let mut w = world();
    let bare = w.state_hash();
    let actions =
        "equip:body:e equip:main_hand:a equip:missiles:i remove:missiles equip:right_ring:g";
    run(&mut w, actions);
    assert_ne!(w.state_hash(), bare);
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content().clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert_eq!(worn(&loaded), worn(&w));

    let mut fresh = World::new(content(), "armory", None).expect("world");
    let mut replay = Replay::new(&fresh);
    for action in parse_actions(&format!("{PICK_ALL} {actions} drop:e")).expect("parse") {
        fresh.apply(&action);
        replay.push(&action, &fresh);
    }
    let text = replay.to_json();
    assert!(text.contains("\"equip:main_hand:a\"") && text.contains("\"remove:missiles\""));
    let (played, divergence) = Replay::from_json(&text)
        .expect("replay parses")
        .play(content())
        .expect("plays");
    assert_eq!(divergence, None);
    assert_eq!(played.state_hash(), fresh.state_hash());

    // A save whose equipment does not fit its inventory is refused.
    let mut state = w.state().clone();
    let player = state.player;
    let head = w.content().slot_id("head").expect("slot");
    if let Some(p) = state.actors.get_mut(&player) {
        p.equipment.put(&[head], 4); // chain mail on the head
    }
    assert!(matches!(
        World::from_state(w.content().clone(), state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn creatures_wear_and_wield_what_they_carry_and_are_seen_doing_it() {
    let w = world();
    let view = w.perception();
    let content = view.content();
    let kobold_in_sight = view
        .actors()
        .iter()
        .find(|a| a.kind == ActorKind::Creature)
        .expect("a kobold in sight");
    let gear: Vec<(String, String)> = view
        .gear(kobold_in_sight.id)
        .map(|g| {
            (
                content.slot(g.slot).id.clone(),
                content.object(g.stack.object).id.clone(),
            )
        })
        .collect();
    assert_eq!(gear, pairs(&[("main_hand", "short_sword")]));
    // Its coins stay unseen; so does everything of the kobold behind the door.
    assert_eq!(view.actors().len(), 2);
    let (_, actors) = w.ground_truth();
    let behind = actors
        .filter(|a| a.kind == ActorKind::Creature && a.id != kobold_in_sight.id)
        .collect::<Vec<_>>();
    assert_eq!(behind.len(), 1);
    assert_eq!(behind[0].equipment.items().len(), 2, "a spear and a jerkin");
    assert_eq!(view.gear(behind[0].id).count(), 0);
    // The dev dump shows what every creature wears and wields.
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Actors).join("\n");
    assert!(
        rows.contains("equipped body a leather jerkin, main_hand a spear;"),
        "{rows}"
    );
}

#[test]
fn spawn_equipment_is_checked() {
    for (entry, message) in [
        (
            r#"{ terrain = "floor", creature = "kobold", wield = "spear" }"#,
            "wields `spear`, which it does not carry",
        ),
        (
            r#"{ terrain = "floor", creature = "kobold", carry = ["leather_jerkin"], wield = "leather_jerkin" }"#,
            "`leather_jerkin` is worn, not wielded: use `wear`",
        ),
        (
            r#"{ terrain = "floor", creature = "kobold", carry = ["spear"], wear = ["spear"] }"#,
            "`spear` is wielded, not worn: use `wield`",
        ),
        (
            r#"{ terrain = "floor", creature = "kobold", carry = ["spear", "club"], wield = ["spear", "club"] }"#,
            "cannot wield `club`: no free slot fits it",
        ),
        (
            r#"{ terrain = "floor", creature = "kobold", carry = ["apple"], wield = "apple" }"#,
            "cannot wield `apple`",
        ),
        (
            r#"{ terrain = "floor", wear = "spear" }"#,
            "but no creature",
        ),
    ] {
        let mut files = files();
        files.insert("levels/armory.toml", &LEVEL.replace(
            r#""v" = { terrain = "floor", creature = "kobold", carry = ["copper_coin*3", "short_sword"], wield = "short_sword" }"#,
            &format!("\"v\" = {entry}"),
        ));
        let err = Content::load(&files).expect_err(entry);
        assert!(err.message.contains(message), "{entry}: {}", err.message);
    }
    // Two rings go on both hands.
    let mut files = files();
    files.insert(
        "levels/armory.toml",
        &LEVEL.replace(
            r#"carry = ["copper_coin*3", "short_sword"], wield = "short_sword""#,
            r#"carry = ["silver_ring*2"], wear = ["silver_ring", "silver_ring"]"#,
        ),
    );
    let content = Content::load(&files).expect("two rings");
    let level = content.level("armory").expect("level");
    let ring = content.object_id("silver_ring").expect("ring");
    let ringed = (level.creatures.iter()).find(|c| c.carry.iter().any(|s| s.object == ring));
    let ringed = ringed.expect("a kobold with two rings");
    let slots: Vec<&str> = (ringed.equipment.iter())
        .map(|(s, _)| content.slot(s).id.as_str())
        .collect();
    assert_eq!(slots, ["left_ring", "right_ring"]);
}
