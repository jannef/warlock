//! The movement talents (#1495): what a talent does to how the holder moves: burden, travel,
//! terrain, water and ice, and speed. [`MovementEffect`] is the data (a talent's `[[effect]]` of
//! one of [`MovementEffect::KINDS`]), [`MovementMods`] the sum every rule reads
//! (`TalentMods::movement`). A new effect of the area is a variant here, its field and its arm in
//! [`MovementMods::add`] here, its wiki text in `xtask/src/wiki/talents/movement.rs`, and its one
//! read.

use crate::content::{Content, TerrainId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MovementEffect {
    /// The carrying limit, this percent more.
    CarryPercent { value: i32 },
    /// Being Burdened costs it nothing, as a dwarf's burden (Porter).
    IgnoreBurdened,
    /// Coins it carries weigh nothing.
    WeightlessCoins,
    /// Journeys on the overworld take this percent more time (negative: quicker).
    TravelPercent { value: i32 },
    /// Wading (water, marsh) costs a plain step, and never gives a disease.
    Swimmer,
    /// A step onto any of `terrains` costs what a step onto `like` does (ids in
    /// `data/terrain/`; Swimmer, Trailblazer).
    TerrainAs { terrains: Vec<String>, like: String },
    /// A plunge through ice into the water never hurts it (Swimmer).
    PlungeProof,
    /// Ice holds it as if it carried nothing (Winter-born).
    LightOnIce,
    /// Climbing out of the water onto ice costs this percent more (negative: less).
    ClimbPercent { value: i32 },
    /// Wading never gives it a disease (Nature's ward).
    WadingProof,
    /// Wading (water, and ground a disease is caught wading in) costs it a plain step (Undertow;
    /// unlike `swimmer`, it may still catch a disease).
    WadesFree,
    /// The burden's cap on its Dexterity in Defence, this much higher.
    BurdenDex { value: i32 },
    /// Percent off the burden's slowdown, on top of what Balance takes off (60: Burdened ×1.10
    /// instead of ×1.25).
    BurdenEase { value: i32 },
    /// It moves and acts this percent quicker (its speed; the number is hidden in the game).
    SpeedPercent { value: i32 },
    /// Its steps cost this percent more (negative: cheaper), only while it wears no body armour
    /// (if `unarmoured`), is Unburdened (if `unburdened`), and holds no shield (if `no_shield`):
    /// Fleet; Headlong, while raging.
    StepPercent {
        value: i32,
        #[serde(default)]
        unarmoured: bool,
        #[serde(default)]
        unburdened: bool,
        #[serde(default)]
        no_shield: bool,
    },
    /// Strained costs it what Burdened costs others (Beast of burden).
    Stout,
}

impl MovementEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`MovementEffect`]).
    pub const KINDS: [&'static str; 16] = [
        "burden_dex",
        "burden_ease",
        "carry_percent",
        "climb_percent",
        "ignore_burdened",
        "light_on_ice",
        "plunge_proof",
        "speed_percent",
        "step_percent",
        "stout",
        "swimmer",
        "terrain_as",
        "travel_percent",
        "wades_free",
        "wading_proof",
        "weightless_coins",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MovementMods {
    /// Percent more carrying limit.
    pub carry_percent: i32,
    /// Being Burdened costs it nothing.
    pub ignore_burdened: bool,
    /// The coins it carries weigh nothing.
    pub weightless_coins: bool,
    /// Percent more time an overworld journey takes (negative: quicker).
    pub travel_percent: i32,
    /// Wading costs a plain step and gives no disease.
    pub swimmer: bool,
    /// Terrain it steps onto at another's price: `(terrain, priced as)`.
    pub terrain_as: Vec<(TerrainId, TerrainId)>,
    /// A plunge through ice never hurts it.
    pub plunge_proof: bool,
    /// Ice holds it as if it carried nothing.
    pub light_on_ice: bool,
    /// Percent more climbing out onto ice costs (negative: less).
    pub climb_percent: i32,
    /// Wading never gives it a disease.
    pub wading_proof: bool,
    /// How much higher the burden's cap on its Dexterity in Defence is.
    pub burden_dex: i32,
    /// Percent off the burden's slowdown, on top of Balance.
    pub burden_ease: i32,
    /// Percent quicker it moves and acts.
    pub speed_percent: i32,
    /// Percents more its steps cost, each while what it asks holds.
    pub step_percent: Vec<StepBonus>,
    /// Strained costs it what Burdened costs others.
    pub stout: bool,
    /// Wading costs it a plain step (Undertow).
    pub wades_free: bool,
}

impl MovementMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: MovementEffect) {
        match effect {
            MovementEffect::CarryPercent { value } => self.carry_percent += value,
            MovementEffect::IgnoreBurdened => self.ignore_burdened = true,
            MovementEffect::WeightlessCoins => self.weightless_coins = true,
            MovementEffect::TravelPercent { value } => self.travel_percent += value,
            MovementEffect::Swimmer => self.swimmer = true,
            MovementEffect::TerrainAs { terrains, like } => {
                if let Some(like) = content.terrain_id(&like) {
                    let terrains = terrains.iter().filter_map(|t| content.terrain_id(t));
                    self.terrain_as.extend(terrains.map(|t| (t, like)));
                }
            }
            MovementEffect::PlungeProof => self.plunge_proof = true,
            MovementEffect::LightOnIce => self.light_on_ice = true,
            MovementEffect::ClimbPercent { value } => self.climb_percent += value,
            MovementEffect::WadingProof => self.wading_proof = true,
            MovementEffect::BurdenDex { value } => self.burden_dex += value,
            MovementEffect::BurdenEase { value } => self.burden_ease += value,
            MovementEffect::SpeedPercent { value } => self.speed_percent += value,
            MovementEffect::WadesFree => self.wades_free = true,
            MovementEffect::StepPercent {
                value,
                unarmoured,
                unburdened,
                no_shield,
            } => self.step_percent.push(StepBonus {
                percent: value,
                unarmoured,
                unburdened,
                no_shield,
            }),
            MovementEffect::Stout => self.stout = true,
        }
    }

    /// The percent its steps cost more, while it wears body armour (`armoured`), is Burdened or
    /// worse (`burdened`), and holds a shield (`shield`) or not.
    pub fn step_percent(&self, armoured: bool, burdened: bool, shield: bool) -> i32 {
        (self.step_percent.iter())
            .filter(|b| !(b.unarmoured && armoured))
            .filter(|b| !(b.unburdened && burdened))
            .filter(|b| !(b.no_shield && shield))
            .map(|b| b.percent)
            .sum()
    }
}

/// A step price change and what it asks ([`TalentEffect::StepPercent`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepBonus {
    pub percent: i32,
    pub unarmoured: bool,
    pub unburdened: bool,
    pub no_shield: bool,
}
