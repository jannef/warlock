//! Potions (spec §8 "Potions", the combat progression design §10): an object with an
//! `[object.drink]` table is drunk from the Consume list (Eat's command and action), and does
//! what its `kind` says, at once.
//!
//! ```toml
//! [object.drink]
//! kind = "heal"              # heal, cure, mana, harm
//! dice = "2d8"               # heal: hit points back; mana: mana back; harm: degrees of sickness
//! cures = ["bandage"]        # heal, cure: means named in the statuses' and diseases' `cured_by`
//! diseases = true            # cure: ends every disease the drinker has
//! status = "poisoned"        # harm: the status it gives, `dice` degrees of it
//! colour = "red"             # what an unknown one is called by ("a fizzy red potion")
//! marks = { toughness = 5 }  # sure training marks it gives, by stat
//! ```
//!
//! Only its kind's fields: a heal has `dice` and maybe `cures`; a cure has `cures` or
//! `diseases`; a mana potion `dice`; a harm `dice` and `status`. The colour's hue is its
//! family's (the design's rule, kept in data as the object's `fg`): potions that share a colour
//! name share their `fg`.

use super::{Content, StatusId};
use crate::dice::Dice;
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The most marks a potion may give a stat.
pub const MAX_POTION_MARKS: u16 = 100;

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
    /// Sickness: degrees of `status` (poisoned), as rotten food gives.
    Harm { dice: Dice, status: StatusId },
}

impl Draught {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            Draught::Heal { .. } => "heal",
            Draught::Cure { .. } => "cure",
            Draught::Mana { .. } => "mana",
            Draught::Harm { .. } => "harm",
        }
    }

    /// What it cures, as means of the statuses' and diseases' `cured_by`.
    pub fn cures(&self) -> &[String] {
        match self {
            Draught::Heal { cures, .. } | Draught::Cure { cures, .. } => cures,
            Draught::Mana { .. } | Draught::Harm { .. } => &[],
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
        let does = match self.kind.as_str() {
            "heal" => {
                only(&[
                    ("diseases", self.diseases),
                    ("status", self.status.is_some()),
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
                ])?;
                Draught::Mana {
                    dice: dice(self.dice)?,
                }
            }
            "harm" => {
                only(&[
                    ("cures", !self.cures.is_empty()),
                    ("diseases", self.diseases),
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
            other => {
                return Err(format!(
                    "`drink`: unknown kind `{other}` (heal, cure, mana, harm)"
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
