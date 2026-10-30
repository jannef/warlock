//! Wands (spec §8 "Magic items": "Wands", the magic design §9): an object with a `[object.wand]`
//! table holds a spell of `data/spells/` and a number of charges, and casts it at a caster level
//! of its own, whoever holds it. It goes in the missile weapon slot and is used as a missile
//! weapon is (`t`, `fire:X,Y`); a charge pays for each cast instead of mana, and an empty wand
//! does nothing. There is no recharging.
//!
//! ```toml
//! [object.wand]
//! spell = "fireball"   # an aimed spell of data/spells/
//! charges = 7          # required: the wand's own, 1 to 99
//! caster_level = 12    # default: the spell's level, at least 1
//! ```

use super::{Content, MAX_LEVEL, SpellId};
use serde::Deserialize;

/// The most charges a wand may hold.
pub const MAX_WAND_CHARGES: u8 = 99;

/// `[object.wand]` as data writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WandSpec {
    spell: String,
    charges: u8,
    caster_level: Option<u8>,
}

/// A wand, resolved: the spell it holds, the charges a new one has, and the caster level it
/// casts at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wand {
    pub spell: SpellId,
    /// The charges a wand of this kind is made with, 1 to [`MAX_WAND_CHARGES`].
    pub charges: u8,
    /// The caster level it casts its spell at, 1 to [`MAX_LEVEL`].
    pub caster_level: u8,
}

impl WandSpec {
    /// Resolves it against the spells (loaded before objects): an aimed spell (a wand is
    /// aimed as a missile weapon is), charges 1 to [`MAX_WAND_CHARGES`] (its own, required), a
    /// caster level 1 to [`MAX_LEVEL`] (default the spell's level, at least 1:
    /// [`super::SpellData::grows_from`]).
    pub(super) fn resolve(self, content: &Content) -> Result<Wand, String> {
        let spell = (content.spell_id(&self.spell))
            .ok_or_else(|| format!("`wand`: no spell `{}` in data/spells/", self.spell))?;
        let def = content.spell(spell);
        if !def.aimed() {
            return Err(format!(
                "`wand`: spell `{}` is cast on oneself; a wand's spell is aimed",
                self.spell
            ));
        }
        let charges = self.charges;
        if !(1..=MAX_WAND_CHARGES).contains(&charges) {
            return Err(format!("`wand`: `charges` is 1 to {MAX_WAND_CHARGES}"));
        }
        let caster_level = self.caster_level.unwrap_or(def.grows_from());
        if !(1..=MAX_LEVEL).contains(&caster_level) {
            return Err(format!("`wand`: `caster_level` is 1 to {MAX_LEVEL}"));
        }
        Ok(Wand {
            spell,
            charges,
            caster_level,
        })
    }
}
