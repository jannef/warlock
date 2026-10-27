//! The summoners (#520): every type with a summoning carries the `summoner` tag and one of
//! another tribe, and every `summoner` summons; the shipped ones keep to #383's formulas for a
//! caster.

use super::{Combat, Content};
use crate::stats::Stat;

/// How far a type's max HP may lie from the formula, in percent.
const TOLERANCE: f64 = 35.0;

/// The summoners of #520 by id. Others (the drow priestess) bring their own roster's checks.
const ROSTER: &[&str] = &["goblin_wizard", "necromancer", "dark_wizard"];

/// #383's caster: Toughness 12 + 0.4 L plus base_hp 2.5 L × 0.8.
fn max_hp(level: f64) -> f64 {
    12.0 + 0.4 * level + 2.5 * level * 0.8
}

#[test]
fn every_summoner_summons_and_belongs_to_another_tribe() {
    let content = Content::builtin().expect("content");
    let tagged: Vec<_> = content.tagged("summoner").collect();
    assert!(tagged.len() >= ROSTER.len());
    for (i, def) in content.creatures.iter().enumerate() {
        let is_tagged = tagged.iter().any(|c| usize::from(c.0) == i);
        assert_eq!(
            def.summon.is_some(),
            is_tagged,
            "{}: a summoning goes with the `summoner` tag",
            def.id
        );
        if is_tagged {
            assert!(
                def.tags.iter().any(|t| t != "summoner" && t != "caster"),
                "{}: a summoner is of some other tribe too",
                def.id
            );
            assert!(def.base_mana > 0, "{}: summoning costs mana", def.id);
        }
    }
}

#[test]
fn the_summoners_keep_to_the_roster() {
    let content = Content::builtin().expect("content");
    let guideline = content.experience().guideline();
    for id in ROSTER {
        let c = content
            .creature_id(id)
            .unwrap_or_else(|| panic!("{id}: no such type"));
        let def = content.creature(c);
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        assert_eq!(def.combat, Combat::Caster, "{id}: a caster");
        assert!(def.tags.iter().any(|t| t == "caster"), "{id}: caster tag");
        assert!(!def.unique, "{id}: not unique");
        assert!(def.gear.racial_weapon.is_some(), "{id}: armed");
        let level = f64::from(def.level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(level);
        assert!(
            ((hp - want) / want * 100.0).abs() <= TOLERANCE,
            "{id}: max HP {hp}, the formula {want:.1}"
        );
        let summon = def.summon.as_ref().expect("a summoning");
        assert_eq!(summon.cooldown, 30, "{id}: half a minute");
        // What it calls is never above it, but for the goblin wizard's hobgoblin.
        let top = (summon.pool.iter())
            .map(|&p| content.creature(p).level)
            .max()
            .unwrap_or(0);
        assert!(top <= def.level + 1, "{id}: calls nothing far above itself");
    }
}
