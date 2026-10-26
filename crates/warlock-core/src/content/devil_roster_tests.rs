//! The devil roster (#448): every `faction = "devil"` type carries the `devil` and `fiend` tags
//! and keeps to the undead roster's formulas (#383 "Conventions") for its level and role.

use super::Content;
use super::creature::CreatureId;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// A role's adjustments (#383 "Conventions"): the ones the devils take.
#[derive(Clone, Copy, Debug)]
enum Role {
    Shambler,
    Brute,
    Standard,
    Small,
}

impl Role {
    /// What the role adds to the base Toughness.
    fn toughness(self) -> f64 {
        match self {
            Role::Shambler | Role::Brute => 5.0,
            Role::Standard => 0.0,
            Role::Small => -4.0,
        }
    }

    /// The factor on `base_hp`.
    fn hp(self) -> f64 {
        match self {
            Role::Shambler => 1.5,
            Role::Brute => 1.6,
            Role::Standard => 1.0,
            Role::Small => 0.5,
        }
    }

    /// The steps it moves the dice along [`LADDER`].
    fn step(self) -> i32 {
        match self {
            Role::Brute => 1,
            Role::Shambler | Role::Standard => 0,
            Role::Small => -1,
        }
    }
}

/// Every devil and its role (#448's table), from the bottom of the hierarchy up.
const ROLES: &[(&str, Role)] = &[
    ("lemure", Role::Shambler),
    ("imp", Role::Small),
    ("bearded_devil", Role::Standard),
    ("barbed_devil", Role::Standard),
    ("bone_devil", Role::Standard),
    ("erinyes", Role::Standard),
    ("chain_devil", Role::Standard),
    ("horned_devil", Role::Brute),
    ("ice_devil", Role::Brute),
    ("pit_fiend", Role::Brute),
];

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (12 + 0.4 L, by role, at most 40) plus base_hp (2.5 L, by
/// role).
fn max_hp(level: f64, role: Role) -> f64 {
    (12.0 + 0.4 * level + role.toughness()).min(40.0) + 2.5 * level * role.hp()
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
fn the_devils_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let devil = content.factions().id("devil").expect("the devil faction");
    let guideline = content.experience().guideline();
    let mut seen = Vec::new();
    for (i, def) in content.creature_defs().iter().enumerate() {
        if def.faction != Some(devil) {
            continue;
        }
        let id = def.id.as_str();
        let role = ROLES.iter().find(|(r, _)| *r == id).map(|&(_, role)| role);
        let role = role.unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        seen.push(id.to_string());
        for tag in ["devil", "fiend"] {
            assert!(
                content.has_tag(CreatureId(i as u16), tag),
                "{id}: a devil carries the `{tag}` tag"
            );
        }
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
    }
    for (id, _) in ROLES {
        assert!(
            seen.iter().any(|s| s == id),
            "{id}: listed in ROLES but no devil type"
        );
    }
}

#[test]
fn the_hierarchy_climbs() {
    // The roster's order is the infernal hierarchy: each devil stands above the one before it.
    let content = Content::builtin().expect("content");
    let levels: Vec<u8> = ROLES
        .iter()
        .map(|(id, _)| content.creature(content.creature_id(id).expect(id)).level)
        .collect();
    assert!(levels.windows(2).all(|w| w[0] < w[1]), "{levels:?}");
}
