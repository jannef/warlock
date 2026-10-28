//! Potions (spec §8 "Potions", the combat progression design §10): an object with an
//! `[object.drink]` table is drunk from the Consume list (Eat's command and action), and does
//! what its `kind` says, at once.
//!
//! ```toml
//! [object.drink]
//! kind = "heal"              # heal, cure, mana, elixir, harm, boost, vision, haste, resistance
//! dice = "2d8"               # heal: hit points back; mana: mana back; harm: degrees of sickness
//! cures = ["bandage"]        # heal, cure: means named in the statuses' and diseases' `cured_by`
//! diseases = true            # cure: ends every disease the drinker has
//! status = "poisoned"        # harm: the status it gives, `dice` degrees of it; boost, vision,
//!                            # resistance: the status it gives (a raise, eyes, a `resist`)
//! severity = 1               # boost, haste: the degrees it gives (1 to the status's `max`)
//! stat = "strength"          # elixir: the stat it raises for good
//! points = 1                 # elixir: by how much (a trained rise each)
//! potential = "1d2"          # elixir: the potential's further rise, rolled once
//! colour = "red"             # what an unknown one is called by ("a fizzy red potion")
//! marks = { toughness = 5 }  # sure training marks it gives, by stat
//! ```
//!
//! Only its kind's fields: a heal has `dice` and maybe `cures`; a cure has `cures` or
//! `diseases`; a mana potion `dice`; an elixir `stat`, `points`, and `potential`; a harm `dice`
//! and `status`; a boost `status` (one that raises a stat, `penalty`) and `severity`; a vision
//! `status` (one with `vision`); a haste `severity` (of Hasted, the Haste spell's status); a
//! resistance `status` (one with `resist`). The colour's hue is its
//! family's (the design's rule, kept in data as the object's `fg`): potions that share a colour
//! name share their `fg`.

use super::{Content, StatusId};
use crate::dice::Dice;
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The most marks a potion may give a stat.
pub const MAX_POTION_MARKS: u16 = 100;

/// The status a haste gives: the Haste spell's.
const HASTE: &str = "hasted";
/// The points an elixir may raise its stat by.
pub const ELIXIR_POINTS: std::ops::RangeInclusive<u8> = 1..=3;

/// What drinking a potion does (spec §8 "Potions").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Draught {
    /// Hit points back at once, never past the maximum, and what `cures` names cured.
    Heal { dice: Dice, cures: Vec<String> },
    /// What `cures` names cured ([`crate::world::World`]'s one cure path), and with `diseases`
    /// every disease.
    Cure { cures: Vec<String>, diseases: bool },
    /// Mana back at once, never past the maximum.
    Mana { dice: Dice },
    /// `stat` raised for good by `points` (each a trained rise: the potential lifted to the score
    /// first if below), then its potential by a roll of `potential` (the design's §10.5).
    Elixir {
        stat: Stat,
        points: u8,
        potential: Dice,
    },
    /// Sickness: degrees of `status` (poisoned), as rotten food gives.
    Harm { dice: Dice, status: StatusId },
    /// A stat raised for a while: `severity` degrees of `status` (Fortified), a raise.
    Boost { status: StatusId, severity: u16 },
    /// Eyes for the dark for a while: `status` (Owl-eyed), which lends a vision profile.
    Vision { status: StatusId },
    /// Quickness for a while: `severity` degrees of `status`, Hasted (the Haste spell's).
    Haste { status: StatusId, severity: u16 },
    /// A resistance for a while: `status` (Warmed), which resists as gear does.
    Resistance { status: StatusId },
}

impl Draught {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            Draught::Heal { .. } => "heal",
            Draught::Cure { .. } => "cure",
            Draught::Mana { .. } => "mana",
            Draught::Elixir { .. } => "elixir",
            Draught::Harm { .. } => "harm",
            Draught::Boost { .. } => "boost",
            Draught::Vision { .. } => "vision",
            Draught::Haste { .. } => "haste",
            Draught::Resistance { .. } => "resistance",
        }
    }

    /// The status it gives the drinker for a while and the degrees of it, for the kinds that
    /// give one (boost, vision, haste, resistance); not a harm's sickness, whose degrees are
    /// rolled.
    pub fn lends(&self) -> Option<(StatusId, u16)> {
        match *self {
            Draught::Boost { status, severity } | Draught::Haste { status, severity } => {
                Some((status, severity))
            }
            Draught::Vision { status } | Draught::Resistance { status } => Some((status, 1)),
            _ => None,
        }
    }

    /// What it cures, as means of the statuses' and diseases' `cured_by`.
    pub fn cures(&self) -> &[String] {
        match self {
            Draught::Heal { cures, .. } | Draught::Cure { cures, .. } => cures,
            _ => &[],
        }
    }
}

/// A potion, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drink {
    pub does: Draught,
    /// The colour's name ("red"): an unknown potion is called by it; its `fg` is the object's.
    pub colour: String,
    /// The training marks it gives, by stat, in stat order (sure marks: the `drank` use).
    pub marks: Vec<(Stat, u16)>,
}

/// `[object.drink]` as data writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrinkSpec {
    kind: String,
    dice: Option<Dice>,
    #[serde(default)]
    cures: Vec<String>,
    #[serde(default)]
    diseases: bool,
    status: Option<String>,
    severity: Option<u16>,
    stat: Option<String>,
    points: Option<u8>,
    potential: Option<Dice>,
    colour: String,
    #[serde(default)]
    marks: BTreeMap<String, u16>,
}

impl DrinkSpec {
    /// Resolves it against the statuses (loaded before objects): only its kind's fields, a
    /// colour, and marks of known stats, 1 to [`MAX_POTION_MARKS`] each.
    pub(super) fn resolve(self, content: &Content) -> Result<Drink, String> {
        let only = |fields: &[(&str, bool)]| match fields.iter().find(|(_, set)| *set) {
            Some((field, _)) => Err(format!("`drink`: a {} has no `{field}`", self.kind)),
            None => Ok(()),
        };
        let dice = |dice: Option<Dice>| {
            dice.ok_or_else(|| format!("`drink`: a {} needs `dice`", self.kind))
        };
        // The kinds that lend a status have no dice, cures, or diseases.
        let lending = [
            ("dice", self.dice.is_some()),
            ("cures", !self.cures.is_empty()),
            ("diseases", self.diseases),
        ];
        let status = |name: Option<&str>| {
            let name =
                name.ok_or_else(|| format!("`drink`: a {} names its `status`", self.kind))?;
            (content.status_id(name))
                .ok_or_else(|| format!("`drink`: no status `{name}` in data/status.toml"))
        };
        let severity = |status: StatusId| {
            let max = content.status(status).max;
            match self.severity {
                Some(s) if (1..=max).contains(&s) => Ok(s),
                Some(_) => Err(format!(
                    "`drink`: `severity` is 1 to {max} (the status's max)"
                )),
                None => Err(format!("`drink`: a {} needs `severity`", self.kind)),
            }
        };
        // An elixir's fields belong to it alone.
        if self.kind != "elixir" {
            only(&[
                ("stat", self.stat.is_some()),
                ("points", self.points.is_some()),
                ("potential", self.potential.is_some()),
            ])?;
        }
        let does = match self.kind.as_str() {
            "heal" => {
                only(&[
                    ("diseases", self.diseases),
                    ("status", self.status.is_some()),
                    ("severity", self.severity.is_some()),
                ])?;
                Draught::Heal {
                    dice: dice(self.dice)?,
                    cures: self.cures.clone(),
                }
            }
            "cure" => {
                only(&[
                    ("dice", self.dice.is_some()),
                    ("status", self.status.is_some()),
                    ("severity", self.severity.is_some()),
                ])?;
                if self.cures.is_empty() && !self.diseases {
                    return Err("`drink`: a cure names `cures` or `diseases = true`".into());
                }
                Draught::Cure {
                    cures: self.cures.clone(),
                    diseases: self.diseases,
                }
            }
            "mana" => {
                only(&[
                    ("cures", !self.cures.is_empty()),
                    ("diseases", self.diseases),
                    ("status", self.status.is_some()),
                    ("severity", self.severity.is_some()),
                ])?;
                Draught::Mana {
                    dice: dice(self.dice)?,
                }
            }
            "elixir" => {
                only(&[
                    ("dice", self.dice.is_some()),
                    ("cures", !self.cures.is_empty()),
                    ("diseases", self.diseases),
                    ("status", self.status.is_some()),
                    ("severity", self.severity.is_some()),
                ])?;
                let name = (self.stat.as_deref())
                    .ok_or_else(|| "`drink`: an elixir names its `stat`".to_string())?;
                let stat = Stat::from_id(name)
                    .ok_or_else(|| format!("`drink`: `stat` names no stat `{name}`"))?;
                let points = self.points.unwrap_or(1);
                if !ELIXIR_POINTS.contains(&points) {
                    return Err(format!(
                        "`drink`: an elixir's `points` are {} to {}",
                        ELIXIR_POINTS.start(),
                        ELIXIR_POINTS.end()
                    ));
                }
                let potential = (self.potential)
                    .ok_or_else(|| "`drink`: an elixir needs `potential` dice".to_string())?;
                if potential.min() < 0 {
                    return Err("`drink`: an elixir's `potential` never lowers it".into());
                }
                Draught::Elixir {
                    stat,
                    points,
                    potential,
                }
            }
            "harm" => {
                only(&[
                    ("cures", !self.cures.is_empty()),
                    ("diseases", self.diseases),
                    ("severity", self.severity.is_some()),
                ])?;
                let name = (self.status.clone())
                    .ok_or_else(|| "`drink`: a harm names its `status`".to_string())?;
                let status = (content.status_id(&name))
                    .ok_or_else(|| format!("`drink`: no status `{name}` in data/status.toml"))?;
                Draught::Harm {
                    dice: dice(self.dice)?,
                    status,
                }
            }
            "boost" => {
                only(&lending)?;
                let status = status(self.status.as_deref())?;
                let def = content.status(status);
                if def
                    .penalty
                    .as_ref()
                    .is_none_or(|p| p.by.iter().all(|(_, v)| v <= 0))
                {
                    return Err(format!(
                        "`drink`: a boost's status `{}` raises no stat",
                        def.id
                    ));
                }
                Draught::Boost {
                    status,
                    severity: severity(status)?,
                }
            }
            "vision" => {
                only(&[lending.as_slice(), &[("severity", self.severity.is_some())]].concat())?;
                let status = status(self.status.as_deref())?;
                if content.status(status).vision.is_none() {
                    let id = &content.status(status).id;
                    return Err(format!(
                        "`drink`: a vision's status `{id}` lends no `vision`"
                    ));
                }
                Draught::Vision { status }
            }
            "haste" => {
                only(&[lending.as_slice(), &[("status", self.status.is_some())]].concat())?;
                let status = status(Some(HASTE))?;
                Draught::Haste {
                    status,
                    severity: severity(status)?,
                }
            }
            "resistance" => {
                only(&[lending.as_slice(), &[("severity", self.severity.is_some())]].concat())?;
                let status = status(self.status.as_deref())?;
                if content.status(status).resist.is_empty() {
                    let id = &content.status(status).id;
                    return Err(format!(
                        "`drink`: a resistance's status `{id}` has no `resist`"
                    ));
                }
                Draught::Resistance { status }
            }
            other => {
                return Err(format!(
                    "`drink`: unknown kind `{other}` (heal, cure, mana, elixir, harm, boost, \
                     vision, haste, resistance)"
                ));
            }
        };
        if self.colour.trim().is_empty() || self.colour.trim() != self.colour {
            return Err("`drink`: `colour` must be a name, not empty or padded".into());
        }
        let mut marks = Vec::new();
        for (name, &count) in &self.marks {
            let stat = Stat::from_id(name)
                .ok_or_else(|| format!("`drink`: `marks` names no stat `{name}`"))?;
            if !(1..=MAX_POTION_MARKS).contains(&count) {
                return Err(format!("`drink`: `marks` are 1 to {MAX_POTION_MARKS}"));
            }
            marks.push((stat, count));
        }
        marks.sort_by_key(|&(stat, _)| Stat::ALL.iter().position(|&s| s == stat));
        Ok(Drink {
            does,
            colour: self.colour,
            marks,
        })
    }
}
