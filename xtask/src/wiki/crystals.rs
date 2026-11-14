//! Magic crystals (docs/design/enchanting.md §5, §10): the colours with their natures, the
//! creatures whose nature each is, and the families an enchanter may roll with it; and the
//! numbers of consuming one and of catching them in salvage. From `data/crystals.toml` and
//! `data/enchanting.toml`.

use super::util::{Term, anchored, capital};
use std::fmt::Write as _;
use warlock_core::combat::{FUMBLE, HIGHEST};
use warlock_core::content::Content;

/// Each colour in its order: its nature, the [creature tags](creatures.md) whose nature it is (in the
/// order they are matched), and its families in words, by name.
pub(super) fn crystal_colours(content: &Content) -> String {
    let Some(crystals) = content.crystals() else {
        return "There are no magic crystals.\n".to_string();
    };
    let mut out =
        String::from("| Crystal | Nature | Creatures tagged | Families |\n|---|---|---|---|\n");
    for c in &crystals.colours {
        let tags: Vec<String> = c.tags.iter().map(|t| format!("`{t}`")).collect();
        let mut words: Vec<&str> = (c.families.iter())
            .filter_map(|f| crystals.family_word(f))
            .collect();
        words.sort_by_key(|w| w.trim_start_matches("the ").to_lowercase());
        let name = format!("{} crystal", capital(&c.name));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            anchored(Term::Crystal, &c.id, &name),
            c.nature,
            tags.join(", "),
            words.join(", ")
        );
    }
    out
}

/// The numbers: what a crystal consumed gives, salvage's affix check, and the rare materials'
/// chance in salvage.
pub(super) fn crystal_rules(content: &Content) -> String {
    let (Some(crystals), Some(rules)) = (content.crystals(), content.enchanting()) else {
        return "There are no magic crystals.\n".to_string();
    };
    let rare: Vec<String> = (crystals.rare.materials.iter())
        .map(|&m| content.object(m).describe(1))
        .collect();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "- **Consuming** a crystal takes a turn and gives {} mana (never past your maximum), {} \
         Willpower [training marks](stats.md#raising-your-stats) (an average score of 10 needs {} for its \
         next point), and a step of {} toward evil, which shrinks the more evil you already are.",
        crystals.consume_mana,
        crystals.consume_marks,
        content.stat_training().base_price(10),
        content
            .alignment()
            .map_or(0, |a| a.shift.consume_crystal)
            .abs(),
    );
    let _ = writeln!(
        out,
        "- **Salvage** checks each affix of a magic item, identified or not: 2d12 plus your Willpower \
         modifier, what talents add to Willpower checks, and what Enchanting gives to its craft, \
         against {} plus one for every {} tiers of the affix (a tier-12 affix is +{}). A natural \
         {FUMBLE} always fails, a natural {HIGHEST} always succeeds.",
        rules.salvage_difficulty,
        rules.salvage_tier_step,
        12 / rules.salvage_tier_step,
    );
    let _ = writeln!(
        out,
        "- **Rare materials**: salvaging a rare item of item level {} or more has a \
         {}% chance of leaving {} (even odds).",
        crystals.rare.salvage_level,
        crystals.rare.salvage_chance,
        rare.join(" or "),
    );
    let yields: Vec<String> = (crystals.yield_levels.iter())
        .enumerate()
        .map(|(i, level)| format!("{} from level {level}", i + 2))
        .collect();
    let _ = writeln!(
        out,
        "- **[Soulglass](magic.md#spell-soulglass)** leaves 1 crystal, {}. A creature of level {} \
         or more has a {}% chance of leaving {} as well (even odds); a dragon always leaves {}.",
        yields.join(", "),
        crystals.rare.level,
        crystals.rare.chance,
        rare.join(" or "),
        content.object(crystals.rare.dragon).describe(1),
    );
    out
}
