//! The on-kill affixes in the world (the affixes design, `docs/design/affixes-systems.md` §4,
//! §5, §8, decision 7; #632): of Siphoning's mana and of the Hunt's point of a pool on a kill by
//! the wearer's own blow, shot, or spell, the slain within three cells (`siphon.rs`), never on
//! a status tick's death or a trap's kill. No shipped file has these affixes yet, so the tests
//! bring their own.

use super::class_active_tests::beside;
use super::spell::Caster;
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::combat::Strike;
use crate::content::ContentFiles;
use crate::damage::Damage;
use crate::magic::{Affix, Magic, Rarity};
use crate::stats::{PerStat, Stats};

const AFFIXES: &str = r#"
[[suffix]]
id = "fixture_of_siphoning"
name = "of Siphoning"
effect = "mana_on_kill"
category = "mana_on_kill"
bases = ["weapon", "launcher", "amulet"]
tiers = [{ level = 1, min = 1, max = 3 }]

[[suffix]]
id = "fixture_of_the_hunt"
name = "of the Hunt"
effect = "resource_on_kill"
resource = "stamina"
category = "stamina_on_kill"
bases = ["weapon", "launcher", "amulet", "boots"]
tiers = [{ level = 1, min = 1, max = 2 }]
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_fixture.toml", AFFIXES);
    Arc::new(Content::load(&files).expect("fixture affixes are valid"))
}

/// A human ranger in the showcase (seed 7) with their kit (a short bow, arrows, stamina), born
/// under no sign, every score 14, with mana enough to show what a kill brings.
fn world() -> World {
    let content = content();
    let spec: CharacterSpec = "human,male,ranger".parse().expect("a character");
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
    w.player_mut().mana_base = 10;
    w
}

/// Actor `who` wears a jade amulet, with the affix `affix` at its value if any (a plain one
/// otherwise, so a twin weighs the same), put on at once.
fn amulet(w: &mut World, who: ActorId, affix: Option<(&str, i32)>) {
    let object = w.content().object_id("jade_amulet").expect("an amulet");
    let stack = match affix {
        Some((id, value)) => Stack::with_magic(
            object,
            1,
            Magic {
                rarity: Rarity::Magic,
                level: 1,
                affixes: vec![Affix {
                    id: id.into(),
                    tier: 1,
                    value,
                }],
                name: None,
                identified: true,
                carried: 0,
                unique: None,
            },
        ),
        None => Stack::new(object, 1),
    };
    let neck = w.content().slot_id("neck").expect("neck");
    let actor = w.state.actors.get_mut(&who).expect("an actor");
    actor.inventory.push(stack);
    let at = actor.inventory.len() - 1;
    actor.equipment.put(&[neck], at);
}

/// A kobold with one hit point `dx` cells east of the player.
fn foe(w: &mut World, dx: i32) -> ActorId {
    let id = beside(w, "kobold", 1);
    put(w, id, dx);
    id
}

fn put(w: &mut World, id: ActorId, dx: i32) {
    let here = w.player_pos();
    w.state.actors.get_mut(&id).expect("the foe").pos = Pos::new(here.x + dx, here.y);
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn stamina(w: &World) -> u16 {
    let id = w.content().resource_id("stamina").expect("stamina");
    (w.state.player().resources.get(id)).map_or(0, |p| p.value)
}

/// The player does `action` at `target`, kept `dx` cells east, until it dies, each try with no
/// mana, no stamina, and full hit points: the mana and stamina they hold after the killing one.
fn kill(w: &mut World, target: ActorId, dx: i32, action: &str) -> (i32, u16) {
    let stamina_id = w.content().resource_id("stamina").expect("stamina");
    let actions = parse_actions(action).expect("an action");
    for _ in 0..80 {
        put(w, target, dx);
        let max = w.state.player().max_hp(w.content());
        let me = w.player_mut();
        (me.hp, me.mana, me.mana_regen) = (max, 0, 0);
        me.resources.set(stamina_id, 0);
        for a in &actions {
            w.apply(a);
        }
        if !w.state.actors.contains_key(&target) {
            return (w.state.player().mana, stamina(w));
        }
    }
    panic!("{action} never killed it");
}

/// What a kill by `action` at `dx` brings with the amulet `affix` over the same kill with a
/// plain amulet: the twin draws the same, so the difference is the affix's alone.
fn gain(affix: (&str, i32), dx: i32, action: impl Fn(Pos) -> String) -> (i32, i32) {
    let mut out = Vec::new();
    for worn in [Some(affix), None] {
        let mut w = world();
        let player = w.player_id();
        amulet(&mut w, player, worn);
        let target = foe(&mut w, dx);
        let at = w.state.actors[&target].pos;
        out.push(kill(&mut w, target, dx, &action(at)));
    }
    let (with, without) = (out[0], out[1]);
    (with.0 - without.0, i32::from(with.1) - i32::from(without.1))
}

fn blow(_: Pos) -> String {
    "move:E".into()
}

fn shot(at: Pos) -> String {
    format!("fire:{},{}", at.x, at.y)
}

#[test]
fn a_blow_beside_siphons_mana_and_a_point_of_stamina() {
    assert_eq!(gain(("fixture_of_siphoning", 2), 1, blow), (2, 0));
    assert_eq!(gain(("fixture_of_the_hunt", 1), 1, blow), (0, 1));
}

#[test]
fn a_shot_counts_at_three_cells_and_not_at_four() {
    assert_eq!(gain(("fixture_of_siphoning", 3), 3, shot), (3, 0));
    assert_eq!(gain(("fixture_of_the_hunt", 2), 3, shot), (0, 2));
    assert_eq!(gain(("fixture_of_siphoning", 3), 4, shot), (0, 0));
    assert_eq!(gain(("fixture_of_the_hunt", 2), 4, shot), (0, 0));
}

/// The player's spell (no roll: a hit) does `amount` to a kobold `dx` east; the mana it left.
fn spell_kill(dx: i32) -> i32 {
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_siphoning", 2)));
    let target = foe(&mut w, dx);
    w.player_mut().mana = 0;
    let here = w.player_pos();
    let damage = (Damage::new(50, None), Strike::Hit { critical: false });
    w.spell_strikes(player, target, (here, true), damage, &mut Vec::new());
    assert!(!w.state.actors.contains_key(&target), "killed");
    w.state.player().mana
}

#[test]
fn a_spell_counts_at_three_cells_and_not_at_four() {
    assert_eq!(spell_kill(3), 2);
    assert_eq!(spell_kill(4), 0);
    // An area's blow reaches the same line.
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_siphoning", 2)));
    let target = foe(&mut w, 2);
    w.player_mut().mana = 0;
    let here = w.player_pos();
    let caster = Caster::Actor(player);
    w.spell_hurts(
        caster,
        target,
        (here, false),
        Damage::new(50, None),
        &mut Vec::new(),
    );
    assert!(!w.state.actors.contains_key(&target), "killed");
    assert_eq!(w.state.player().mana, 2);
}

#[test]
fn never_past_the_maximum() {
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_siphoning", 3)));
    let target = foe(&mut w, 1);
    let max = w.state.player().max_mana(w.content());
    assert!(max > 3, "room to show it: {max}");
    w.player_mut().mana = max - 1;
    w.siphon(player, (target, w.state.actors[&target].pos));
    assert_eq!(w.state.player().mana, max);
    w.siphon(player, (target, w.state.actors[&target].pos));
    assert_eq!(w.state.player().mana, max);
    // Nor a pool's.
    let mut w = world();
    amulet(&mut w, player, Some(("fixture_of_the_hunt", 2)));
    let id = w.content().resource_id("stamina").expect("stamina");
    let level = w.player_level();
    let full = (w.state.player()).resource_max(w.content(), id, level);
    w.player_mut().resources.set(id, full - 1);
    let target = foe(&mut w, 1);
    w.siphon(player, (target, w.state.actors[&target].pos));
    assert_eq!(stamina(&w), full);
}

#[test]
fn a_poison_death_brings_nothing() {
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_siphoning", 3)));
    let target = foe(&mut w, 2);
    let poisoned = w.content().status_id("poisoned").expect("poisoned");
    w.afflict(target, poisoned, 6, Some(player), &mut Vec::new());
    for _ in 0..300 {
        if !w.state.actors.contains_key(&target) {
            break;
        }
        put(&mut w, target, 2);
        let max = w.state.player().max_hp(w.content());
        let me = w.player_mut();
        (me.hp, me.mana, me.mana_regen) = (max, 0, 0);
        run(&mut w, "wait");
    }
    assert!(
        !w.state.actors.contains_key(&target),
        "the poison killed it"
    );
    assert_eq!(
        w.state.player().mana,
        0,
        "a status tick's death is not a kill of one's own"
    );
}

#[test]
fn a_traps_kill_brings_nothing() {
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_siphoning", 3)));
    let target = foe(&mut w, 1);
    w.player_mut().mana = 0;
    let trap = w.content().trap_id("arrow").expect("a trap");
    let at = w.state.actors[&target].pos;
    w.trap_hurts(
        trap,
        target,
        (at, false),
        Damage::new(50, None),
        &mut Vec::new(),
    );
    assert!(!w.state.actors.contains_key(&target), "the trap killed it");
    assert_eq!(w.state.player().mana, 0);
}

#[test]
fn a_creatures_kill_siphons_its_own_mana() {
    let mut w = world();
    let kobold = foe(&mut w, 1);
    amulet(&mut w, kobold, Some(("fixture_of_siphoning", 3)));
    let me = w.state.actors.get_mut(&kobold).expect("the kobold");
    (me.mana_base, me.hp, me.hp_base) = (10, 100, 100);
    let max = w.state.actors[&kobold].max_mana(w.content());
    assert!(max > 3, "room to show it: {max}");
    for _ in 0..200 {
        if w.state.death.is_some() {
            break;
        }
        let me = w.state.actors.get_mut(&kobold).expect("the kobold");
        (me.mana, me.mana_regen) = (0, 0);
        w.player_mut().hp = 1;
        run(&mut w, "wait");
    }
    assert!(w.state.death.is_some(), "the kobold killed the player");
    assert_eq!(w.state.actors[&kobold].mana, 3);
}

#[test]
fn one_without_the_pool_or_the_gear_gets_nothing() {
    // No stamina held: of the Hunt never grants it.
    let mut w = world();
    let player = w.player_id();
    amulet(&mut w, player, Some(("fixture_of_the_hunt", 2)));
    let id = w.content().resource_id("stamina").expect("stamina");
    w.player_mut().resources = Default::default();
    let target = foe(&mut w, 1);
    w.siphon(player, (target, w.state.actors[&target].pos));
    assert!(w.state.player().resources.get(id).is_none());
    // Nothing worn: the kill changes nothing at all.
    let mut w = world();
    w.player_mut().mana = 0;
    let target = foe(&mut w, 1);
    let before = w.state_hash();
    w.siphon(player, (target, w.state.actors[&target].pos));
    assert_eq!(w.state_hash(), before);
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}
