//! The demon roster (#449): every `faction = "demon"` type keeps to the undead roster's formulas
//! (#383, `undead_roster_tests`) for its level and role, carries the tags `demon` and `fiend`,
//! and leaves no corpse.

use super::Content;
use super::undead_roster_tests::{Role, dice_average, max_hp, within};
use crate::stats::Stat;

/// Every demon, its level, and its role (#449's table).
const DEMONS: &[(&str, u8, Role)] = &[
    ("dretch", 8, Role::Small),
    ("quasit", 12, Role::Small),
    ("balgura", 24, Role::Brute),
    ("chasme", 28, Role::Quick),
    ("hezrou", 30, Role::Brute),
    ("vrock", 34, Role::Standard),
    ("marilith", 40, Role::Standard),
    ("glabrezu", 44, Role::Brute),
    ("nalfeshnee", 50, Role::Brute),
    ("balor", 62, Role::Brute),
];

#[test]
fn the_demons_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let demon = content.factions().id("demon").expect("the demon faction");
    let guideline = content.experience().guideline();
    let demons: Vec<_> = (content.creature_defs().iter())
        .filter(|c| c.faction == Some(demon))
        .collect();
    assert_eq!(demons.len(), DEMONS.len(), "a demon missing from DEMONS");
    for &(id, level, role) in DEMONS {
        let def = content.creature_id(id).map(|c| content.creature(c));
        let def = def.unwrap_or_else(|| panic!("{id}: no such type"));
        assert_eq!(def.faction, Some(demon), "{id}: not a demon");
        assert_eq!(def.level, level, "{id}: level");
        assert_eq!(def.xp, guideline.xp(level), "{id}: off the XP guideline");
        for tag in ["demon", "fiend"] {
            assert!(def.tags.iter().any(|t| t == tag), "{id}: no `{tag}` tag");
        }
        assert!(!def.corpse, "{id}: a demon's body comes apart");
        let level = f64::from(level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level, role);
        assert!(within(hp, want), "{id}: max HP {hp}, the formula {want:.1}");
        let attack = def
            .attack
            .as_ref()
            .unwrap_or_else(|| panic!("{id}: no attack"));
        let damage = attack.attack.damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let want = dice_average(level, role, false);
        assert!(
            within(average, want),
            "{id}: dice average {average}, the formula {want}"
        );
    }
}

#[test]
fn the_demons_hunt_the_living_and_leave_the_dead_be() {
    let content = Content::builtin().expect("content");
    let id = |n: &str| content.creature_id(n).expect(n);
    use super::Stance::{Hostile, Neutral};
    for (other, want) in [
        ("player", Hostile),
        ("kobold", Hostile),
        ("goblin", Hostile),
        ("duergar_warrior", Hostile),
        ("marsh_troll", Hostile),
        ("imp", Hostile),
        ("skeleton", Neutral),
        ("ember_beetle", Neutral),
        ("rat", Neutral),
        ("balor", Neutral),
    ] {
        assert_eq!(content.stance(id("dretch"), id(other)), want, "{other}");
    }
}
