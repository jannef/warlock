//! Creature rarities (`assets/data/creature_rarities.toml`, spec §7 "Danger levels"): how often a
//! creature type is drawn for a level's population or a spawn, beside its level. A tier's
//! `factor` (in percent of a common type's) multiplies the type's weight in a draw
//! ([`super::danger::weight`]): the dangerous and rewarding types are fewer.
//!
//! Every creature type has a rarity, [`COMMON`] unless it names another. Without the file there
//! is only [`CreatureRarity::common`].

use super::ContentError;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The rarity of every creature type that names none; the file must have it.
pub const COMMON: &str = "common";
/// A tier's factor, in percent of a common type's weight.
pub const FACTOR: RangeInclusive<u16> = 1..=1000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RarityFile {
    rarity: Vec<CreatureRarity>,
}

/// A rarity tier, in file order.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureRarity {
    pub id: String,
    /// Its name, as the wiki shows it ("Very rare").
    pub name: String,
    /// What a type's weight in a draw is multiplied by, in percent (100: as a common one).
    pub factor: u16,
}

impl CreatureRarity {
    /// The one tier there is without `data/creature_rarities.toml`: common, at full weight.
    pub fn common() -> Self {
        Self {
            id: COMMON.into(),
            name: "Common".into(),
            factor: 100,
        }
    }
}

/// Parses `data/creature_rarities.toml`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<CreatureRarity>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: RarityFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut tiers: Vec<CreatureRarity> = Vec::new();
    for tier in parsed.rarity {
        if tiers.iter().any(|t| t.id == tier.id) {
            return Err(err(format!("duplicate rarity id `{}`", tier.id)));
        }
        if !FACTOR.contains(&tier.factor) {
            return Err(err(format!(
                "rarity `{}`: factor {} is out of range ({} to {})",
                tier.id,
                tier.factor,
                FACTOR.start(),
                FACTOR.end()
            )));
        }
        tiers.push(tier);
    }
    if !tiers.iter().any(|t| t.id == COMMON) {
        return Err(err(format!(
            "a rarity with id `{COMMON}` is required: it is the rarity of whatever names none"
        )));
    }
    Ok(tiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rarities_parse_in_file_order() {
        let text = "[[rarity]]\nid = \"common\"\nname = \"Common\"\nfactor = 100\n\
                    [[rarity]]\nid = \"rare\"\nname = \"Rare\"\nfactor = 25\n";
        let tiers = parse("creature_rarities.toml", text).expect("valid");
        assert_eq!(tiers[0], CreatureRarity::common());
        assert_eq!((tiers[1].id.as_str(), tiers[1].factor), ("rare", 25));
    }

    #[test]
    fn bad_rarities_are_refused() {
        let common = "[[rarity]]\nid = \"common\"\nname = \"Common\"\nfactor = 100\n";
        for (text, why) in [
            (
                common.replace("\"common\"", "\"rare\""),
                "`common` is required",
            ),
            (format!("{common}{common}"), "duplicate rarity id `common`"),
            (common.replace("100", "0"), "factor 0 is out of range"),
            (common.replace("100", "1001"), "factor 1001 is out of range"),
            (
                common.replace("factor = 100\n", ""),
                "missing field `factor`",
            ),
            (format!("{common}weight = 3\n"), "unknown field `weight`"),
        ] {
            let e = parse("creature_rarities.toml", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
