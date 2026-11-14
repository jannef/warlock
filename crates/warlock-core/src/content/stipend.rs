//! The aristocrat's stipend (`assets/data/stipend.toml`, spec §8 "Class abilities": Stipend;
//! #1405): what each level-up of a character with the Stipend class ability adds to what their
//! family holds for them, until their servant hands it over (`world/stipend.rs`).
//!
//! Each level reached adds `gold.each` plus `gold.dice`, and, by `loot.chance` percent, one loot roll at
//! the level as item level, with the magic and rare rarities `loot.magic` percent more likely
//! and a loose gem `loot.gems` percent more likely (the finder's luck of
//! [`crate::magic::loot::Luck`]); what is rolled is kept when it is magic or better or of one
//! of the `loot.keep` item classes, and otherwise thrown away.
//!
//! The file is optional: without it the stipend never grows.

use super::{Content, ContentError, ItemClassId};
use crate::dice::Dice;
use serde::Deserialize;

/// The stipend's rules that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StipendRules {
    /// Gold each level reached adds.
    pub each: u32,
    /// Gold rolled on top for each level reached.
    pub dice: Dice,
    /// The percent chance a level-up makes its loot roll at all.
    pub chance: u32,
    /// Percent more likely the magic and rare rarities are in its loot roll.
    pub magic: i32,
    /// Percent more likely a loose gem is.
    pub gems: i32,
    /// The item classes kept whatever their rarity (potions, scrolls, gems).
    pub keep: Vec<ItemClassId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StipendFile {
    gold: GoldSpec,
    loot: LootSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldSpec {
    each: u32,
    dice: Dice,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LootSpec {
    chance: u32,
    magic: i32,
    gems: i32,
    #[serde(default)]
    keep: Vec<String>,
}

impl StipendRules {
    /// Parses `data/stipend.toml`: the classes kept must be known.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: StipendFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let keep = (parsed.loot.keep.iter())
            .map(|name| {
                (content.item_class_id(name))
                    .ok_or_else(|| err(format!("unknown item class `{name}`")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            each: parsed.gold.each,
            dice: parsed.gold.dice,
            chance: parsed.loot.chance.min(100),
            magic: parsed.loot.magic,
            gems: parsed.loot.gems,
            keep,
        })
    }
}

impl super::RulesFile for StipendRules {
    const PATH: &'static str = "data/stipend.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/stipend.toml";

    #[test]
    fn the_stipend_keeps_potions_scrolls_and_gems() {
        let content = crate::test_support::builtin();
        let rules = content.stipend().expect("the stipend is in the game");
        let kept: Vec<&str> = (rules.keep.iter())
            .map(|&c| content.item_class(c).id.as_str())
            .collect();
        assert_eq!(kept, ["potions", "scrolls", "gems"]);
        assert!(rules.each > 0 && rules.magic > 0);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_stipend() {
        let text = "gold = { each = 1, dice = \"1d4\" }\nloot = { chance = 50, magic = 0, gems = 0, keep = [\"nothing\"] }\n";
        let mut files = ContentFiles::builtin();
        files.insert(PATH, text);
        let err = Content::load(&files).expect_err(text);
        assert!(
            err.message.contains("unknown item class"),
            "{}",
            err.message
        );
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the stipend is optional");
        assert!(content.stipend().is_none());
    }
}
