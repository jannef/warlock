//! Miscast backfires (`backfire.rs`; docs/design/miscasts-review.md §5.3, #1291): the rebound
//! heal on the nearest hostile, the buff on it, the hostile summon, and the fallback harm.

use super::backfire::fallback_dice;
use super::cast::Cast;
use super::caster_tests::{kobold, world_with};
use super::miscast::Aim;
use super::*;
use crate::content::{Miscast, Stance};
use crate::test_support::{builtin, said};

/// The built-in entry `id`.
fn entry(c: &Content, id: &str) -> Miscast {
    (c.miscasts().iter())
        .find(|m| m.id == id)
        .cloned()
        .expect("a built-in entry")
}

/// A wizard on the showcase with every other actor sent far off, at full hit points.
fn alone() -> World {
    let c = builtin();
    let mut w = world_with(c, "wizard");
    let player = w.state.player;
    for a in w.state.actors.values_mut() {
        if a.id != player {
            a.pos = Pos::new(-100, -100);
        }
    }
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w
}

/// A hurt kobold `dx` east of the player, at 1 of its 50 hit points.
fn hurt_kobold(w: &mut World, dx: i32) -> ActorId {
    let k = kobold(w, dx, 50);
    w.state.actors.get_mut(&k).expect("the kobold").hp = 1;
    k
}

/// The player's `spell` backfires by entry `id` (`backlash` or `rebound`); the lines said.
fn backfire(w: &mut World, spell: &str, id: &str) -> Vec<String> {
    let c = w.content().clone();
    let spell = c.spell_id(spell).expect("the spell");
    let me = w.state.player().clone();
    let cast = Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let before = said(w).len();
    let player = w.state.player;
    w.miscast(
        player,
        &cast,
        Aim::Player(None),
        &entry(&c, id),
        &mut Vec::new(),
    );
    said(w)[before..].to_vec()
}

#[test]
fn the_fallback_harm_grows_with_the_spell_level() {
    // The design's examples: Cure wounds 3, Haste 11, Benison 12.
    assert_eq!((fallback_dice(3, false), fallback_dice(3, true)), (2, 2));
    assert_eq!((fallback_dice(11, false), fallback_dice(11, true)), (4, 6));
    assert_eq!((fallback_dice(12, false), fallback_dice(12, true)), (5, 7));
    assert_eq!((fallback_dice(0, false), fallback_dice(0, true)), (1, 1));
}

#[test]
fn a_heal_rebounds_on_the_nearest_hostile() {
    let mut w = alone();
    let far = hurt_kobold(&mut w, 4);
    let near = hurt_kobold(&mut w, 2);
    w.player_mut().hp -= 3;
    let hp = w.state.player().hp;
    let lines = backfire(&mut w, "cure_wounds", "backlash");
    assert!(w.state.actors[&near].hp > 1, "the nearer kobold is mended");
    assert_eq!(w.state.actors[&far].hp, 1, "the farther one is not");
    assert_eq!(
        w.state.player().hp,
        hp,
        "the caster is neither mended nor harmed"
    );
    assert!(
        lines.contains(&"Your healing flows into the kobold.".to_string()),
        "{lines:?}"
    );
}

#[test]
fn a_heal_with_no_hostile_in_sight_harms_the_caster() {
    let mut w = alone();
    let max = w.state.player().hp;
    backfire(&mut w, "cure_wounds", "rebound");
    // 2d6 force: 2 to 12 off, none resisted by a human.
    assert!((max - 12..=max - 2).contains(&w.state.player().hp));
}

#[test]
fn a_buff_takes_hold_of_the_nearest_hostile() {
    let mut w = alone();
    let k = kobold(&mut w, 2, 50);
    let hasted = w.content().status_id("hasted").expect("hasted");
    let lines = backfire(&mut w, "haste", "backlash");
    assert!(w.state.actors[&k].effects.has(hasted), "{lines:?}");
    assert!(!w.state.player().effects.has(hasted));
    assert!(lines.contains(&"Your spell takes hold of the kobold instead.".to_string()));
}

/// Two kobolds called, three at once.
const KOBOLDS: &str = r#"
[[spell]]
id = "zz_call_kobolds"
name = "Call kobolds"
description = "x"
level = 1
school = "enchantment"
kind = "summon"
shape = "self"
resolution = "auto"
effect = { summon = { creatures = ["kobold"], count = "2", most = 3, minutes = 5 } }
"#;

/// The actors `w` holds that are not in `before`, in id order.
fn new_actors(w: &World, before: &[ActorId]) -> Vec<ActorId> {
    (w.state.actors.keys())
        .filter(|id| !before.contains(id))
        .copied()
        .collect()
}

#[test]
fn a_summon_comes_hostile() {
    let mut files = crate::content::ContentFiles::builtin();
    files.insert("data/spells/zz_call_kobolds.toml", KOBOLDS);
    let c = Arc::new(Content::load(&files).expect("the content loads"));
    let mut w = world_with(c, "wizard");
    let player = w.state.player;
    let before: Vec<ActorId> = w.state.actors.keys().copied().collect();
    let lines = backfire(&mut w, "zz_call_kobolds", "backlash");
    let came = new_actors(&w, &before);
    assert_eq!(came.len(), 2, "{lines:?}");
    for &id in &came {
        let it = &w.state.actors[&id];
        assert_eq!(w.content().creature(it.creature).id, "kobold");
        assert!(it.ally.is_none() && it.summoner.is_none());
        // Berserk (#1311): hostile to the caster, to a kobold of its own kind, and to the other.
        assert_eq!(
            w.content()
                .factions()
                .def(it.faction_override.expect("a faction"))
                .id,
            "berserk"
        );
        assert_eq!(w.stance(id, player), Stance::Hostile);
    }
    assert_eq!(w.stance(came[0], came[1]), Stance::Hostile);
    let kin = kobold(&mut w, 3, 50);
    assert_eq!(w.stance(came[0], kin), Stance::Hostile);
    assert!(
        lines.contains(&"Two kobolds come to your call, but not to your side!".to_string()),
        "{lines:?}"
    );
    assert!(w.allies_of(player).is_empty());

    // A backfired wolf is no wildlife: berserk, hostile to the player (#1311), no ally of anyone's.
    let mut w = alone();
    let before: Vec<ActorId> = w.state.actors.keys().copied().collect();
    backfire(&mut w, "call_of_the_wild", "rebound");
    let came = new_actors(&w, &before);
    assert_eq!(came.len(), 1);
    let wolf = &w.state.actors[&came[0]];
    assert_eq!(w.content().creature(wolf.creature).id, "wolf");
    assert!(wolf.ally.is_none());
    let wolf = wolf.id;
    assert_eq!(w.stance(wolf, w.state.player), Stance::Hostile);
    assert!(w.allies_of(w.state.player).is_empty());
}

#[test]
fn a_cleanse_falls_back_on_harm_and_a_damage_spell_on_its_burst() {
    let mut w = alone();
    let max = w.state.player().hp;
    // Remove disease, level 11: 4d6 force at base.
    backfire(&mut w, "remove_disease", "backlash");
    assert!((max - 24..=max - 4).contains(&w.state.player().hp));
    let mut w = alone();
    let max = w.state.player().hp;
    backfire(&mut w, "spark", "backlash");
    assert!(w.state.player().hp < max, "the spark bursts on its caster");
}

#[test]
fn a_creatures_heal_rebounds_on_the_player() {
    let mut w = alone();
    let shaman = kobold(&mut w, 2, 50);
    w.player_mut().hp = 1;
    let c = w.content().clone();
    let spell = c.spell_id("cure_wounds").expect("cure wounds");
    let me = w.state.actors[&shaman].clone();
    let cast = Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let player = w.state.player;
    w.miscast(
        shaman,
        &cast,
        Aim::Creature(player),
        &entry(&c, "backlash"),
        &mut Vec::new(),
    );
    assert!(w.state.player().hp > 1, "{:?}", said(&w));
    assert_eq!(w.state.actors[&shaman].hp, 50);
}
