//! Sneak attacks (spec §8 "Surprise and sneak attacks"; `ambush.rs`): the Sneak attack skill's
//! ladder of dice, their sum with the talents', the reach of missiles and Long reach, Surprised
//! as the second door, the spell path, the marks, and the craft and Initiative in the check.

use super::cast_tests::cast_at;
use super::caster_tests::{kobold, reach, world_with};
use super::vitals_tests::{lull, one, train, world};
use super::*;
use crate::combat::{Blow, Strike};
use crate::content::ContentFiles;
use crate::skills::{Effect, Use};

#[test]
fn the_skill_buys_a_d6_trained_and_one_more_at_each_step() {
    let w = world(true);
    let content = w.content();
    let skill = content.skill(content.skill_id("sneak_attack").expect("the skill"));
    let dice = skill
        .benefits
        .iter()
        .find(|b| b.effect == Effect::SneakDice);
    let dice = dice.expect("its dice");
    for (value, n) in [
        (0, 0),
        (1, 1),
        (24, 1),
        (25, 2),
        (50, 3),
        (75, 4),
        (99, 4),
        (100, 5),
        (149, 5),
        (150, 6),
        (200, 7),
    ] {
        assert_eq!(dice.at(value), n, "at {value}");
    }
    let craft = skill.benefits.iter().find(|b| b.effect == Effect::Surprise);
    let craft = craft.expect("its craft");
    assert_eq!((craft.at(0), craft.at(100), craft.at(200)), (0, 8, 12));
    let initiative = content.skill(content.skill_id("initiative").expect("the skill"));
    let at = |effect, value| {
        (initiative.benefits.iter())
            .find(|b| b.effect == effect)
            .map_or(0, |b| b.at(value))
    };
    assert_eq!(
        (at(Effect::Initiative, 100), at(Effect::Initiative, 200)),
        (8, 12)
    );
    assert_eq!(
        (at(Effect::SurpriseEase, 100), at(Effect::SurpriseEase, 200)),
        (50, 75)
    );
}

#[test]
fn the_skills_dice_and_the_talents_add_up() {
    let mut w = world(true);
    let player = w.state.player;
    train(&mut w, "sneak_attack", 0);
    assert_eq!(w.sneak_dice(player), None, "nothing at 0");
    train(&mut w, "sneak_attack", 30);
    assert_eq!(
        w.sneak_dice(player).map(|d| d.to_string()),
        Some("2d6".into())
    );
    // Ambusher's 1d6 on top.
    w.player_mut().talents.take("ambusher", None);
    assert_eq!(
        w.sneak_dice(player).map(|d| d.to_string()),
        Some("3d6".into())
    );
    // The talent alone, untrained.
    let mut bare = world(true);
    bare.player_mut().talents.take("ambusher", None);
    let player = bare.state.player;
    assert_eq!(
        bare.sneak_dice(player).map(|d| d.to_string()),
        Some("1d6".into())
    );
}

#[test]
fn a_blow_counts_at_any_reach_a_missile_within_six_and_long_reach_three_more() {
    let mut w = world(true);
    let player = w.state.player;
    train(&mut w, "sneak_attack", 10);
    let goblin = one(&w, "goblin");
    lull(&mut w, goblin);
    let at = |w: &mut World, x: i32, missile: bool| {
        w.state.actors.get_mut(&goblin).expect("goblin").pos = Pos::new(1 + x, 2);
        w.sneaking(player, goblin, missile)
    };
    assert!(at(&mut w, 6, true));
    assert!(!at(&mut w, 7, true), "a shot from too far");
    assert!(at(&mut w, 7, false), "a blow, however far it reaches");
    w.player_mut().talents.take("long_reach", None);
    assert!(at(&mut w, 9, true));
    assert!(!at(&mut w, 10, true));
}

#[test]
fn a_surprised_foe_takes_the_dice_though_it_sees_you() {
    let mut w = world(true);
    let player = w.state.player;
    train(&mut w, "sneak_attack", 10);
    let goblin = one(&w, "goblin");
    // Awake, in the light, hunting the player: aware.
    w.state
        .actors
        .get_mut(&goblin)
        .expect("goblin")
        .mind
        .last_seen = Some(Pos::new(1, 1));
    assert!(!w.sneaking(player, goblin, false));
    let surprised = w.content.status_id("surprised").expect("surprised");
    let def = w.content.status(surprised).clone();
    let actor = w.state.actors.get_mut(&goblin).expect("goblin");
    actor.effects.add(surprised, &def, 1, None);
    assert!(w.sneaking(player, goblin, false));
}

#[test]
fn a_sneak_attack_that_hits_marks_sneak_attack_and_dexterity() {
    let mut w = world(true);
    let player = w.state.player;
    train(&mut w, "sneak_attack", 10);
    assert!(w.trains(Use::SneakAttack));
    let id = w.content.skill_id("sneak_attack").expect("the skill");
    let hit = Blow {
        strike: Strike::Hit { critical: false },
        damage: 2,
        kind: None,
    };
    let gameplay = w.state.rng.clone();
    for _ in 0..40 {
        let damage = w.ambushed(player, true, hit).damage;
        assert!((3..=8).contains(&damage), "{damage}");
    }
    assert_ne!(w.state.rng, gameplay, "the dice are gameplay draws");
    let marks = w.state.player().skills.get(id).expect("trained").marks;
    assert!((5..=35).contains(&marks), "about half: {marks}");
    // A seen blow, or a miss, marks nothing and draws nothing.
    let rng = (w.state.rng.clone(), w.state.training.clone());
    w.ambushed(player, false, hit);
    w.ambushed(player, true, Blow::missed(None));
    assert_eq!((w.state.rng.clone(), w.state.training.clone()), rng);
}

#[test]
fn a_spell_that_rolls_to_hit_sneak_attacks_within_reach_past_its_cap() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/spells/arcane_bolt.toml",
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         tier = 2\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n\
         shape = \"ray\"\nresolution = \"attack\"\nmax_range = 10\ndamage = { dice = \"3d6\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("the spell loads"));
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    train(&mut w, "sneak_attack", 200);
    let target = kobold(&mut w, 3, 10_000);
    // Asleep three cells off, the bolt's dice are held to its cap (18) but the 7d6 come on
    // top; seven cells off, beyond the reach, never.
    let most = |w: &mut World, dx: i32| {
        (0..40)
            .map(|_| {
                lull(w, target);
                cast_at(w, (dx, 0), (target, (dx, 0))).1
            })
            .max()
            .unwrap_or(0)
    };
    assert!(most(&mut w, 3) > 18, "the dice past the cap");
    assert!(most(&mut w, 7) <= 18, "too far for a sneak attack");
}

#[test]
fn the_attackers_craft_and_the_targets_initiative_shape_the_check() {
    // Against a fixture's difficulty of 1 everyone but a natural 2 makes it; a master's craft
    // (+12) and the thief's +2 make it 15, and Initiative 200 (+12) eases it again.
    let made = |sneak: u16, initiative: u16| {
        let mut made = 0;
        for seed in 0..200u64 {
            let mut w = world(true);
            w.state.rng = crate::rng::Rng::stream(seed, "craft-test");
            let player = w.state.player;
            train(&mut w, "sneak_attack", sneak);
            let goblin = one(&w, "goblin");
            // The player stands where the goblin cannot see: it makes the check.
            if initiative > 0 {
                let id = w.content.skill_id("initiative").expect("the skill");
                let actor = w.state.actors.get_mut(&goblin).expect("goblin");
                actor.skills.add(id, i32::from(initiative));
            }
            let mut events = Vec::new();
            w.startle(goblin, Some(player), &mut events);
            made += i32::from(!w.surprised(goblin));
        }
        made
    };
    let (plain, master, eased) = (made(1, 0), made(200, 0), made(200, 200));
    assert!(master < plain - 40, "{master} of 200 against {plain}");
    assert!(eased > master + 40, "{eased} of 200 against {master}");
}
