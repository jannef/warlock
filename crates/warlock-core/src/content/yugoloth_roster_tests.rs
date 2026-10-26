//! The yugoloth roster (#446): every `faction = "yugoloth"` type keeps to the undead roster's
//! formulas (#383 "Conventions": XP, max HP, and dice by level and role), carries the `yugoloth`
//! and `fiend` tags, and the roster's five are all there.

use super::Content;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"), the ones the yugoloths take.
#[derive(Clone, Copy, Debug)]
enum Role {
    Brute,
    Standard,
    Quick,
    Caster,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Brute => 5.0,
            Role::Standard | Role::Caster => 0.0,
            Role::Quick => -1.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Quick => 0.9,
            Role::Caster => 0.8,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Standard | Role::Quick => 0,
            Role::Caster => -1,
        }
    }
}

/// Every yugoloth, its level, and its role (#446's table).
const ROSTER: &[(&str, u8, Role)] = &[
    ("mezzoloth", 22, Role::Standard),
    ("canoloth", 29, Role::Quick),
    ("nycaloth", 36, Role::Brute),
    ("arcanaloth", 46, Role::Caster),
    ("ultroloth", 58, Role::Standard),
];

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
}

/// The formula's dice average: the ladder's nearest to `2 + 0.55 L` (flattening past L20),
/// moved by the role's step.
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
fn the_yugoloths_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let yugoloth = content
        .factions()
        .id("yugoloth")
        .expect("the yugoloth faction");
    let guideline = content.experience().guideline();
    let mut seen = Vec::new();
    for def in content.creature_defs() {
        let id = def.id.as_str();
        let tagged = def.tags.iter().any(|t| t == "yugoloth");
        assert_eq!(
            def.faction == Some(yugoloth),
            tagged,
            "{id}: the yugoloth faction and tag go together"
        );
        if !tagged {
            continue;
        }
        seen.push(id);
        assert!(
            def.tags.iter().any(|t| t == "fiend"),
            "{id}: tag it `fiend`"
        );
        let &(_, level, role) = (ROSTER.iter().find(|(r, ..)| *r == id))
            .unwrap_or_else(|| panic!("{id}: give it a level and a role in ROSTER"));
        assert_eq!(def.level, level, "{id}: off its roster level");
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let caster = def.tags.iter().any(|t| t == "caster");
        assert_eq!(caster, matches!(role, Role::Caster), "{id}: caster tag");
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
    }
    for (id, ..) in ROSTER {
        assert!(
            seen.contains(id),
            "{id}: listed in ROSTER but no yugoloth type"
        );
    }
}

#[test]
fn the_formulas_give_the_tables_numbers() {
    // #446's table: the mezzoloth's 76 hit points and 3d8, the nycaloth's 175 and 4d8 (a brute's
    // step above 3d10), the arcanaloth's 4d8 (a caster's step below 3d12).
    assert_eq!(max_hp(22.0, Role::Standard).round(), 76.0);
    assert_eq!(dice_average(22.0, Role::Standard), 13.5);
    assert_eq!(max_hp(36.0, Role::Brute).round(), 175.0);
    assert_eq!(dice_average(36.0, Role::Brute), 18.0);
    assert_eq!(dice_average(46.0, Role::Caster), 18.0);
}
