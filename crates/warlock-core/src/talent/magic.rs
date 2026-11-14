//! The magic talents (#1495): what a talent does to the holder's mana and spells: regeneration,
//! costs, the spells given and their hooks, hexes, scrolls, and potions. [`MagicEffect`] is the
//! data (a talent's `[[effect]]` of one of [`MagicEffect::KINDS`]), [`MagicMods`] the sum every
//! rule reads (`TalentMods::magic`). A new effect of the area is a variant here, its field and its
//! arm in [`MagicMods::add`] here, its wiki text in `xtask/src/wiki/talents/magic.rs`, and its one
//! read.

use super::{MAX_MANA_CUT, TalentMods, per_levels_at};
use crate::content::Content;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MagicEffect {
    /// Mana comes back this percent faster (negative: slower; Tidal), on
    /// [`crate::world::World::mana_rate`]'s pace.
    ManaRegenPercent { value: i32 },
    /// Maximum mana: 1 per `per_levels` character levels, at least `at_least`, handed out into
    /// the actor's base mana when the talent is taken and as each later level comes, as
    /// `max_hp_per_levels` does hit points (Deep well).
    MaxManaPerLevels { per_levels: u8, at_least: i32 },
    /// Maximum mana this percent more (negative: less, never below 0; Fell-minded), in
    /// [`crate::world::Actor::max_mana`], as `max_hp_percent` is for hit points.
    MaxManaPercent { value: i32 },
    /// Resting brings mana back this percent faster on top of resting's own pace (Meditation).
    ManaRestingPercent { value: i32 },
    /// What abilities cost in mana, this percent more (negative: less; never below 1; Frugal
    /// caster), in [`crate::world::World::spend_mana`].
    ManaCostPercent { value: i32 },
    /// What rods cost in mana, this percent more (negative: less; never below 1; Channeler; spec
    /// §8 "Magic items": "Rods"), before [`TalentEffect::ManaCostPercent`].
    RodCostPercent { value: i32 },
    /// The active Blood magic: give `hp` hit points (never the last) for `mana` mana.
    BloodMagic { hp: i32, mana: i32 },
    /// Reading a scroll takes this percent more time (negative: less; Scroll savant).
    ReadPercent { value: i32 },
    /// Studying a spell (spec §8 "Spells"): `bonus` points off the check's difficulty, it
    /// takes `percent` more time (negative: less; Quick study), and its overreach counts
    /// `overreach_percent` more in the check (negative: less, rounded down; Overreach, the
    /// spell scaling design §7.2). Each defaults to 0.
    Study {
        #[serde(default)]
        bonus: i32,
        #[serde(default)]
        percent: i32,
        #[serde(default)]
        overreach_percent: i32,
    },
    /// A scroll it reads is not used up, by this chance in percent (one gameplay draw a read;
    /// Scroll savant).
    ScrollKept { chance: u8 },
    /// A potion it drinks is not used up, by this chance in percent (one gameplay draw a drink;
    /// Thrifty drinker).
    PotionKept { chance: u8 },
    /// Its own bursts never catch it (Scroll savant).
    SparesSelf,
    /// The class active Cackle: a voice `noise` loud; every hex of the player's on a creature
    /// they see grows `degrees` and starts its degree afresh, to its status's `max`
    /// (`world/witch.rs`).
    Cackle { degrees: u16, noise: u16 },
    /// A caster's spell (design §4.1 B2, spec §8 "Spells"): its holder has the spell ability
    /// `spell`, the spell of that id in `data/spells/` (a class ability's; each given once).
    Spell { spell: String },
    /// What it does to its spells through the six hooks (spec §8 "Spells": "Hooks";
    /// `content/spell_mod.rs`): rungs, dice, rerolls, riders, reach, degrees, and a cost
    /// percent, for every spell or those of its `when` tags (a talent chosen for a school or an
    /// element: those of the choice). Empower, Archmage, Keen mind, School focus, Scroll
    /// adept, Wand mastery, Sure hand, Brimming.
    SpellMod(crate::content::SpellModSpec),
    /// What it does to a spell type's own rule (`[types]` in `data/spells.toml`) for the damage
    /// types in `when`: `arc_chance`, the percent its spells of them arc at, read in
    /// `world/spell_arc.rs` (Conductor).
    SpellType { arc_chance: u8, when: Vec<String> },
    /// Its spells cost this percent more mana (negative: less; Archmage, Empower), at least 1.
    SpellManaPercent { value: i32 },
    /// When a creature dies under a hex its holder laid (`Content::hex_statuses`), whoever
    /// killed it, the holder's mana rises by the dead creature's level divided by `per_levels`,
    /// at least 1, never past its maximum (Malison; `world/malison.rs`).
    HexTithe { per_levels: u8 },
    /// The class active Knock: for `mana` mana, the lock of the locked door or chest in a
    /// direction springs open (`world/knock.rs`; spec §8 "Chests").
    Knock { mana: i32 },
    /// The class active Scribe: a scroll of a spell the player knows written on a blank scroll,
    /// for `mana_times` the spell's mana and a study's time (`world/scribe.rs`; spec §8
    /// "Scrolls").
    Scribe { mana_times: u8 },
}

impl MagicEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`MagicEffect`]).
    pub const KINDS: [&'static str; 20] = [
        "blood_magic",
        "cackle",
        "hex_tithe",
        "knock",
        "mana_cost_percent",
        "mana_regen_percent",
        "mana_resting_percent",
        "max_mana_per_levels",
        "max_mana_percent",
        "potion_kept",
        "read_percent",
        "rod_cost_percent",
        "scribe",
        "scroll_kept",
        "spares_self",
        "spell",
        "spell_mana_percent",
        "spell_mod",
        "spell_type",
        "study",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MagicMods {
    /// Percent faster mana comes back.
    pub mana_regen_percent: i32,
    /// Mana by level, one entry per talent: `(per_levels, at_least)`
    /// ([`MagicMods::mana_at`]).
    pub mana_per_levels: Vec<(u8, i32)>,
    /// Percent more maximum mana (negative: less).
    pub max_mana_percent: i32,
    /// Percent faster resting brings mana back, on top of resting's own pace.
    pub mana_resting_percent: i32,
    /// Percent more abilities cost in mana (negative: less; [`MagicMods::mana_cost`]).
    pub mana_cost_percent: i32,
    /// Percent more mana a rod costs (negative: less; Channeler).
    pub rod_cost_percent: i32,
    /// What it does to its spells through the six hooks (spec §8 "Spells": "Hooks"), one entry
    /// per `spell_mod` effect, each with its tag filter
    /// ([`MagicMods::spell_mods`]).
    pub spell: Vec<crate::content::SpellMod>,
    /// The percent chance its spells of a damage type arc at, one entry per type a `spell_type`
    /// effect names ([`MagicMods::arc_chance`]; Conductor).
    pub arc_chance: Vec<(crate::content::DamageTypeId, u8)>,
    /// The active Blood magic, if it has it.
    pub blood_magic: Option<BloodMagic>,
    /// Percent more time reading a scroll takes (negative: less).
    pub read_percent: i32,
    /// The chance in percent a scroll it reads is not used up (the most counts).
    pub scroll_kept: u8,
    /// The chance in percent a potion it drinks is not used up (the most counts).
    pub potion_kept: u8,
    /// Its own bursts never catch it.
    pub spares_self: bool,
    /// Points off the difficulty of its studies, and percent more time they take (negative:
    /// less; Quick study, spec §8 "Spells").
    pub study_bonus: i32,
    pub study_percent: i32,
    /// Percent more a study's overreach counts in its check (negative: less; Overreach).
    pub study_overreach_percent: i32,
    /// The active Cackle, if it has it.
    pub cackle: Option<Cackle>,
    /// The caster's spells it has (design §4.1 B2), in the order taken.
    pub spells: Vec<crate::content::SpellId>,
    /// Percent more mana its spells cost (negative: less).
    pub spell_mana_percent: i32,
    /// Malison's tithe, if it has it: a creature dying under its hex gives it the creature's
    /// level divided by this in mana, at least 1 (`world/malison.rs`).
    pub hex_tithe: Option<u8>,
    /// The active Knock, if it has it.
    pub knock: Option<Knock>,
    /// The active Scribe, if it has it: how many times a spell's mana a scroll of it costs.
    pub scribe: Option<u8>,
}

impl MagicMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: MagicEffect, choice: Option<&str>) {
        match effect {
            MagicEffect::ManaRegenPercent { value } => self.mana_regen_percent += value,
            MagicEffect::MaxManaPercent { value } => self.max_mana_percent += value,
            MagicEffect::MaxManaPerLevels {
                per_levels,
                at_least,
            } => self.mana_per_levels.push((per_levels, at_least)),
            MagicEffect::ManaRestingPercent { value } => self.mana_resting_percent += value,
            MagicEffect::ManaCostPercent { value } => self.mana_cost_percent += value,
            MagicEffect::RodCostPercent { value } => self.rod_cost_percent += value,
            MagicEffect::BloodMagic { hp, mana } => {
                self.blood_magic = Some(BloodMagic { hp, mana });
            }
            MagicEffect::ReadPercent { value } => self.read_percent += value,
            MagicEffect::Study {
                bonus,
                percent,
                overreach_percent,
            } => {
                self.study_bonus += bonus;
                self.study_percent += percent;
                self.study_overreach_percent += overreach_percent;
            }
            MagicEffect::ScrollKept { chance } => self.scroll_kept = self.scroll_kept.max(chance),
            MagicEffect::PotionKept { chance } => self.potion_kept = self.potion_kept.max(chance),
            MagicEffect::SparesSelf => self.spares_self = true,
            MagicEffect::Spell { spell } => {
                if let Some(spell) = content.spell_id(&spell)
                    && !self.spells.contains(&spell)
                {
                    self.spells.push(spell);
                }
            }
            MagicEffect::SpellMod(spec) => self.spell.push(spec.resolve(content, choice)),
            MagicEffect::SpellType { arc_chance, when } => {
                let kinds = when.iter().filter_map(|id| content.damage_type_id(id));
                self.arc_chance.extend(kinds.map(|k| (k, arc_chance)));
            }
            MagicEffect::SpellManaPercent { value } => self.spell_mana_percent += value,
            MagicEffect::HexTithe { per_levels } => self.hex_tithe = Some(per_levels),
            MagicEffect::Cackle { degrees, noise } => {
                self.cackle = Some(Cackle { degrees, noise });
            }
            MagicEffect::Knock { mana } => self.knock = Some(Knock { mana }),
            MagicEffect::Scribe { mana_times } => self.scribe = Some(mana_times),
        }
    }

    /// Whether it has the caster's spell `spell` (spec §8 "Class abilities").
    pub fn has_spell(&self, spell: crate::content::SpellId) -> bool {
        self.spells.contains(&spell)
    }

    /// What its talents do to a spell with `tags` (spec §8 "Spells": "Hooks"): every
    /// `spell_mod` whose filter the spell fits, summed.
    pub fn spell_mods(&self, tags: &[&str]) -> crate::content::SpellMods {
        let mut sum = crate::content::SpellMods::default();
        for one in self.spell.iter().filter(|m| m.fits(tags)) {
            sum.add(&one.mods);
        }
        sum
    }

    /// The chance its spell of damage type `kind` arcs at, by its talents: the highest any gives
    /// (0: none; the type's own stands then).
    pub fn arc_chance(&self, kind: crate::content::DamageTypeId) -> u8 {
        (self.arc_chance.iter())
            .filter(|(k, _)| *k == kind)
            .map(|&(_, chance)| chance)
            .max()
            .unwrap_or(0)
    }

    /// The mana one of its spells that costs `mana` costs it: scaled by `spell_mana_percent`,
    /// at least 1.
    pub fn spell_mana(&self, mana: u16) -> i32 {
        self.spell_mana_with(mana, 0)
    }

    /// [`MagicMods::spell_mana`] with `more` percent besides (School focus's, for a spell of
    /// its tag), added to `spell_mana_percent`.
    pub fn spell_mana_with(&self, mana: u16, more: i32) -> i32 {
        let scaled = TalentMods::scaled(u64::from(mana), self.spell_mana_percent + more);
        i32::try_from(scaled).unwrap_or(i32::MAX).max(1)
    }

    /// The mana the per-level effects give at character `level` (Deep well), as
    /// [`VigourMods::hp_at`] does hit points.
    pub fn mana_at(&self, level: u8) -> i32 {
        per_levels_at(&self.mana_per_levels, level)
    }

    /// What an ability whose cost is `base` mana costs with these talents and `worn_off` percent
    /// less from magic items (of Thrift's `mana_cost`): `mana_cost_percent` more (Frugal
    /// caster's −20: a fifth less) and the items' less, the two summed before the one rounding
    /// and never more than [`MAX_MANA_CUT`] off; rounded to the nearest, never below 1 for a
    /// cost above 0.
    pub fn mana_cost(&self, base: i32, worn_off: i32) -> i32 {
        let percent = (self.mana_cost_percent - worn_off.max(0)).max(-MAX_MANA_CUT);
        if base <= 0 || percent == 0 {
            return base;
        }
        let scaled = (base * (100 + percent) + 50) / 100;
        scaled.max(1)
    }

    /// What a rod whose cost is `base` mana costs with these talents, before
    /// [`MagicMods::mana_cost`]: `rod_cost_percent` more (Channeler's −25: a quarter less),
    /// rounded to the nearest, never below 1.
    pub fn rod_cost(&self, base: i32) -> i32 {
        if base <= 0 || self.rod_cost_percent == 0 {
            return base;
        }
        let scaled = (base * (100 + self.rod_cost_percent).max(0) + 50) / 100;
        scaled.max(1)
    }
}

/// The active Cackle (spec §8 "Class abilities"; `world/witch.rs`): how many degrees it adds
/// to each hex of its holder's in sight, and how loud it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cackle {
    pub degrees: u16,
    pub noise: u16,
}

/// The active Knock (spec §8 "Class abilities", "Chests"): its mana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knock {
    pub mana: i32,
}

/// The active Blood magic (spec §8 "Talents"): `hp` hit points given for `mana` mana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BloodMagic {
    pub hp: i32,
    pub mana: i32,
}
