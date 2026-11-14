//! The sign talents (#1495): what a birth sign's gifts do (spec §8 "Birth sign"): the sign actives
//! and what makes them stronger. [`SignEffect`] is the data (a talent's `[[effect]]` of one of
//! [`SignEffect::KINDS`]), [`SignMods`] the sum every rule reads (`TalentMods::sign`). A new effect
//! of the area is a variant here, its field and its arm in [`SignMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/sign.rs`, and its one read.

use crate::content::{Content, DamageTypeId, LightId, StatusId};
use crate::dice::Dice;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SignEffect {
    /// The active Blaze (a birth sign's, spec §8 "Birth sign"): a burst of `radius` around the
    /// player that does `damage` of kind `damage_type` (an id in `data/damage_types.toml`) to
    /// every creature it reaches but them, melts the ice it reaches, and sets each cell it
    /// reaches but theirs burning for `burns` seconds with light preset `fire` (as a burning
    /// missile where it lands); shown as effect `spell` (`data/effects.toml`); then the
    /// `cooldown` status.
    Blaze {
        radius: i32,
        damage: Dice,
        damage_type: String,
        spell: String,
        fire: String,
        burns: u32,
        cooldown: String,
    },
    /// Blaze bursts out to `radius` with `damage` instead of its own (Wildfire).
    Wildfire { radius: i32, damage: Dice },
    /// The active Fire breath (a birth sign's pick, spec §8 "Birth sign"), aimed at a cell
    /// within [`crate::world::BREATH_RANGE`]: a bolt of `damage` of kind `damage_type` along the
    /// line toward it, `length` cells from the player, through everyone on it; shown as effect
    /// `spell` (a bolt); then `degrees` of the `cooldown` status.
    FireBreath {
        length: i32,
        damage: Dice,
        damage_type: String,
        spell: String,
        cooldown: String,
        degrees: u16,
    },
    /// The active Hound's bay (a birth sign's pick): a howl; every hostile that hears it and is
    /// of a lower level than the player suffers `severity` degrees of `status` (one that
    /// `flees`), then the `cooldown` status.
    HoundsBay {
        status: String,
        severity: u16,
        cooldown: String,
    },
    /// The active Horn's grace (a birth sign's): the statuses `cures` end and up to `diseases`
    /// diseases that show are shaken off, then `degrees` of the `cooldown` status.
    HornsGrace {
        cures: Vec<String>,
        diseases: u8,
        cooldown: String,
        #[serde(default = "super::one_degree")]
        degrees: u16,
    },
    /// Horn's grace also gives back `per_level` hit points per character level, and leaves
    /// `degrees` of its cooldown instead of its own (Horn of plenty).
    HornOfPlenty { per_level: u8, degrees: u16 },
    /// The active Transfixing stare (a birth sign's): a hostile within
    /// [`crate::world::STARE_RANGE`] cells that the player sees and that sees them suffers
    /// `severity` degrees of `status`, then `degrees` of the `cooldown` status.
    Stare {
        status: String,
        severity: u16,
        cooldown: String,
        #[serde(default = "super::one_degree")]
        degrees: u16,
    },
    /// The Transfixing stare leaves `severity` degrees of its status and `degrees` of its
    /// cooldown instead of its own (Petrifying gaze).
    PetrifyingGaze { severity: u16, degrees: u16 },
}

impl SignEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`SignEffect`]).
    pub const KINDS: [&'static str; 8] = [
        "blaze",
        "fire_breath",
        "horn_of_plenty",
        "horns_grace",
        "hounds_bay",
        "petrifying_gaze",
        "stare",
        "wildfire",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignMods {
    /// The birth sign's active Horn's grace, if it has it.
    pub horns_grace: Option<HornsGrace>,
    /// The birth sign's active Transfixing stare, if it has it; of two (a sign's and a profane
    /// gift's), the larger severity.
    pub stare: Option<Stare>,
    /// The birth sign's active Blaze, if it has it.
    pub blaze: Option<Blaze>,
    /// Horn of plenty: the hit points per level Horn's grace gives back, and the degrees of its
    /// cooldown it leaves instead.
    pub horn_of_plenty: Option<(u8, u16)>,
    /// Petrifying gaze: the degrees of its status the stare leaves, and of its cooldown.
    pub petrifying_gaze: Option<(u16, u16)>,
    /// Wildfire: Blaze's radius and damage instead of its own.
    pub wildfire: Option<(i32, Dice)>,
    /// The birth sign's active Fire breath, if it has it; of two (the Wyvern's pick and
    /// Hate-wyrm's heart), the larger damage.
    pub fire_breath: Option<FireBreath>,
    /// The birth sign's active Hound's bay, if it has it.
    pub hounds_bay: Option<HoundsBay>,
}

impl SignMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: SignEffect) {
        match effect {
            SignEffect::HornOfPlenty { per_level, degrees } => {
                self.horn_of_plenty = Some((per_level, degrees));
            }
            SignEffect::PetrifyingGaze { severity, degrees } => {
                self.petrifying_gaze = Some((severity, degrees));
            }
            SignEffect::Wildfire { radius, damage } => self.wildfire = Some((radius, damage)),
            SignEffect::FireBreath {
                length,
                damage,
                damage_type,
                spell,
                cooldown,
                degrees,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    let bolt = crate::content::Bolt {
                        spell,
                        length,
                        damage: damage.into(),
                        reroll: 0,
                        kind: content.damage_type_id(&damage_type),
                    };
                    // Two breaths (the Wyvern's pick and Hate-wyrm's heart): the larger
                    // damage, with its own cooldown.
                    let larger = (self.fire_breath.as_ref())
                        .is_none_or(|held| bolt.damage.mean_x2() > held.bolt.damage.mean_x2());
                    if larger {
                        self.fire_breath = Some(FireBreath {
                            bolt,
                            cooldown,
                            degrees,
                        });
                    }
                }
            }
            SignEffect::HoundsBay {
                status,
                severity,
                cooldown,
            } => {
                if let (Some(status), Some(cooldown)) =
                    (content.status_id(&status), content.status_id(&cooldown))
                {
                    self.hounds_bay = Some(HoundsBay {
                        status,
                        severity,
                        cooldown,
                    });
                }
            }
            SignEffect::Blaze {
                radius,
                damage,
                damage_type,
                spell,
                fire,
                burns,
                cooldown,
            } => {
                if let (Some(fire), Some(cooldown)) =
                    (content.light_id(&fire), content.status_id(&cooldown))
                {
                    self.blaze = Some(Blaze {
                        radius,
                        damage,
                        kind: content.damage_type_id(&damage_type),
                        spell,
                        fire,
                        burns,
                        cooldown,
                    });
                }
            }
            SignEffect::HornsGrace {
                cures,
                diseases,
                cooldown,
                degrees,
            } => {
                if let Some(cooldown) = content.status_id(&cooldown) {
                    let cures = cures.iter().filter_map(|s| content.status_id(s)).collect();
                    self.horns_grace = Some(HornsGrace {
                        cures,
                        diseases,
                        cooldown,
                        degrees,
                    });
                }
            }
            SignEffect::Stare {
                status,
                severity,
                cooldown,
                degrees,
            } => {
                if let (Some(status), Some(cooldown)) =
                    (content.status_id(&status), content.status_id(&cooldown))
                {
                    // Two stares: the larger severity.
                    if self.stare.is_none_or(|held| severity > held.severity) {
                        self.stare = Some(Stare {
                            status,
                            severity,
                            cooldown,
                            degrees,
                        });
                    }
                }
            }
        }
    }

    /// Whether it has the birth sign's active `active` (spec §8 "Birth sign").
    pub fn has_sign_active(&self, active: crate::content::SignActive) -> bool {
        use crate::content::SignActive as A;
        match active {
            A::HornsGrace => self.horns_grace.is_some(),
            A::TransfixingStare => self.stare.is_some(),
            A::Blaze => self.blaze.is_some(),
            A::FireBreath => self.fire_breath.is_some(),
            A::HoundsBay => self.hounds_bay.is_some(),
        }
    }

    /// The cooldown status of the birth sign's active `active`, if it has it.
    pub fn sign_cooldown(&self, active: crate::content::SignActive) -> Option<StatusId> {
        use crate::content::SignActive as A;
        match active {
            A::HornsGrace => self.horns_grace.as_ref().map(|g| g.cooldown),
            A::TransfixingStare => self.stare.map(|s| s.cooldown),
            A::Blaze => self.blaze.as_ref().map(|b| b.cooldown),
            A::FireBreath => self.fire_breath.as_ref().map(|b| b.cooldown),
            A::HoundsBay => self.hounds_bay.map(|b| b.cooldown),
        }
    }
}

/// The active Fire breath (spec §8 "Birth sign"): a bolt of fire, then a cooldown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireBreath {
    pub bolt: crate::content::Bolt,
    pub cooldown: StatusId,
    pub degrees: u16,
}

/// The active Hound's bay (spec §8 "Birth sign"): the status a howl leaves on lesser foes
/// who hear it, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoundsBay {
    pub status: StatusId,
    pub severity: u16,
    pub cooldown: StatusId,
}

/// The birth sign's active Horn's grace (spec §8 "Birth sign"): what it cures, then a cooldown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HornsGrace {
    pub cures: Vec<StatusId>,
    /// How many of the diseases that show it shakes off.
    pub diseases: u8,
    pub cooldown: StatusId,
    pub degrees: u16,
}

/// The birth sign's active Transfixing stare (spec §8 "Birth sign"): the status it leaves on
/// the creature stared at, then a cooldown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stare {
    pub status: StatusId,
    pub severity: u16,
    pub cooldown: StatusId,
    pub degrees: u16,
}

/// The birth sign's active Blaze (spec §8 "Birth sign"): a burst of flame around the player,
/// fires on the cells it reaches, then a cooldown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blaze {
    pub radius: i32,
    pub damage: Dice,
    /// The kind of damage it does; `None` without damage types.
    pub kind: Option<DamageTypeId>,
    /// The effect presentation shows (`data/effects.toml`).
    pub spell: String,
    /// The light the fires it leaves give, and how long they burn, in seconds.
    pub fire: LightId,
    pub burns: u32,
    pub cooldown: StatusId,
}
