//! The hags (#451): every type of the `hags` faction keeps to the undead roster's formulas
//! (#383 "Conventions", the same numbers as `undead_roster_tests`), and every hag carries the
//! `hag` tag, the bog hag (of the `marsh`) too.

use super::{Affliction, Content};
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"): the ones the hags take.
#[derive(Clone, Copy, Debug)]
enum Role {
    Standard,
    Brute,
    Caster,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Brute => 5.0,
            Role::Standard | Role::Caster => 0.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Caster => 0.8,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Standard => 0,
            Role::Caster => -1,
        }
    }
}

/// Every type of the `hags` faction and its role (#451's table).
const ROLES: &[(&str, Role)] = &[
    ("green_hag", Role::Caster),
    ("sea_hag", Role::Standard),
    ("annis_hag", Role::Brute),
    ("bheur_hag", Role::Caster),
];

/// The dice the roster picks from, by average (#383 "Damage dice").
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

/// The formula's dice average: the ladder's nearest to `2 + 0.55 L` (flattening past L20),
/// moved by the role's step and one step lower for a drain.
fn dice_average(level: f64, role: Role, drains: bool) -> f64 {
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
    let index = nearest as i32 + role.step() - i32::from(drains);
    LADDER[index.clamp(0, LADDER.len() as i32 - 1) as usize]
}

fn within(actual: f64, expected: f64) -> bool {
    ((actual - expected) / expected * 100.0).abs() <= TOLERANCE
}

#[test]
fn the_hags_keep_to_the_formulas() {
    let content = Content::builtin().expect("content");
    let hags = content.factions().id("hags").expect("the hags faction");
    let guideline = content.experience().guideline();
    for def in (content.creature_defs().iter()).filter(|c| c.faction == Some(hags)) {
        let id = def.id.as_str();
        let role = ROLES.iter().find(|(r, _)| *r == id).map(|&(_, role)| role);
        let role = role.unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
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
        let drains = (attack.on_hit.iter()).any(|h| matches!(h.what, Affliction::Drain(_)));
        let want = dice_average(level, role, drains);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
    }
    for (id, _) in ROLES {
        let def = content.creature_id(id).map(|c| content.creature(c));
        assert!(
            def.is_some_and(|d| d.faction == Some(hags)),
            "{id}: listed in ROLES but of no hags type"
        );
    }
}

#[test]
fn every_hag_carries_the_hag_tag() {
    let content = Content::builtin().expect("content");
    let tagged: Vec<String> = (content.tagged("hag"))
        .map(|c| content.creature(c).id.clone())
        .collect();
    for id in ["bog_hag", "green_hag", "sea_hag", "annis_hag", "bheur_hag"] {
        assert!(tagged.contains(&id.to_string()), "{id}: tag it `hag`");
    }
}

#[test]
fn the_formulas_give_the_issues_numbers() {
    // #451's table: the green hag's 55 and 2d8, the sea hag's 87 and 3d8 (4d6 a step down for
    // her drain), the annis hag's 167 and 4d8, the bheur hag's 113 and 3d10.
    assert_eq!(max_hp(18.0, Role::Caster).round(), 55.0);
    assert_eq!(dice_average(18.0, Role::Caster, false), 9.0);
    assert_eq!(max_hp(26.0, Role::Standard).round(), 87.0);
    assert_eq!(dice_average(26.0, Role::Standard, true), 13.5);
    assert_eq!(max_hp(34.0, Role::Brute).round(), 167.0);
    assert_eq!(dice_average(34.0, Role::Brute, false), 18.0);
    assert_eq!(max_hp(42.0, Role::Caster).round(), 113.0);
    assert_eq!(dice_average(42.0, Role::Caster, false), 16.5);
}
