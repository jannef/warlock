//! The one cast path (spec §8 "Spells", #395): resolution, the cap, the shapes, the cost's
//! armour and Willpower's reach, and the draws a cast makes. Spells the casters do not have are
//! tried as the wizard's Arcane bolt, its file replaced (the ability's id is the spell's).

use super::caster_tests::{kobold, kobold_at, last, reach, run, world_with};
use super::*;
use crate::action::Event;
use crate::combat::Strike;
use crate::content::{ClassSpell, ContentFiles};
use crate::stats::{PerStat, Stat, Stats};

/// The built-in content with the wizard's Arcane bolt (tier 2) as `body` says: its shape,
/// resolution, and dice.
fn content_with_bolt(body: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let text = format!(
        "[[spell]]\nid = \"arcane_bolt\"\nname = \"Arcane bolt\"\ndescription = \"x\"\n\
         tier = 2\nschool = \"evocation\"\nkind = \"damage\"\ndamage_type = \"force\"\n{body}\n"
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
fn cast_at(
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
    let out = w.apply(&Action::Cast(ClassSpell::ArcaneBolt, Some(at)));
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
fn an_attack_spell_can_miss_and_critical_and_never_passes_its_cap() {
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
    // Tier 2's cap for an attack is its ceiling, 18: a critical's doubled roll is held to it.
    assert_eq!(most, 18);
}

#[test]
fn auto_and_area_spells_never_miss_nor_critical() {
    // Scroll adept's +1 a die: 2d6+2, held to tier 2's cap (13 for auto, 14 for an area).
    for (body, cap) in [
        ("shape = \"bolt\"\nresolution = \"auto\"\nmax_range = 6", 13),
        (
            "shape = \"line\"\nresolution = \"area\"\nmax_length = 6",
            14,
        ),
    ] {
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
fn empower_reaches_the_cap_sooner_and_never_passes_it() {
    // The shipped Arcane bolt: a line of 2d6, tier 2's area cap 14; Empower makes it 3d6.
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 32);
    assert_eq!(
        w.state.player().talent_mods(w.content()).spell_dice,
        1,
        "Empower"
    );
    let target = kobold(&mut w, 2, 10_000);
    let mut most = 0;
    for _ in 0..200 {
        let (_, hurt) = cast_at(&mut w, (2, 0), (target, (2, 0)));
        assert!(hurt <= 14, "{hurt}");
        most = most.max(hurt);
    }
    assert_eq!(most, 14, "3d6 reaches the cap");
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
            ClassSpell::ArcaneBolt,
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
        ClassSpell::ArcaneBolt,
        Some(Pos::new(here.x + 3, here.y)),
    ));
    assert!(w.state.actors[&ahead].hp < 100, "in the cone");
    assert_eq!(w.state.actors[&aside].hp, 100, "beside it");
    // Tier 2's 6 × 1.6 = 9.6: 10 mana.
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
        // Tier 2's touch cap: 18 × 120 % = 21.
        assert!(hurt <= 21);
    }
    assert!(hits > 0);
    // Tier 2's 6 × 0.7: 4 mana.
    let spent = 40 - w.state.player().mana;
    assert_eq!(spent, 4);
}

#[test]
fn armour_makes_spells_dearer() {
    let mut w = world_with(Arc::new(Content::builtin().expect("content")), "wizard");
    reach(&mut w, 18);
    let content = w.content().clone();
    let bolt = content
        .class_spell(ClassSpell::ArcaneBolt)
        .expect("the bolt");
    let level = w.caster_level(w.state.player);
    assert_eq!(w.state.player().spell_cost(&content, bolt, level), 11);
    let plate = content.object_id("plate_armour").expect("plate");
    let me = w.player_mut();
    me.inventory.push(crate::object::Stack::new(plate, 1));
    let item = me.inventory.len() - 1;
    run(&mut w, &format!("equip:body:{item}"));
    assert_eq!(w.state.player().armour_spell_cost(&content), 40);
    // 6 × 1.8 × 1.4 = 15.12.
    assert_eq!(w.state.player().spell_cost(&content, bolt, level), 15);
    assert_eq!(
        w.perception().own_spell_cost(ClassSpell::ArcaneBolt),
        Some(15)
    );
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
            .own_spell_reach(ClassSpell::ArcaneBolt)
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
    let rng = w.state.rng.clone();
    let mut events = Vec::new();
    let spent = w.cast_spell(ClassSpell::Blessing, None, &mut events);
    assert!(matches!(spent, Spent::Turn(_)));
    assert!(w.state.player().effects.iter().next().is_some(), "blessed");
    assert_eq!(w.state.rng, rng, "the cast itself drew nothing");
}
