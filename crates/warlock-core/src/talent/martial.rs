//! The martial talents (#1495): the warrior's, berserker's, ascetic's, and hunter's class actives
//! and what changes them: Veteran's wind, Rage, War cry, Charge, the ascetic's strikes and steps,
//! Rapid shot, Volley, and the spirit weapon. [`MartialEffect`] is the data (a talent's
//! `[[effect]]` of one of [`MartialEffect::KINDS`]), [`MartialMods`] the sum every rule reads
//! (`TalentMods::martial`). A new effect of the area is a variant here, its field and its arm in
//! [`MartialMods::add`] here, its wiki text in `xtask/src/wiki/talents/martial.rs`, and its one
//! read.

use super::SecondWind;
use crate::content::{Content, ResourceCost, ResourceId, StatusId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MartialEffect {
    /// The class active Veteran's wind ([`ClassActive::VeteransWind`]): as Second wind.
    VeteransWind {
        per_level: u8,
        at_least: i32,
        cooldown: String,
    },
    /// The class active Rage: the player is granted `resource` (Rage, spec §8 "Resources") and
    /// suffers its status (Raging) while it lasts; no rage while that or the status it `follows`
    /// into lasts.
    Rage { resource: String },
    /// The class active War cry: a roar as loud as a howl; every hostile that hears it, sees the
    /// player, and has vitals rolls Willpower against `difficulty` + the player's Strength
    /// modifier + their Intimidate; failed, it suffers `severity` degrees of `status` (one that
    /// `flees`; `world/war_cry.rs`).
    WarCry {
        status: String,
        severity: u16,
        difficulty: i32,
    },
    /// While raging: the statuses named tick nothing, and a blow that would take the last hit
    /// point leaves it for `cost` of the rage, when that much is held (Unstoppable).
    Unstoppable { statuses: Vec<String>, cost: u16 },
    /// The class active Charge (the warrior's, `world/charge.rs`): a rush along the line to a
    /// hostile the player sees, ending beside it with a blow, provoking no free blows on the
    /// way: it costs the talent's `resource` (Stamina) and `per_cell` more for every cell
    /// moved.
    Charge { per_cell: u16 },
    /// Charge also strikes every hostile beside a cell it passes, each once, a free blow (the
    /// warrior's Whirling charge, `world/charge.rs`); the one charged at gets only its own.
    WhirlingCharge,
    /// The class active Patient defence: a wait, then a degree of `status` (Poised: Dodge up
    /// and the riposte switched on until the next action; what it costs is the talent's
    /// `resource`: Ki; `world/ascetic.rs`).
    PatientDefence { status: String },
    /// The class active Quivering palm: an unarmed blow in a direction; a hit that wounds
    /// leaves the target `severity` degrees of `status` (what it costs is the talent's
    /// `resource`: Ki).
    QuiveringPalm { status: String, severity: u16 },
    /// The class active Stunning strike (the ascetic's, #848): an unarmed blow in a direction;
    /// a hit that wounds leaves the target `severity` degrees of `status` (Stunned; what it
    /// costs is the talent's `resource`: Ki). Quivering palm's blow with another status.
    StunningStrike { status: String, severity: u16 },
    /// The class active Step of the wind: a degree of `status` (Light-footed) on the player,
    /// taking no time (what it costs is the talent's `resource`: Ki; `world/ascetic.rs`).
    WindStep { status: String },
    /// The class active Stillness (the ascetic's, on Still mind, #850): every status of
    /// `statuses` the player suffers ends at once, taking no time (what it costs is the talent's
    /// `resource`: Ki; `world/ascetic.rs`). `name` is the abilities menu's, as the talent that
    /// carries it has a passive name of its own.
    Stillness { name: String, statuses: Vec<String> },
    /// A shot it looses while it holds `cost` of `resource` spends them and takes `saving`
    /// percent less time (Rapid shot, on Focus).
    RapidShot {
        resource: String,
        cost: u16,
        saving: u8,
    },
    /// Each second blow of its flurry spends `cost` of `resource`; holding less, it strikes
    /// none (Flurry, on Ki).
    FlurryCost { resource: String, cost: u16 },
    /// A missile (a shot, a throw, a trap's) that would hit it while its hands hold no weapon
    /// and no shield spends `cost` of `resource` by itself and misses; holding less, it hits
    /// (Diamond soul's catch, on Ki; `world/ascetic.rs`).
    CatchMissiles { resource: String, cost: u16 },
    /// The class active Volley: `shots` shots at a cell for the price of one (what it costs is
    /// the talent's `resource`, if anything).
    Volley { shots: u8 },
    /// The class active Spirit weapon: a melee weapon of spirit-stuff called into the hand in a
    /// shape chosen (`world/spirit_weapon.rs`; spec §8 "Prestige classes"), which strikes
    /// `to_hit` surer, threatens `threat` lower, and ignores all Protection if
    /// `ignores_protection`; `phrase` is its description's power line.
    SpiritWeapon {
        to_hit: i32,
        threat: u8,
        #[serde(default)]
        ignores_protection: bool,
        phrase: String,
    },
}

impl MartialEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`MartialEffect`]).
    pub const KINDS: [&'static str; 16] = [
        "catch_missiles",
        "charge",
        "flurry_cost",
        "patient_defence",
        "quivering_palm",
        "rage",
        "rapid_shot",
        "spirit_weapon",
        "stillness",
        "stunning_strike",
        "unstoppable",
        "veterans_wind",
        "volley",
        "war_cry",
        "whirling_charge",
        "wind_step",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MartialMods {
    /// The active Veteran's wind, if it has it.
    pub veterans_wind: Option<SecondWind>,
    /// The active Rage: the resource it grants (Rage, which brings the Raging status), if it
    /// has it.
    pub rage: Option<ResourceId>,
    /// The active War cry, if it has it.
    pub war_cry: Option<WarCry>,
    /// While raging, the statuses that tick nothing, and what of the rage holding on to the
    /// last hit point costs; `None` without Unstoppable.
    pub unstoppable: Option<Unstoppable>,
    /// The active Charge (`world/charge.rs`): what each cell moved costs on top of its
    /// talent's `resource`, if it has it.
    pub charge: Option<u16>,
    /// Whirling charge: Charge strikes the hostiles beside its way (`world/charge.rs`).
    pub whirling_charge: bool,
    /// The active Patient defence, if it has it: the status it gives (Poised).
    pub patient_defence: Option<StatusId>,
    /// The active Stunning strike, if it has it.
    pub stunning_strike: Option<PalmBlow>,
    /// The active Quivering palm, if it has it.
    pub quivering_palm: Option<PalmBlow>,
    /// The active Step of the wind, if it has it: the status it gives (Light-footed).
    pub wind_step: Option<StatusId>,
    /// The active Stillness, if it has it: the statuses it ends.
    pub stillness: Option<Vec<StatusId>>,
    /// The active Spirit weapon, and what a spirit weapon in its hand gives, if it has it.
    pub spirit_weapon: Option<SpiritWeapon>,
    /// Rapid shot's quicker shots on a resource, if it has it.
    pub rapid_shot: Option<RapidShot>,
    /// What each second blow of its flurry costs, if anything (Flurry, on Ki).
    pub flurry_cost: Option<ResourceCost>,
    /// What a missile caught costs, if it catches them (Diamond soul, on Ki; `world/ascetic.rs`).
    pub catch_missiles: Option<ResourceCost>,
    /// The active Volley, if it has it.
    pub volley: Option<Volley>,
}

impl MartialMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: MartialEffect) {
        match effect {
            MartialEffect::VeteransWind {
                per_level,
                at_least,
                cooldown,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    self.veterans_wind = Some(SecondWind {
                        per_level,
                        at_least,
                        cooldown,
                    });
                }
            }
            MartialEffect::Rage { resource } => {
                self.rage = content.resource_id(&resource).or(self.rage);
            }
            MartialEffect::WarCry {
                status,
                severity,
                difficulty,
            } => {
                if let Some(status) = content.status_id(&status) {
                    self.war_cry = Some(WarCry {
                        status,
                        severity,
                        difficulty,
                    });
                }
            }
            MartialEffect::Unstoppable { statuses, cost } => {
                let ids = statuses.iter().filter_map(|s| content.status_id(s));
                let held = self.unstoppable.get_or_insert_with(Unstoppable::default);
                held.numb.extend(ids);
                held.cost = held.cost.max(cost);
            }
            MartialEffect::WhirlingCharge => self.whirling_charge = true,
            MartialEffect::Charge { per_cell } => self.charge = Some(per_cell),
            MartialEffect::QuiveringPalm { status, severity } => {
                if let Some(status) = content.status_id(&status) {
                    self.quivering_palm = Some(PalmBlow { status, severity });
                }
            }
            MartialEffect::StunningStrike { status, severity } => {
                if let Some(status) = content.status_id(&status) {
                    self.stunning_strike = Some(PalmBlow { status, severity });
                }
            }
            MartialEffect::WindStep { status } => {
                self.wind_step = self.wind_step.or(content.status_id(&status));
            }
            MartialEffect::Stillness { statuses, .. } => {
                let ends = self.stillness.get_or_insert_with(Vec::new);
                for id in statuses.iter().filter_map(|s| content.status_id(s)) {
                    if !ends.contains(&id) {
                        ends.push(id);
                    }
                }
            }
            MartialEffect::PatientDefence { status } => {
                self.patient_defence = content.status_id(&status).or(self.patient_defence);
            }
            MartialEffect::SpiritWeapon {
                to_hit,
                threat,
                ignores_protection,
                ..
            } => {
                self.spirit_weapon = Some(SpiritWeapon {
                    to_hit,
                    threat,
                    ignores_protection,
                });
            }
            MartialEffect::RapidShot {
                resource,
                cost,
                saving,
            } => {
                if let Some(resource) = content.resource_id(&resource) {
                    self.rapid_shot = Some(RapidShot {
                        resource,
                        cost,
                        saving,
                    });
                }
            }
            MartialEffect::Volley { shots } => self.volley = Some(Volley { shots }),
            MartialEffect::FlurryCost { resource, cost } => {
                if let Some(resource) = content.resource_id(&resource) {
                    self.flurry_cost = Some(ResourceCost {
                        resource,
                        amount: cost,
                    });
                }
            }
            MartialEffect::CatchMissiles { resource, cost } => {
                if let Some(resource) = content.resource_id(&resource) {
                    self.catch_missiles = Some(ResourceCost {
                        resource,
                        amount: cost,
                    });
                }
            }
        }
    }

    /// The status the active Rage brings (Raging: its resource's `status`), if it has Rage.
    pub fn rage_status(&self, content: &Content) -> Option<StatusId> {
        content.resource(self.rage?).status
    }
}

/// The active Volley (spec §8 "Class abilities"): shots at a cell for the price of one; it costs
/// what its talent's `resource` says (Focus).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Volley {
    pub shots: u8,
}

/// Rapid shot (spec §8 "Resources"): a shot spends `cost` of `resource` (Focus), when there is
/// that much, and takes `saving` percent less time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RapidShot {
    pub resource: ResourceId,
    pub cost: u16,
    pub saving: u8,
}

/// The active War cry (spec §8 "Class abilities"): the status a roar leaves on the hostiles who
/// hear and see its holder and fail their check, its degrees, and the check's base difficulty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarCry {
    pub status: StatusId,
    pub severity: u16,
    pub difficulty: i32,
}

/// The active Spirit weapon (spec §8 "Prestige classes"; `world/spirit_weapon.rs`): what a
/// spirit weapon in the hand gives, more to hit, a threat lower, and Protection ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpiritWeapon {
    pub to_hit: i32,
    pub threat: u8,
    pub ignores_protection: bool,
}

/// A bare-handed blow that leaves a wounded foe `severity` degrees of `status`: the actives
/// Stunning strike and Quivering palm (spec §8 "Class abilities"; `World::palm_blow`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PalmBlow {
    pub status: StatusId,
    pub severity: u16,
}

/// Unstoppable (spec §8 "Class abilities"): while raging, the statuses in `numb` tick nothing,
/// and a blow that would take the last hit point leaves it for `cost` of the rage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Unstoppable {
    pub numb: Vec<StatusId>,
    pub cost: u16,
}
