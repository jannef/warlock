//! The summoners (#520): every type with a summoning carries the `summoner` tag and one of
//! another tribe, and every `summoner` summons.

use super::Content;

/// The summoners of #520 by id.
const ROSTER: &[&str] = &["goblin_wizard", "necromancer", "dark_wizard"];

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
