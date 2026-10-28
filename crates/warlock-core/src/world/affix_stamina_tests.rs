//! The stamina and resource affixes in the world (the affixes design,
//! `docs/design/affixes-systems.md` §5; #630): what is worn raises a pool's maximum
//! (`resource_max`, read in `Actor::resource_max`), speeds its pace (`resource_rate`,
//! `World::resource_rate`), and takes stamina off every weapon active (`active_stamina`,
//! `Actor::weapon_stamina`), keyed by the pool the affix names; taking it off lowers what is held
//! at once (`World::keep_resources_within_max`). No shipped file has these affixes yet, so the
//! tests bring their own.

use super::class_active_tests::beside;
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, ResourceId};
use crate::dump::{self, Layer};
use crate::magic::{Affix, Magic, Rarity};
use crate::stats::{PerStat, Stats};

const AFFIXES: &str = r#"
[[prefix]]
id = "fixture_tireless"
name = "Tireless"
effect = "resource_max"
resource = "stamina"
category = "stamina"
bases = ["ring", "body"]
tiers = [{ level = 1, min = 1, max = 5 }]

[[suffix]]
id = "fixture_of_breath"
name = "of Breath"
effect = "resource_rate"
resource = "stamina"
category = "stamina_pace"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 10, max = 60 }]

[[suffix]]
id = "fixture_of_ease"
name = "of Ease"
effect = "active_stamina"
resource = "stamina"
category = "stamina_cost"
bases = ["ring", "gloves"]
tiers = [{ level = 1, min = 1, max = 2 }]

[[suffix]]
id = "fixture_of_fury"
name = "of Fury"
effect = "resource_max"
resource = "rage"
category = "rage"
bases = ["ring"]
tiers = [{ level = 1, min = 2, max = 8 }]
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_fixture.toml", AFFIXES);
    Arc::new(Content::load(&files).expect("fixture affixes are valid"))
}

/// A human of `class` in the showcase (seed 7) with their kit, born under no sign, every score
/// 14.
fn world(class: &str) -> World {
    let content = content();
    let spec: CharacterSpec = format!("human,male,{class}").parse().expect("a character");
    let spec = spec.with_talents(vec![
        "keen_ears".into(),
        "lean".into(),
        "long_strider".into(),
    ]);
    let w = World::start_run(
        content.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    let mut state = w.state().clone();
    state.character.as_mut().expect("a character").sign = None;
    let mut w = World::from_state(content, state).expect("restores");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

/// The showcase's own player, its talents taken away: no pool until one is granted.
fn plain_world() -> World {
    let mut w = World::new(content(), "showcase", Some(7)).expect("world");
    w.player_mut().talents = Default::default();
    w
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn pool(w: &World, id: &str) -> ResourceId {
    w.content().resource_id(id).expect(id)
}

fn held(w: &World, id: &str) -> Option<u16> {
    (w.state.player().resources.get(pool(w, id))).map(|p| p.value)
}

fn max(w: &World, id: &str) -> u16 {
    let level = w.player_level();
    (w.state.player()).resource_max(w.content(), pool(w, id), level)
}

fn set(w: &mut World, id: &str, value: u16) {
    let id = pool(w, id);
    w.player_mut().resources.set(id, value);
}

/// One `object` with the affix `id` at `value`, as loot would make it, identified or not.
fn magic(w: &World, object: &str, id: &str, value: i32, identified: bool) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified,
        carried: 0,
        unique: None,
    };
    let object = w.content().object_id(object).expect("object");
    Stack::with_magic(object, 1, magic)
}

/// The player is handed `stack` and puts it on in `slot` (the equip action, taking its time).
fn wear(w: &mut World, slot: &str, stack: Stack) {
    let at = w.state.player().inventory.len();
    w.player_mut().inventory.push(stack);
    run(w, &format!("equip:{slot}:{at}"));
    let slot = w.content().slot_id(slot).expect("slot");
    assert!(w.state.player().equipment.get(slot).is_some(), "worn");
}

fn lunge(w: &World) -> u16 {
    let id = w.content().weapon_ability_id("lunge").expect("lunge");
    w.state.player().weapon_stamina(w.content(), id)
}

#[test]
fn tireless_raises_the_pool_and_putting_it_on_fills_nothing() {
    let mut w = world("warrior");
    let (before, full) = (max(&w, "stamina"), held(&w, "stamina"));
    let ring = magic(&w, "iron_ring", "fixture_tireless", 3, true);
    wear(&mut w, "left_ring", ring);
    assert_eq!(max(&w, "stamina"), before + 3);
    assert_eq!(held(&w, "stamina"), full, "nothing more held yet");
    // Two pieces add.
    let mail = magic(&w, "chain_mail", "fixture_tireless", 2, true);
    let at = w.state.player().inventory.len();
    w.player_mut().inventory.push(mail);
    let body = w.content().slot_id("body").expect("body");
    w.player_mut().equipment.put(&[body], at);
    assert_eq!(max(&w, "stamina"), before + 5);
    // The perception and the dump show the larger pool.
    let header = dump::ascii(&w, &[]);
    let shown = format!("stamina {}/{}", full.unwrap(), before + 5);
    assert!(header.contains(&shown), "{header}");
}

#[test]
fn an_unidentified_piece_works_all_the_same() {
    let mut w = world("warrior");
    let before = max(&w, "stamina");
    let ring = magic(&w, "iron_ring", "fixture_tireless", 4, false);
    wear(&mut w, "left_ring", ring);
    assert_eq!(max(&w, "stamina"), before + 4);
}

#[test]
fn taking_it_off_lowers_what_is_held_at_once() {
    let mut w = world("warrior");
    let before = max(&w, "stamina");
    let ring = magic(&w, "iron_ring", "fixture_tireless", 3, true);
    wear(&mut w, "left_ring", ring);
    set(&mut w, "stamina", before + 3);
    run(&mut w, "remove:left_ring");
    assert_eq!(max(&w, "stamina"), before);
    assert_eq!(
        held(&w, "stamina"),
        Some(before),
        "down to the maximum without it"
    );
    // The clamp itself, with no time passing: the ring slipped off by hand.
    let ring = magic(&w, "iron_ring", "fixture_tireless", 3, true);
    wear(&mut w, "left_ring", ring);
    set(&mut w, "stamina", before + 3);
    let slot = w.content().slot_id("left_ring").expect("slot");
    let item = w.state.player().equipment.get(slot).expect("worn");
    w.player_mut().equipment.take_off(item);
    let player = w.player_id();
    w.keep_resources_within_max(player);
    assert_eq!(held(&w, "stamina"), Some(before));
}

#[test]
fn of_breath_speeds_the_pace_beside_the_talents() {
    let mut w = world("warrior");
    let player = w.player_id();
    let stamina = pool(&w, "stamina");
    assert_eq!(w.resource_rate(player, stamina), 100);
    let amulet = magic(&w, "jade_amulet", "fixture_of_breath", 30, true);
    wear(&mut w, "neck", amulet);
    assert_eq!(w.resource_rate(player, stamina), 130);
    w.player_mut().talents.take("steady_breathing", None);
    assert_eq!(
        w.resource_rate(player, stamina),
        180,
        "Steady breathing's 50 beside"
    );
    // Another pool's pace is untouched.
    let rage = pool(&w, "rage");
    w.player_mut().talents.forget("steady_breathing");
    assert_eq!(w.resource_rate(player, rage), 100);
    run(&mut w, "remove:neck");
    assert_eq!(w.resource_rate(player, stamina), 100);
}

#[test]
fn of_ease_cuts_every_active_down_to_one() {
    let mut w = world("warrior");
    assert_eq!(lunge(&w), 3, "a lunge's own");
    let gloves = magic(&w, "leather_gloves", "fixture_of_ease", 1, true);
    wear(&mut w, "hands", gloves);
    assert_eq!(lunge(&w), 2);
    w.player_mut().talents.take("drillmaster", None);
    assert_eq!(lunge(&w), 1, "Drillmaster's 1 beside");
    let ring = magic(&w, "iron_ring", "fixture_of_ease", 2, true);
    wear(&mut w, "left_ring", ring);
    assert_eq!(lunge(&w), 1, "never below 1");
}

#[test]
fn a_wearer_without_talents_gets_the_affixes_too() {
    let mut w = plain_world();
    run(&mut w, "resource:stamina");
    let player = w.player_id();
    let stamina = pool(&w, "stamina");
    let before = max(&w, "stamina");
    assert_eq!(lunge(&w), 3);
    for (slot, id, value) in [
        ("left_ring", "fixture_tireless", 2),
        ("right_ring", "fixture_of_ease", 1),
        ("neck", "fixture_of_breath", 50),
    ] {
        let object = if slot == "neck" {
            "jade_amulet"
        } else {
            "iron_ring"
        };
        let stack = magic(&w, object, id, value, true);
        let at = w.state.player().inventory.len();
        w.player_mut().inventory.push(stack);
        let slot = w.content().slot_id(slot).expect("slot");
        w.player_mut().equipment.put(&[slot], at);
    }
    assert!(w.state.player().talents.is_empty());
    assert_eq!(max(&w, "stamina"), before + 2);
    assert_eq!(lunge(&w), 2);
    assert_eq!(w.resource_rate(player, stamina), 150);
}

#[test]
fn one_without_the_pool_gets_nothing() {
    let mut w = world("wizard");
    assert_eq!(held(&w, "stamina"), None);
    let ring = magic(&w, "iron_ring", "fixture_tireless", 5, true);
    wear(&mut w, "left_ring", ring);
    run(&mut w, "wait*20");
    assert_eq!(held(&w, "stamina"), None, "an affix never grants the pool");
    let player = w.player_id();
    assert!(w.resource_paces(player).is_empty());
}

#[test]
fn of_fury_raises_a_barbarians_rage_and_does_nothing_for_a_wizard() {
    let mut w = world("barbarian");
    let (before, stamina) = (max(&w, "rage"), max(&w, "stamina"));
    let ring = magic(&w, "iron_ring", "fixture_of_fury", 4, true);
    wear(&mut w, "left_ring", ring);
    assert_eq!(max(&w, "rage"), before + 4);
    assert_eq!(max(&w, "stamina"), stamina, "only the pool it names");
    run(&mut w, "use:rage");
    assert!(held(&w, "rage").is_some(), "raging");
    set(&mut w, "rage", before + 4);
    let slot = w.content().slot_id("left_ring").expect("slot");
    let item = w.state.player().equipment.get(slot).expect("worn");
    w.player_mut().equipment.take_off(item);
    let player = w.player_id();
    w.keep_resources_within_max(player);
    assert_eq!(held(&w, "rage"), Some(before), "down with the ring off");

    let mut w = world("wizard");
    let ring = magic(&w, "iron_ring", "fixture_of_fury", 4, true);
    wear(&mut w, "left_ring", ring);
    run(&mut w, "wait*5");
    assert_eq!(held(&w, "rage"), None, "dead loot on a wizard");
}

#[test]
fn a_creatures_gear_does_nothing_for_it() {
    let mut w = world("warrior");
    let kobold = beside(&mut w, "kobold", 50);
    let ring = magic(&w, "iron_ring", "fixture_tireless", 5, true);
    let slot = w.content().slot_id("left_ring").expect("slot");
    let me = w.state.actors.get_mut(&kobold).expect("kobold");
    me.inventory.push(ring);
    me.equipment.put(&[slot], 0);
    run(&mut w, "wait*3");
    let me = w.state.actors.get(&kobold).expect("kobold");
    assert!(me.resources.is_empty(), "creatures hold no pools");
    assert!(w.resource_paces(kobold).is_empty());
    let actors = dump::layer_rows(&w, Layer::Actors).join("\n");
    assert!(actors.contains("Tireless"), "the gear as it is: {actors}");
}
