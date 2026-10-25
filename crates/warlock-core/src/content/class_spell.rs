//! The casters' spells (spec §8 "Class abilities", design §4.1 B2): the priest's, the druid's,
//! the wizard's, and the witch's actives, **placeholders** for the spell system to come. Each is
//! a class ability's `[[talent.spell]]` table in `data/talents/`, used from the abilities menu
//! (`use:arcane_bolt:12,5`), paid in mana (spec §8 "Mana"; no cooldown besides: mana comes back
//! with time), and cast through the one shared spell code the scrolls and traps use
//! (`world/caster.rs`).
//!
//! ```toml
//! [[talent.spell]]
//! spell = "arcane_bolt"   # which ability it is: its id, its replay text (`use:arcane_bolt`)
//! mana = 3                # what a cast costs
//! casts = 1               # its time, in `cast` actions (default 1)
//! name = "Flame"          # the ability's name and description, when not the talent's (a
//! description = "..."     # talent with two spells)
//! [talent.spell.cast]     # a strike's burst or bolt, exactly as a scroll's `read` table
//! kind = "bolt"
//! ...
//! ```
//!
//! What a spell does is its kind's ([`ClassSpell::kind`]): a **boon** (`status`, `severity`: on
//! the caster), a **mend** (`heal` dice, and one more hit point per `per_levels` levels), a
//! **cleanse** (the statuses `cures` names end, and with `diseases` every disease), a **hex**
//! (`status`, `severity` on a creature the caster sees within `range`), or a **strike** (`cast`,
//! a burst or a bolt aimed at a cell; `open_sky`: only under the open sky; `status`, `severity`:
//! everyone it reaches suffers that too).

use super::scroll::{ReadSpec, Reading, ScrollSpell};
use super::{Content, StatusId};
use crate::dice::Dice;
use serde::{Deserialize, Serialize};

/// The farthest a hex reaches, in cells.
pub const MAX_HEX_RANGE: i32 = 12;
/// The most mana a spell may cost.
pub const MAX_SPELL_MANA: u16 = 100;

/// A caster's spell (design §4.9–4.12): which ability it is. Each is had by whoever holds the
/// class ability whose `[[talent.spell]]` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassSpell {
    /// The priest's Blessing: a boon on the caster.
    Blessing,
    /// The priest's Cure wounds: hit points back.
    CureWounds,
    /// The priest's Remove disease: diseases and poison gone.
    RemoveDisease,
    /// The druid's Call lightning: a bolt, under the open sky only.
    CallLightning,
    /// The druid's Flame (Flame and frost): a fireball's burst.
    Flame,
    /// The druid's Frost (Flame and frost): a frost burst.
    Frost,
    /// The wizard's Arcane bolt.
    ArcaneBolt,
    /// The wizard's Fire burst.
    FireBurst,
    /// The witch's Evil eye: a hex.
    EvilEye,
    /// The witch's Hex of weakness.
    HexOfWeakness,
    /// The witch's Curse of sloth.
    CurseOfSloth,
    /// The witch's Blight: a burst that poisons.
    Blight,
}

/// What a kind of spell does (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellKind {
    Boon,
    Mend,
    Cleanse,
    Hex,
    Strike,
}

impl ClassSpell {
    /// Every spell, in menu order (by class: priest, druid, wizard, witch).
    pub const ALL: &[ClassSpell] = &[
        ClassSpell::Blessing,
        ClassSpell::CureWounds,
        ClassSpell::RemoveDisease,
        ClassSpell::CallLightning,
        ClassSpell::Flame,
        ClassSpell::Frost,
        ClassSpell::ArcaneBolt,
        ClassSpell::FireBurst,
        ClassSpell::EvilEye,
        ClassSpell::HexOfWeakness,
        ClassSpell::CurseOfSloth,
        ClassSpell::Blight,
    ];

    /// Its id: the ability's, and its replay text (`use:arcane_bolt:12,5`).
    pub const fn id(self) -> &'static str {
        match self {
            ClassSpell::Blessing => "blessing",
            ClassSpell::CureWounds => "cure_wounds",
            ClassSpell::RemoveDisease => "remove_disease",
            ClassSpell::CallLightning => "call_lightning",
            ClassSpell::Flame => "flame",
            ClassSpell::Frost => "frost",
            ClassSpell::ArcaneBolt => "arcane_bolt",
            ClassSpell::FireBurst => "fire_burst",
            ClassSpell::EvilEye => "evil_eye",
            ClassSpell::HexOfWeakness => "hex_of_weakness",
            ClassSpell::CurseOfSloth => "curse_of_sloth",
            ClassSpell::Blight => "blight",
        }
    }

    /// The spell with id `id`.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|s| s.id() == id)
    }

    /// What it does.
    pub const fn kind(self) -> SpellKind {
        match self {
            ClassSpell::Blessing => SpellKind::Boon,
            ClassSpell::CureWounds => SpellKind::Mend,
            ClassSpell::RemoveDisease => SpellKind::Cleanse,
            ClassSpell::EvilEye | ClassSpell::HexOfWeakness | ClassSpell::CurseOfSloth => {
                SpellKind::Hex
            }
            ClassSpell::CallLightning
            | ClassSpell::Flame
            | ClassSpell::Frost
            | ClassSpell::ArcaneBolt
            | ClassSpell::FireBurst
            | ClassSpell::Blight => SpellKind::Strike,
        }
    }

    /// Whether it asks for a cell before it is cast (a hex, a strike).
    pub const fn aimed(self) -> bool {
        matches!(self.kind(), SpellKind::Hex | SpellKind::Strike)
    }
}

/// `[[talent.spell]]` as a talent's file writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpellSpec {
    spell: ClassSpell,
    name: Option<String>,
    description: Option<String>,
    mana: u16,
    casts: Option<u8>,
    status: Option<String>,
    severity: Option<u16>,
    heal: Option<Dice>,
    per_levels: Option<u8>,
    #[serde(default)]
    cures: Vec<String>,
    #[serde(default)]
    diseases: bool,
    range: Option<i32>,
    cast: Option<ReadSpec>,
    #[serde(default)]
    open_sky: bool,
}

/// A caster's spell, checked: what it is, costs, and does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellDef {
    pub spell: ClassSpell,
    /// The ability's own name and description (`None`: the talent's).
    pub name: Option<String>,
    pub description: Option<String>,
    /// What a cast costs, in mana (before talents: Archmage).
    pub mana: u16,
    /// Its time, in `cast` actions.
    pub casts: u8,
    pub does: Does,
}

/// What a spell does (its kind's, module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Does {
    /// The caster suffers `severity` degrees of `status` (a boon: Blessed).
    Boon { status: StatusId, severity: u16 },
    /// The caster gets `heal` hit points back, and one more per `per_levels` levels (0: none).
    Mend { heal: Dice, per_levels: u8 },
    /// The statuses `cures` end, and with `diseases` every disease the caster has.
    Cleanse {
        cures: Vec<StatusId>,
        diseases: bool,
    },
    /// A creature the caster sees within `range` cells suffers `severity` degrees of `status`.
    Hex {
        status: StatusId,
        severity: u16,
        range: i32,
    },
    /// A burst or a bolt at a cell (`open_sky`: only under the open sky); everyone it reaches
    /// suffers `afflict` too.
    Strike {
        spell: ScrollSpell,
        open_sky: bool,
        afflict: Option<(StatusId, u16)>,
    },
}

impl SpellDef {
    /// How far a cell it aims at may be: a hex's range, a strike's burst range or bolt length.
    pub fn range(&self) -> Option<i32> {
        match &self.does {
            Does::Hex { range, .. } => Some(*range),
            Does::Strike { spell, .. } => Some(spell.range),
            _ => None,
        }
    }

    /// The burst or bolt it casts, for a strike.
    pub fn strike(&self) -> Option<&ScrollSpell> {
        match &self.does {
            Does::Strike { spell, .. } => Some(spell),
            _ => None,
        }
    }
}

impl SpellSpec {
    /// The spell it names.
    pub(super) fn spell(&self) -> ClassSpell {
        self.spell
    }

    /// Checks the spec against `content` (statuses and damage types loaded; its effect is checked
    /// once effects are, [`super::scroll::check_spells`]).
    pub(super) fn resolve(self, content: &Content) -> Result<SpellDef, String> {
        let id = self.spell.id();
        let err = |m: &str| format!("spell `{id}`: {m}");
        if !(1..=MAX_SPELL_MANA).contains(&self.mana) {
            return Err(err(&format!("`mana` is 1 to {MAX_SPELL_MANA}")));
        }
        let casts = self.casts.unwrap_or(1);
        if !(1..=10).contains(&casts) {
            return Err(err("`casts` is 1 to 10"));
        }
        for text in [&self.name, &self.description].into_iter().flatten() {
            if text.trim().is_empty() {
                return Err(err("`name` and `description` must not be empty"));
            }
        }
        let status = |name: &Option<String>| -> Result<Option<StatusId>, String> {
            match name {
                None => Ok(None),
                Some(s) if content.status_ids().next().is_none() => {
                    Err(err(&format!("names a status `{s}`, but there are none")))
                }
                Some(s) => (content.status_id(s))
                    .map(Some)
                    .ok_or_else(|| err(&format!("names an unknown status `{s}`"))),
            }
        };
        let kind = self.spell.kind();
        let only = |field: &str, given: bool, kinds: &[SpellKind]| {
            if given && !kinds.contains(&kind) {
                Err(err(&format!("`{field}` is not for this kind of spell")))
            } else {
                Ok(())
            }
        };
        use SpellKind as K;
        only(
            "status",
            self.status.is_some(),
            &[K::Boon, K::Hex, K::Strike],
        )?;
        only(
            "severity",
            self.severity.is_some(),
            &[K::Boon, K::Hex, K::Strike],
        )?;
        only("heal", self.heal.is_some(), &[K::Mend])?;
        only("per_levels", self.per_levels.is_some(), &[K::Mend])?;
        only("cures", !self.cures.is_empty(), &[K::Cleanse])?;
        only("diseases", self.diseases, &[K::Cleanse])?;
        only("range", self.range.is_some(), &[K::Hex])?;
        only("cast", self.cast.is_some(), &[K::Strike])?;
        only("open_sky", self.open_sky, &[K::Strike])?;
        let severity = self.severity.unwrap_or(1);
        if severity == 0 {
            return Err(err("`severity` is at least 1"));
        }
        let does = match kind {
            K::Boon => Does::Boon {
                status: status(&self.status)?.ok_or_else(|| err("a boon needs a `status`"))?,
                severity,
            },
            K::Mend => {
                let heal = self.heal.ok_or_else(|| err("a mend needs `heal` dice"))?;
                if heal.max() < 1 || heal.min() < 0 {
                    return Err(err("`heal` dice must heal"));
                }
                Does::Mend {
                    heal,
                    per_levels: self.per_levels.unwrap_or(0),
                }
            }
            K::Cleanse => {
                let cures = (self.cures.iter())
                    .map(|s| {
                        (content.status_id(s))
                            .ok_or_else(|| err(&format!("names an unknown status `{s}`")))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if cures.is_empty() && !self.diseases {
                    return Err(err("a cleanse `cures` a status or ends `diseases`"));
                }
                Does::Cleanse {
                    cures,
                    diseases: self.diseases,
                }
            }
            K::Hex => {
                let range = self.range.unwrap_or(0);
                if !(1..=MAX_HEX_RANGE).contains(&range) {
                    return Err(err(&format!("a hex's `range` is 1 to {MAX_HEX_RANGE}")));
                }
                Does::Hex {
                    status: status(&self.status)?.ok_or_else(|| err("a hex needs a `status`"))?,
                    severity,
                    range,
                }
            }
            K::Strike => {
                let cast = self.cast.ok_or_else(|| err("a strike needs a `cast`"))?;
                let Reading::Spell(spell) = cast.resolve(content).map_err(|m| err(&m))? else {
                    return Err(err("a strike's `cast` is a burst or a bolt"));
                };
                Does::Strike {
                    spell,
                    open_sky: self.open_sky,
                    afflict: status(&self.status)?.map(|s| (s, severity)),
                }
            }
        };
        Ok(SpellDef {
            spell: self.spell,
            name: self.name,
            description: self.description,
            mana: self.mana,
            casts,
            does,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_kinds_aim() {
        for &spell in ClassSpell::ALL {
            assert_eq!(ClassSpell::from_id(spell.id()), Some(spell));
        }
        assert_eq!(ClassSpell::from_id("rage"), None);
        assert!(ClassSpell::ArcaneBolt.aimed() && ClassSpell::EvilEye.aimed());
        assert!(!ClassSpell::Blessing.aimed() && !ClassSpell::RemoveDisease.aimed());
    }
}
