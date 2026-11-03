//! The collar (`collar.rs`; spec §8 "Allies": "The collar"): the hand-over and its price, the old
//! one on the floor, the refusals, the amulet's magic on the beast, the pile on its death, what
//! the player is shown, and a save and a replay.

use super::fight::Harm;
use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::{Action, parse_actions};
use crate::content::Content;
use crate::geom::{Dir, Pos};
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use crate::replay::Replay;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// A kitted run of `spec` in the courtyard.
fn kitted(spec: &str) -> World {
    let picks = vec!["keen_ears".into(), "lean".into(), "healthy".into()];
    let spec = (spec.parse::<crate::character::CharacterSpec>())
        .expect("a character")
        .with_talents(picks);
    World::start_run(
        BUILTIN.clone(),
        "courtyard",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

/// A kitted ranger's run, its beast beside them.
fn ranger() -> World {
    let w = kitted("human,male,ranger");
    assert!(way(&w).is_some(), "the beast starts beside the player");
    w
}

fn way(w: &World) -> Option<Dir> {
    (allies_beside(&w.state).into_iter())
        .find(|(_, a)| a.companion().is_some())
        .map(|(d, _)| d)
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn plain(id: &str) -> Stack {
    Stack::one(BUILTIN.object_id(id).expect(id))
}

/// An amulet of warmth: it resists the cold.
fn warm() -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 8,
        affixes: vec![Affix {
            id: "of_warmth".into(),
            tier: 9,
            value: 30,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = BUILTIN.object_id("gold_amulet").expect("gold_amulet");
    (magic.check(BUILTIN.object(object), &BUILTIN)).expect("a valid item");
    Stack::with_magic(object, 1, magic)
}

/// Puts `stack` in the player's pack; its position.
fn carry(w: &mut World, stack: Stack) -> usize {
    let pack = w.state.inventory_mut();
    pack.push(stack);
    pack.len() - 1
}

fn beast(w: &World) -> &Actor {
    &w.state.actors[&w.companion().expect("a companion")]
}

fn name(w: &World) -> String {
    beast(w).companion().expect("data").name.clone()
}

/// What the beast has in its neck slot, by object id.
fn collar(w: &World) -> Option<String> {
    let neck = BUILTIN.slot_id("neck").expect("neck");
    let a = beast(w);
    let on = a.equipment.get(neck)?;
    Some(BUILTIN.object(a.inventory[on].object).id.clone())
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// How long putting an amulet on takes the same character.
fn wearing() -> crate::time::Duration {
    let mut w = ranger();
    let item = carry(&mut w, plain("jade_amulet"));
    let time = w.time();
    run(&mut w, &format!("equip:neck:{item}"));
    w.time().since(time)
}

fn give(item: usize) -> Action {
    Action::Give { item, ally: None }
}

#[test]
fn giving_fastens_one_on_the_beast_at_a_wears_price() {
    let mut w = ranger();
    let before = w.state.player().inventory.len();
    let item = carry(&mut w, plain("jade_amulet"));
    let time = w.time();
    w.apply(&give(item));
    assert_eq!(collar(&w).as_deref(), Some("jade_amulet"));
    assert_eq!(w.state.player().inventory.len(), before, "out of the pack");
    assert_eq!(w.time().since(time), wearing(), "a wear's time");
    let line = format!("You fasten the jade amulet about {}'s neck.", name(&w));
    assert!(said(&w).contains(&line), "{:?}", said(&w));
}

#[test]
fn only_one_of_a_stack_goes_and_the_way_may_be_named() {
    let mut w = ranger();
    let mut two = plain("jade_amulet");
    two.count = 2;
    let item = carry(&mut w, two);
    let dir = way(&w).expect("beside");
    let wrong = (Dir::ALL.into_iter()).find(|d| *d != dir).expect("a way");
    w.apply(&Action::Give {
        item,
        ally: Some(wrong),
    });
    assert_eq!(collar(&w), None, "nobody that way");
    w.apply(&Action::Give {
        item,
        ally: Some(dir),
    });
    assert_eq!(collar(&w).as_deref(), Some("jade_amulet"));
    assert_eq!(w.state.player().inventory[item].count, 1);
    assert_eq!(beast(&w).inventory.len(), 1);
    assert_eq!(beast(&w).inventory[0].count, 1);
}

#[test]
fn what_it_wore_falls_on_its_cell() {
    let mut w = ranger();
    let item = carry(&mut w, plain("jade_amulet"));
    w.apply(&give(item));
    let item = carry(&mut w, plain("bone_amulet"));
    w.apply(&give(item));
    assert_eq!(collar(&w).as_deref(), Some("bone_amulet"));
    let a = beast(&w);
    assert_eq!(a.inventory.len(), 1, "it carries the one it wears");
    let jade = BUILTIN.object_id("jade_amulet").expect("jade");
    assert!(
        (w.state.current().objects_at(a.pos).iter()).any(|s| s.object == jade),
        "the old one lies under it"
    );
    let line = format!("{}'s jade amulet falls to the floor.", name(&w));
    assert!(said(&w).contains(&line), "{:?}", said(&w));
}

#[test]
fn what_the_player_wears_comes_off_first_and_pays_for_it() {
    let mut w = ranger();
    let item = carry(&mut w, plain("jade_amulet"));
    run(&mut w, &format!("equip:neck:{item}"));
    let neck = BUILTIN.slot_id("neck").expect("neck");
    assert_eq!(w.state.player().equipment.get(neck), Some(item));
    let time = w.time();
    w.apply(&give(item));
    assert_eq!(w.state.player().equipment.get(neck), None);
    assert_eq!(collar(&w).as_deref(), Some("jade_amulet"));
    assert!(w.time().since(time) > wearing(), "the taking off too");
}

#[test]
fn the_refusals_take_no_time() {
    let mut w = ranger();
    let sword = carry(&mut w, plain("long_sword"));
    let time = w.time();
    w.apply(&give(sword));
    assert_eq!(collar(&w), None);
    let line = format!("{} has no use for that.", name(&w));
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    w.apply(&give(999));
    assert!(said(&w).contains(&"You carry no such thing.".to_string()));
    // In the retinue it stands on no cell of the level.
    let id = w.companion().expect("a companion");
    let actor = w.state.actors.get_mut(&id).expect("the beast");
    actor.ally.as_mut().expect("an ally").retinue = true;
    let amulet = carry(&mut w, plain("jade_amulet"));
    w.apply(&give(amulet));
    assert_eq!(collar(&w), None);
    let line = format!("{} is not beside you.", name(&w));
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    assert_eq!(w.time(), time);
}

/// A kitted witch's run, her familiar beside her.
fn witch() -> World {
    let w = kitted("human,female,witch");
    assert!(w.familiar().is_some(), "she starts with one");
    w
}

#[test]
fn a_familiar_wears_a_collar_as_a_companion_does() {
    let mut w = witch();
    assert!(
        (w.perception().abilities().iter())
            .any(|a| a.ability == Ability::Give && a.unavailable.is_none()),
        "listed, the familiar beside her"
    );
    let id = w.familiar().expect("a familiar");
    let amulet = carry(&mut w, plain("jade_amulet"));
    let time = w.time();
    w.apply(&give(amulet));
    assert_eq!(w.time().since(time), wearing());
    let familiar = &w.state.actors[&id];
    let slot = w.content.slot_id("neck").expect("the neck slot");
    let on = familiar.equipment.get(slot).expect("worn");
    let worn = w.content.object(familiar.inventory[on].object);
    assert_eq!(worn.id, "jade_amulet");
    let name = familiar.kept().expect("kept").name.clone();
    let line = format!("You fasten the jade amulet about {name}'s neck.");
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    // The dumps show it, by the familiar's own layer.
    let layers = crate::dump::parse_layers("familiar").unwrap();
    let dump = crate::dump::ascii(&w, &layers);
    assert!(dump.contains("wearing a jade amulet"), "{dump}");
    assert_eq!(
        crate::dump::json(&w, &[], false)["familiar"]["wears"],
        "a jade amulet"
    );
    // With her familiar away, the row is greyed, and the rules name it.
    let actor = w.state.actors.get_mut(&id).expect("the familiar");
    actor.ally.as_mut().expect("an ally").retinue = true;
    actor.pos = Pos::new(0, 0);
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    let row = (w.perception().abilities().into_iter())
        .find(|a| a.ability == Ability::Give)
        .expect("listed");
    assert_eq!(row.unavailable, Some(Unavailable::NotBeside));
}

#[test]
fn with_two_kept_beasts_beside_the_way_says_which_wears_it() {
    let mut w = witch();
    run(&mut w, "ally:hound");
    let hound = w.content.creature_id("hound").expect("hounds");
    let snel = (w.allies_of(w.state.player).into_iter())
        .find(|id| w.state.actors[id].creature == hound)
        .expect("the hound");
    (w.state
        .actors
        .get_mut(&snel)
        .unwrap()
        .ally
        .as_mut()
        .unwrap())
    .source = AllySource::Companion(CompanionData {
        name: "Snel".into(),
        grade: 1,
        scent: None,
    });
    let tibb = w.familiar().expect("a familiar");
    let me = w.state.player().pos;
    for (id, d) in [(tibb, Dir::S), (snel, Dir::E)] {
        let at = me.step(d);
        if let Some(there) = w.state.actor_at(&w.state.current_level, at).map(|a| a.id) {
            w.state.actors.get_mut(&there).unwrap().pos = Pos::new(0, 0);
        }
        w.state.actors.get_mut(&id).unwrap().pos = at;
    }
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    assert_eq!(w.perception().kept_beside(), [Dir::E, Dir::S]);
    let slot = w.content.slot_id("neck").expect("the neck slot");
    let wearing_it = |w: &World, id| w.state.actors[&id].equipment.get(slot).is_some();
    let on_tibb = carry(&mut w, plain("jade_amulet"));
    w.apply(&Action::Give {
        item: on_tibb,
        ally: Some(Dir::S),
    });
    assert!(wearing_it(&w, tibb) && !wearing_it(&w, snel));
    let on_snel = carry(&mut w, plain("jade_amulet"));
    w.apply(&Action::Give {
        item: on_snel,
        ally: Some(Dir::E),
    });
    assert!(wearing_it(&w, snel), "{:?}", said(&w));
    assert!(said(&w).contains(&"You fasten the jade amulet about Snel's neck.".to_string()));
    // A way with no beast of theirs: refused, no time, the stack kept.
    let spare = carry(&mut w, plain("jade_amulet"));
    let (time, count) = (w.time(), w.state.player().inventory[spare].count);
    w.apply(&Action::Give {
        item: spare,
        ally: Some(Dir::NW),
    });
    assert_eq!(w.time(), time);
    assert_eq!(w.state.player().inventory[spare].count, count);
    assert!(said(&w).contains(&"No beast of yours stands that way.".to_string()));
}

#[test]
fn a_character_with_no_companion_is_refused_and_has_no_give() {
    let mut w = kitted("human,male,warrior");
    assert!(
        (w.perception().abilities().iter()).all(|a| a.ability != Ability::Give),
        "not listed"
    );
    let item = carry(&mut w, plain("jade_amulet"));
    let time = w.time();
    w.apply(&give(item));
    assert_eq!(w.time(), time);
    assert_eq!(w.state.player().inventory.len(), item + 1);
}

#[test]
fn its_magic_works_on_the_beast_and_identifies_nothing() {
    let mut w = ranger();
    let id = w.companion().expect("a companion");
    let cold = Some(BUILTIN.damage_type_id("cold").expect("cold"));
    assert_eq!(w.resistances_of(id).get(cold), 0);
    let item = carry(&mut w, warm());
    w.apply(&give(item));
    assert_eq!(w.resistances_of(id).get(cold), 30);
    let worn = &beast(&w).inventory[0];
    assert!(
        !w.lore().knows(worn),
        "wearing it on the beast teaches nothing"
    );
}

#[test]
fn on_its_death_the_collar_is_in_the_pile() {
    let mut w = ranger();
    let item = carry(&mut w, plain("jade_amulet"));
    w.apply(&give(item));
    let id = w.companion().expect("a companion");
    let at: Pos = beast(&w).pos;
    let player = w.player_id();
    assert!(w.hurt(id, 10_000, Harm::Actor(player), &mut Vec::new()));
    assert!(w.companion().is_none());
    let jade = BUILTIN.object_id("jade_amulet").expect("jade");
    assert!((w.state.current().objects_at(at).iter()).any(|s| s.object == jade));
}

#[test]
fn the_menu_and_the_dumps_show_it() {
    let mut w = ranger();
    let row = |w: &World| {
        (w.perception().abilities().into_iter())
            .find(|a| a.ability == Ability::Give)
            .expect("Give is listed")
    };
    assert_eq!(row(&w).unavailable, None);
    let layers = crate::dump::parse_layers("companion").unwrap();
    assert!(!crate::dump::ascii(&w, &layers).contains("wearing"));
    assert!(crate::dump::json(&w, &[], false)["companion"]["wears"].is_null());
    let item = carry(&mut w, plain("jade_amulet"));
    w.apply(&give(item));
    assert!(
        crate::dump::ascii(&w, &layers).contains("wearing a jade amulet"),
        "{}",
        crate::dump::ascii(&w, &layers)
    );
    assert_eq!(
        crate::dump::json(&w, &[], false)["companion"]["wears"],
        "a jade amulet"
    );
    let id = w.companion().expect("a companion");
    assert!(
        (w.perception().gear(id))
            .any(|g| g.stack.object == BUILTIN.object_id("jade_amulet").unwrap()),
        "look mode's gear sees it"
    );
    // Out of reach: listed, greyed out.
    let actor = w.state.actors.get_mut(&id).expect("the beast");
    actor.ally.as_mut().expect("an ally").retinue = true;
    run(&mut w, "wait");
    assert_eq!(row(&w).unavailable, Some(Unavailable::NotBeside));
}

#[test]
fn the_collar_survives_a_save() {
    let mut w = ranger();
    let item = carry(&mut w, plain("jade_amulet"));
    w.apply(&give(item));
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(collar(&loaded).as_deref(), Some("jade_amulet"));
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn give_replays_the_same_way() {
    let mut w = ranger();
    let mut replay = Replay::new(&w);
    // Refused without a neck's thing in the pack, and said the same way every time.
    for action in parse_actions("give:a wait give:a:E").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let replay = Replay::from_json(&replay.to_json()).expect("parses");
    let (_, diverged) = replay.play(BUILTIN.clone()).expect("plays");
    assert!(diverged.is_none());
}
