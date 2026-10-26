//! The troll tribe (#456): every living troll (a type tagged `troll` outside the undead faction,
//! whose rotting troll keeps to the undead roster's numbers) keeps to the tribe's formulas: the
//! undead roster's (#383) on the troll's own base, Toughness 4 over it (a troll heals by it).

use super::Content;
use crate::stats::Stat;

/// How far a type's max HP and dice average may lie from the formulas, in percent.
const TOLERANCE: f64 = 35.0;

/// The tribe's span: a troll of the tribe at each end, none past it.
const SPAN: (u8, u8) = (13, 42);

/// A role's adjustments (#383 "Conventions", the roles trolls take).
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

/// Every living troll and its role (#456's table).
const ROLES: &[(&str, Role)] = &[
    // Kin before the tribe: the fen's (faction `marsh`) and the first of the tribe.
    ("marsh_troll", Role::Standard),
    ("troll", Role::Standard),
    // The tribe (#456).
    ("hill_troll", Role::Brute),
    ("forest_troll", Role::Quick),
    ("troll_shaman", Role::Caster),
    ("ice_troll", Role::Brute),
    ("stone_troll", Role::Standard),
    ("two_headed_troll", Role::Brute),
    ("fire_troll", Role::Standard),
    ("troll_chieftain", Role::Standard),
    ("elder_troll", Role::Brute),
];

/// The dice the roster picks from, by average (#383 "Damage dice": 1d3 to 5d12).
const LADDER: &[f64] = &[
    2.0, 2.5, 3.5, 4.5, 5.5, 7.0, 9.0, 10.5, 13.5, 14.0, 16.5, 18.0, 19.5, 22.0, 26.0, 27.5, 32.5,
];

/// The formula's max HP: Toughness (16 + 0.4 L, by role: #383's base and the troll's 4) plus
/// base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    16.0 + 0.4 * level + role.toughness() + 2.5 * level * role.hp()
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
fn the_trolls_keep_to_the_tribe() {
    let content = Content::builtin().expect("content");
    let undead = content.factions().id("undead").expect("the undead faction");
    let tribe = content.factions().id("troll").expect("the troll faction");
    let guideline = content.experience().guideline();
    let mut tribe_levels = Vec::new();
    for creature in content.tagged("troll") {
        let def = content.creature(creature);
        if def.faction == Some(undead) {
            continue;
        }
        let id = def.id.as_str();
        let role = ROLES.iter().find(|(r, _)| *r == id).map(|&(_, role)| role);
        let role = role.unwrap_or_else(|| panic!("{id}: give it a role in ROLES"));
        if def.faction == Some(tribe) {
            tribe_levels.push(def.level);
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
        let attack = def.attack.as_ref();
        let damage = attack
            .unwrap_or_else(|| panic!("{id}: no attack"))
            .attack
            .damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let want = dice_average(level, role);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
        // Fire stops a troll, all but the fire troll's kind.
        let fire = content.damage_type_id("fire").expect("fire");
        let resists = def.resist.iter().find(|(t, _)| *t == fire).map(|r| r.1);
        if id != "fire_troll" && id != "marsh_troll" {
            assert!(
                resists.is_some_and(|r| r < 0),
                "{id}: fire hurts a troll more"
            );
        }
    }
    for (id, _) in ROLES {
        let tagged = content
            .creature_id(id)
            .is_some_and(|c| content.has_tag(c, "troll"));
        assert!(tagged, "{id}: listed in ROLES but no troll");
    }
    let (low, high) = SPAN;
    assert_eq!(tribe_levels.iter().min(), Some(&low));
    assert_eq!(tribe_levels.iter().max(), Some(&high));
}

#[test]
fn the_formulas_give_the_plans_numbers() {
    // #456's table: the hill troll's 87 hit points and 3d8, the elder troll's 206 and 3d12.
    assert_eq!(max_hp(15.0, Role::Brute).round(), 87.0);
    assert_eq!(dice_average(15.0, Role::Brute), 13.5);
    assert_eq!(max_hp(42.0, Role::Brute).round(), 206.0);
    assert_eq!(dice_average(42.0, Role::Brute), 19.5);
}
