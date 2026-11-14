//! The fortune talents (#1495): what a talent does to the holder's wealth and standing: the
//! stipend, investments, quest gold, prices, gems, loot, what is handed down, and who trusts it.
//! [`FortuneEffect`] is the data (a talent's `[[effect]]` of one of [`FortuneEffect::KINDS`]),
//! [`FortuneMods`] the sum every rule reads (`TalentMods::fortune`). A new effect of the area is a
//! variant here, its field and its arm in [`FortuneMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/fortune.rs`, and its one read.

use serde::Deserialize;
use std::collections::BTreeMap;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FortuneEffect {
    /// Each level reached adds to the family's stipend, which its servant hands over (Stipend:
    /// `data/rules/stipend.toml`, `world/classes/stipend.rs`).
    Stipend,
    /// The stipend's gold is `gold` percent more, its loot roll made by `chance` percent more
    /// (percentage points), and its magic and rare rarities `magic` percent more likely (Landed
    /// noble).
    StipendMore { gold: i32, chance: i32, magic: i32 },
    /// A business partner in town invests the gold put with them (Invest: `data/rules/invest.toml`,
    /// `world/classes/invest.rs`).
    Invest,
    /// A patron in town commissions graded work from its trades and advances the stock (Patron:
    /// `data/rules/patron.toml`, `world/crafting/patron.rs`).
    Patron,
    /// Loot it finds (what generated levels hold, what creatures drop) rolls magic and rare this
    /// percent more often: their rarity weights (Treasure hunter).
    LootMagicPercent { value: i32 },
    /// The gold a quest gives, this percent more.
    QuestGoldPercent { value: i32 },
    /// Prices move this many percent more in its favour (spec §7 "Shops"), before the cap.
    FavourPercent { value: i32 },
    /// Item descriptions show what each gem in a thing is worth.
    GemWorth,
    /// A magic item it has carried this many hours since it held the effect is identified.
    IdentifyCarried { hours: u32 },
    /// Loot it finds and creatures drop is a loose gem this percent more often (Hoard sense):
    /// the gems' chance, never above certain.
    GemFindPercent { value: i32 },
    /// Handed out once, as it is taken (a talent only for character creation: Family wealth):
    /// gold coins, `percent` of those the character carries more, at least `at_least` and at
    /// most `at_most` (`world/character/heirloom.rs`).
    StartingGold {
        percent: u32,
        at_least: u32,
        at_most: u32,
    },
    /// Handed out once, as it is taken (Family wealth): `min` to `max` loose gems found at item
    /// level `level`, drawn from the stream `character/family-wealth/<race>,<gender>,<class>`.
    StartingGems { min: u8, max: u8, level: u8 },
    /// Handed out once, as it is taken (Father's sword): the race's heirloom weapon (`weapons`,
    /// by race id; `fallback` for a race without one), magic with `min` to `max` affixes rolled at
    /// item level `level` and identified, from the stream
    /// `character/fathers-sword/<race>,<gender>,<class>`, of `quality` if given; wielded in the
    /// main hand when it is free, else carried.
    Heirloom {
        weapons: BTreeMap<String, String>,
        fallback: String,
        min: u8,
        max: u8,
        level: u8,
        /// Its make (`quality = "masterwork"`), so its capacity holds its affixes
        /// (docs/dev/enchanting.md); none: plain.
        #[serde(default)]
        quality: Option<crate::magic::Quality>,
    },
    /// No one makes way for it (Trusted by none), whatever else would have them.
    Distrusted,
    /// The class active Merchant's eye: a stack the player carries is identified, for the time of
    /// a reading (`world/classes/merchants_eye.rs`; spec §8 "Class abilities").
    MerchantsEye,
}

impl FortuneEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`FortuneEffect`]).
    pub const KINDS: [&'static str; 15] = [
        "distrusted",
        "favour_percent",
        "gem_find_percent",
        "gem_worth",
        "heirloom",
        "identify_carried",
        "invest",
        "loot_magic_percent",
        "merchants_eye",
        "patron",
        "quest_gold_percent",
        "starting_gems",
        "starting_gold",
        "stipend",
        "stipend_more",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FortuneMods {
    /// Each level reached adds to the family's stipend (Stipend, `world/classes/stipend.rs`).
    pub stipend: bool,
    /// Percent more gold the stipend grows by (Landed noble).
    pub stipend_gold_percent: i32,
    /// Percentage points more chance of the stipend's loot roll (Landed noble).
    pub stipend_chance: i32,
    /// Percent more likely magic and rare finds are in the stipend's loot roll (Landed noble).
    pub stipend_magic_percent: i32,
    /// A business partner in town invests for it (Invest, `world/classes/invest.rs`).
    pub invest: bool,
    /// A patron in town commissions work from it (Patron, `world/crafting/patron.rs`).
    pub patron: bool,
    /// Percent more often loot it finds rolls magic and rare.
    pub loot_magic_percent: i32,
    /// Percent more gold from quests.
    pub quest_gold_percent: i32,
    /// Percent more favour in shops, before the cap.
    pub favour_percent: i32,
    /// Item descriptions show its gems' worth.
    pub gem_worth: bool,
    /// The hours a magic item must be carried to be identified, if it has the effect (the
    /// fewest counts).
    pub identify_carried: Option<u32>,
    /// Percent more often its finds are loose gems.
    pub gem_find_percent: i32,
    /// No one makes way for it.
    pub distrusted: bool,
    /// The active Merchant's eye: a stack it carries identified, for the time of a reading.
    pub merchants_eye: bool,
}

impl FortuneMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, effect: FortuneEffect) {
        match effect {
            FortuneEffect::Stipend => self.stipend = true,
            FortuneEffect::StipendMore {
                gold,
                chance,
                magic,
            } => {
                self.stipend_gold_percent += gold;
                self.stipend_chance += chance;
                self.stipend_magic_percent += magic;
            }
            FortuneEffect::Invest => self.invest = true,
            FortuneEffect::Patron => self.patron = true,
            // Handed out as the talent is taken (`World::take_talent`), never read after.
            FortuneEffect::StartingGold { .. }
            | FortuneEffect::StartingGems { .. }
            | FortuneEffect::Heirloom { .. } => {}
            FortuneEffect::LootMagicPercent { value } => self.loot_magic_percent += value,
            FortuneEffect::QuestGoldPercent { value } => self.quest_gold_percent += value,
            FortuneEffect::FavourPercent { value } => self.favour_percent += value,
            FortuneEffect::GemWorth => self.gem_worth = true,
            FortuneEffect::IdentifyCarried { hours } => {
                self.identify_carried = Some(self.identify_carried.map_or(hours, |h| h.min(hours)));
            }
            FortuneEffect::GemFindPercent { value } => self.gem_find_percent += value,
            FortuneEffect::Distrusted => self.distrusted = true,
            FortuneEffect::MerchantsEye => self.merchants_eye = true,
        }
    }
}
