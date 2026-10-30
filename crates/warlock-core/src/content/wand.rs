//! Wands (spec §8 "Magic items": "Wands", the magic design §9, the spell scaling design §7.4): an
//! object with a `[object.wand]` table holds a spell of `data/spells/` and a number of charges,
//! and casts it at a rung of its own on the spell's damage ladder, whoever holds it. It goes in
//! the missile weapon slot and is used as a missile weapon is (`t`, `fire:X,Y`); a charge pays
//! for each cast instead of mana, and an empty wand does nothing. There is no recharging.
//!
//! ```toml
//! [object.wand]
//! spell = "fireball"            # an aimed spell of data/spells/
//! charges = 7                   # required: the wand's own, 1 to 99
//! rungs = { min = 0, max = 2 }  # the rungs a wand of the kind casts at (default min 0, max the
//!                               # spell's damage or heal ladder's length; at most that length)
//! ```
//!
//! Until each wand draws its own rung in its range (S5), a wand casts at `min`, plus the user's
//! wand rungs (Wand mastery, of Wands).

use super::{Content, SpellId};
use serde::Deserialize;

/// The most charges a wand may hold.
pub const MAX_WAND_CHARGES: u8 = 99;

/// `[object.wand]` as data writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WandSpec {
    spell: String,
    charges: u8,
    rungs: Option<RungsSpec>,
    /// Refused: the spell scaling design §7.4 replaced it with `rungs`.
    caster_level: Option<toml::Value>,
}

/// `rungs = { min, max }` as written.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RungsSpec {
    min: Option<u8>,
    max: Option<u8>,
}

/// A wand, resolved: the spell it holds, the charges a new one has, and the rungs it casts at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wand {
    pub spell: SpellId,
    /// The charges a wand of this kind is made with, 1 to [`MAX_WAND_CHARGES`].
    pub charges: u8,
    /// The lowest rung of its spell's damage or heal ladder a wand of the kind casts at.
    pub min_rung: u8,
    /// The highest, at most the ladder's length.
    pub max_rung: u8,
}

impl WandSpec {
    /// Resolves it against the spells (loaded before objects): an aimed spell (a wand is
    /// aimed as a missile weapon is), charges 1 to [`MAX_WAND_CHARGES`] (its own, required), its
    /// `rungs` within the spell's damage or heal ladder (`min` at most `max`).
    pub(super) fn resolve(self, content: &Content) -> Result<Wand, String> {
        if self.caster_level.is_some() {
            return Err(
                "`wand`: `caster_level` is gone: a wand casts at its `rungs` on its spell's \
                        ladder, `rungs = { min = 0, max = 2 }` (docs/design/spell-scaling.md §7.4)"
                    .into(),
            );
        }
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
        let most = u8::try_from(def.grows.roll().map_or(0, |l| l.len())).unwrap_or(u8::MAX);
        let given = self.rungs.unwrap_or(RungsSpec {
            min: None,
            max: None,
        });
        let (min_rung, max_rung) = (given.min.unwrap_or(0), given.max.unwrap_or(most));
        if min_rung > max_rung || max_rung > most {
            return Err(format!(
                "`wand`: `rungs` is `min` to `max` within 0 to {most}, the length of `{}`'s \
                 damage or heal ladder",
                self.spell
            ));
        }
        Ok(Wand {
            spell,
            charges,
            min_rung,
            max_rung,
        })
    }
}
