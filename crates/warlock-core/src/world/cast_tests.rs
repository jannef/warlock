//! The one cast path (spec §8 "Spells", #395, the spell scaling design §13): resolution, no cap
//! (decision 16), the shapes, the cost's armour, the ladders' rungs, the roll and rider hooks,
//! and the draws a cast makes. Spells the casters do not have are
//! tried as the wizard's Arcane bolt, its file replaced (the ability's id is the spell's).

use super::caster_tests::{kobold, kobold_at, last, reach, run, sp, world_with};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::ContentFiles;
use crate::stats::{PerStat, Stat, Stats};

/// The built-in content with the wizard's Arcane bolt (level 6) as `body` says: its shape,
/// resolution, and dice.
fn content_with_bolt(body: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let text = format!(
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n{body}\n"
    );
    files.insert("data/spells/arcane_bolt.toml", &text);
    Arc::new(Content::load(&files).expect("the spell loads"))
}

/// A level-18 wizard (Arcane bolt) with `content`.
fn wizard(content: Arc<Content>) -> World {
    let mut w = world_with(content, "wizard");
    reach(&mut w, 18);
    w
}

/// The player casts Arcane bolt at the cell `(dx, dy)` from them, with full hit points and
/// mana, `target` put back first where it stood (`(tx, ty)` from the player) with plenty of
/// hit points; what its blows on `target` were, and the hit points it lost.
pub(super) fn cast_at(
    w: &mut World,
    (dx, dy): (i32, i32),
    (target, (tx, ty)): (ActorId, (i32, i32)),
) -> (Vec<Strike>, i32) {
    let here = w.player_pos();
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    w.player_mut().mana = 40;
    // Untrained in every school, so no rank its marks raise grows the dice between casts.
    w.player_mut().weapon_skills = Default::default();
    let kobold = w.state.actors.get_mut(&target).expect("the kobold");
    kobold.pos = Pos::new(here.x + tx, here.y + ty);
    kobold.hp = 10_000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let at = Pos::new(here.x + dx, here.y + dy);
    let out = w.apply(&Action::Cast("arcane_bolt".into(), Some(at)));
    let strikes = (out.events.iter())
        .filter_map(|e| match e {
            Event::Attacked {
                target: t,
                strike: Some(strike),
                ..
            } if *t == target => Some(*strike),
            _ => None,
        })
        .collect();
    (strikes, 10_000 - w.state.actors[&target].hp)
}

#[test]
fn an_attack_spell_can_miss_and_critical_and_nothing_holds_its_roll() {
    let content =
        content_with_bolt("shape = \"ray\"\nresolution = \"attack\"\nrange = 6\ndamage = \"3d6\"");
    let mut w = wizard(content);
    let target = kobold(&mut w, 1, 10_000);
    let (mut missed, mut critical, mut most) = (0, 0, 0);
    for _ in 0..300 {
        let (strikes, hurt) = cast_at(&mut w, (3, 0), (target, (1, 0)));
        assert_eq!(
            strikes.len(),
            1,
            "the first creature on its way: {strikes:?}"
        );
        match strikes[0] {
            Strike::Missed { .. } => {
                missed += 1;
                // Less a hit point the kobold may have mended as the turn passed.
                assert!((-1..=0).contains(&hurt), "{hurt}");
            }
            Strike::Hit { critical: true } => critical += 1,
            _ => {}
        }
        most = most.max(hurt);
    }
    assert!(
        missed > 0 && critical > 0,
        "{missed} missed, {critical} criticals"
    );
    // 3d6 rolls 18 at most; a critical rolls it again, and nothing holds it (decisions 4, 16).
    assert!(most > 18 && most <= 36, "{most}");
}

#[test]
fn auto_and_area_spells_never_miss_nor_critical() {
    // 2d6, 12 at most: Scroll adept's rung is the spell scrolls' only (decision 12).
    for body in [
        "shape = \"bolt\"\nresolution = \"auto\"\nrange = 6",
        "shape = \"line\"\nresolution = \"area\"\nlength = 6",
    ] {
        let most_of = 12;
        let body = format!("{body}\ndamage = \"2d6\"");
        let mut w = wizard(content_with_bolt(&body));
        let target = kobold(&mut w, 2, 10_000);
        let mut most = 0;
        for _ in 0..100 {
            let (strikes, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
            assert_eq!(strikes, [Strike::Hit { critical: false }], "{body}");
            assert!((2..=most_of).contains(&hurt), "{body}: {hurt}");
            most = most.max(hurt);
        }
        assert_eq!(most, most_of, "{body}");
    }
}

#[test]
fn empower_climbs_a_rung_and_nothing_holds_it() {
    // The shipped Arcane bolt: a line of 2d6, then 3d6 at Intelligence 13; Empower's rung
    // reaches 3d6 at Intelligence 10, 18 at most.
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 32);
    scores(&mut w, &[]);
    let content = w.content().clone();
    let bolt = content.spell(sp("arcane_bolt"));
    let roll = w.state.player().spell_roll(&content, bolt, CastBy::Own);
    assert_eq!(
        roll.map(|r| r.to_string()).as_deref(),
        Some("3d6"),
        "Empower"
    );
    let target = kobold(&mut w, 2, 10_000);
    let mut most = 0;
    for _ in 0..200 {
        let (_, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
        most = most.max(hurt);
    }
    assert!(most > 12, "3d6 passes the base's 12: {most}");
}

#[test]
fn evasion_halves_an_area_but_not_a_bolt() {
    for (body, halved) in [
        ("shape = \"line\"\nresolution = \"area\"\nlength = 6", true),
        ("shape = \"bolt\"\nresolution = \"auto\"\nrange = 6", false),
    ] {
        let content = content_with_bolt(&format!("{body}\ndamage = \"2d6\""));
        // The same cast against the same roll, the kobold with Evasion and without.
        let hurt = |evades: bool| {
            let mut w = wizard(content.clone());
            let target = kobold(&mut w, 2, 10_000);
            if evades {
                let kobold = w.state.actors.get_mut(&target).expect("there");
                kobold.talents.take("evasion", None);
            }
            cast_at(&mut w, (2, 0), (target, (2, 0))).1
        };
        let (full, evaded) = (hurt(false), hurt(true));
        assert!(full >= 2, "{body}: {full}");
        let expected = if halved { full / 2 } else { full };
        assert_eq!(evaded, expected, "{body}");
    }
}

#[test]
fn a_line_strikes_everyone_on_it_and_a_bolt_the_first() {
    for (body, both) in [
        ("shape = \"line\"\nresolution = \"area\"\nlength = 6", true),
        ("shape = \"bolt\"\nresolution = \"auto\"\nrange = 6", false),
    ] {
        let mut w = wizard(content_with_bolt(&format!("{body}\ndamage = \"2d6\"")));
        let near = kobold(&mut w, 2, 100);
        let far = kobold(&mut w, 4, 100);
        w.apply(&Action::Cast(
            "arcane_bolt".into(),
            Some(Pos::new(w.player_pos().x + 4, w.player_pos().y)),
        ));
        assert!(w.state.actors[&near].hp < 100, "{body}");
        let far_hp = w.state.actors[&far].hp;
        assert_eq!(far_hp < 100, both, "{body}");
    }
}

#[test]
fn a_cone_fans_out_toward_its_aim() {
    let mut w = wizard(content_with_bolt(
        "shape = \"cone\"\nresolution = \"area\"\nlength = 3\ndamage = \"2d6\"\n\
         cast = { effect = \"fireball\" }",
    ));
    let ahead = kobold_at(&mut w, (2, 1), 100);
    let aside = kobold_at(&mut w, (0, 2), 100);
    let here = w.player_pos();
    let mana = w.state.player().mana;
    let out = w.apply(&Action::Cast(
        "arcane_bolt".into(),
        Some(Pos::new(here.x + 3, here.y)),
    ));
    assert!(w.state.actors[&ahead].hp < 100, "in the cone");
    assert_eq!(w.state.actors[&aside].hp, 100, "beside it");
    // Level 6 × 1.6 = 9.6: 10 mana.
    assert_eq!(mana - w.state.player().mana, 10);
    // Drawn as a burst around the caster over the cone's cells.
    let drawn = out.events.iter().any(|e| {
        matches!(e, Event::Spell { shape: crate::action::SpellShape::Burst { center, radius: 3 }, cells, .. }
            if *center == here && cells.contains(&Pos::new(here.x + 2, here.y + 1))
                && !cells.contains(&here))
    });
    assert!(drawn, "{:?}", out.events);
}

#[test]
fn a_touch_reaches_only_the_next_cell() {
    let mut w = wizard(content_with_bolt(
        "shape = \"touch\"\nresolution = \"touch\"\ndamage = \"3d6\"",
    ));
    let target = kobold(&mut w, 2, 10_000);
    let turn = w.state.turn;
    let (strikes, _) = cast_at(&mut w, (2, 0), (target, (2, 0)));
    assert!(strikes.is_empty());
    assert_eq!(last(&w), "You can only touch someone next to you.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, 40));
    let mut hits = 0;
    for _ in 0..50 {
        let (strikes, hurt) = cast_at(&mut w, (1, 0), (target, (1, 0)));
        assert_eq!(strikes.len(), 1);
        hits += i32::from(strikes[0].is_hit());
        // 3d6, a critical's twice: nothing holds it.
        assert!(hurt <= 36);
    }
    assert!(hits > 0);
    // Level 6 × 0.7: 4 mana.
    let spent = 40 - w.state.player().mana;
    assert_eq!(spent, 4);
}

#[test]
fn armour_makes_spells_dearer() {
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 18);
    let content = w.content().clone();
    let bolt = content.class_spell(sp("arcane_bolt")).expect("the bolt");
    assert_eq!(w.state.player().spell_cost(&content, bolt), 11);
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("equip:body:{item}"));
    assert_eq!(w.state.player().armour_spell_cost(&content), 40);
    // 6 × 1.8 × 1.4 = 15.12.
    assert_eq!(w.state.player().spell_cost(&content, bolt), 15);
    assert_eq!(w.perception().own_spell_cost(sp("arcane_bolt")), Some(15));
}

#[test]
fn willpower_climbs_the_reach_ladder_to_its_last_rung() {
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 18);
    // The shipped Arcane bolt: length 6; `willpower 14 7, 18 8`, the old rounding's values.
    let mut reach_at = |score: u8| {
        let mut scores = PerStat::splat(14);
        scores[Stat::Willpower] = score;
        w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
        w.perception()
            .own_spell_reach(sp("arcane_bolt"))
            .and_then(|r| r.range)
    };
    assert_eq!(
        [
            reach_at(10),
            reach_at(13),
            reach_at(14),
            reach_at(17),
            reach_at(18),
            reach_at(30)
        ],
        [Some(6), Some(6), Some(7), Some(7), Some(8), Some(8)]
    );
}

#[test]
fn a_spell_without_a_roll_draws_nothing() {
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "priest");
    reach(&mut w, 3);
    let rng = w.state.rng.clone();
    let mut events = Vec::new();
    let spent = w.cast_spell("blessing", None, &mut events);
    assert!(matches!(spent, Spent::Turn(_)));
    assert!(w.state.player().effects.iter().next().is_some(), "blessed");
    assert_eq!(w.state.rng, rng, "the cast itself drew nothing");
}

/// Sets the player's scores in `scores` (every other 10, so its modifier is 0).
pub(super) fn scores(w: &mut World, scores: &[(Stat, u8)]) {
    let mut all = PerStat::splat(10);
    for &(stat, score) in scores {
        all[stat] = score;
    }
    w.player_mut().stats = Stats::new(all, PerStat::splat(40));
}

/// An auto bolt of level 6 that never misses: 2d6, a die more at Intelligence 13, a bigger
/// die at 16, a hotter core at 19.
const LADDER_BOLT: &str = "shape = \"bolt\"\nresolution = \"auto\"\nrange = 6\ndamage = \"2d6\"\n\
                           [spell.grows]\n\
                           damage = \"intelligence 13 3d6, 16 3d8, 19 3d8+1d10\"";

/// The player's own roll for Arcane bolt now, as text.
fn own_roll(w: &World) -> String {
    let content = w.content().clone();
    let bolt = content.spell(sp("arcane_bolt"));
    let roll = w.state.player().spell_roll(&content, bolt, CastBy::Own);
    roll.map(|r| r.to_string()).unwrap_or_default()
}

#[test]
fn the_rung_is_the_count_of_rungs_the_score_reaches() {
    let mut w = wizard(content_with_bolt(LADDER_BOLT));
    for (score, roll) in [
        (3, "2d6"),
        (12, "2d6"),
        (13, "3d6"),
        (15, "3d6"),
        (16, "3d8"),
        (19, "3d8+1d10"),
        (40, "3d8+1d10"),
    ] {
        scores(&mut w, &[(Stat::Intelligence, score)]);
        assert_eq!(own_roll(&w), roll, "Intelligence {score}");
    }
    // What a drain or a worn item does to the score counts: the effective score climbs.
    scores(&mut w, &[(Stat::Intelligence, 15)]);
    w.player_mut().damage.take(Stat::Intelligence, 3, 15);
    assert_eq!(own_roll(&w), "2d6", "15 drained to 12");
}

/// A cast at each rung draws exactly the dice of that rung, one draw a die in group order, and
/// nothing before them (an auto bolt has no attack roll): the damage is that roll on the same
/// stream.
#[test]
fn a_cast_at_every_rung_draws_the_dice_of_that_rung() {
    let mut w = wizard(content_with_bolt(LADDER_BOLT));
    let target = kobold(&mut w, 2, 10_000);
    for score in [10, 13, 16, 19] {
        scores(&mut w, &[(Stat::Intelligence, score)]);
        let roll: crate::dice::Roll = own_roll(&w).parse().expect("a roll");
        for _ in 0..20 {
            let mut rng = w.state.rng.clone();
            let (_, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
            // Less a hit point the kobold may have mended as the turn passed.
            let rolled = roll.roll(&mut rng, 0);
            assert!(
                (rolled - 1..=rolled).contains(&hurt),
                "Intelligence {score}: {hurt}"
            );
            assert_eq!(
                w.state.rng, rng,
                "nothing more drawn at Intelligence {score}"
            );
        }
    }
}

/// The boundary of a rung (Intelligence 15 → 16) changes the roll and nothing else: the reach,
/// the cost, the to-hit stay.
#[test]
fn a_rungs_boundary_changes_the_roll_and_nothing_else() {
    let mut w = wizard(Arc::new(Content::builtin().expect("content")));
    let content = w.content().clone();
    let fireball = content.spell(sp("fireball"));
    let read = |w: &World| {
        let me = w.state.player();
        (
            me.spell_roll(&content, fireball, CastBy::Own)
                .map(|r| r.to_string()),
            me.spell_reach(&content, fireball),
            me.spell_cost(&content, fireball),
            me.spell_to_hit(&content, fireball),
        )
    };
    scores(&mut w, &[(Stat::Intelligence, 15)]);
    let before = read(&w);
    scores(&mut w, &[(Stat::Intelligence, 16)]);
    let after = read(&w);
    assert_eq!(before.0.as_deref(), Some("3d6"));
    assert_eq!(after.0.as_deref(), Some("4d6"));
    assert_eq!((before.1, before.2, before.3), (after.1, after.2, after.3));
}

/// Sure hand's reroll (the roll hook): a die showing 1 is drawn once more, and that draw stands;
/// the cast's damage is the roll with that reroll on the same stream, and no die is sent back
/// twice.
#[test]
fn a_reroll_draws_once_more_per_die_it_sends_back_and_never_twice() {
    let mut w = wizard(content_with_bolt(LADDER_BOLT));
    w.player_mut().talents.take("sure_hand", None);
    scores(&mut w, &[(Stat::Intelligence, 13)]);
    let target = kobold(&mut w, 2, 10_000);
    let roll: crate::dice::Roll = own_roll(&w).parse().expect("a roll");
    let (mut ones, mut low) = (0, 0);
    for _ in 0..300 {
        let mut rng = w.state.rng.clone();
        let mut plain = rng.clone();
        let (_, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
        let rolled = roll.roll(&mut rng, 1);
        assert!(
            (rolled - 1..=rolled).contains(&hurt),
            "Sure hand rerolls the 1s: {hurt}"
        );
        assert_eq!(w.state.rng, rng, "nothing more drawn");
        // The same stream without the reroll: three draws, of which any 1 is sent back.
        let dice: Vec<i32> = (0..3).map(|_| plain.range(1, 6)).collect();
        ones += dice.iter().filter(|&&d| d == 1).count();
        low += i32::from(rolled == 3);
    }
    assert!(ones > 0, "some 1s were sent back");
    assert!(
        low < 3,
        "three 1s that stand are rare with the reroll: {low}"
    );
}

/// Numbing cold's rider (the rider hook): every target a cold spell harms is Sluggish besides,
/// none it misses, and no other spell's.
#[test]
fn a_rider_lands_on_every_target_harmed_and_on_none_missed() {
    let mut files = ContentFiles::builtin();
    let text = "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
                level = 6\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"cold\"\n\
                shape = \"ray\"\nresolution = \"attack\"\nrange = 6\ndamage = \"1d6\"\n";
    files.insert("data/spells/arcane_bolt.toml", text);
    let mut w = wizard(Arc::new(Content::load(&files).expect("the spell loads")));
    w.player_mut().talents.take("numbing_cold", None);
    let content = w.content().clone();
    let sluggish = content.status_id("sluggish").expect("sluggish");
    let target = kobold(&mut w, 1, 10_000);
    let (mut hit, mut missed) = (0, 0);
    for _ in 0..200 {
        w.state.actors.get_mut(&target).expect("there").effects = Default::default();
        let (strikes, _) = cast_at(&mut w, (3, 0), (target, (1, 0)));
        let slowed = w.state.actors[&target].effects.has(sluggish);
        if strikes[0].is_hit() {
            hit += 1;
            assert!(slowed, "harmed: Sluggish");
        } else {
            missed += 1;
            assert!(!slowed, "missed: nothing");
        }
    }
    assert!(hit > 0 && missed > 0, "{hit} hit, {missed} missed");
    // A line of force harms everyone on it; the rider is a cold spell's only.
    let mut w = wizard(content_with_bolt(
        "shape = \"line\"\nresolution = \"area\"\nlength = 6\ndamage = \"2d6\"",
    ));
    w.player_mut().talents.take("numbing_cold", None);
    let near = kobold(&mut w, 2, 100);
    cast_at(&mut w, (4, 0), (near, (2, 0)));
    assert!(
        !w.state.actors[&near].effects.has(sluggish),
        "force is not cold"
    );
}

#[test]
fn an_intelligent_caster_aims_a_spell_that_names_intelligence() {
    let mut w = wizard(content_with_bolt(
        "shape = \"ray\"\nresolution = \"attack\"\nrange = 6\ndamage = \"3d6\"\n\
         aim = \"intelligence\"",
    ));
    let content = w.content().clone();
    let bolt = content.spell(sp("arcane_bolt"));
    scores(&mut w, &[(Stat::Intelligence, 18), (Stat::Dexterity, 4)]);
    assert_eq!(
        w.state.player().spell_to_hit(&content, bolt),
        4,
        "Intelligence's"
    );
    let hits = |w: &mut World| {
        let target = kobold(w, 1, 10_000);
        (0..300)
            .filter(|_| {
                let (strikes, _) = cast_at(w, (3, 0), (target, (1, 0)));
                strikes.iter().all(|s| s.is_hit())
            })
            .count()
    };
    let by_wits = hits(&mut w);
    // The same ray aimed by Dexterity (-3).
    let mut d = wizard(content_with_bolt(
        "shape = \"ray\"\nresolution = \"attack\"\nrange = 6\ndamage = \"3d6\"",
    ));
    scores(&mut d, &[(Stat::Intelligence, 18), (Stat::Dexterity, 4)]);
    let by_hand = hits(&mut d);
    assert!(
        by_wits > by_hand + 30,
        "{by_wits} by Intelligence, {by_hand} by Dexterity"
    );
}
