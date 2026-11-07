//! The chest bands (`assets/data/chests.toml`, spec §7 "Chests"; [`crate::chest`]): ten
//! `[[band]]`s from the coffer to the gilded ark, each from a least level up, and `[smash]`.
//!
//! A chest's band is the last whose `from` its level reaches ([`ChestRules::band_of`]): it names
//! the chest, colours it, and gives how sturdy it is against a kick (a door's scale), how hard
//! against a blast (a wall's scale), and, for generated chests (later slices), how often one is
//! locked, trapped, or a mimic, a mimic's hold, and the loot table. A load checks only that
//! the ids are unique and the `from`s rise. Without the file there are no bands: a chest is then named by its terrain.

use super::ContentError;
use crate::color::Rgb;
use crate::dice::Dice;
use serde::Deserialize;

/// A band of chests: what chests of its levels are called and like.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChestBand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub fg: Rgb,
    /// The least level of the band.
    pub from: u8,
    /// A kick breaks it with a chance of the kicker's Strength in this.
    pub sturdiness: u8,
    /// What a blast must beat to blow it apart.
    pub hardness: u16,
    /// Percent of generated chests of the band that come locked.
    pub locked: u8,
    /// Percent that come trapped.
    pub trapped: u8,
    /// Percent that are a mimic instead.
    pub mimic: u8,
    /// A mimic of the band holds with this chance on a hit, in percent.
    pub hold: u8,
    /// What a generated chest of the band holds.
    pub loot: ChestLoot,
}

/// A band's loot table (spec §8 "Chest loot"; rolled by a later slice).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChestLoot {
    /// How many items.
    pub items: Dice,
    /// Levels above the floor's the items and gems are rolled at.
    pub loot_level: u8,
    /// Percent of items at least magic (the rare ones counted).
    pub magic: u8,
    /// Percent of items rare.
    pub rare: u8,
    /// A purse of coins.
    pub gold: ChestGold,
    /// How many loose gems (0 or less: none).
    pub gems: Dice,
}

/// A loot table's purse: `dice` × `each` gold coins.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChestGold {
    pub dice: Dice,
    pub each: u32,
}

/// The chest rules of `data/chests.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChestRules {
    /// The bands, least first.
    pub bands: Vec<ChestBand>,
    /// Percent of each stack destroyed when a chest is kicked open or blown apart.
    pub destroyed: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChestFile {
    band: Vec<ChestBand>,
    smash: SmashSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SmashSpec {
    destroyed: u8,
}

impl ChestRules {
    /// Parses and checks `data/chests.toml`.
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: ChestFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (i, band) in file.band.iter().enumerate() {
            band.check(i, &file.band)
                .map_err(|m| err(format!("band `{}`: {m}", band.id)))?;
        }
        Ok(Self {
            bands: file.band,
            destroyed: file.smash.destroyed,
        })
    }

    /// The band of a chest of `level`: the last whose `from` it reaches (the first below
    /// them all). `None` without bands.
    pub fn band_of(&self, level: u8) -> Option<&ChestBand> {
        self.band_index(level).map(|i| &self.bands[i])
    }

    /// The index of [`ChestRules::band_of`] in [`ChestRules::bands`].
    pub fn band_index(&self, level: u8) -> Option<usize> {
        if self.bands.is_empty() {
            return None;
        }
        Some(
            self.bands
                .iter()
                .rposition(|b| b.from <= level)
                .unwrap_or(0),
        )
    }

    /// The band with id `id`.
    pub fn band(&self, id: &str) -> Option<&ChestBand> {
        self.bands.iter().find(|b| b.id == id)
    }
}

impl ChestBand {
    /// Checks band number `i` of `all`: its id not used twice ([`ChestRules::band`] would find
    /// the first) and its `from` above the band before it ([`ChestRules::band_index`] reads them
    /// in order).
    fn check(&self, i: usize, all: &[ChestBand]) -> Result<(), String> {
        if all.iter().filter(|b| b.id == self.id).count() > 1 {
            return Err("the id is used twice".into());
        }
        if i > 0 && self.from <= all[i - 1].from {
            return Err("`from` must rise from band to band".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn band(id: &str, from: u8) -> String {
        format!(
            "[[band]]\nid = \"{id}\"\nname = \"{id}\"\ndescription = \"A box.\"\nfg = \"#808080\"\n\
             from = {from}\nsturdiness = 20\nhardness = 200\nlocked = 60\ntrapped = 25\n\
             mimic = 5\nhold = 10\nloot = {{ items = \"1d3\", loot_level = 0, magic = 10, \
             rare = 1, gold = {{ dice = \"2d6\", each = 1 }}, gems = \"1d4-3\" }}\n"
        )
    }

    fn parse(text: &str) -> Result<ChestRules, ContentError> {
        ChestRules::parse("data/chests.toml", text)
    }

    #[test]
    fn the_shipped_bands_parse_and_rise() {
        let text = include_str!("../../../../assets/data/chests.toml");
        let rules = parse(text).expect("data/chests.toml");
        assert_eq!(rules.bands.len(), 10);
        assert_eq!(rules.destroyed, 50);
        assert_eq!(rules.bands[0].id, "coffer");
        assert_eq!(rules.bands[9].id, "gilded_ark");
        assert!(rules.bands.windows(2).all(|w| w[0].from < w[1].from));
        assert!(
            rules
                .bands
                .windows(2)
                .all(|w| w[0].sturdiness < w[1].sturdiness)
        );
    }

    #[test]
    fn a_level_falls_in_the_last_band_it_reaches() {
        let text = include_str!("../../../../assets/data/chests.toml");
        let rules = parse(text).expect("data/chests.toml");
        let id = |level| rules.band_of(level).map(|b| b.id.as_str());
        assert_eq!(id(1), Some("coffer"));
        assert_eq!(id(2), Some("coffer"));
        assert_eq!(id(3), Some("chest"));
        assert_eq!(id(7), Some("hutch"));
        assert_eq!(id(10), Some("banded_chest"));
        assert_eq!(id(12), Some("strongbox"));
        assert_eq!(id(60), Some("gilded_ark"));
        assert_eq!(id(0), Some("coffer"), "below every band: the first");
        assert_eq!(ChestRules::default().band_of(5), None);
    }

    #[test]
    fn bad_bands_are_refused() {
        let smash = "[smash]\ndestroyed = 50\n";
        let ok = format!("{}{}{smash}", band("a", 1), band("b", 3));
        assert!(parse(&ok).is_ok());
        let odd = format!("{}[smash]\ndestroyed = 101\n", band("a", 2))
            .replace("sturdiness = 20", "sturdiness = 0")
            .replace("rare = 1", "rare = 20");
        assert!(parse(&odd).is_ok(), "the numbers are the author's");
        let falling = format!("{}{}{}{smash}", band("a", 1), band("b", 5), band("c", 4));
        assert!(parse(&falling).is_err(), "from must rise");
        let twice = format!("{}{}{smash}", band("a", 1), band("a", 3));
        assert!(parse(&twice).is_err(), "ids are unique");
    }
}
