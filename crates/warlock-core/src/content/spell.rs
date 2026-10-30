//! Spells in data (spec §8 "Traps", "Spells"): the bursts and bolts that traps
//! (`data/traps.toml`) cast, with the rules and checks the spells of `data/spells/` share (their
//! bursts and lines go off through the same code). Who casts one decides only who deals its harm
//! (`world/spell.rs`).
//!
//! - A **burst** (`Burst`, a fireball): everyone on the cells within its `radius` that a
//!   shadowcast from its center reaches takes one roll of its `damage`, half of it (at least 1)
//!   beyond radius − 1; with a `blast` it breaks the walls it reaches whose hardness is below
//!   that; a hot one (`heat`) melts the ice it reaches, a cold one (`frost`, mm) freezes the
//!   water it reaches to at least that thick.
//! - A **bolt** (`Bolt`, lightning): everyone on its line takes one roll of its `damage`, up to
//!   its `length` cells or the first wall.
//!
//! Both name the `spell` effect presentation shows (`data/effects.toml`), which must be an
//! effect of their shape when that file exists ([`check_effect`]), and the kind of damage they
//! do (`damage_type`, spec §8 "Damage types": the fireball fire, the frost cold, the lightning
//! electricity), which the targets resist and no armour soaks.

use super::effect::EffectShape;
use super::terrain::MAX_HARDNESS;
use super::{Content, DamageTypeId};
use crate::dice::{Dice, Roll};
use crate::ice;
use crate::perception::VIEW_RADIUS;
use std::ops::RangeInclusive;

/// The radii a burst may have, in cells.
pub const BURST_RADII: RangeInclusive<i32> = 1..=5;
/// The lengths a bolt may have, in cells (a trap's own cell is the first).
pub const BOLT_LENGTHS: RangeInclusive<i32> = 1..=VIEW_RADIUS;

/// What a burst does to the ice and the water it reaches (spec §5 "Ice").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurstIce {
    /// Heat (a fireball): it melts all the ice, and whoever stood on it falls in.
    Melts,
    /// Frost: the water freezes at once to at least this many µm of ice.
    Freezes(u32),
}

/// A blast of `radius` cells around a center: `damage` to everyone in it (half beyond the inner
/// cells), no roll to hit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Burst {
    /// The effect presentation shows (`data/effects.toml`).
    pub spell: String,
    pub radius: i32,
    pub damage: Roll,
    /// A die of its roll showing this or less is drawn once more (a caster's roll hook; a
    /// trap's 0).
    pub reroll: u32,
    /// The kind of damage it does (spec §8 "Damage types"); `None` without damage types.
    pub kind: Option<DamageTypeId>,
    /// What it does to walls (spec §8 "Digging"): it breaks those in the blast whose hardness is
    /// below this; 0 breaks none.
    pub blast: u16,
    /// What it does to the ice and water it reaches (spec §5 "Ice"), if anything.
    pub ice: Option<BurstIce>,
}

/// A line until a wall, at most `length` cells: `damage` to everyone on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bolt {
    /// The effect presentation shows (`data/effects.toml`).
    pub spell: String,
    pub length: i32,
    pub damage: Roll,
    /// A die of its roll showing this or less is drawn once more (a caster's roll hook; a
    /// trap's 0).
    pub reroll: u32,
    /// The kind of damage it does (spec §8 "Damage types"); `None` without damage types.
    pub kind: Option<DamageTypeId>,
}

/// A spell's fields as data writes them, for [`Burst::new`] and [`Bolt::new`]: what a trap and a
/// scroll share.
#[derive(Clone, Debug, Default)]
pub(super) struct SpellFields {
    pub spell: Option<String>,
    pub damage: Option<Dice>,
    /// The one kind of damage it does (resolved by the caller, which knows the content).
    pub kind: Option<DamageTypeId>,
    pub radius: Option<i32>,
    pub blast: Option<u16>,
    pub heat: bool,
    pub frost: Option<u16>,
    pub length: Option<i32>,
}

impl SpellFields {
    fn spell(&self) -> Result<String, String> {
        (self.spell.clone())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "needs the `spell` effect it shows".into())
    }

    fn damage(&self) -> Result<Dice, String> {
        self.damage
            .filter(|d| d.min() >= 0 && d.max() >= 1)
            .ok_or_else(|| "needs `damage` dice that can do harm, e.g. \"1d6\"".into())
    }
}

impl Burst {
    /// The burst `fields` describe, checked: a radius in [`BURST_RADII`], a blast up to
    /// [`MAX_HARDNESS`], `heat` or `frost` (1 to [`ice::MAX_MM`] mm) but not both.
    pub(super) fn new(fields: &SpellFields) -> Result<Self, String> {
        let radius = fields.radius.unwrap_or(0);
        if !BURST_RADII.contains(&radius) {
            return Err(format!(
                "`radius` must be {} to {}",
                BURST_RADII.start(),
                BURST_RADII.end()
            ));
        }
        let blast = fields.blast.unwrap_or(0);
        if blast > MAX_HARDNESS {
            return Err(format!("`blast` must be 0 to {MAX_HARDNESS}"));
        }
        let ice = match (fields.heat, fields.frost) {
            (true, Some(_)) => return Err("a burst has `heat` or `frost`, not both".into()),
            (true, None) => Some(BurstIce::Melts),
            (false, Some(mm)) if (1..=ice::MAX_MM).contains(&mm) => {
                Some(BurstIce::Freezes(u32::from(mm) * 1000))
            }
            (false, Some(_)) => return Err(format!("`frost` must be 1 to {} mm", ice::MAX_MM)),
            (false, None) => None,
        };
        Ok(Self {
            spell: fields.spell()?,
            radius,
            damage: fields.damage()?.into(),
            reroll: 0,
            kind: fields.kind,
            blast,
            ice,
        })
    }
}

impl Bolt {
    /// The bolt `fields` describe, checked: a length in [`BOLT_LENGTHS`].
    pub(super) fn new(fields: &SpellFields) -> Result<Self, String> {
        let length = fields.length.unwrap_or(0);
        if !BOLT_LENGTHS.contains(&length) {
            return Err(format!(
                "`length` must be {} to {}",
                BOLT_LENGTHS.start(),
                BOLT_LENGTHS.end()
            ));
        }
        Ok(Self {
            spell: fields.spell()?,
            length,
            damage: fields.damage()?.into(),
            reroll: 0,
            kind: fields.kind,
        })
    }
}

/// The one kind of damage a spell or a trap does, from its `damage_type` (`names`): one at
/// most; none is an untyped spell.
pub(super) fn one_kind(
    content: &Content,
    names: Option<&super::damage_type::TypeNames>,
) -> Result<Option<DamageTypeId>, String> {
    let kinds = content.resolve_kinds(names)?;
    if kinds.iter().count() > 1 {
        return Err("`damage_type` names one kind here".into());
    }
    Ok(kinds.main())
}

/// What a blow of `kind` is said with when data gives no `verb`: the kind's verb ("burns"),
/// else "hits".
pub(super) fn default_verb(content: &Content, kind: Option<DamageTypeId>) -> String {
    kind.map_or_else(
        || "hits".to_string(),
        |k| content.damage_type(k).verb.clone(),
    )
}

/// With spell effects in content, `spell` must name an effect of `shape` (`data/effects.toml`).
pub(super) fn check_effect(
    content: &Content,
    spell: &str,
    shape: EffectShape,
) -> Result<(), String> {
    if !content.effects().is_empty() && content.effect(spell).is_none_or(|e| e.shape != shape) {
        let kind = match shape {
            EffectShape::Burst => "burst",
            EffectShape::Bolt => "bolt",
        };
        return Err(format!(
            "`spell = \"{spell}\"` must name a {kind} effect in data/effects.toml"
        ));
    }
    Ok(())
}
