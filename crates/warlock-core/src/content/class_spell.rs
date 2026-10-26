//! The casters' spells (spec §8 "Class abilities", "Spells"): the priest's, the druid's, the
//! wizard's, and the witch's spell abilities, used from the abilities menu
//! (`use:arcane_bolt:12,5`). Each names a spell of `data/spells/` by its id, which a class
//! ability gives (the talent effect `{ kind = "spell", spell = "arcane_bolt" }`), and is cast by
//! the one cast path (`world/cast.rs`): its cost, resolution, and dice are the spell's.
//!
//! These twelve are what the placeholders of #245 became (#395): the abilities, their ids, and
//! their replay text are unchanged. Learned spells (#396) will join them.

use serde::{Deserialize, Serialize};

/// A caster's spell ability (design §4.9–4.12): which it is. Each is had by whoever holds the
/// class ability whose `spell` effect names it, and is the spell of `data/spells/` of its id.
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

    /// Whether it asks for a cell before it is cast: anything but a spell on oneself. The
    /// content check holds its spell's shape to this ([`super::SpellData::aimed`]).
    pub const fn aimed(self) -> bool {
        !matches!(
            self,
            ClassSpell::Blessing | ClassSpell::CureWounds | ClassSpell::RemoveDisease
        )
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
