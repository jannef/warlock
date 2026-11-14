//! What reading does (`read` in an object's file, spec §8 "Scrolls"): a scroll casts a spell
//! of `data/spells/` through the one cast path at its own rungs, free of mana. A scroll teaches
//! nothing: its spell's grimoire does (`grimoire.rs`). The one other kind, `cleanse`, has no
//! spell behind it: the scroll of restoration mends the reader's stat damage and gives back what
//! drains took (the priest design §11).
//!
//! ```toml
//! [object.read]
//! kind = "spell"        # the spell of data/spells/ named by `spell`
//! spell = "fireball"
//! rungs = 1             # the rung of the spell's damage or heal ladder it casts at (default 0,
//!                       # the base; one past the ladder casts at its last); the reader's Scroll
//!                       # adept adds
//!
//! [object.read]
//! kind = "cleanse"      # no spell: read on oneself
//! mends = true          # every point of stat damage mends (default false)
//! restores = true       # every point drains took comes back (default false)
//! ```

use super::{Content, SpellId};
use crate::talent::HungerEffect;
use serde::Deserialize;

/// What kind of reading data names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Spell,
    Cleanse,
}

/// `read` as an object's file writes it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadSpec {
    kind: Kind,
    /// The spell of `data/spells/` it casts.
    spell: Option<String>,
    /// The rung of the spell's roll ladder a read casts at (default 0).
    rungs: Option<u8>,
    /// A `cleanse` reading's: it mends stat damage.
    #[serde(default)]
    mends: bool,
    /// A `cleanse` reading's: it restores drained points.
    #[serde(default)]
    restores: bool,
}

/// What reading a scroll does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// It casts a spell of `data/spells/` through the one cast path.
    Cast(SpellScroll),
    /// No spell: it cleanses its reader (the scroll of restoration).
    Cleanse(Cleansing),
}

/// What a `cleanse` reading does to its reader (one of the two at least).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cleansing {
    /// Every point of stat damage mends.
    pub mends: bool,
    /// Every point drains took comes back.
    pub restores: bool,
}

/// A scroll of a spell of `data/spells/` (spec §8 "Scrolls", "Spells"): read, it casts the
/// spell at its `rungs`, free of mana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellScroll {
    pub spell: SpellId,
    /// The rung of the spell's damage or heal ladder a read casts at (0: the base), before the
    /// reader's `scroll_rungs` (Scroll adept); at most the ladder's length.
    pub rungs: u8,
    /// How far the reader may aim it at the spell's base (its data's reach); `None` for a spell
    /// on oneself, read without aiming.
    pub reach: Option<i32>,
    /// Whether it is read on a stack the reader carries (Identify's scroll): the read list asks
    /// which.
    pub on_item: bool,
    /// Whether it is read at a corpse the reader sees (`SpellData::aims_corpse`).
    pub at_corpse: bool,
}

impl Reading {
    /// How far the reader may aim it; `None` for what is not aimed (identify, a spell on
    /// oneself, a cleansing).
    pub fn range(&self) -> Option<i32> {
        self.scroll().and_then(|scroll| scroll.reach)
    }

    /// Whether it is read at a cell, a burst's or a corpse's: a scroll that raises a corpse has
    /// no range and is aimed all the same.
    pub fn aimed(&self) -> bool {
        self.scroll()
            .is_some_and(|scroll| scroll.reach.is_some() || scroll.at_corpse)
    }

    /// Whether it is read on a stack the reader carries (the scroll of identify).
    pub fn on_item(&self) -> bool {
        self.scroll().is_some_and(|scroll| scroll.on_item)
    }

    /// The spell scroll it is, if it is one.
    pub fn scroll(&self) -> Option<&SpellScroll> {
        match self {
            Reading::Cast(scroll) => Some(scroll),
            Reading::Cleanse(_) => None,
        }
    }

    /// The spell of `data/spells/` it casts (its grimoire teaches it); none for a
    /// cleansing.
    pub fn spell(&self) -> Option<SpellId> {
        self.scroll().map(|scroll| scroll.spell)
    }
}

impl ReadSpec {
    /// Whether it casts a spell (kind `spell`).
    pub(super) fn casts(&self) -> bool {
        self.kind == Kind::Spell
    }

    /// Resolves the spec: a `spell` reading of a spell of `data/spells/` (spells load before
    /// objects); a `cleanse` reading with no spell or rungs, and `mends` and `restores` only on
    /// it (neither would be read on the other kind).
    pub(super) fn resolve(self, content: &Content) -> Result<Reading, String> {
        if self.kind == Kind::Cleanse {
            if self.spell.is_some() || self.rungs.is_some() {
                return Err("`read`: a `cleanse` reading has no `spell` or `rungs`".into());
            }
            let (mends, restores) = (self.mends, self.restores);
            return Ok(Reading::Cleanse(Cleansing { mends, restores }));
        }
        if self.mends || self.restores {
            return Err("`read`: `mends` and `restores` belong to a `cleanse` reading".into());
        }
        let id = self
            .spell
            .ok_or("`read`: a `spell` reading needs its `spell`")?;
        let spell = (content.spell_id(&id))
            .ok_or_else(|| format!("`read`: no spell `{id}` in data/spells/"))?;
        let def = content.spell(spell);
        let rungs = self.rungs.unwrap_or(0);
        Ok(Reading::Cast(SpellScroll {
            spell,
            rungs,
            reach: def.base_range(),
            on_item: def.aims_item(),
            at_corpse: def.aims_corpse(),
        }))
    }
}

/// The meals the talents name are objects (checked once objects are loaded).
pub(super) fn check_spells(content: &Content) -> Result<(), String> {
    // The meals the talents name are objects (loaded after the talents).
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            if let super::TalentEffect::Hunger(HungerEffect::MealHeals {
                object: Some(object),
                ..
            }) = effect
                && !content.object_defs().is_empty()
                && content.object_id(object).is_none()
            {
                return Err(format!(
                    "talent `{}`: `meal_heals` names an unknown object `{object}`",
                    def.id
                ));
            }
        }
    }
    Ok(())
}
