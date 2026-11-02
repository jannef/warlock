//! The beasts (#839, `docs/dev/creatures/beasts.md`): the wild animals the druid's calls bring,
//! every type tagged `beast`. Wildlife (neutral to everyone, so never a stray), on the XP
//! guideline, no unique, and each at the numbers of the druid design's table (§7.3).

use super::Content;
use crate::stats::Stat;

/// Every beast: its id, level, max hit points (Toughness + `base_hp`), and attack dice.
const BEASTS: &[(&str, u8, i32, &str)] = &[("bear", 12, 60, "2d6"), ("wolf", 4, 16, "1d6")];

#[test]
fn the_beasts_keep_to_their_table() {
    let content = Content::builtin().expect("content");
    let wildlife = content.factions().id("wildlife").expect("the wildlife");
    let guideline = content.experience().guideline();
    let mut seen: Vec<&str> = Vec::new();
    for creature in content.tagged("beast") {
        let def = content.creature(creature);
        let id = def.id.as_str();
        let &(_, level, hp, dice) = (BEASTS.iter())
            .find(|(b, ..)| *b == id)
            .unwrap_or_else(|| panic!("{id}: give it a row in BEASTS"));
        seen.push(id);
        assert_eq!(def.faction, Some(wildlife), "{id}: wildlife");
        assert!(!content.may_stray(creature), "{id}: never a stray");
        assert!(!def.unique, "{id}: no unique");
        assert_eq!(def.level, level, "{id}: level");
        assert_eq!(
            def.xp,
            guideline.xp(def.level),
            "{id}: off the XP guideline"
        );
        let max = i32::from(def.stats.score(Stat::Toughness)) + def.base_hp;
        assert_eq!(max, hp, "{id}: max hit points");
        let attack = def.attack.as_ref().expect("an attack");
        assert_eq!(attack.attack.damage.to_string(), dice, "{id}: attack");
        assert!(def.depart.is_some(), "{id}: its own departure words");
    }
    seen.sort_unstable();
    let listed: Vec<&str> = BEASTS.iter().map(|b| b.0).collect();
    assert_eq!(seen, listed, "every row a beast");
}
