//! Spirit weapon (spec §8 "Prestige classes"; #881): the Hunter's class ability calls a weapon
//! of spirit-stuff into the main hand in any melee weapon's shape, for the time of a wield; in
//! the hand it strikes surer, threatens lower, and ignores Protection; it fades the moment it
//! leaves the hand (removed, swapped, dropped, sold), is never thrown, and is never handed in.

use super::*;
use crate::ability::{Ability, Aim};
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ActionKind, ClassActive, ContentFiles};
use crate::dump::{self, Layer};
use crate::magic::describe::{self, LineKind};
use crate::replay::Replay;
use crate::test_support::builtin;
use crate::test_support::run;
use std::sync::LazyLock;

const SPIRIT: ClassActive = ClassActive::SpiritWeapon;

/// A lit hall; the player carries a long sword (a) and a buckler (b).
static HALL: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n#######\n#@....#\n#######\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\n\
         carry = [\"long_sword\", \"buckler\"]\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// The hall, the player without the ability.
fn plain() -> World {
    World::new(HALL.clone(), "hall", Some(3)).expect("world")
}

/// The hall, the player with Spirit weapon.
fn hunter() -> World {
    let mut w = plain();
    w.player_mut().talents.take("spirit_weapon", None);
    w
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.display())
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display() == text)
}

/// The positions of the spirit stacks the player carries.
fn spirits(w: &World) -> Vec<usize> {
    let pack = &w.state.player().inventory;
    (0..pack.len()).filter(|&i| pack[i].is_spirit()).collect()
}

/// What the player holds in `slot`, by object id.
fn held(w: &World, slot: &str) -> Option<String> {
    let content = w.content();
    let player = w.state.player();
    let item = player.equipment.get(content.slot_id(slot)?)?;
    Some(content.object(player.inventory[item].object).id.clone())
}

#[test]
fn the_hunter_gains_it_at_its_first_level() {
    let hunter = HALL.prestige("hunter").expect("the Hunter");
    assert_eq!(hunter.levels, 30);
    let at = (hunter.abilities.iter()).find(|(_, t)| t == "spirit_weapon");
    assert_eq!(at.map(|(level, _)| *level), Some(1));
    let talents = HALL.talents();
    let id = talents.id("spirit_weapon").expect("the talent");
    assert!(talents.get(id).granted, "a class ability, never picked");
    assert_eq!(Ability::Class(SPIRIT).aim(&HALL), Aim::Shape);
    assert_eq!(SPIRIT.cost(), ActionKind::Wield, "the time of a wield");
}

#[test]
fn every_melee_weapon_is_a_shape_by_skill_then_id() {
    let shapes = crate::world::weapon_shapes(&HALL);
    let ids: Vec<&str> = (shapes.iter())
        .map(|&o| HALL.object(o).id.as_str())
        .collect();
    assert_eq!(ids.len(), 23, "{ids:?}");
    assert_eq!(
        &ids[..3],
        ["dagger", "knife", "stiletto"],
        "daggers first, by id"
    );
    for id in [
        "long_sword",
        "great_axe",
        "javelin",
        "quarterstaff",
        "war_hammer",
    ] {
        assert!(ids.contains(&id), "{id}");
    }
    for id in ["short_bow", "arrows", "buckler", "pick_axe"] {
        assert!(!ids.contains(&id), "{id}");
    }
    let skills: Vec<_> = (shapes.iter()).map(|&o| HALL.object(o).skill).collect();
    assert!(skills.windows(2).all(|p| p[0] <= p[1]), "grouped by skill");
    assert_eq!(plain().perception().weapon_shapes(), shapes);
}

#[test]
fn without_it_the_call_is_refused_with_no_time() {
    let mut w = plain();
    let out = run(&mut w, "use:spirit_weapon:long_sword");
    assert!(!out[0].took_turn);
    assert_eq!(last(&w), "You have no such ability.");
    assert!(spirits(&w).is_empty());
}

#[test]
fn a_call_puts_a_spirit_weapon_in_the_hand_for_a_wield() {
    let mut w = hunter();
    let rows = dump::layer_rows(&w, Layer::Abilities);
    assert!(
        rows.contains(&"Spirit weapon (class): a weapon's shape".to_string()),
        "{rows:?}"
    );
    // Without a shape it asks for one; a shape that is none is refused; no time either way.
    for (action, says) in [
        ("use:spirit_weapon", "Call it in what shape?"),
        (
            "use:spirit_weapon:longsword",
            "There is no weapon `longsword` to call.",
        ),
        (
            "use:spirit_weapon:short_bow",
            "You cannot call a short bow: it is no weapon for the hand.",
        ),
        (
            "use:spirit_weapon:arrow",
            "You cannot call an arrow: it is no weapon for the hand.",
        ),
    ] {
        let out = run(&mut w, action);
        assert!(!out[0].took_turn, "{action}");
        assert_eq!(last(&w), says, "{action}");
    }
    let out = run(&mut w, "use:spirit_weapon:long_sword");
    assert!(out[0].took_turn);
    assert_eq!(last(&w), "A long sword of pale light is in your hand.");
    assert_eq!(spirits(&w), [2], "made into the pack");
    assert_eq!(held(&w, "main_hand").as_deref(), Some("long_sword"));
    let stack = &w.state.player().inventory[2];
    assert!(w.lore().knows(stack), "known for what it is");
    assert_eq!(
        stack.weight(w.content()),
        Stack::one(stack.object).weight(w.content())
    );
    // Priced as wielding steel with the hands empty.
    let mut steel = hunter();
    let wield = run(&mut steel, "equip:main_hand:a");
    assert_eq!(out[0].elapsed, wield[0].elapsed);
    // Nothing else spent, and no limit: again at once.
    let out = run(&mut w, "use:spirit_weapon:long_sword");
    assert!(out[0].took_turn);
    assert_eq!(spirits(&w), [2], "the one held faded first");
    assert!(said(&w, "The long sword fades from your hand."));
}

#[test]
fn a_two_handed_shape_puts_away_what_the_hands_held() {
    let mut w = hunter();
    run(&mut w, "equip:main_hand:a equip:off_hand:b");
    let out = run(&mut w, "use:spirit_weapon:great_axe");
    assert!(out[0].took_turn);
    assert!(said(&w, "You put away a long sword."));
    assert!(said(&w, "You put away a buckler."));
    assert_eq!(last(&w), "A great axe of pale light is in your hands.");
    assert_eq!(held(&w, "main_hand").as_deref(), Some("great_axe"));
    assert_eq!(held(&w, "off_hand").as_deref(), Some("great_axe"));
    assert_eq!(
        w.state.player().inventory.len(),
        3,
        "steel kept in the pack"
    );
    // Reshaped to a dagger for the corridor: the axe fades, the off hand stays empty.
    run(&mut w, "use:spirit_weapon:dagger");
    assert_eq!(held(&w, "main_hand").as_deref(), Some("dagger"));
    assert_eq!(held(&w, "off_hand"), None);
    assert_eq!(spirits(&w), [2]);
}

#[test]
fn in_the_hand_it_strikes_surer_keener_and_through_armour() {
    let mut steel = hunter();
    run(&mut steel, "equip:main_hand:a");
    let mut spirit = hunter();
    run(&mut spirit, "use:spirit_weapon:long_sword");
    let content = HALL.clone();
    let a = steel.state.player().melee(&content);
    let b = spirit.state.player().melee(&content);
    assert_eq!(b.to_hit, a.to_hit + 3);
    assert_eq!(b.threat, a.threat - 2);
    assert_eq!(a.penetration, 0);
    assert!(crate::world::pierces_all(&b));
    assert_eq!(b.damage, a.damage, "its base's dice");
    // Without the talent the same stack is plain steel.
    let mut lost = spirit;
    lost.player_mut().talents = Default::default();
    let c = lost.state.player().melee(&content);
    assert_eq!((c.to_hit, c.threat, c.penetration), (a.to_hit, a.threat, 0));
}

#[test]
fn a_light_spirit_weapon_in_the_off_hand_is_one_too() {
    let mut w = hunter();
    run(&mut w, "equip:main_hand:a use:spirit_weapon:dagger");
    // The dagger, moved to the off hand, is still in a hand: it stays.
    let dagger = spirits(&w)[0];
    run(&mut w, &format!("equip:off_hand:{dagger}"));
    run(&mut w, "equip:main_hand:a");
    assert_eq!(held(&w, "off_hand").as_deref(), Some("dagger"));
    let content = HALL.clone();
    let (_, off) = w
        .state
        .player()
        .off_hand_blow(&content)
        .expect("two weapons");
    assert!(crate::world::pierces_all(&off));
}

#[test]
fn it_fades_when_taken_off_or_swapped_for_steel() {
    let mut w = hunter();
    run(&mut w, "use:spirit_weapon:long_sword");
    let out = run(&mut w, "remove:main_hand");
    assert!(out[0].took_turn);
    assert!(said(&w, "You put away a long sword."));
    assert_eq!(last(&w), "The long sword fades from your hand.");
    assert!(spirits(&w).is_empty());
    assert_eq!(w.state.player().inventory.len(), 2);
    run(&mut w, "use:spirit_weapon:mace equip:main_hand:a");
    assert!(spirits(&w).is_empty(), "swapped for steel");
    assert_eq!(held(&w, "main_hand").as_deref(), Some("long_sword"));
    // Readied as a missile weapon it is out of the hands: it fades.
    run(&mut w, "use:spirit_weapon:dagger");
    let dagger = spirits(&w)[0];
    run(&mut w, &format!("equip:missile_weapon:{dagger}"));
    assert!(spirits(&w).is_empty());
}

#[test]
fn a_drop_lets_it_fade_and_drops_the_rest() {
    let mut w = hunter();
    run(&mut w, "use:spirit_weapon:long_sword");
    let out = run(&mut w, "drop:c");
    assert!(out[0].took_turn, "taking it off takes its time");
    assert!(spirits(&w).is_empty());
    assert!(w.state.current().objects_at(w.player_pos()).is_empty());
    // Everything at once: the steel lies on the floor, the spirit-stuff is gone.
    run(&mut w, "use:spirit_weapon:long_sword drop");
    let pile = w.state.current().objects_at(w.player_pos());
    assert_eq!(pile.len(), 2, "{pile:?}");
    assert!(pile.iter().all(|s| !s.is_spirit()));
    assert!(w.state.player().inventory.is_empty());
    assert!(w.state.player().equipment.is_empty());
}

#[test]
fn it_is_never_thrown_or_handed_in() {
    let mut w = hunter();
    run(&mut w, "use:spirit_weapon:dagger");
    let out = run(&mut w, "throw:c:4,1");
    assert!(!out[0].took_turn);
    assert_eq!(last(&w), "It will not leave your hand.");
    assert_eq!(spirits(&w), [2]);
    let dagger = w.content().object_id("dagger").expect("dagger");
    assert_eq!(
        super::shop::count_of(&w.state.player().inventory, dagger),
        0
    );
}

#[test]
fn a_shop_never_sees_it() {
    // Grimbold's General Store in the courtyard, the player carrying their lantern, gold, and
    // a scroll of light (`shop_tests.rs`).
    let mut w = World::new(builtin(), "courtyard", None).expect("courtyard");
    run(
        &mut w,
        "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5",
    );
    w.player_mut().talents.take("spirit_weapon", None);
    run(&mut w, "use:spirit_weapon:long_sword");
    let at = spirits(&w)[0];
    let keeper = w.state.current().shops[0].keeper;
    let currency = w.content().shops().expect("shops").currency;
    let purse = super::shop::count_of(&w.state.actors[&keeper].inventory, currency);
    let floor = w.state.current().objects_at(w.player_pos()).len();
    let out = run(&mut w, &format!("sell:{at}"));
    assert!(out[0].took_turn);
    assert!(spirits(&w).is_empty());
    assert_eq!(
        super::shop::count_of(&w.state.actors[&keeper].inventory, currency),
        purse
    );
    assert_eq!(w.state.current().objects_at(w.player_pos()).len(), floor);
}

#[test]
fn its_description_and_the_dumps_say_what_it_is() {
    let mut w = hunter();
    run(&mut w, "use:spirit_weapon:long_sword");
    let stack = w.state.player().inventory[2].clone();
    let lines = describe::lines(&stack, w.content());
    let text = |kind: LineKind| -> Vec<String> {
        (lines.iter())
            .filter(|l| l.kind == kind)
            .map(|l| l.text.clone())
            .collect()
    };
    assert_eq!(
        text(LineKind::Power),
        [
            "A weapon of spirit-stuff: it finds the gaps in any armour, and fades when it leaves your hand."
        ]
    );
    let stats = text(LineKind::Stat);
    assert!(stats.iter().any(|t| t == "Pierces armour"), "{stats:?}");
    assert!(
        stats.iter().any(|t| t.contains("criticals on 18–24")),
        "{stats:?}"
    );
    assert!(
        stats.iter().any(|t| t.starts_with("To hit +3")),
        "{stats:?}"
    );
    let plain = describe::lines(&Stack::one(stack.object), w.content());
    assert!(!plain.iter().any(|l| l.kind == LineKind::Power));
    // The item lists, the attacks, and JSON.
    assert_eq!(
        w.lore().listed(&stack),
        "a long sword (+3 hit, 1d9) (spirit)"
    );
    let inventory = dump::layer_rows(&w, Layer::Inventory);
    assert!(
        inventory
            .iter()
            .any(|r| r.contains("a long sword (+3 hit, 1d9) (spirit)")),
        "{inventory:?}"
    );
    let attacks = dump::layer_rows(&w, Layer::Attacks);
    assert!(
        attacks[0].starts_with("Main hand long sword:") && attacks[0].ends_with(", pierces armour"),
        "{attacks:?}"
    );
    let json = dump::json(&w, &[], false);
    let abilities = json["abilities"].as_array().expect("abilities");
    let row = (abilities.iter()).find(|a| a["id"] == "spirit_weapon");
    assert_eq!(row.expect("the ability")["aim"], "a weapon's shape");
}

#[test]
fn a_save_holds_it_in_the_hand() {
    let mut w = hunter();
    run(&mut w, "use:spirit_weapon:long_sword");
    let state = w.state.as_ref().clone();
    let json = serde_json::to_string(&state).expect("json");
    assert!(json.contains("\"spirit\":true"), "saved");
    let back = World::from_state(HALL.clone(), state).expect("restore");
    assert_eq!(back.state_hash(), w.state_hash());
    assert_eq!(spirits(&back), [2]);
    assert_eq!(held(&back, "main_hand").as_deref(), Some("long_sword"));
    // Nothing of spirit-stuff, nothing saved for it.
    let json = serde_json::to_string(plain().state.as_ref()).expect("json");
    assert!(!json.contains("spirit"));
}

/// The built-in content with the bard capped at 12 and a fixture calling into the shipped
/// Hunter from it in the showcase (the shipped bard's cap and callings are other issues'), the
/// class tables stripped (the bard's reach past 12).
static CALLED: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    crate::content::strip_class_tables(&mut files);
    files.edit_table("classes", |text| match text.contains("id = \"bard\"") {
        true => text.replace("max_level = 30", "max_level = 12"),
        false => text.to_string(),
    });
    files.insert(
        "campaigns/showcase/data/callings/hunter.toml",
        "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 12\n\
         message = \"The oath is in you now, as a blade is in its sheath. You are a hunter.\"\n",
    );
    Arc::new(Content::load(&files).expect("fixture content"))
});

/// A human bard in the showcase, of [`CALLED`].
fn bard() -> World {
    let spec: CharacterSpec = "human,male,bard".parse().expect("a character");
    World::with_character(CALLED.clone(), "showcase", Some(7), Some(&spec)).expect("world")
}

#[test]
fn the_calling_into_the_hunter_gives_spirit_weapon() {
    let mut w = bard();
    run(&mut w, "call:hunter");
    assert_eq!(w.player_level(), 13, "Hunter 1");
    assert!(w.state.player().talents.holds("spirit_weapon", None));
    assert!(said(&w, "You gain Spirit weapon."));
    let talents = dump::layer_rows(&w, Layer::Talents);
    assert!(
        talents.contains(&"Spirit weapon (class)".to_string()),
        "{talents:?}"
    );
    while w.due().is_some() {
        run(&mut w, "levelup:auto");
    }
    let out = run(&mut w, "use:spirit_weapon:long_sword");
    assert!(out[0].took_turn, "{}", last(&w));
}

#[test]
fn its_action_text_round_trips_and_replays() {
    let action = Action::Shape(SPIRIT, "long_sword".into());
    assert_eq!(action.to_string(), "use:spirit_weapon:long_sword");
    assert_eq!(
        parse_actions("use:spirit_weapon:long_sword").expect("parses"),
        [action]
    );
    assert!(parse_actions("use:spirit_weapon:Long-sword").is_err());
    let mut w = bard();
    let actions = parse_actions(
        "call:hunter levelup:auto*12 use:spirit_weapon:long_sword wait remove:main_hand \
         use:spirit_weapon:great_axe wait",
    )
    .expect("actions");
    let mut replay = Replay::new(&w);
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    assert_eq!(held(&w, "main_hand").as_deref(), Some("great_axe"));
    let back = replay.run(CALLED.clone()).expect("replays");
    assert_eq!(back.state_hash(), w.state_hash());
}
