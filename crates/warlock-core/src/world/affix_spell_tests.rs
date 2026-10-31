//! The spellcasting affixes' hooks (docs/design/affixes-systems.md §4, #629), each read where
//! the matching talent's is: of Thrift's `mana_cost` in [`Actor::mana_cost`], Mage-wrought's
//! `spell_ease` in [`Actor::worn_spell_cost`], of the Magus's `spell_to_hit`
//! ([`Actor::spell_to_hit`]), Potent's `spell_bonus`, of Lingering's `spell_duration`, and of
//! Farcasting's `spell_reach` through the spell hooks ([`Actor::spell_mods`]), and Baleful's
//! `spell_threat` in [`Actor::spell_threat`]. The tests bring fixture affixes with ranges of
//! their own, as `world/unique_tests.rs` brings its own unique.

use super::cast_tests::{cast_at, scores};
use super::caster_tests::{kobold, reach, run, sp, world_with};
use super::*;
use crate::combat::{DEFAULT_THREAT, Strike, THREATS};
use crate::content::{ContentFiles, Grip};
use crate::magic::{Affix, Magic, Rarity};
use crate::stats::Stat;

/// Fixture affixes, one per spellcasting effect, with ranges wide enough for every test.
const AFFIXES: &str = r#"
[[prefix]]
id = "fixture_mage_wrought"
name = "Mage-wrought"
effect = "spell_ease"
category = "spell_armour"
bases = ["body", "helmet", "shield"]
tiers = [{ level = 1, min = 1, max = 40 }]

[[prefix]]
id = "fixture_baleful"
name = "Baleful"
effect = "spell_threat"
category = "spell_critical"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 1, max = 3 }]

[[prefix]]
id = "fixture_potent"
name = "Potent"
effect = "spell_bonus"
category = "spell_power"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 1, max = 10 }]

[[suffix]]
id = "fixture_of_thrift"
name = "of Thrift"
effect = "mana_cost"
category = "mana_cost"
bases = ["ring", "amulet", "helmet"]
tiers = [{ level = 1, min = 1, max = 50 }]

[[suffix]]
id = "fixture_of_the_magus"
name = "of the Magus"
effect = "spell_to_hit"
category = "spell_to_hit"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 1, max = 10 }]

[[suffix]]
id = "fixture_of_farcasting"
name = "of Farcasting"
effect = "spell_reach"
category = "spell_reach"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 1, max = 5 }]

[[suffix]]
id = "fixture_of_lingering"
name = "of Lingering"
effect = "spell_duration"
category = "spell_duration"
bases = ["ring", "amulet"]
tiers = [{ level = 1, min = 1, max = 5 }]
"#;

/// An attack ray of level 6, 3d6, a die more at Intelligence 13 and two at 16: the wizard's
/// Arcane bolt, its file replaced.
const RAY: &str = "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
                   level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n\
                   shape = \"ray\"\nresolution = \"attack\"\nrange = 6\n\
                   damage = \"3d6\"\n[spell.grows]\n\
                   damage = \"intelligence 13 4d6, 16 5d6\"\n";

/// The built-in content with the fixture affixes, and Arcane bolt as [`RAY`] when `ray`.
fn content(ray: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("data/affixes/zz_spell_fixture.toml", AFFIXES);
    if ray {
        files.insert("data/spells/arcane_bolt.toml", RAY);
    }
    Arc::new(Content::load(&files).expect("the fixture affixes load"))
}

/// A level-18 wizard with `content`.
fn wizard(content: Arc<Content>) -> World {
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    w
}

/// One `object` with the affix `id` at `value`, unidentified (affixes work all the same).
fn item(w: &World, object: &str, id: &str, value: i32) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = w.content().object_id(object).expect("object");
    Stack::with_magic(object, 1, magic)
}

/// Actor `id` carries `stack` and wears it in `slot`; where it lies.
fn wear(w: &mut World, id: ActorId, slot: &str, stack: Stack) -> usize {
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    actor.equipment.put(&taken, item);
    item
}

/// The player wears a ring with the affix `id` at `value` on the left hand.
fn ring(w: &mut World, id: &str, value: i32) {
    let stack = item(w, "iron_ring", id, value);
    let player = w.player_id();
    wear(w, player, "left_ring", stack);
}

fn cost(w: &World, base: i32) -> i32 {
    w.state.player().mana_cost(w.content(), base)
}

#[test]
fn of_thrift_cuts_every_mana_cost_until_taken_off() {
    let mut w = wizard(content(false));
    let content = w.content().clone();
    let bolt = content.class_spell(sp("arcane_bolt")).expect("the bolt");
    assert_eq!((cost(&w, 10), cost(&w, 1), cost(&w, 0)), (10, 1, 0));
    assert_eq!(w.state.player().spell_cost(&content, bolt), 11);
    ring(&mut w, "fixture_of_thrift", 20);
    assert!(!w.state.player().inventory.last().unwrap().is_identified());
    assert_eq!(cost(&w, 10), 8, "a fifth less, unidentified as it is");
    assert_eq!(cost(&w, 1), 1, "never below 1");
    assert_eq!(cost(&w, 0), 0);
    // 11 × 0.8 = 8.8: the spell's price, the menu's too.
    assert_eq!(w.state.player().spell_cost(&content, bolt), 9);
    assert_eq!(w.perception().own_spell_cost(sp("arcane_bolt")), Some(9));
    // The dev drain spends through the same hook.
    w.player_mut().mana = 40;
    run(&mut w, "drain:10");
    assert_eq!(w.state.player().mana, 32);
    run(&mut w, "remove:left_ring");
    assert_eq!(cost(&w, 10), 10, "taken off, the cut is gone");
}

#[test]
fn of_thrift_joins_frugal_caster_before_the_rounding_and_never_past_half() {
    let mut w = wizard(content(false));
    w.player_mut().talents.take("frugal_caster", None);
    // Frugal caster alone: 7 × 0.8 = 5.6, 6.
    assert_eq!(cost(&w, 7), 6);
    // With 10 % more off, summed before the one rounding: 7 × 0.7 = 4.9, 5.
    ring(&mut w, "fixture_of_thrift", 10);
    assert_eq!(cost(&w, 7), 5);
    // Three pieces of 20 % and the talent's 20: 80 % off, held to 50.
    let player = w.player_id();
    let right = item(&w, "iron_ring", "fixture_of_thrift", 20);
    wear(&mut w, player, "right_ring", right);
    let amulet = item(&w, "jade_amulet", "fixture_of_thrift", 20);
    wear(&mut w, player, "neck", amulet);
    let worn = w
        .state
        .player()
        .mods(w.content())
        .get(crate::content::Effect::ManaCost);
    assert_eq!(worn, 50);
    assert_eq!((cost(&w, 10), cost(&w, 7), cost(&w, 1)), (5, 4, 1));
}

#[test]
fn mage_wrought_eases_its_own_piece_and_a_fully_eased_one_leaves_the_armoured_grip() {
    let mut w = wizard(content(false));
    let content = w.content().clone();
    let player = w.player_id();
    let grip = |w: &World| {
        let ray = content.spell(content.spell_id("ray_of_frost").expect("ray of frost"));
        let style = w.state.player().spell_skills(&content, ray).style;
        let table = content.weapon_skills();
        if style == table.style_for(Grip::ArmouredSpell, &[]) {
            "armoured"
        } else {
            assert_eq!(style, table.style_for(Grip::RangedSpell, &[]));
            "ranged"
        }
    };
    assert_eq!(grip(&w), "ranged");
    // Plate, 40 %, eased 15 points: 25.
    let plate = item(&w, "plate_armour", "fixture_mage_wrought", 15);
    let body = wear(&mut w, player, "body", plate);
    assert_eq!(w.state.player().worn_spell_cost(&content), 25);
    assert_eq!(grip(&w), "armoured");
    // A great helm (10 %) beside it, unhelped: its own 10, the plate's ease never spills.
    let helm = content.object_id("great_helm").expect("helm");
    wear(&mut w, player, "head", Stack::new(helm, 1));
    assert_eq!(w.state.player().worn_spell_cost(&content), 35);
    run(&mut w, "remove:head");
    // Chain mail (10 %) eased by 12: never below 0, and no longer armour for the grip.
    run(&mut w, "remove:body");
    let worn = w.state.player().equipment.items();
    assert!(worn.into_iter().all(|(_, i)| i != body), "the plate is off");
    let chain = item(&w, "chain_mail", "fixture_mage_wrought", 12);
    wear(&mut w, player, "body", chain);
    assert_eq!(w.state.player().worn_spell_cost(&content), 0);
    assert_eq!(w.state.player().armour_spell_cost(&content), 0);
    assert_eq!(grip(&w), "ranged", "nothing left to ease");
    // And back: taken off and plain chain mail put on, the grip is armoured again.
    run(&mut w, "remove:body");
    let plain = content.object_id("chain_mail").expect("chain");
    wear(&mut w, player, "body", Stack::new(plain, 1));
    assert_eq!(w.state.player().worn_spell_cost(&content), 10);
    assert_eq!(grip(&w), "armoured");
}

#[test]
fn the_magus_potent_and_lingering_join_the_hooks() {
    let mut w = wizard(content(true));
    let content = w.content().clone();
    let bolt = content.spell(sp("arcane_bolt"));
    let evil_eye = content.spell(sp("evil_eye"));
    scores(&mut w, &[(Stat::Dexterity, 16)]);
    let read = |w: &World| {
        let me = w.state.player();
        let roll = me.spell_roll(&content, bolt, CastBy::Own).expect("a roll");
        (
            me.spell_to_hit(&content, bolt),
            roll.to_string(),
            me.spell_degrees(&content, evil_eye),
        )
    };
    assert_eq!(read(&w), (3, "3d6".into(), Some(1)));
    ring(&mut w, "fixture_of_the_magus", 2);
    let player = w.player_id();
    let amulet = item(&w, "jade_amulet", "fixture_of_lingering", 2);
    wear(&mut w, player, "neck", amulet);
    assert_eq!(
        read(&w),
        (5, "3d6".into(), Some(3)),
        "unidentified as they are"
    );
    // Potent's +2 on the roll after the ladder's rung: Intelligence 16 reaches 5d6.
    let potent = item(&w, "iron_ring", "fixture_potent", 2);
    wear(&mut w, player, "right_ring", potent);
    assert_eq!(read(&w).1, "3d6+2");
    scores(&mut w, &[(Stat::Intelligence, 16), (Stat::Dexterity, 16)]);
    assert_eq!(read(&w).1, "5d6+2");
    // Keen mind's bigger dice on top, for its school only.
    w.player_mut().talents.take("keen_mind", Some("evocation"));
    assert_eq!(read(&w).1, "5d7+2");
    run(&mut w, "remove:left_ring remove:right_ring remove:neck");
    scores(&mut w, &[(Stat::Dexterity, 16)]);
    assert_eq!(
        read(&w),
        (3, "3d7".into(), Some(1)),
        "taken off: Keen mind's size stays"
    );
}

/// A save from before the spell scaling design (§7.3) holds a Potent of 4 to 15 at its old
/// tiers; the lenient read puts it in the tier that holds it or the nearest and holds its value
/// there: an old 13 is a +3.
#[test]
fn an_old_potent_reads_as_the_top_tiers_bonus() {
    let w = wizard(Arc::new(Content::builtin().expect("content")));
    let content = w.content().clone();
    let object = content.object_id("iron_ring").expect("ring");
    for (tier, value, now) in [
        (6, 13, (3, 3)),
        (1, 4, (3, 3)),
        (1, 1, (1, 1)),
        (2, 2, (2, 2)),
    ] {
        let mut magic = Magic {
            rarity: Rarity::Magic,
            level: 3,
            affixes: vec![Affix {
                id: "potent".into(),
                tier,
                value,
            }],
            name: None,
            identified: false,
            carried: 0,
            unique: None,
        };
        magic.retier(&content);
        let a = &magic.affixes[0];
        assert_eq!((a.tier, a.value), now, "tier {tier} value {value}");
        let stack = Stack::with_magic(object, 1, magic);
        assert!(stack.is_valid_for(&content), "tier {tier} value {value}");
    }
}

#[test]
fn of_farcasting_reaches_farther_on_the_range_never_the_radius() {
    let mut w = wizard(content(false));
    let content = w.content().clone();
    scores(&mut w, &[]);
    let reach = |w: &World, id: &str| {
        w.state
            .player()
            .spell_reach(&content, content.spell(sp(id)))
    };
    // The shipped Arcane bolt: length 6 at Willpower 10.
    let bolt = reach(&w, "arcane_bolt").range;
    let blight = reach(&w, "blight");
    assert_eq!(bolt, Some(6));
    ring(&mut w, "fixture_of_farcasting", 1);
    assert_eq!(reach(&w, "arcane_bolt").range, Some(7));
    let farther = reach(&w, "blight");
    assert_eq!(farther.range, blight.range.map(|r| r + 1));
    assert_eq!(farther.radius, blight.radius, "never the radius");
    run(&mut w, "remove:left_ring");
    ring(&mut w, "fixture_of_farcasting", 5);
    assert_eq!(
        reach(&w, "arcane_bolt").range,
        Some(11),
        "past the ladder's last rung: only the engine's reach holds it"
    );
}

#[test]
fn baleful_widens_an_own_spells_threat_down_to_the_floor() {
    let mut w = wizard(content(true));
    let content = w.content().clone();
    assert_eq!(w.state.player().spell_threat(&content), DEFAULT_THREAT);
    ring(&mut w, "fixture_baleful", 2);
    assert_eq!(w.state.player().spell_threat(&content), DEFAULT_THREAT - 2);
    let player = w.player_id();
    for (slot, object) in [("right_ring", "iron_ring"), ("neck", "jade_amulet")] {
        let more = item(&w, object, "fixture_baleful", 30);
        wear(&mut w, player, slot, more);
    }
    assert_eq!(w.state.player().spell_threat(&content), *THREATS.start());
}

/// Criticals in 300 casts of the ray at a kobold next to the player.
fn criticals(w: &mut World) -> usize {
    let target = kobold(w, 1, 10_000);
    (0..300)
        .filter(|_| {
            let (strikes, _) = cast_at(w, (3, 0), (target, (1, 0)));
            strikes == [Strike::Hit { critical: true }]
        })
        .count()
}

#[test]
fn baleful_makes_an_own_cast_critical_more_often() {
    let mut plain = wizard(content(true));
    let mut baleful =
        World::from_state(plain.content().clone(), plain.state().clone()).expect("restores");
    let player = baleful.player_id();
    for (slot, object) in [("left_ring", "iron_ring"), ("right_ring", "iron_ring")] {
        let stack = item(&baleful, object, "fixture_baleful", 30);
        wear(&mut baleful, player, slot, stack);
    }
    let (few, many) = (criticals(&mut plain), criticals(&mut baleful));
    assert!(many > few + 30, "{few} plain, {many} with Baleful");
}

/// Zaps wand stack `wand` at the kobold `target` until it is empty, then reads scroll stack
/// `scroll` at it 40 times, the player's and the kobold's hit points put back each time; every
/// blow on it.
fn zap_and_read(w: &mut World, (scroll, wand): (usize, usize), target: ActorId) -> Vec<Strike> {
    let at = w.state.actors[&target].pos;
    run(w, &format!("equip:missile_weapon:{wand}"));
    let charges = w.state.player().inventory[wand].charges_left(w.content());
    let zaps = usize::from(charges.expect("a wand"));
    let actions = (0..zaps + 40).map(|i| {
        if i < zaps {
            format!("fire:{},{}", at.x, at.y)
        } else {
            format!("read:{scroll}:{},{}", at.x, at.y)
        }
    });
    let mut strikes = Vec::new();
    for text in actions {
        let action = &crate::action::parse_actions(&text).expect("parses")[0];
        w.state.actors.get_mut(&target).expect("kobold").hp = 10_000;
        let max = w.state.player().max_hp(w.content());
        w.player_mut().hp = max;
        let out = w.apply(action);
        assert!(out.took_turn, "{text}");
        strikes.extend(out.events.iter().filter_map(|e| match e {
            crate::action::Event::Attacked {
                target: t,
                strike: Some(strike),
                ..
            } if *t == target => Some(*strike),
            _ => None,
        }));
    }
    strikes
}

#[test]
fn a_scroll_and_a_wand_keep_their_own_aim_and_threat() {
    let mut plain = wizard(content(false));
    let target = kobold(&mut plain, 1, 10_000);
    let content = plain.content().clone();
    let scroll = content
        .object_id("scroll_of_scorching_ray")
        .expect("scroll");
    let wand = content.object_id("wand_of_scorching_ray").expect("wand");
    let me = plain.player_mut();
    me.inventory.push(Stack::new(scroll, 40));
    me.inventory.push(Stack::new(wand, 1));
    let at = (me.inventory.len() - 2, me.inventory.len() - 1);
    let mut worn = World::from_state(content.clone(), plain.state().clone()).expect("restores");
    let player = worn.player_id();
    for (slot, object, id, value) in [
        ("left_ring", "iron_ring", "fixture_baleful", 30),
        ("right_ring", "iron_ring", "fixture_of_the_magus", 10),
    ] {
        let stack = item(&worn, object, id, value);
        wear(&mut worn, player, slot, stack);
    }
    let without = zap_and_read(&mut plain, at, target);
    let with = zap_and_read(&mut worn, at, target);
    assert!(without.len() >= 41, "{without:?}");
    assert_eq!(with, without, "the reader's rings change nothing");
}

#[test]
fn a_creature_wearing_them_reads_them_too() {
    let mut w = wizard(content(true));
    let content = w.content().clone();
    let kobold = kobold(&mut w, 2, 10);
    let bolt = content.spell(sp("arcane_bolt"));
    let before = {
        let k = &w.state.actors[&kobold];
        (k.mana_cost(&content, 10), k.spell_to_hit(&content, bolt))
    };
    let thrift = item(&w, "iron_ring", "fixture_of_thrift", 30);
    wear(&mut w, kobold, "left_ring", thrift);
    let magus = item(&w, "iron_ring", "fixture_of_the_magus", 2);
    wear(&mut w, kobold, "right_ring", magus);
    let baleful = item(&w, "jade_amulet", "fixture_baleful", 1);
    wear(&mut w, kobold, "neck", baleful);
    let k = &w.state.actors[&kobold];
    assert_eq!(before.0, 10);
    assert_eq!(k.mana_cost(&content, 10), 7);
    assert_eq!(k.spell_to_hit(&content, bolt), before.1 + 2);
    assert_eq!(k.spell_threat(&content), DEFAULT_THREAT - 1);
}
