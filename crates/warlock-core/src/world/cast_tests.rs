//! The one cast path (spec §8 "Spells", #395): resolution, no cap (decision 16), the shapes,
//! the cost's armour and Willpower's reach, and the draws a cast makes. Spells the casters do not have are
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
    let content = content_with_bolt(
        "shape = \"ray\"\nresolution = \"attack\"\nmax_range = 6\ndamage = { dice = \"3d6\" }",
    );
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
                assert_eq!(hurt, 0);
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
    // 3d6 rolls 18 at most; a critical rolls it again, and nothing holds it (decision 16).
    assert!(most > 18 && most <= 36, "{most}");
}

#[test]
fn auto_and_area_spells_never_miss_nor_critical() {
    // Scroll adept's +1 a die: 2d6+2, 14 at most, nothing holding it.
    for body in [
        "shape = \"bolt\"\nresolution = \"auto\"\nmax_range = 6",
        "shape = \"line\"\nresolution = \"area\"\nmax_length = 6",
    ] {
        let cap = 14;
        let body = format!("{body}\ndamage = {{ dice = \"2d6\" }}");
        let mut w = wizard(content_with_bolt(&body));
        let target = kobold(&mut w, 2, 10_000);
        let mut most = 0;
        for _ in 0..100 {
            let (strikes, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
            assert_eq!(strikes, [Strike::Hit { critical: false }], "{body}");
            assert!((4..=cap).contains(&hurt), "{body}: {hurt}");
            most = most.max(hurt);
        }
        assert_eq!(most, cap, "{body}");
    }
}

#[test]
fn empower_adds_a_die_and_nothing_holds_it() {
    // The shipped Arcane bolt: a line of 2d6; Empower makes it 3d6, 18 at most.
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 32);
    // No modifier, so no lift.
    scores(&mut w, &[]);
    assert_eq!(
        w.state.player().talent_mods(w.content()).spell_dice,
        1,
        "Empower"
    );
    let target = kobold(&mut w, 2, 10_000);
    let mut most = 0;
    for _ in 0..200 {
        let (_, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
        most = most.max(hurt);
    }
    assert!(most > 14, "3d6 passes the old cap of 14: {most}");
}

#[test]
fn evasion_halves_an_area_but_not_a_bolt() {
    for (body, halved) in [
        (
            "shape = \"line\"\nresolution = \"area\"\nmax_length = 6",
            true,
        ),
        (
            "shape = \"bolt\"\nresolution = \"auto\"\nmax_range = 6",
            false,
        ),
    ] {
        let content = content_with_bolt(&format!("{body}\ndamage = {{ dice = \"2d6\" }}"));
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
        assert!(full >= 4, "{body}: {full}");
        let expected = if halved { full / 2 } else { full };
        assert_eq!(evaded, expected, "{body}");
    }
}

#[test]
fn a_line_strikes_everyone_on_it_and_a_bolt_the_first() {
    for (body, both) in [
        (
            "shape = \"line\"\nresolution = \"area\"\nmax_length = 6",
            true,
        ),
        (
            "shape = \"bolt\"\nresolution = \"auto\"\nmax_range = 6",
            false,
        ),
    ] {
        let mut w = wizard(content_with_bolt(&format!(
            "{body}\ndamage = {{ dice = \"2d6\" }}"
        )));
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
        "shape = \"cone\"\nresolution = \"area\"\nmax_length = 3\ndamage = { dice = \"2d6\" }\n\
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
        "shape = \"touch\"\nresolution = \"touch\"\ndamage = { dice = \"3d6\" }",
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
fn willpower_reaches_farther_up_to_the_spells_cap() {
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 18);
    // The shipped Arcane bolt: length 6, at most 8; a cell per two points of the modifier.
    let mut reach_at = |score: u8| {
        let mut scores = PerStat::splat(14);
        scores[Stat::Willpower] = score;
        w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
        w.perception()
            .own_spell_reach(sp("arcane_bolt"))
            .and_then(|r| r.range)
    };
    assert_eq!(
        [reach_at(10), reach_at(14), reach_at(18), reach_at(30)],
        [Some(6), Some(7), Some(8), Some(8)]
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

/// An attack ray of level 6, 3d6, its dice lifted by Intelligence 10 % and Willpower 5 % a
/// point, aimed with Intelligence.
const SCALED_RAY: &str = "shape = \"ray\"\nresolution = \"attack\"\nmax_range = 6\n\
                          damage = { dice = \"3d6\" }\n[spell.scales]\n\
                          damage = { intelligence = 10, willpower = 5 }\n\
                          to_hit = { intelligence = 100 }";

#[test]
fn the_stats_a_spell_names_lift_its_dice_with_no_most() {
    let mut w = wizard(content_with_bolt(SCALED_RAY));
    let content = w.content().clone();
    let bolt = content.spell(sp("arcane_bolt"));
    let at = |w: &World| w.state.player().spell_lift(&content, bolt);
    scores(&mut w, &[]);
    assert_eq!(at(&w), 0, "no modifier: no lift");
    // The design's level-10 wizard: Intelligence +4, Willpower +2: 50 %.
    scores(&mut w, &[(Stat::Intelligence, 18), (Stat::Willpower, 14)]);
    assert_eq!(at(&w), 50);
    // Low stats never lower the dice.
    scores(&mut w, &[(Stat::Intelligence, 4), (Stat::Willpower, 4)]);
    assert_eq!(at(&w), 0);
    // Intelligence +10, Willpower +10: 150 %, nothing holding it (decision 16).
    scores(&mut w, &[(Stat::Intelligence, 30), (Stat::Willpower, 30)]);
    assert_eq!(at(&w), 150);
    // Keen mind for evocation: 10 points more of lift, only for its school.
    let before = w.state.player().talents.clone();
    w.player_mut().talents.take("keen_mind", Some("evocation"));
    assert_eq!(at(&w), 160);
    w.player_mut().talents = before;
    w.player_mut().talents.take("keen_mind", Some("necromancy"));
    assert_eq!(at(&w), 150, "another school's mind");
}

#[test]
fn a_lifted_spell_rolls_more_and_its_criticals_more_again() {
    let mut w = wizard(content_with_bolt(SCALED_RAY));
    scores(&mut w, &[(Stat::Intelligence, 18), (Stat::Willpower, 14)]);
    let target = kobold(&mut w, 1, 10_000);
    let (mut critical, mut most, mut plain) = (0, 0, 0);
    for _ in 0..400 {
        let (strikes, hurt) = cast_at(&mut w, (3, 0), (target, (1, 0)));
        if strikes == [Strike::Hit { critical: true }] {
            critical += 1;
        } else {
            plain = plain.max(hurt);
        }
        most = most.max(hurt);
    }
    // 4d6 now (the lift's 50 % of three dice, a die more), past 3d6's 18; a critical more.
    assert!(critical > 0);
    assert!(plain > 18, "{plain}");
    assert!(most > plain, "{most} against {plain}");
}

#[test]
fn a_spell_without_scales_keeps_the_old_rules() {
    // The shipped Spark, a cantrip with no `damage` scales: no lift whatever the stats.
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    let content = w.content().clone();
    let spark = content.spell(sp("spark"));
    scores(&mut w, &[(Stat::Intelligence, 30), (Stat::Willpower, 30)]);
    let me = w.state.player();
    assert_eq!(me.spell_lift(&content, spark), 0);
    // Its to-hit and reach are the old Dexterity's and Willpower's.
    scores(&mut w, &[(Stat::Dexterity, 16), (Stat::Willpower, 15)]);
    let me = w.state.player();
    assert_eq!(me.spell_to_hit(&content, spark), 3);
    assert_eq!(me.spell_reach_more(&content, spark), 1);
    // Evil eye's degrees: Presence +2 at 50 %, Willpower +4 at 25 %: two more.
    let evil_eye = content.spell(sp("evil_eye"));
    scores(&mut w, &[(Stat::Presence, 14), (Stat::Willpower, 18)]);
    let me = w.state.player();
    assert_eq!(me.spell_degrees_more(&content, evil_eye), 2);
    // Blessing names nothing: Presence alone, a degree per two points.
    let blessing = content.spell(sp("blessing"));
    assert_eq!(me.spell_degrees_more(&content, blessing), 1);
}

#[test]
fn an_intelligent_caster_aims_a_spell_that_names_intelligence() {
    let mut w = wizard(content_with_bolt(SCALED_RAY));
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
        "shape = \"ray\"\nresolution = \"attack\"\nmax_range = 6\ndamage = { dice = \"3d6\" }",
    ));
    scores(&mut d, &[(Stat::Intelligence, 18), (Stat::Dexterity, 4)]);
    let by_hand = hits(&mut d);
    assert!(
        by_wits > by_hand + 30,
        "{by_wits} by Intelligence, {by_hand} by Dexterity"
    );
}
