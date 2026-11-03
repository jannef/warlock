//! The beasts (#839, #916, #929, `docs/dev/creatures/beasts.md`): the wild animals the druid's
//! calls bring, the ranger's companions, and the witch's familiars, every type tagged `beast`.
//! Wildlife (neutral to everyone, so never a stray), on the XP guideline, no unique, tameable
//! but for the familiars' own kinds (called, never tamed), and each at the numbers of the druid
//! design's table (§7.3), the companion design's (§12.2), and the familiar design's (§6.1).

use super::Content;
use crate::stats::Stat;

/// Every beast: its id, level, max hit points (Toughness + `base_hp`), attack dice, and whether
/// it may be tamed, by id.
const BEASTS: &[(&str, u8, i32, &str, bool)] = &[
    ("badger", 2, 19, "1d6", true),
    ("bear", 12, 60, "2d6", true),
    ("black_hound", 5, 24, "1d8", true),
    ("boar", 2, 20, "1d8", true),
    ("cat", 1, 10, "1d2", false),
    ("cave_lizard", 2, 16, "1d6", true),
    ("great_boar", 6, 30, "2d6", true),
    ("hare", 1, 10, "1d2", false),
    ("hound", 1, 12, "1d4", true),
    ("large_spider", 2, 13, "1d4", true),
    ("lynx", 2, 13, "1d6", true),
    ("panther", 6, 24, "1d8", true),
    ("polecat", 1, 10, "1d4", true),
    ("raven", 1, 9, "1d3", false),
    ("toad", 1, 10, "1d2", false),
    ("wolf", 4, 16, "1d6", true),
];

#[test]
fn the_beasts_keep_to_their_table() {
    let content = Content::builtin().expect("content");
    let wildlife = content.factions().id("wildlife").expect("the wildlife");
    let guideline = content.experience().guideline();
    let mut seen: Vec<&str> = Vec::new();
    for creature in content.tagged("beast") {
        let def = content.creature(creature);
        let id = def.id.as_str();
        let &(_, level, hp, dice, tameable) = (BEASTS.iter())
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
        assert_eq!(
            def.tameable, tameable,
            "{id}: tameable, or a familiar's kind"
        );
    }
    seen.sort_unstable();
    let listed: Vec<&str> = BEASTS.iter().map(|b| b.0).collect();
    assert_eq!(seen, listed, "every row a beast");
}
