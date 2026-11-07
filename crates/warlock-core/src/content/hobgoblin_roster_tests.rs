//! The hobgoblins (#454), the goblin tribe's second tier: every `hobgoblin` type keeps to the
//! issue's formulas, belongs to the goblins, and stands above the plain goblins.

use super::variance_roster_tests::{self, Shape};
use super::{Combat, Content};
use crate::stats::Stat;

/// How far a type's max HP may lie from the formula, in percent.
const TOLERANCE: f64 = 20.0;

/// A hobgoblin's role (#454 "Conventions"): what multiplies its `base_hp`.
#[derive(Clone, Copy, Debug)]
enum Role {
    Soldier,
    Archer,
    Caster,
    Officer,
}

impl Role {
    fn hp(self) -> f64 {
        match self {
            Role::Soldier => 1.0,
            Role::Archer => 0.9,
            Role::Caster => 0.5,
            Role::Officer => 1.1,
        }
    }

    /// What the role does to the to-hit and the blow (the variance pass, #1134).
    fn shape(self) -> Shape {
        match self {
            Role::Soldier | Role::Archer | Role::Officer => Shape::Standard,
            Role::Caster => Shape::Caster,
        }
    }
}

/// Every hobgoblin type and its role, by level.
const ROLES: &[(&str, Role)] = &[
    ("hobgoblin", Role::Soldier),
    ("hobgoblin_archer", Role::Archer),
    ("hobgoblin_legionnaire", Role::Soldier),
    ("hobgoblin_warcaster", Role::Caster),
    ("hobgoblin_sergeant", Role::Officer),
    ("hobgoblin_captain", Role::Officer),
    ("hobgoblin_warlord", Role::Officer),
];

/// The formula's max HP: Toughness (12 + 0.4 L) plus base_hp (2.5 L, by role).
fn max_hp(level: f64, role: Role) -> f64 {
    12.0 + 0.4 * level + 2.5 * level * role.hp()
}

#[test]
fn the_hobgoblins_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let goblins = content
        .factions()
        .id("goblins")
        .expect("the goblins' faction");
    let guideline = content.experience().guideline();
    let hobgoblins: Vec<_> = content.tagged("hobgoblin").collect();
    assert_eq!(hobgoblins.len(), ROLES.len(), "every hobgoblin in ROLES");
    let top_goblin = (content.tagged("goblin"))
        .map(|c| content.creature(c))
        .filter(|d| !d.tags.iter().any(|t| t == "hobgoblin"))
        .map(|d| d.level)
        .max()
        .expect("goblins");
    for &(id, role) in ROLES {
        let c = content
            .creature_id(id)
            .unwrap_or_else(|| panic!("{id}: no such type"));
        let def = content.creature(c);
        assert!(hobgoblins.contains(&c), "{id}: not tagged `hobgoblin`");
        assert!(
            def.tags.iter().any(|t| t == "goblin"),
            "{id}: a hobgoblin is of the goblin tribe"
        );
        assert_eq!(def.faction, Some(goblins), "{id}: the goblins' faction");
        assert!(def.level > top_goblin, "{id}: above the goblin bracket");
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        assert!(!def.unique, "{id}: no hobgoblin is unique");
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role) * super::variance_roster_tests::starter(def.level);
        assert!(
            ((hp - want) / want * 100.0).abs() <= TOLERANCE,
            "{id}: max HP {hp}, the formula {want:.1}"
        );
        variance_roster_tests::check(&content, def, role.shape());
        if matches!(role, Role::Soldier | Role::Archer) {
            variance_roster_tests::rank_and_file(def);
        }
        assert!(def.gear.racial_weapon.is_some(), "{id}: armed");
        let tagged = |t: &str| def.tags.iter().any(|x| x == t);
        assert_eq!(
            tagged("archer"),
            def.combat == Combat::Ranged,
            "{id}: archer tag"
        );
        assert_eq!(
            tagged("caster"),
            def.combat == Combat::Caster,
            "{id}: caster tag"
        );
    }
}
