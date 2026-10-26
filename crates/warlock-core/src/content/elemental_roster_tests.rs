//! The elemental roster (#453): every type tagged `elemental` keeps to the undead roster's
//! formulas (#383, `undead_roster_tests`) for its level and role, carries one element's tag, and
//! each element has its ladder of rungs.

use super::Content;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"; the elementals use four of its seven).
#[derive(Clone, Copy, Debug)]
enum Role {
    Brute,
    Standard,
    Quick,
    Small,
}

impl Role {
    fn toughness(self) -> f64 {
        match self {
            Role::Brute => 5.0,
            Role::Standard => 0.0,
            Role::Quick => -1.0,
            Role::Small => -4.0,
        }
    }

    fn hp(self) -> f64 {
        match self {
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Small => 0.5,
        }
    }

    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Standard | Role::Quick => 0,
            Role::Small => -1,
        }
    }
}

/// The elements: the tag, the role of the rungs above the mote (a mote is [`Role::Small`]),
/// and the damage type each of its rungs is immune to besides poison, if any.
const ELEMENTS: &[(&str, Role, Option<&str>)] = &[
    ("fire", Role::Standard, Some("fire")),
    ("earth", Role::Brute, Some("electricity")),
    ("air", Role::Quick, None),
    ("water", Role::Standard, None),
    ("ice", Role::Brute, Some("cold")),
    ("lightning", Role::Quick, Some("electricity")),
];

/// The fewest rungs an element's ladder has.
const MIN_RUNGS: usize = 6;

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

fn dice_average(level: f64, role: Role) -> f64 {
    let target = if level <= 20.0 {
        2.0 + 0.55 * level
    } else {
        13.0 + 0.25 * (level - 20.0)
    };
    let nearest = (0..LADDER.len())
        .min_by(|&a, &b| {
            (LADDER[a] - target)
                .abs()
                .total_cmp(&(LADDER[b] - target).abs())
        })
        .expect("a ladder");
    let index = (nearest as i32 + role.step()).clamp(0, LADDER.len() as i32 - 1);
    LADDER[index as usize]
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_elementals_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let guideline = content.experience().guideline();
    let small = content.sizes().iter().position(|s| s.id == "small");
    let factions = ["elemental", "fire"].map(|f| content.factions().id(f).expect(f));
    let poison = content.damage_type_id("poison").expect("poison");
    let mut rungs = vec![Vec::new(); ELEMENTS.len()];
    for c in content.tagged("elemental") {
        let def = content.creature(c);
        let id = def.id.as_str();
        let elements: Vec<usize> = (0..ELEMENTS.len())
            .filter(|&e| content.has_tag(c, ELEMENTS[e].0))
            .collect();
        let [element] = elements[..] else {
            panic!("{id}: give it exactly one element's tag");
        };
        let (_, role, immune) = ELEMENTS[element];
        let role = if Some(def.size) == small {
            Role::Small
        } else {
            role
        };
        rungs[element].push(def.level);
        assert!(
            def.faction.is_some_and(|f| factions.contains(&f)),
            "{id}: of the elemental faction (the cinder elemental: the delf's fire)"
        );
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let attack = (def.attack.as_ref()).unwrap_or_else(|| panic!("{id}: no attack"));
        let damage = attack.attack.damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let want = dice_average(level, role);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
        let resists = |t| def.resist.iter().any(|&(r, v)| r == t && v == 100);
        assert!(resists(poison), "{id}: immune to poison");
        if let Some(immune) = immune {
            let t = content.damage_type_id(immune).expect("a damage type");
            assert!(resists(t), "{id}: immune to {immune}");
        }
        assert!(!def.corpse, "{id}: nothing is left of an elemental");
    }
    for (e, levels) in rungs.iter_mut().enumerate() {
        levels.sort_unstable();
        let element = ELEMENTS[e].0;
        assert!(
            levels.len() >= MIN_RUNGS,
            "{element}: {} rungs, at least {MIN_RUNGS}",
            levels.len()
        );
        assert!(
            levels.first().is_some_and(|&l| l <= 10) && levels.last().is_some_and(|&l| l >= 50),
            "{element}: its ladder spans the levels ({levels:?})"
        );
    }
}

#[test]
fn the_cinder_elemental_is_the_fire_lines_rung() {
    let content = Content::builtin().expect("content");
    let cinder = content.creature_id("cinder_elemental").expect("cinder");
    assert!(content.has_tag(cinder, "elemental") && content.has_tag(cinder, "fire"));
    assert!(content.has_tag(cinder, "fire-thing"), "still the delf's");
}
